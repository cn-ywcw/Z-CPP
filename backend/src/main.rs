/// Z-CPP 后端 — 入口
///
/// 启动 HTTP 服务，提供编译、运行、文件管理、设置等 API。
/// 生产模式下同时 serve 前端静态文件。

mod compile;
mod models;

use axum::{
    http::Method,
    routing::{get, post},
    Extension, Json, Router,
};
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use tower_http::cors::{Any, CorsLayer};
use tower_http::services::ServeDir;
use tracing::info;

/// 应用状态
struct AppState {
    settings: Mutex<models::Settings>,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "z_cpp_backend=info".into()),
        )
        .init();

    let production = std::env::var("ZCPP_MODE").unwrap_or_default() == "production"
        || std::env::args().any(|a| a == "--prod");

    info!(
        "Z-CPP 后端服务启动中... (mode: {})",
        if production { "production" } else { "development" }
    );

    let settings = compile::load_settings();
    check_compilers(&settings);

    let state = Arc::new(AppState { settings: Mutex::new(settings) });

    // API 路由 — 使用 Extension 传递 State
    let api_routes = Router::new()
        .route("/api/health", get(handle_health))
        .route("/api/compile", post(handle_compile))
        .route("/api/languages", get(handle_languages))
        .route("/api/compilers", get(handle_compilers))
        .route("/api/files", get(handle_list_files))
        .route("/api/files", post(handle_create_file))
        .route("/api/save", post(handle_save_file))
        .route("/api/load/{filename}", get(handle_load_file))
        .route("/api/settings", get(handle_get_settings))
        .route("/api/settings", post(handle_save_settings))
        .layer(Extension(state.clone()));

    let app = if production {
        let static_dir = if cfg!(target_os = "windows") {
            ".\\frontend\\dist"
        } else {
            "./frontend/dist"
        };
        let dist_path = std::path::Path::new(static_dir);
        if !dist_path.exists() {
            info!("静态文件目录不存在: {}", dist_path.display());
        }
        api_routes
            .fallback_service(ServeDir::new(static_dir).append_index_html_on_directories(true))
    } else {
        let cors = CorsLayer::new()
            .allow_origin(Any)
            .allow_methods([Method::GET, Method::POST, Method::OPTIONS])
            .allow_headers(Any);
        api_routes.layer(cors)
    };

    let port = parse_port();
    let host = if production { "0.0.0.0" } else { "127.0.0.1" };
    let addr = SocketAddr::new(host.parse().unwrap(), port);
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .unwrap_or_else(|_| panic!("无法绑定地址 {}", addr));
    info!("后端服务已启动: http://{}:{}", host, port);
    axum::serve(listener, app).await.expect("服务器启动失败");
}

// ── 辅助 ──────────────────────────────────────────────

fn parse_port() -> u16 {
    let args: Vec<String> = std::env::args().collect();
    let mut i = 1;
    while i < args.len() {
        if args[i] == "--port" && i + 1 < args.len() {
            return args[i + 1].parse().unwrap_or_else(|_| {
                eprintln!("错误: 无效端口号 '{}'", args[i + 1]);
                std::process::exit(1);
            });
        }
        if args[i].starts_with("--port=") {
            return args[i][7..].parse().unwrap_or_else(|_| {
                eprintln!("错误: 无效端口号"); std::process::exit(1);
            });
        }
        i += 1;
    }
    std::env::var("ZCPP_PORT").ok().and_then(|v| v.parse().ok()).unwrap_or(3000)
}

fn check_compilers(settings: &models::Settings) {
    for (name, _, available) in compile::get_available_compilers_with_settings(settings) {
        if available { info!("✓ {} 可用", name); }
        else { info!("✗ {} 不可用（如需使用请安装或配置路径）", name); }
    }
}

// ── 获取 State 辅助宏 ─────────────────────────────────
macro_rules! get_settings {
    ($ext:expr) => { $ext.settings.lock().unwrap().clone() };
}

// ── Handlers ──────────────────────────────────────────

async fn handle_health(Extension(state): Extension<Arc<AppState>>) -> Json<models::HealthResponse> {
    let settings = get_settings!(state);
    let compilers = compile::get_available_compilers_with_settings(&settings);
    let gcc_avail = compilers.iter().any(|(n, _, a)| n == "GCC" && *a);
    let clang_avail = compilers.iter().any(|(n, _, a)| n == "Clang" && *a);
    Json(models::HealthResponse { status: "ok".to_string(), version: "0.1.0".to_string(), gcc_available: gcc_avail, clang_available: clang_avail })
}

async fn handle_compile(
    Extension(state): Extension<Arc<AppState>>,
    Json(req): Json<models::CompileRequest>,
) -> Json<models::CompileResponse> {
    let settings = get_settings!(state);
    info!("编译: compiler={}, file={}", req.compiler, req.filename);
    let result = compile::compile_and_run(req, &settings).await;
    Json(result)
}

