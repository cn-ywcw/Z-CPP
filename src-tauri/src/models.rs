use serde::{Deserialize, Serialize};

// ── 编辑器设置 ──────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EditorSettings {
    #[serde(default = "default_font_family")]
    pub font_family: String,
    #[serde(default = "default_font_size")]
    pub font_size: u32,
    #[serde(default = "default_tab_size")]
    pub tab_size: u32,
    #[serde(default = "default_theme")]
    pub theme: String,
    #[serde(default)]
    pub word_wrap: String,
}

impl Default for EditorSettings {
    fn default() -> Self {
        Self {
            font_family: default_font_family(),
            font_size: default_font_size(),
            tab_size: default_tab_size(),
            theme: default_theme(),
            word_wrap: "off".to_string(),
        }
    }
}

fn default_font_family() -> String { "'Cascadia Code', 'Fira Code', 'Consolas', monospace".to_string() }
fn default_font_size() -> u32 { 14 }
fn default_tab_size() -> u32 { 4 }
fn default_theme() -> String { "vs-dark".to_string() }

// ── 外观设置 ────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppearanceSettings {
    #[serde(default)]
    pub background_image: String,
    #[serde(default = "default_opacity")]
    pub opacity: f64,
    #[serde(default)]
    pub frosted_glass: bool,
    #[serde(default = "default_blur_amount")]
    pub blur_amount: u32,
    #[serde(default = "default_background_opacity")]
    pub background_opacity: f64,
}

impl Default for AppearanceSettings {
    fn default() -> Self {
        Self {
            background_image: String::new(),
            opacity: default_opacity(),
            frosted_glass: false,
            blur_amount: default_blur_amount(),
            background_opacity: default_background_opacity(),
        }
    }
}

fn default_opacity() -> f64 { 1.0 }
fn default_blur_amount() -> u32 { 10 }
fn default_background_opacity() -> f64 { 1.0 }

// ── 应用元数据（只读）──────────────────────────────

#[derive(Debug, Serialize)]
pub struct AppMeta {
    pub version: String,
    pub license: String,
}

// ── 编译选项 ────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompileOptions {
    #[serde(default = "default_opt_level")]
    pub optimization: String,
    #[serde(default = "default_warnings")]
    pub warnings: String,
    #[serde(default)]
    pub standard: String,
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

#[derive(Debug, Deserialize)]
pub struct CompileRequest {
    pub code: String,
    #[serde(default = "default_filename")]
    pub filename: String,
    #[serde(default = "default_compiler")]
    pub compiler: String,
    #[serde(default)]
    pub compile_options: Option<CompileOptions>,
    #[serde(default)]
    pub options: String,
    pub std: Option<String>,
    #[serde(default)]
    pub compile_only: bool,
    #[serde(default)]
    pub input_text: String,
}

fn default_filename() -> String { "main.cpp".to_string() }
fn default_compiler() -> String { "gcc".to_string() }

#[derive(Debug, Serialize)]
pub struct CompileResponse {
    pub success: bool,
    pub compile_output: String,
    pub run_output: String,
    pub run_time_ms: Option<u64>,
    pub exit_code: Option<i32>,
}

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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    #[serde(default)]
    pub gcc_path: String,
    #[serde(default)]
    pub clang_path: String,
    #[serde(default = "default_compiler")]
    pub default_compiler: String,
    #[serde(default)]
    pub default_options: CompileOptions,
    #[serde(default = "default_workspace")]
    pub workspace: String,
    #[serde(default)]
    pub editor: EditorSettings,
    #[serde(default)]
    pub appearance: AppearanceSettings,
    #[serde(default)]
    pub auto_save: bool,
    #[serde(default)]
    pub default_compile_only: bool,
    #[serde(default = "default_true")]
    pub restore_tabs: bool,
}

fn default_workspace() -> String {
    let mut p = std::env::current_exe().unwrap_or_default();
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
            editor: EditorSettings::default(),
            appearance: AppearanceSettings::default(),
            auto_save: false,
            default_compile_only: false,
            restore_tabs: true,
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

fn default_true() -> bool { true }

fn default_new_file_content() -> String {
    String::new()
}

// ── 会话持久化 ──────────────────────────────────────

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SessionTab {
    pub filename: String,
    pub language: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SessionData {
    pub tabs: Vec<SessionTab>,
    pub active_tab: usize,
}

#[derive(Debug, Deserialize)]
pub struct SaveSessionRequest {
    pub session: SessionData,
}

// ── 文件操作（右键菜单）────────────────────────────

#[derive(Debug, Deserialize)]
pub struct CreateDirRequest {
    pub name: String,
}

#[derive(Debug, Deserialize)]
pub struct FileOpRequest {
    pub filename: String,
}

#[derive(Debug, Deserialize)]
pub struct RenameRequest {
    pub old_name: String,
    pub new_name: String,
}

#[derive(Debug, Deserialize)]
pub struct CopyRequest {
    pub source: String,
    pub dest: String,
}

#[derive(Debug, Serialize)]
pub struct FileOpResponse {
    pub success: bool,
    pub message: String,
}
