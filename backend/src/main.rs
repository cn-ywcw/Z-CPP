/// Z-CPP 后端 — 入口
///
/// 启动 HTTP 服务，提供编译、运行、文件管理等 API。
/// 生产模式下同时 serve 前端静态文件。

mod compile;
mod models;

use axum::{
    extract::State,
    http::Method,
    routing::{get, post},
    Json, Router,
};
use std::net::SocketAddr;
use std::sync::Arc;
use tower_http::cors::{Any, CorsLayer};
use tower_http::services::ServeDir;
use tracing::info;

/// 应用状态
struct AppState {
    // 可在此处添加运行时配置
}

#[tokio::main]
async fn main() {
    // 初始化日志
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "z_cpp_backend=info".into()),
        )
        .init();

    // 检测模式：ZCPP_MODE=production 或 --prod 参数
    let production = std::env::var("ZCPP_MODE").unwrap_or_default() == "production"
        || std::env::args().any(|a| a == "--prod");

    info!(
        "Z-CPP 后端服务启动中... (mode: {})",
        if production { "production" } else { "development" }
    );

    // 检查编译器可用性
    check_compilers();

    // 共享状态
    let state = Arc::new(AppState {});

    // API 路由
    let api_routes = Router::new()
        .route("/api/health", get(health_check))
        .route("/api/compile", post(handle_compile))
        .route("/api/languages", get(handle_languages))
        .route("/api/compilers", get(handle_compilers))
        .route("/api/save", post(handle_save_file))
        .route("/api/load/{filename}", get(handle_load_file));

    let app = if production {
        // 生产模式：serve 前端静态文件，API 路由优先
        let static_dir = if cfg!(target_os = "windows") {
            ".\\frontend\\dist"
        } else {
            "./frontend/dist"
        };
        let dist_path = std::path::Path::new(static_dir);

        if !dist_path.exists() {
            info!(
                "静态文件目录不存在: {}，将只提供 API 服务",
                dist_path.display()
            );
        } else {
            info!("静态文件目录: {}", dist_path.canonicalize().unwrap_or_else(|_| dist_path.to_path_buf()).display());
        }

        api_routes
            .fallback_service(
                ServeDir::new(static_dir)
                    .append_index_html_on_directories(true),
            )
            .with_state(state)
    } else {
        // 开发模式：仅 API，允许 CORS（前端 dev server 代理 API）
        let cors = CorsLayer::new()
            .allow_origin(Any)
            .allow_methods([Method::GET, Method::POST, Method::OPTIONS])
            .allow_headers(Any);

        api_routes.layer(cors).with_state(state)
    };

    // 解析端口
    let port = parse_port();

    // 监听地址
    let host = if production { "0.0.0.0" } else { "127.0.0.1" };
    let addr = SocketAddr::new(
        host.parse().unwrap(),
        port,
    );

    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .unwrap_or_else(|_| {
            panic!("无法绑定地址 {}", addr)
        });

    info!("后端服务已启动: http://{}:{}", host, port);

    axum::serve(listener, app)
        .await
        .expect("服务器启动失败");
}

/// 从命令行参数解析端口号
/// 支持: --port 8080 或 --port=8080
fn parse_port() -> u16 {
    let args: Vec<String> = std::env::args().collect();
    let mut i = 1;
    while i < args.len() {
        if args[i] == "--port" {
            if i + 1 < args.len() {
                return args[i + 1].parse().unwrap_or_else(|_| {
                    eprintln!("错误: 无效端口号 '{}'", args[i + 1]);
                    std::process::exit(1);
                });
            }
        } else if args[i].starts_with("--port=") {
            let val = &args[i][7..];
            return val.parse().unwrap_or_else(|_| {
                eprintln!("错误: 无效端口号 '{}'", val);
                std::process::exit(1);
            });
        }
        i += 1;
    }
    // 环境变量 ZCPP_PORT 覆盖默认值
    std::env::var("ZCPP_PORT")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(3000)
}

