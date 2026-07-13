mod compile;
mod models;

use std::sync::Mutex;
use tauri::State;

pub struct AppState {
    settings: Mutex<models::Settings>,
}



#[tauri::command]
fn check_health(state: State<AppState>) -> Result<models::HealthResponse, String> {
    let settings = state.settings.lock().map_err(|e| e.to_string())?.clone();
    let compilers = compile::get_available_compilers_with_settings(&settings);
    let gcc_avail = compilers.iter().any(|(n, _, a)| n == "GCC" && *a);
    let clang_avail = compilers.iter().any(|(n, _, a)| n == "Clang" && *a);
    Ok(models::HealthResponse {
        status: "ok".to_string(),
        version: "0.1.0".to_string(),
        gcc_available: gcc_avail,
        clang_available: clang_avail,
    })
}

#[tauri::command]
async fn compile_code(
    state: State<'_, AppState>,
    req: models::CompileRequest,
) -> Result<models::CompileResponse, String> {
    let settings = state.settings.lock().map_err(|e| e.to_string())?.clone();
    Ok(compile::compile_and_run(req, &settings).await)
}

#[tauri::command]
fn list_files(state: State<AppState>, subdir: Option<String>) -> Result<models::FileListResponse, String> {
    let settings = state.settings.lock().map_err(|e| e.to_string())?.clone();
    let ws = if settings.workspace.is_empty() {
        compile::workspace_dir_override()
    } else {
        settings.workspace.clone()
    };
    let mut ws_path = std::path::PathBuf::from(&ws);
    if let Some(ref sub) = subdir {
        if !sub.is_empty() {
            // 安全检查：不允许 .. 逃逸工作目录
            let clean = sub.replace('\\', "/");
            for part in clean.split('/') {
                if part == ".." || part.is_empty() { continue; }
                ws_path = ws_path.join(part);
            }
        }
    }
    std::fs::create_dir_all(&ws_path).map_err(|e| e.to_string())?;

    let mut files = Vec::new();
    if let Ok(entries) = std::fs::read_dir(&ws_path) {
        for entry in entries.flatten() {
            let path = entry.path();
            if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                if name.starts_with('.')
                    || name.ends_with(".exe")
                    || name.ends_with(".out")
                    || name.ends_with(".o")
                    || name.ends_with(".obj")
                    || name.ends_with(".pdb")
                    || name.ends_with(".ilk")
                    || name.ends_with(".swp")
                    || name.ends_with('~')
                {
                    continue;
                }
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
    Ok(models::FileListResponse { files, workspace: ws })
}

#[tauri::command]
fn create_file(
    state: State<AppState>,
    req: models::CreateFileRequest,
) -> Result<models::CreateFileResponse, String> {
    let settings = state.settings.lock().map_err(|e| e.to_string())?.clone();
    let path = resolve_ws_path(&settings, &req.filename);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).ok();
    }
    match std::fs::write(&path, &req.content) {
        Ok(_) => Ok(models::CreateFileResponse {
            success: true,
            message: format!("文件 '{}' 创建成功", req.filename),
        }),
        Err(e) => Ok(models::CreateFileResponse {
            success: false,
            message: format!("创建失败: {}", e),
        }),
    }
}

#[tauri::command]
fn save_file(
    state: State<AppState>,
    req: models::SaveFileRequest,
) -> Result<serde_json::Value, String> {
    let settings = state.settings.lock().map_err(|e| e.to_string())?.clone();
    let path = resolve_ws_path(&settings, &req.filename);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).ok();
    }
    match std::fs::write(&path, &req.content) {
        Ok(_) => Ok(serde_json::json!({"success": true, "message": "文件保存成功"})),
        Err(e) => Ok(serde_json::json!({"success": false, "message": format!("保存失败: {}", e)})),
    }
}

#[tauri::command]
fn load_file(
    state: State<AppState>,
    filename: String,
) -> Result<serde_json::Value, String> {
    let settings = state.settings.lock().map_err(|e| e.to_string())?.clone();
    let path = resolve_ws_path(&settings, &filename);
    match std::fs::read_to_string(&path) {
        Ok(content) => Ok(serde_json::json!({"success": true, "content": content})),
        Err(e) => Ok(serde_json::json!({"success": false, "message": format!("读取失败: {}", e)})),
    }
}

