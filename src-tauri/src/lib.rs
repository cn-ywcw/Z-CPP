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
async fn run_testcases(
    state: State<'_, AppState>,
    req: models::TestCasesRequest,
) -> Result<models::TestCasesResponse, String> {
    let settings = state.settings.lock().map_err(|e| e.to_string())?.clone();
    let opts = req.compile_options.clone().unwrap_or_else(|| settings.default_options.clone());

    let output_path = match compile::compile_program(
        &req.code,
        &req.filename,
        &req.compiler,
        &settings,
        &opts,
        &req.options,
        &req.std,
    )
    .await
    {
        Ok(p) => p,
        Err(e) => {
            return Ok(models::TestCasesResponse {
                success: false,
                compile_output: format!("编译失败:\n{}", e),
                results: vec![],
            })
        }
    };

    if req.compile_only {
        return Ok(models::TestCasesResponse {
            success: true,
            compile_output: String::new(),
            results: vec![],
        });
    }

    let mut results = Vec::with_capacity(req.testcases.len());
    for (i, tc) in req.testcases.iter().enumerate() {
        let (out, err, code, ms) = compile::run_capture(&output_path, &tc.input);
        let output = if err.is_empty() {
            out.clone()
        } else {
            format!("{}\n{}", out, err)
        };
        let passed = tc.expected.as_ref().map(|exp| {
            compile::normalize_output(&output) == compile::normalize_output(exp)
        });
        results.push(models::TestCaseResult {
            index: i,
            output,
            exit_code: code,
            time_ms: ms,
            passed,
        });
    }

    Ok(models::TestCasesResponse {
        success: true,
        compile_output: String::new(),
        results,
    })
}

#[tauri::command]
async fn stress_test(
    state: State<'_, AppState>,
    req: models::StressRequest,
) -> Result<models::StressResponse, String> {
    let settings = state.settings.lock().map_err(|e| e.to_string())?.clone();
    let opts = req.compile_options.clone().unwrap_or_else(|| settings.default_options.clone());

    let gen_path = match compile::compile_program(
        &req.generator_code, &req.generator_filename, &req.compiler, &settings, &opts, &req.options, &req.std,
    )
    .await
    {
        Ok(p) => p,
        Err(e) => {
            return Ok(models::StressResponse {
                found: false,
                iterations: 0,
                compile_error: Some(format!("生成器编译失败:\n{}", e)),
                runtime_error: None,
                counterexample_input: None,
                solution_output: None,
                reference_output: None,
                timed_out: false,
            })
        }
    };
    let sol_path = match compile::compile_program(
        &req.solution_code, &req.solution_filename, &req.compiler, &settings, &opts, &req.options, &req.std,
    )
    .await
    {
        Ok(p) => p,
        Err(e) => {
            return Ok(models::StressResponse {
                found: false,
                iterations: 0,
                compile_error: Some(format!("被测程序编译失败:\n{}", e)),
                runtime_error: None,
                counterexample_input: None,
                solution_output: None,
                reference_output: None,
                timed_out: false,
            })
        }
    };
    let ref_path = match compile::compile_program(
        &req.reference_code, &req.reference_filename, &req.compiler, &settings, &opts, &req.options, &req.std,
    )
    .await
    {
        Ok(p) => p,
        Err(e) => {
            return Ok(models::StressResponse {
                found: false,
                iterations: 0,
                compile_error: Some(format!("参考解编译失败:\n{}", e)),
                runtime_error: None,
                counterexample_input: None,
                solution_output: None,
                reference_output: None,
                timed_out: false,
            })
        }
    };

    let mut found = false;
    let mut iterations: u32 = 0;
    let mut timed_out = false;
    let mut runtime_error: Option<String> = None;
    let mut counterexample: Option<String> = None;
    let mut sol_out: Option<String> = None;
    let mut ref_out: Option<String> = None;

    for _ in 0..req.iterations.max(1) {
        iterations += 1;
        let (gen_out, gen_err, gen_code, _, gen_to) =
            compile::run_capture_timeout(&gen_path, "", req.timeout_ms).await;
        if gen_to {
            found = true;
            timed_out = true;
            runtime_error = Some(format!("生成器运行超时（>{} ms）", req.timeout_ms));
            break;
        }
        if !gen_err.is_empty() {
            runtime_error = Some(format!("生成器 stderr:\n{}", gen_err));
        }
        if gen_code != Some(0) {
            runtime_error = Some(format!("生成器运行失败（退出码 {:?}），输出:\n{}", gen_code, gen_out));
            break;
        }
        let input = gen_out;

        let (sol_o, sol_e, sol_c, _, sol_to) =
            compile::run_capture_timeout(&sol_path, &input, req.timeout_ms).await;
        let (ref_o, ref_e, ref_c, _, ref_to) =
            compile::run_capture_timeout(&ref_path, &input, req.timeout_ms).await;

        if sol_to || ref_to {
            found = true;
            timed_out = true;
            runtime_error = Some(format!(
                "运行超时（被测 {} / 参考 {}，限制 {} ms）",
                if sol_to { "超时" } else { "正常" },
                if ref_to { "超时" } else { "正常" },
                req.timeout_ms
            ));
            counterexample = Some(input);
            sol_out = Some(sol_o);
            ref_out = Some(ref_o);
            break;
        }

        if sol_c != Some(0) || ref_c != Some(0) {
            found = true;
            runtime_error = Some(format!(
                "运行异常（被测退出码 {:?}，参考退出码 {:?}）\n被测 stderr: {}\n参考 stderr: {}",
                sol_c, ref_c, sol_e, ref_e
            ));
            counterexample = Some(input);
            sol_out = Some(sol_o);
            ref_out = Some(ref_o);
            break;
        }

        if compile::normalize_output(&sol_o) != compile::normalize_output(&ref_o) {
            found = true;
            counterexample = Some(input);
            sol_out = Some(sol_o);
            ref_out = Some(ref_o);
            break;
        }
    }

    Ok(models::StressResponse {
        found,
        iterations,
        compile_error: None,
        runtime_error,
        counterexample_input: counterexample,
        solution_output: sol_out,
        reference_output: ref_out,
        timed_out,
    })
}

#[tauri::command]
fn save_testcases(filename: String, cases: Vec<models::TestCase>) -> Result<(), String> {
    compile::save_testcases(&filename, &cases)
}

#[tauri::command]
fn load_testcases(filename: String) -> Result<Vec<models::TestCase>, String> {
    Ok(compile::load_testcases(&filename))
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
            run_testcases,
            stress_test,
            save_testcases,
            load_testcases,
        ])
        .run(tauri::generate_context!())
        .expect("启动 Z-CPP 失败");
}
