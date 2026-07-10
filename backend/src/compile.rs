/// Z-CPP 后端 — 编译模块
///
/// 负责接收源代码，调用 GCC/Clang 编译并运行。

use std::path::PathBuf;
use std::process::Command;
use std::time::Instant;
use tracing::{info, warn};

use crate::models::{CompileRequest, CompileResponse};

/// 工作目录：存放待编译的源代码文件
const WORKSPACE_DIR: &str = "../workspace";

/// 执行编译请求
pub async fn compile_and_run(req: CompileRequest) -> CompileResponse {
    // 1. 确定编译器
    let compiler = resolve_compiler(&req.compiler);
    if !check_compiler_available(&compiler) {
        return CompileResponse {
            success: false,
            compile_output: format!("错误: 找不到编译器 '{}'，请确保已安装。", compiler),
            run_output: String::new(),
            run_time_ms: None,
            exit_code: None,
        };
    }

    // 2. 确定语言标准
    let lang_std = if req.filename.ends_with(".c") {
        // C 文件
        req.std.as_deref().unwrap_or("c17")
    } else {
        // C++ 文件
        req.std.as_deref().unwrap_or("c++17")
    };

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

    // 4. 写源代码到工作目录
    let workspace_path = PathBuf::from(WORKSPACE_DIR);
    tokio::fs::create_dir_all(&workspace_path).await.unwrap_or_else(|e| {
        warn!("创建工作目录失败: {}", e);
    });

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

    // 5. 编译
    let compile_result = compile_source(
        &compiler,
        &source_path,
        &output_path,
        &lang_std,
        &req.options,
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

    // 如果只编译不运行
    if req.compile_only {
        return CompileResponse {
            success: true,
            compile_output,
            run_output: String::new(),
            run_time_ms: None,
            exit_code: Some(0),
        };
    }

    // 6. 运行
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

/// 获取可用的编译器列表
pub fn get_available_compilers() -> Vec<(String, String, bool)> {
    let compilers = vec![
        ("GCC".to_string(), get_gcc_command(), true),
        ("Clang".to_string(), get_clang_command(), true),
    ];
    compilers
        .into_iter()
        .map(|(name, cmd, _)| {
            let available = check_compiler_available(&cmd);
            (name, cmd, available)
        })
        .collect()
}

fn resolve_compiler(compiler: &str) -> String {
    match compiler.to_lowercase().as_str() {
        "clang" => get_clang_command(),
        _ => get_gcc_command(),
    }
}

fn get_gcc_command() -> String {
    if cfg!(target_os = "windows") {
        "g++".to_string()
    } else {
        "g++".to_string()
    }
}

fn get_clang_command() -> String {
    if cfg!(target_os = "windows") {
        "clang++".to_string()
    } else {
        "clang++".to_string()
    }
}

fn check_compiler_available(compiler: &str) -> bool {
    Command::new(compiler)
        .arg("--version")
        .output()
        .is_ok()
}

fn compile_source(
    compiler: &str,
    source: &PathBuf,
    output: &PathBuf,
    lang_std: &str,
    extra_options: &str,
) -> CompileOutput {
    let source_str = source.to_string_lossy().to_string();
    let output_str = output.to_string_lossy().to_string();

    let is_cpp = source_str.ends_with(".cpp") || source_str.ends_with(".cc");
    let std_flag = if is_cpp {
        format!("-std={}", lang_std)
    } else {
        format!("-std={}", lang_std)
    };

    let mut cmd = Command::new(compiler);
    cmd.arg(&source_str)
        .arg("-o")
        .arg(&output_str)
        .arg(&std_flag)
        .arg("-O2") // 算法竞赛常用优化
        .arg("-Wall") // 开启警告
        .arg("-Wextra");

    // 添加额外选项
    if !extra_options.is_empty() {
        for opt in extra_options.split_whitespace() {
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

fn run_program(program: &PathBuf) -> RunResult {
    let program_str = program.to_string_lossy().to_string();

    let start = Instant::now();

    let output = Command::new(&program_str)
        .output();

    let elapsed = start.elapsed();

    match output {
        Ok(out) => {
            let stdout = String::from_utf8_lossy(&out.stdout).to_string();
            let stderr = String::from_utf8_lossy(&out.stderr).to_string();
            let full_output = if stderr.is_empty() {
                stdout
            } else {
                format!("{}\n{}", stdout, stderr)
            };

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
