use std::path::PathBuf;
use std::process::Command;
use std::time::Instant;
use tracing::{info, warn};

#[cfg(windows)]
use std::os::windows::process::CommandExt;

use crate::models::{CompileOptions, CompileRequest, CompileResponse, Settings};

fn validate_filename(filename: &str) -> Result<(), String> {
    if filename.contains("..") || filename.contains('/') || filename.contains('\\') {
        return Err("文件名不能包含路径分隔符或 ..".into());
    }
    if filename.contains('\0') {
        return Err("文件名包含非法字符".into());
    }
    let lower = filename.to_lowercase();
    for reserved in &["con", "prn", "aux", "nul", "com1", "com2", "com3", "com4", "com5", "com6", "com7", "com8", "com9", "lpt1", "lpt2", "lpt3", "lpt4", "lpt5", "lpt6", "lpt7", "lpt8", "lpt9"] {
        if lower == *reserved || lower.starts_with(&format!("{}.", reserved)) {
            return Err(format!("文件名 '{}' 是 Windows 保留名", filename));
        }
    }
    Ok(())
}

fn sanitize_extra_flags(flags: &str) -> Vec<String> {
    let dangerous_prefixes = ["-include", "-plugin", "-fplugin=", "-specs=", "-wrapper"];
    flags.split_whitespace()
        .filter(|f| {
            let lower = f.to_lowercase();
            !dangerous_prefixes.iter().any(|p| lower.starts_with(p))
        })
        .map(|s| s.to_string())
        .collect()
}

pub fn workspace_dir_override() -> String {
    std::env::var("ZCPP_WORKSPACE")
        .ok()
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| {
            let mut p = std::env::current_exe().unwrap_or_default();
            p.pop();
            p.push("workspace");
            p.to_string_lossy().to_string()
        })
}

fn settings_file_path() -> PathBuf {
    let mut p = std::env::current_exe().unwrap_or_default();
    p.pop();
    p.push("z-cpp-settings.json");
    p
}

pub fn load_settings() -> Settings {
    let path = settings_file_path();
    if path.exists() {
        match std::fs::read_to_string(&path) {
            Ok(s) => serde_json::from_str(&s).unwrap_or_default(),
            Err(_) => Settings::default(),
        }
    } else {
        Settings::default()
    }
}

pub fn save_settings(settings: &Settings) -> Result<(), String> {
    let path = settings_file_path();
    let json = serde_json::to_string_pretty(settings).map_err(|e| e.to_string())?;
    std::fs::write(&path, json).map_err(|e| e.to_string())?;
    info!("设置已保存");
    Ok(())
}

pub fn get_available_compilers_with_settings(settings: &Settings) -> Vec<(String, String, bool)> {
    let gcc_cmd = compiler_cmd("gcc", &settings.gcc_path);
    let clang_cmd = compiler_cmd("clang", &settings.clang_path);
    vec![
        ("GCC".to_string(), gcc_cmd.clone(), check_available(&gcc_cmd)),
        ("Clang".to_string(), clang_cmd.clone(), check_available(&clang_cmd)),
    ]
}

fn compiler_cmd(kind: &str, custom_path: &str) -> String {
    if !custom_path.is_empty() {
        custom_path.to_string()
    } else {
        match kind {
            "clang" => "clang++".to_string(),
            _ => "g++".to_string(),
        }
    }
}

fn check_available(cmd: &str) -> bool {
    Command::new(cmd).arg("--version").output().is_ok()
}