/// 检查并打印编译器状态
fn check_compilers() {
    for (name, _, available) in compile::get_available_compilers() {
        if available {
            info!("✓ {} 可用", name);
        } else {
            info!("✗ {} 不可用（如需使用请安装）", name);
        }
    }
}

/// 健康检查
async fn health_check() -> Json<models::HealthResponse> {
    let (_, _, gcc_avail) = compile::get_available_compilers()
        .into_iter()
        .find(|(n, _, _)| n == "GCC")
        .unwrap_or_default();

    let (_, _, clang_avail) = compile::get_available_compilers()
        .into_iter()
        .find(|(n, _, _)| n == "Clang")
        .unwrap_or_default();

    Json(models::HealthResponse {
        status: "ok".to_string(),
        version: "0.1.0".to_string(),
        gcc_available: gcc_avail,
        clang_available: clang_avail,
    })
}

/// 处理编译请求
async fn handle_compile(
    State(_state): State<Arc<AppState>>,
    Json(req): Json<models::CompileRequest>,
) -> Json<models::CompileResponse> {
    info!("收到编译请求: compiler={}, file={}", req.compiler, req.filename);
    let result = compile::compile_and_run(req).await;
    info!("编译完成: success={}", result.success);
    Json(result)
}

/// 获取支持的语言列表
async fn handle_languages() -> Json<Vec<models::LanguageInfo>> {
    Json(vec![
        models::LanguageInfo {
            name: "C".to_string(),
            extension: ".c".to_string(),
            compilers: vec![
                models::CompilerInfo {
                    name: "GCC".to_string(),
                    command: "gcc".to_string(),
                    available: compile::get_available_compilers()
                        .iter()
                        .any(|(n, _, a)| n == "GCC" && *a),
                },
                models::CompilerInfo {
                    name: "Clang".to_string(),
                    command: "clang".to_string(),
                    available: compile::get_available_compilers()
                        .iter()
                        .any(|(n, _, a)| n == "Clang" && *a),
                },
            ],
        },
        models::LanguageInfo {
            name: "C++".to_string(),
            extension: ".cpp".to_string(),
            compilers: vec![
                models::CompilerInfo {
                    name: "GCC".to_string(),
                    command: "g++".to_string(),
                    available: compile::get_available_compilers()
                        .iter()
                        .any(|(n, _, a)| n == "GCC" && *a),
                },
                models::CompilerInfo {
                    name: "Clang".to_string(),
                    command: "clang++".to_string(),
                    available: compile::get_available_compilers()
                        .iter()
                        .any(|(n, _, a)| n == "Clang" && *a),
                },
            ],
        },
    ])
}

/// 获取可用的编译器列表
async fn handle_compilers() -> Json<Vec<models::CompilerInfo>> {
    let compilers = compile::get_available_compilers();
    Json(
        compilers
            .into_iter()
            .map(|(name, cmd, available)| models::CompilerInfo {
                name,
                command: cmd,
                available,
            })
            .collect(),
    )
}

/// 保存文件
async fn handle_save_file(
    Json(req): Json<models::SaveFileRequest>,
) -> Json<serde_json::Value> {
    let workspace_path = std::path::PathBuf::from(compile::workspace_dir_override());
    let file_path = workspace_path.join(&req.filename);

    match tokio::fs::write(&file_path, &req.content).await {
        Ok(_) => Json(serde_json::json!({"success": true, "message": "文件保存成功"})),
        Err(e) => Json(serde_json::json!({"success": false, "message": format!("保存失败: {}", e)})),
    }
}

/// 加载文件
async fn handle_load_file(
    axum::extract::Path(filename): axum::extract::Path<String>,
) -> Json<serde_json::Value> {
    let workspace_path = std::path::PathBuf::from(compile::workspace_dir_override()).join(&filename);

    match tokio::fs::read_to_string(&workspace_path).await {
        Ok(content) => Json(serde_json::json!({"success": true, "content": content})),
        Err(e) => Json(serde_json::json!({"success": false, "message": format!("读取失败: {}", e)})),
    }
}
