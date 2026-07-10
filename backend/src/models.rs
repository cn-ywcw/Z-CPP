/// Z-CPP 后端 — 数据模型
use serde::{Deserialize, Serialize};

/// 编译请求
#[derive(Debug, Deserialize)]
pub struct CompileRequest {
    /// 源代码内容
    pub code: String,
    /// 文件名（可选，默认 main.cpp）
    #[serde(default = "default_filename")]
    pub filename: String,
    /// 编译器类型: "gcc" | "clang"
    #[serde(default = "default_compiler")]
    pub compiler: String,
    /// 编译选项（额外参数）
    #[serde(default)]
    pub options: String,
    /// C 标准（可选，如 "c11", "c17"）
    pub std: Option<String>,
    /// 是否只编译不运行
    #[serde(default)]
    pub compile_only: bool,
}

fn default_filename() -> String {
    "main.cpp".to_string()
}

fn default_compiler() -> String {
    "gcc".to_string()
}

/// 编译运行结果
#[derive(Debug, Serialize)]
pub struct CompileResponse {
    /// 是否成功
    pub success: bool,
    /// 编译输出（错误信息等）
    pub compile_output: String,
    /// 运行输出（仅在成功时有效）
    pub run_output: String,
    /// 运行时间（毫秒）
    pub run_time_ms: Option<u64>,
    /// 退出码
    pub exit_code: Option<i32>,
}

/// 健康检查响应
#[derive(Debug, Serialize)]
pub struct HealthResponse {
    pub status: String,
    pub version: String,
    pub gcc_available: bool,
    pub clang_available: bool,
}

/// 保存文件请求
#[derive(Debug, Deserialize)]
pub struct SaveFileRequest {
    pub filename: String,
    pub content: String,
}

/// 语言列表响应
#[derive(Debug, Serialize)]
pub struct LanguageInfo {
    pub name: String,
    pub extension: String,
    pub compilers: Vec<CompilerInfo>,
}

#[derive(Debug, Serialize)]
pub struct CompilerInfo {
    pub name: String,
    pub command: String,
    pub available: bool,
}