#[tauri::command]
fn get_settings(state: State<AppState>) -> Result<models::SettingsResponse, String> {
    let settings = state.settings.lock().map_err(|e| e.to_string())?.clone();
    Ok(models::SettingsResponse { settings })
}

#[tauri::command]
fn save_settings(
    state: State<AppState>,
    req: models::SaveSettingsRequest,
) -> Result<serde_json::Value, String> {
    compile::save_settings(&req.settings)?;
    {
        let mut s = state.settings.lock().map_err(|e| e.to_string())?;
        *s = req.settings.clone();
    }
    Ok(serde_json::json!({"success": true, "message": "设置已保存"}))
}

#[tauri::command]
fn get_languages(state: State<AppState>) -> Result<Vec<models::LanguageInfo>, String> {
    let settings = state.settings.lock().map_err(|e| e.to_string())?.clone();
    let compilers = compile::get_available_compilers_with_settings(&settings);
    let gcc = compilers.iter().any(|(n, _, a)| n == "GCC" && *a);
    let clang = compilers.iter().any(|(n, _, a)| n == "Clang" && *a);
    Ok(vec![
        models::LanguageInfo {
            name: "C".into(),
            extension: ".c".into(),
            compilers: vec![
                models::CompilerInfo { name: "GCC".into(), command: "gcc".into(), available: gcc },
                models::CompilerInfo { name: "Clang".into(), command: "clang".into(), available: clang },
            ],
        },
        models::LanguageInfo {
            name: "C++".into(),
            extension: ".cpp".into(),
            compilers: vec![
                models::CompilerInfo { name: "GCC".into(), command: "g++".into(), available: gcc },
                models::CompilerInfo { name: "Clang".into(), command: "clang++".into(), available: clang },
            ],
        },
    ])
}

#[tauri::command]
fn get_compilers(state: State<AppState>) -> Result<Vec<models::CompilerInfo>, String> {
    let settings = state.settings.lock().map_err(|e| e.to_string())?.clone();
    let compilers = compile::get_available_compilers_with_settings(&settings);
    Ok(compilers
        .into_iter()
        .map(|(n, c, a)| models::CompilerInfo {
            name: n,
            command: c,
            available: a,
        })
        .collect())
}

fn resolve_ws_path(settings: &models::Settings, path: &str) -> std::path::PathBuf {
    let ws = if settings.workspace.is_empty() {
        compile::workspace_dir_override()
    } else {
        settings.workspace.clone()
    };
    let mut ws_path = std::path::PathBuf::from(&ws);
    let clean = path.replace('\\', "/");
    for part in clean.split('/') {
        if part == ".." || part.is_empty() { continue; }
        ws_path = ws_path.join(part);
    }
    ws_path
}

#[tauri::command]
fn create_dir(
    state: State<AppState>,
    req: models::CreateDirRequest,
) -> Result<models::FileOpResponse, String> {
    let settings = state.settings.lock().map_err(|e| e.to_string())?.clone();
    let path = resolve_ws_path(&settings, &req.name);
    match std::fs::create_dir_all(&path) {
        Ok(_) => Ok(models::FileOpResponse {
            success: true,
            message: format!("目录 '{}' 创建成功", req.name),
        }),
        Err(e) => Ok(models::FileOpResponse {
            success: false,
            message: format!("创建失败: {}", e),
        }),
    }
}

#[tauri::command]
fn delete_file(
    state: State<AppState>,
    req: models::FileOpRequest,
) -> Result<models::FileOpResponse, String> {
    let settings = state.settings.lock().map_err(|e| e.to_string())?.clone();
    let path = resolve_ws_path(&settings, &req.filename);
    if !path.exists() {
        return Ok(models::FileOpResponse {
            success: false,
            message: "文件不存在".into(),
        });
    }
    let result = if path.is_dir() {
        std::fs::remove_dir_all(&path)
    } else {
        std::fs::remove_file(&path)
    };
    match result {
        Ok(_) => Ok(models::FileOpResponse {
            success: true,
            message: format!("'{}' 已删除", req.filename),
        }),
        Err(e) => Ok(models::FileOpResponse {
            success: false,
            message: format!("删除失败: {}", e),
        }),
    }
}

