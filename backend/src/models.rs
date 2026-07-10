/// Z-CPP 后端 — 数据模型
use serde::{Deserialize, Serialize};

// ── 编译请求 ──────────────────────────────────────────

/// 编译选项
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompileOptions {
    /// 优化等级: "O0", "O1", "O2", "O3", "Os", "Ofast"
    #[serde(default = "default_opt_level")]
    pub optimization: String,
    /// 警告选项: "Wall", "Wall-Wextra", "Wall-Wextra-Werror", "none"
    #[serde(default = "default_warnings")]
    pub warnings: String,
    /// C/C++ 标准: "c++11", "c++14", "c++17", "c++20", "c++23" / "c11", "c17"
    #[serde(default)]
    pub standard: String,
    /// 额外编译参数
    #[serde(default)]
    pub extra_flags: String,
}

fn default_opt_level() -> String { "O2".to_string() }
fn default_warnings() -> String { "Wall-Wextra".to_string() }

impl Default for CompileOptions {
    fn default() -> Self {
        Self {
            optimization: "O2".to_string(),
            warnings: "Wall-Wextra".to_string(),
            standard: String::new(),
            extra_flags: String::new(),
        }
    }
}

/// 编译请求
#[derive(Debug, Deserialize)]
pub struct CompileRequest {
    /// 源代码内容
    pub code: String,
    /// 文件名
    #[serde(default = "default_filename")]
    pub filename: String,
    /// 编译器类型: "gcc" | "clang"
    #[serde(default = "default_compiler")]
    pub compiler: String,
    /// 编译选项（结构化，与 options/std 二选一使用）
    #[serde(default)]
    pub compile_options: Option<CompileOptions>,
    /// 旧式：额外编译选项字符串（当 compile_options 为 None 时使用）
    #[serde(default)]
    pub options: String,
    /// 旧式：C/C++ 标准
    pub std: Option<String>,
    /// 是否只编译不运行
    #[serde(default)]
    pub compile_only: bool,
}

fn default_filename() -> String { "main.cpp".to_string() }
fn default_compiler() -> String { "gcc".to_string() }

// ── 编译响应 ──────────────────────────────────────────

#[derive(Debug, Serialize)]
pub struct CompileResponse {
    pub success: bool,
    pub compile_output: String,
    pub run_output: String,
    pub run_time_ms: Option<u64>,
    pub exit_code: Option<i32>,
}

// ── 文件管理 ──────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct SaveFileRequest {
    pub filename: String,
    pub content: String,
}

#[derive(Debug, Serialize)]
pub struct FileInfo {
    pub name: String,
    pub path: String,
    pub size: u64,
    pub is_dir: bool,
}

#[derive(Debug, Serialize)]
pub struct FileListResponse {
    pub files: Vec<FileInfo>,
    pub workspace: String,
}

// ── 设置管理 ──────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    /// 自定义 GCC 路径（空 = 使用 PATH）
    #[serde(default)]
    pub gcc_path: String,
    /// 自定义 Clang 路径（空 = 使用 PATH）
    #[serde(default)]
    pub clang_path: String,
    /// 默认编译器
    #[serde(default = "default_compiler")]
    pub default_compiler: String,
    /// 默认编译选项
    #[serde(default)]
    pub default_options: CompileOptions,
    /// 工作目录路径
    #[serde(default = "default_workspace")]
    pub workspace: String,
}

fn default_workspace() -> String {
    let mut p = std::env::current_exe()
        .unwrap_or_default();
    p.pop();
    p.push("workspace");
    p.to_string_lossy().to_string()
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            gcc_path: String::new(),
            clang_path: String::new(),
            default_compiler: "gcc".to_string(),
            default_options: CompileOptions::default(),
            workspace: default_workspace(),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct SaveSettingsRequest {
    pub settings: Settings,
}

#[derive(Debug, Serialize)]
pub struct SettingsResponse {
    pub settings: Settings,
}

// ── 其他 ──────────────────────────────────────────────

#[derive(Debug, Serialize)]
pub struct HealthResponse {
    pub status: String,
    pub version: String,
    pub gcc_available: bool,
    pub clang_available: bool,
}

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

#[derive(Debug, Serialize)]
pub struct CreateFileResponse {
    pub success: bool,
    pub message: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateFileRequest {
    pub filename: String,
    #[serde(default = "default_new_file_content")]
    pub content: String,
}

fn default_new_file_content() -> String {
    String::new()
}
