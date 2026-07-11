mod compile;
mod models;

use std::sync::Mutex;
use tauri::State;

pub struct AppState {
    settings: Mutex<models::Settings>,
}

fn check_compilers(state: &AppState) {
    let settings = match state.settings.lock() {
        Ok(s) => s,
        Err(e) => {
            eprintln!("获取设置锁失败: {}", e);
            return;
        }
    };
    for (name, _, available) in compile::get_available_compilers_with_settings(&settings) {
        if available {
            println!("✓ {} 可用", name);
        } else {
            println!("✗ {} 不可用（如需使用请安装或配置路径）", name);
        }
    }
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
fn list_files(state: State<AppState>) -> Result<models::FileListResponse, String> {
    let settings = state.settings.lock().map_err(|e| e.to_string())?.clone();
    let ws = if settings.workspace.is_empty() {
        compile::workspace_dir_override()
    } else {
        settings.workspace.clone()
    };
    let ws_path = std::path::PathBuf::from(&ws);
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
    let ws = if settings.workspace.is_empty() {
        compile::workspace_dir_override()
    } else {
        settings.workspace.clone()
    };
    if req.filename.contains("..") || req.filename.contains('/') || req.filename.contains('\\') {
        return Ok(models::CreateFileResponse {
            success: false,
            message: "文件名不能包含路径分隔符".into(),
        });
    }
    let path = std::path::PathBuf::from(&ws).join(&req.filename);
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
    let ws = if settings.workspace.is_empty() {
        compile::workspace_dir_override()
    } else {
        settings.workspace.clone()
    };
    if req.filename.contains("..") || req.filename.contains('/') || req.filename.contains('\\') || req.filename.contains('\0') {
        return Ok(serde_json::json!({"success": false, "message": "文件名包含非法字符"}));
    }
    let path = std::path::PathBuf::from(&ws).join(&req.filename);
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
    let ws = if settings.workspace.is_empty() {
        compile::workspace_dir_override()
    } else {
        settings.workspace.clone()
    };
    if filename.contains("..") || filename.contains('/') || filename.contains('\\') || filename.contains('\0') {
        return Ok(serde_json::json!({"success": false, "message": "文件名包含非法字符"}));
    }
    let path = std::path::PathBuf::from(&ws).join(&filename);
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
    check_compilers(&state);
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
    check_compilers(&state);

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
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
        ])
        .run(tauri::generate_context!())
        .expect("启动 Z-CPP 失败");
}