#[tauri::command]
fn rename_file(
    state: State<AppState>,
    req: models::RenameRequest,
) -> Result<models::FileOpResponse, String> {
    let settings = state.settings.lock().map_err(|e| e.to_string())?.clone();
    let old = resolve_ws_path(&settings, &req.old_name);
    let new = resolve_ws_path(&settings, &req.new_name);
    match std::fs::rename(&old, &new) {
        Ok(_) => Ok(models::FileOpResponse {
            success: true,
            message: format!("已重命名为 '{}'", req.new_name),
        }),
        Err(e) => Ok(models::FileOpResponse {
            success: false,
            message: format!("重命名失败: {}", e),
        }),
    }
}

#[tauri::command]
fn copy_file(
    state: State<AppState>,
    req: models::CopyRequest,
) -> Result<models::FileOpResponse, String> {
    let settings = state.settings.lock().map_err(|e| e.to_string())?.clone();
    let src = resolve_ws_path(&settings, &req.source);
    let dst = resolve_ws_path(&settings, &req.dest);
    if !src.exists() {
        return Ok(models::FileOpResponse {
            success: false,
            message: "源文件不存在".into(),
        });
    }
    match std::fs::copy(&src, &dst) {
        Ok(_) => Ok(models::FileOpResponse {
            success: true,
            message: format!("已复制到 '{}'", req.dest),
        }),
        Err(e) => Ok(models::FileOpResponse {
            success: false,
            message: format!("复制失败: {}", e),
        }),
    }
}

fn session_path() -> std::path::PathBuf {
    let mut p = std::env::current_exe().unwrap_or_default();
    p.pop();
    p.push("z-cpp-session.json");
    p
}

#[tauri::command]
fn save_session(req: models::SaveSessionRequest) -> Result<(), String> {
    let path = session_path();
    let json = serde_json::to_string_pretty(&req.session).map_err(|e| e.to_string())?;
    std::fs::write(path, json).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
fn load_session() -> Result<models::SessionData, String> {
    let path = session_path();
    if !path.exists() {
        return Ok(models::SessionData { tabs: vec![], active_tab: 0 });
    }
    let json = std::fs::read_to_string(path).map_err(|e| e.to_string())?;
    serde_json::from_str(&json).map_err(|e| e.to_string())
}

#[tauri::command]
fn get_system_fonts() -> Vec<String> {
    use font_kit::source::SystemSource;
    let mut fonts: Vec<String> = Vec::new();
    let source = SystemSource::new();
    if let Ok(handles) = source.all_fonts() {
        for handle in handles {
            if let Ok(font) = handle.load() {
                let family = font.family_name().to_string();
                if !family.is_empty() && !fonts.contains(&family) {
                    fonts.push(family);
                }
            }
        }
    }
    fonts.sort();
    fonts
}

#[tauri::command]
fn get_app_meta() -> models::AppMeta {
    models::AppMeta {
        version: env!("CARGO_PKG_VERSION").to_string(),
        license: env!("CARGO_PKG_LICENSE").to_string(),
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let settings = compile::load_settings();
    let state = AppState {
        settings: Mutex::new(settings),
    };
    // 后台检测编译器，不阻塞窗口启动
    let s = state.settings.lock().unwrap().clone();
    std::thread::spawn(move || {
        for (name, _, available) in compile::get_available_compilers_with_settings(&s) {
            if available { println!("✓ {} 可用", name); }
            else { println!("✗ {} 不可用", name); }
        }
    });

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .manage(state)
        .invoke_handler(tauri::generate_handler![
            check_health,
            compile_code,
            list_files,
            create_file,
            save_file,
            load_file,
            get_settings,
            save_settings,
            get_languages,
            get_compilers,
            get_app_meta,
            get_system_fonts,
            create_dir,
            delete_file,
            rename_file,
            copy_file,
            save_session,
            load_session,
        ])
        .run(tauri::generate_context!())
        .expect("启动 Z-CPP 失败");
}