async fn handle_languages(Extension(state): Extension<Arc<AppState>>) -> Json<Vec<models::LanguageInfo>> {
    let settings = get_settings!(state);
    let compilers = compile::get_available_compilers_with_settings(&settings);
    let gcc = compilers.iter().any(|(n, _, a)| n == "GCC" && *a);
    let clang = compilers.iter().any(|(n, _, a)| n == "Clang" && *a);
    Json(vec![
        models::LanguageInfo { name: "C".into(), extension: ".c".into(), compilers: vec![
            models::CompilerInfo { name: "GCC".into(), command: "gcc".into(), available: gcc },
            models::CompilerInfo { name: "Clang".into(), command: "clang".into(), available: clang },
        ]},
        models::LanguageInfo { name: "C++".into(), extension: ".cpp".into(), compilers: vec![
            models::CompilerInfo { name: "GCC".into(), command: "g++".into(), available: gcc },
            models::CompilerInfo { name: "Clang".into(), command: "clang++".into(), available: clang },
        ]},
    ])
}

async fn handle_compilers(Extension(state): Extension<Arc<AppState>>) -> Json<Vec<models::CompilerInfo>> {
    let settings = get_settings!(state);
    let compilers = compile::get_available_compilers_with_settings(&settings);
    Json(compilers.into_iter().map(|(n, c, a)| models::CompilerInfo { name: n, command: c, available: a }).collect())
}

async fn handle_list_files(Extension(state): Extension<Arc<AppState>>) -> Json<models::FileListResponse> {
    let settings = get_settings!(state);
    let ws = if settings.workspace.is_empty() { compile::workspace_dir_override() } else { settings.workspace.clone() };
    let ws_path = std::path::PathBuf::from(&ws);
    tokio::fs::create_dir_all(&ws_path).await.unwrap_or_default();

    let mut files = Vec::new();
    if let Ok(entries) = std::fs::read_dir(&ws_path) {
        for entry in entries.flatten() {
            let path = entry.path();
            if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                if name.ends_with(".exe") || name.ends_with(".out") || name.ends_with(".o") { continue; }
                let meta = entry.metadata().ok();
                files.push(models::FileInfo {
                    name: name.to_string(),
                    path: name.to_string(),
                    size: meta.as_ref().map(|m| m.len()).unwrap_or(0),
                    is_dir: meta.map(|m| m.is_dir()).unwrap_or(false),
                });
            }
        }
    }
    files.sort_by(|a, b| a.name.cmp(&b.name));
    Json(models::FileListResponse { files, workspace: ws })
}

async fn handle_create_file(
    Extension(state): Extension<Arc<AppState>>,
    Json(req): Json<models::CreateFileRequest>,
) -> Json<models::CreateFileResponse> {
    let settings = get_settings!(state);
    let ws = if settings.workspace.is_empty() { compile::workspace_dir_override() } else { settings.workspace.clone() };
    if req.filename.contains("..") || req.filename.contains('/') || req.filename.contains('\\') {
        return Json(models::CreateFileResponse { success: false, message: "文件名不能包含路径分隔符".into() });
    }
    let path = std::path::PathBuf::from(&ws).join(&req.filename);
    match tokio::fs::write(&path, &req.content).await {
        Ok(_) => Json(models::CreateFileResponse { success: true, message: format!("文件 '{}' 创建成功", req.filename) }),
        Err(e) => Json(models::CreateFileResponse { success: false, message: format!("创建失败: {}", e) }),
    }
}

async fn handle_save_file(
    Extension(state): Extension<Arc<AppState>>,
    Json(req): Json<models::SaveFileRequest>,
) -> Json<serde_json::Value> {
    let settings = get_settings!(state);
    let ws = if settings.workspace.is_empty() { compile::workspace_dir_override() } else { settings.workspace.clone() };
    let path = std::path::PathBuf::from(&ws).join(&req.filename);
    match tokio::fs::write(&path, &req.content).await {
        Ok(_) => Json(serde_json::json!({"success": true, "message": "文件保存成功"})),
        Err(e) => Json(serde_json::json!({"success": false, "message": format!("保存失败: {}", e)})),
    }
}

async fn handle_load_file(
    Extension(state): Extension<Arc<AppState>>,
    axum::extract::Path(filename): axum::extract::Path<String>,
) -> Json<serde_json::Value> {
    let settings = get_settings!(state);
    let ws = if settings.workspace.is_empty() { compile::workspace_dir_override() } else { settings.workspace.clone() };
    let path = std::path::PathBuf::from(&ws).join(&filename);
    match tokio::fs::read_to_string(&path).await {
        Ok(content) => Json(serde_json::json!({"success": true, "content": content})),
        Err(e) => Json(serde_json::json!({"success": false, "message": format!("读取失败: {}", e)})),
    }
}

async fn handle_get_settings(Extension(state): Extension<Arc<AppState>>) -> Json<models::SettingsResponse> {
    let settings = get_settings!(state);
    Json(models::SettingsResponse { settings })
}

async fn handle_save_settings(
    Extension(state): Extension<Arc<AppState>>,
    Json(req): Json<models::SaveSettingsRequest>,
) -> Json<serde_json::Value> {
    match compile::save_settings(&req.settings) {
        Ok(_) => {
            *state.settings.lock().unwrap() = req.settings.clone();
            check_compilers(&req.settings);
            Json(serde_json::json!({"success": true, "message": "设置已保存"}))
        }
        Err(e) => Json(serde_json::json!({"success": false, "message": format!("保存失败: {}", e)})),
    }
}
