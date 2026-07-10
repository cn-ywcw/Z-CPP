/// Z-CPP 后端 — 编译模块
///
/// 负责接收源代码，调用 GCC/Clang 编译并运行。
/// 支持自定义编译器路径、结构化编译选项。

use std::path::PathBuf;
use std::process::Command;
use std::time::Instant;
use tracing::{info, warn};

use crate::models::{CompileOptions, CompileRequest, CompileResponse, Settings};

// ── 工作目录 ──────────────────────────────────────────

/// 工作目录，可通过 ZCPP_WORKSPACE 或设置中的 workspace 覆盖
pub fn workspace_dir_override() -> String {
    std::env::var("ZCPP_WORKSPACE")
        .ok()
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "workspace".to_string())
}

// ── 设置管理 ──────────────────────────────────────────

const SETTINGS_FILE: &str = "z-cpp-settings.json";

/// 加载持久化的设置
pub fn load_settings() -> Settings {
    let path = PathBuf::from(SETTINGS_FILE);
    if path.exists() {
        match std::fs::read_to_string(&path) {
            Ok(s) => serde_json::from_str(&s).unwrap_or_default(),
            Err(_) => Settings::default(),
        }
    } else {
        Settings::default()
    }
}

/// 保存设置到磁盘
pub fn save_settings(settings: &Settings) -> Result<(), String> {
    let path = PathBuf::from(SETTINGS_FILE);
    let json = serde_json::to_string_pretty(settings).map_err(|e| e.to_string())?;
    std::fs::write(&path, json).map_err(|e| e.to_string())?;
    info!("设置已保存");
    Ok(())
}

// ── 编译器路径 ────────────────────────────────────────

/// 获取编译器可用性列表（考虑自定义路径）
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

/// 执行编译请求
pub async fn compile_and_run(req: CompileRequest, settings: &Settings) -> CompileResponse {
    // 1. 确定编译器路径
    let compiler = if req.compiler.eq_ignore_ascii_case("clang") {
        compiler_cmd("clang", &settings.clang_path)
    } else {
        compiler_cmd("gcc", &settings.gcc_path)
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

    // 2. 确定工作目录
    let workspace_path = if settings.workspace.is_empty() {
        PathBuf::from(workspace_dir_override())
    } else {
        PathBuf::from(&settings.workspace)
    };
    tokio::fs::create_dir_all(&workspace_path).await.unwrap_or_else(|e| {
        warn!("创建工作目录失败: {}", e);
    });

    // 3. 确定输出文件
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

    // 4. 写源代码
    if let Err(e) = tokio::fs::write(&source_path, &req.code).await {
        return CompileResponse {
            success: false,
            compile_output: format!("错误: 写入源文件失败: {}", e),
            run_output: String::new(),
            run_time_ms: None,
            exit_code: None,
        };
    }

    // 5. 确定编译选项
    let opts = req.compile_options
        .clone()
        .unwrap_or_else(|| settings.default_options.clone());

    // 从旧字段兼容
    let std_str = req.std.clone().unwrap_or_default();
    if !std_str.is_empty() && opts.standard.is_empty() {
        // 旧请求格式，使用 req.std
    }

    // 6. 编译
    let compile_result = compile_source(
        &compiler,
        &source_path,
        &output_path,
        &req.filename,
        &opts,
        &req.options,
        &req.std,
    );

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

    // 如果只编译
    if req.compile_only {
        return CompileResponse {
            success: true,
            compile_output,
            run_output: String::new(),
            run_time_ms: None,
            exit_code: Some(0),
        };
    }

    // 7. 运行
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

/// 构建编译命令并执行
fn compile_source(
    compiler: &str,
    source: &PathBuf,
    output: &PathBuf,
    filename: &str,
    opts: &CompileOptions,
    legacy_options: &str,
    legacy_std: &Option<String>,
) -> CompileOutput {
    let source_str = source.to_string_lossy().to_string();
    let output_str = output.to_string_lossy().to_string();

    let _is_cpp = filename.ends_with(".cpp") || filename.ends_with(".cc") || filename.ends_with(".cxx");
    let is_c = filename.ends_with(".c");

    let mut cmd = Command::new(compiler);
    cmd.arg(&source_str).arg("-o").arg(&output_str);

    // --- 标准 ---
    let std_flag = if !opts.standard.is_empty() {
        Some(if is_c { format!("-std={}", opts.standard) } else { format!("-std={}", opts.standard) })
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

    // --- 优化等级 ---
    if !opts.optimization.is_empty() {
        cmd.arg(format!("-{}", opts.optimization));
    }

    // --- 警告选项 ---
    match opts.warnings.as_str() {
        "Wall" => { cmd.arg("-Wall"); }
        "Wall-Wextra" => { cmd.arg("-Wall"); cmd.arg("-Wextra"); }
        "Wall-Wextra-Werror" => { cmd.arg("-Wall"); cmd.arg("-Wextra"); cmd.arg("-Werror"); }
        _ => {} // "none" 或其它 = 无警告
    }

    // --- 旧式额外选项（向后兼容） ---
    if !legacy_options.is_empty() {
        for opt in legacy_options.split_whitespace() {
            cmd.arg(opt);
        }
    }

    // --- 新式额外选项 ---
    if !opts.extra_flags.is_empty() {
        for opt in opts.extra_flags.split_whitespace() {
            cmd.arg(opt);
        }
    }

    info!("编译命令: {:?}", cmd);

    let output = cmd.output().expect("编译进程启动失败");

    CompileOutput {
        status: output.status,
        stdout: output.stdout,
        stderr: output.stderr,
    }
}

/// 运行已编译的程序
fn run_program(program: &PathBuf) -> RunResult {
    let program_str = program.to_string_lossy().to_string();
    let start = Instant::now();

    let output = Command::new(&program_str).output();
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