pub async fn compile_and_run(req: CompileRequest, settings: &Settings) -> CompileResponse {
    if let Err(e) = validate_filename(&req.filename) {
        return CompileResponse {
            success: false,
            compile_output: format!("错误: {}", e),
            run_output: String::new(),
            run_time_ms: None,
            exit_code: None,
        };
    }

    let is_c = req.filename.ends_with(".c");
    let compiler = if req.compiler.eq_ignore_ascii_case("clang") {
        if is_c { "clang".to_string() } else { compiler_cmd("clang", &settings.clang_path) }
    } else {
        if is_c { "gcc".to_string() } else { compiler_cmd("gcc", &settings.gcc_path) }
    };

    if !check_available(&compiler) {
        return CompileResponse {
            success: false,
            compile_output: format!(
                "错误: 找不到编译器 '{}'，请在设置中配置正确的路径。",
                compiler
            ),
            run_output: String::new(),
            run_time_ms: None,
            exit_code: None,
        };
    }

    let workspace_path = if settings.workspace.is_empty() {
        PathBuf::from(workspace_dir_override())
    } else {
        PathBuf::from(&settings.workspace)
    };
    tokio::fs::create_dir_all(&workspace_path).await.unwrap_or_else(|e| {
        warn!("创建工作目录失败: {}", e);
    });

    let file_stem = std::path::Path::new(&req.filename)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("output");
    let output_name = if cfg!(target_os = "windows") {
        format!("{}.exe", file_stem)
    } else {
        file_stem.to_string()
    };
    let source_path = workspace_path.join(&req.filename);
    let output_path = workspace_path.join(&output_name);

    if let Err(e) = tokio::fs::write(&source_path, &req.code).await {
        return CompileResponse {
            success: false,
            compile_output: format!("错误: 写入源文件失败: {}", e),
            run_output: String::new(),
            run_time_ms: None,
            exit_code: None,
        };
    }

    let opts = req.compile_options
        .clone()
        .unwrap_or_else(|| settings.default_options.clone());

    let compile_result = match compile_source(
        &compiler,
        &source_path,
        &output_path,
        &req.filename,
        &opts,
        &req.options,
        &req.std,
    ) {
        Ok(r) => r,
        Err(e) => {
            return CompileResponse {
                success: false,
                compile_output: e,
                run_output: String::new(),
                run_time_ms: None,
                exit_code: None,
            };
        }
    };

    if !compile_result.status.success() {
        let stderr = String::from_utf8_lossy(&compile_result.stderr);
        return CompileResponse {
            success: false,
            compile_output: format!("编译失败:\n{}", stderr),
            run_output: String::new(),
            run_time_ms: None,
            exit_code: compile_result.status.code(),
        };
    }

    let compile_output = String::from_utf8_lossy(&compile_result.stderr).to_string();

    if req.compile_only {
        return CompileResponse {
            success: true,
            compile_output,
            run_output: String::new(),
            run_time_ms: None,
            exit_code: Some(0),
        };
    }

    let run_result = run_program(&output_path);

    CompileResponse {
        success: run_result.exit_code == Some(0),
        compile_output,
        run_output: run_result.output,
        run_time_ms: Some(run_result.time_ms),
        exit_code: run_result.exit_code,
    }
}

struct CompileOutput {
    status: std::process::ExitStatus,
    #[allow(dead_code)]
    stdout: Vec<u8>,
    stderr: Vec<u8>,
}

struct RunResult {
    output: String,
    exit_code: Option<i32>,
    time_ms: u64,
}

fn compile_source(
    compiler: &str,
    source: &PathBuf,
    output: &PathBuf,
    _filename: &str,
    opts: &CompileOptions,
    legacy_options: &str,
    legacy_std: &Option<String>,
) -> Result<CompileOutput, String> {
    let source_str = source.to_string_lossy().to_string();
    let output_str = output.to_string_lossy().to_string();

    let mut cmd = Command::new(compiler);
    #[cfg(windows)]
    cmd.creation_flags(0x08000000); // CREATE_NO_WINDOW
    cmd.arg(&source_str).arg("-o").arg(&output_str);

    let std_flag = if !opts.standard.is_empty() {
        Some(format!("-std={}", opts.standard))
    } else if let Some(std) = legacy_std {
        if !std.is_empty() {
            Some(format!("-std={}", std))
        } else {
            None
        }
    } else {
        None
    };
    if let Some(f) = std_flag {
        cmd.arg(f);
    }

    if !opts.optimization.is_empty() {
        cmd.arg(format!("-{}", opts.optimization));
    }

    match opts.warnings.as_str() {
        "Wall" => { cmd.arg("-Wall"); }
        "Wall-Wextra" => { cmd.arg("-Wall"); cmd.arg("-Wextra"); }
        "Wall-Wextra-Werror" => { cmd.arg("-Wall"); cmd.arg("-Wextra"); cmd.arg("-Werror"); }
        _ => {}
    }

    if !legacy_options.is_empty() {
        for opt in sanitize_extra_flags(legacy_options) {
            cmd.arg(opt);
        }
    }

    if !opts.extra_flags.is_empty() {
        for opt in sanitize_extra_flags(&opts.extra_flags) {
            cmd.arg(opt);
        }
    }

    info!("编译命令: {:?}", cmd);

    let output = cmd.output().map_err(|e| format!("编译进程启动失败: {}", e))?;

    Ok(CompileOutput {
        status: output.status,
        stdout: output.stdout,
        stderr: output.stderr,
    })
}

fn run_program(program: &PathBuf) -> RunResult {
    let program_str = program.to_string_lossy().to_string();
    let start = Instant::now();

    let mut cmd = Command::new(&program_str);
    #[cfg(windows)]
    cmd.creation_flags(0x08000000);
    let output = cmd.output();
    let elapsed = start.elapsed();

    match output {
        Ok(out) => {
            let stdout = String::from_utf8_lossy(&out.stdout).to_string();
            let stderr = String::from_utf8_lossy(&out.stderr).to_string();
            let full_output = if stderr.is_empty() { stdout } else { format!("{}\n{}", stdout, stderr) };
            RunResult {
                output: full_output,
                exit_code: out.status.code(),
                time_ms: elapsed.as_millis() as u64,
            }
        }
        Err(e) => RunResult {
            output: format!("运行失败: {}", e),
            exit_code: None,
            time_ms: elapsed.as_millis() as u64,
        },
    }
}
