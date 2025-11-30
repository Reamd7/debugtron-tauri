use crate::state::AppState;
use crate::targets::types::{AppInfo, AppMetadata, RemoteDeviceOptions, TargetInfo, TargetType};
use tauri::{Manager, State};

#[tauri::command]
pub async fn debug(app_info: AppInfo, inspect_brk: Option<bool>, state: State<'_, AppState>) -> Result<(), String> {
    state
        .debug_app(&app_info, inspect_brk)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn debug_path(path: String, inspect_brk: Option<bool>, state: State<'_, AppState>) -> Result<(), String> {
    println!("[COMMAND] debug_path called with path: {}", path);

    use std::path::Path;

    // Create a copy of the path for the app name
    let path_copy = path.clone();

    // Handle macOS app bundles (.app directories)
    let actual_exe_path = if cfg!(target_os = "macos") && path.ends_with(".app") {
        // Check if it's a directory
        let app_bundle_path = Path::new(&path);
        if app_bundle_path.is_dir() {
            // Try to find the executable inside Contents/MacOS
            let app_name = app_bundle_path.file_name().and_then(|s| s.to_str()).unwrap_or("").replace(".app", "");
            let exe_candidate = app_bundle_path.join("Contents/MacOS").join(app_name);

            // Check if this executable exists
            if exe_candidate.exists() {
                println!("[COMMAND] Detected macOS app bundle, using executable: {}", exe_candidate.display());
                exe_candidate.to_string_lossy().to_string()
            } else {
                // Fallback: try to find any executable in Contents/MacOS
                let contents_macos_path = app_bundle_path.join("Contents/MacOS");
                let mut found_exe = None;

                if let Ok(dir_entries) = std::fs::read_dir(contents_macos_path) {
                    for entry in dir_entries.filter_map(|e| e.ok()) {
                        let entry_path = entry.path();
                        if entry_path.is_file() {
                            // Check if it's executable or not a dylib
                            if entry_path.extension().and_then(|s| s.to_str()) != Some("dylib") && entry_path.extension().and_then(|s| s.to_str()) != Some("framework") {
                                // Check for executable (platform-specific)
                                let is_executable = entry.metadata().map(|meta| {
                                    #[cfg(unix)]
                                    {
                                        use std::os::unix::fs::PermissionsExt;
                                        (meta.permissions().mode() & 0o111) != 0
                                    }
                                    #[cfg(windows)]
                                    {
                                        // On Windows, assume executable if it looks like an executable file
                                        entry_path.extension().map(|ext|
                                            matches!(ext.to_str(), Some("exe" | "cmd" | "bat"))
                                        ).unwrap_or(true)
                                    }
                                }).unwrap_or(true); // Assume executable if metadata can't be read

                                if is_executable {
                                    found_exe = Some(entry_path.to_string_lossy().to_string());
                                    break;
                                }
                            }
                        }
                    }
                }

                // If found an executable, use it; otherwise fall back to original path
                found_exe.unwrap_or(path)
            }
        } else {
            // Not a directory, use original path
            path
        }
    } else {
        // Not a macOS app bundle, use original path
        path
    };

    // Create an AppInfo struct for the path
    let app_info = AppInfo {
        id: format!("path-{}", uuid::Uuid::new_v4()), // Generate a unique ID
        name: Path::new(&path_copy)
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or(&path_copy)
            .to_string(),
        icon: String::new(), // Empty icon for path input
        exe_path: Some(actual_exe_path),
        target_id: format!("local-{}", std::env::consts::OS), // Local target ID
        target_type: TargetType::Local,
        metadata: Some(AppMetadata {
            platform: Some(std::env::consts::OS.to_string()),
            device_info: None,
            package_name: None,
        }),
    };

    println!("[COMMAND] Created AppInfo: {:?}", app_info);

    // Add the app to the global app state so frontend can access it
    let mut apps = state.apps.lock().await;
    apps.insert(app_info.id.clone(), app_info.clone());
    drop(apps);

    // Emit an event to notify frontend about the new app
    println!("[COMMAND] Emitting apps-updated event with new path app");
    let apps_list = state.apps.lock().await;
    let apps_vec: Vec<AppInfo> = apps_list.values().cloned().collect();
    let _ = state.app_handle.emit_all("apps-updated", &apps_vec);
    drop(apps_list);

    // Use the existing debug_app flow
    state
        .debug_app(&app_info, inspect_brk)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn add_remote_device(
    options: RemoteDeviceOptions,
    state: State<'_, AppState>,
) -> Result<(), String> {
    state
        .add_remote_device(options)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn remove_device(target_id: String, state: State<'_, AppState>) -> Result<(), String> {
    state
        .remove_device(&target_id)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn refresh_device_apps(
    target_id: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    state
        .refresh_apps(&target_id)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn open_devtools(url: String, app_handle: tauri::AppHandle) -> Result<(), String> {
    println!("[COMMAND] open_devtools called with URL: {}", url);

    // 启动或获取 DevTools 代理服务器
    let devtools_port = crate::devtools_server::get_or_start_server(&app_handle)
        .await
        .map_err(|e| format!("Failed to start DevTools server: {}", e))?;

    println!("[COMMAND] DevTools server running on port {}", devtools_port);

    // Extract WebSocket URL from the devtoolsFrontendUrl
    let ws_url = if let Some(ws_start) = url.find("ws=") {
        let ws_part = &url[ws_start + 3..];
        let ws_end = ws_part.find('&').unwrap_or(ws_part.len());
        ws_part[..ws_end].to_string()
    } else {
        return Err("No WebSocket URL found in devtoolsFrontendUrl".to_string());
    };

    println!("[COMMAND] Extracted WebSocket URL: {}", ws_url);

    // Determine the DevTools type based on the URL
    let devtools_page = if url.contains("js_app.html") || url.contains("v8only=true") {
        "js_app.html"
    } else {
        "inspector.html"
    };

    // 构建本地 DevTools URL (注意：DevTools 前端文件在 front_end/ 子目录下)
    let local_devtools_url = format!(
        "http://127.0.0.1:{}/front_end/{}?ws={}",
        devtools_port, devtools_page, ws_url
    );

    println!("[COMMAND] Opening DevTools at: {}", local_devtools_url);

    // 在系统默认浏览器中打开 DevTools
    // 由于 Tauri v1 的安全限制，无法直接在 Tauri 窗口中加载外部 HTTP URL
    // 使用系统浏览器可以避免这些限制，并提供完整的 DevTools 功能
    opener::open(&local_devtools_url)
        .map_err(|e| format!("Failed to open DevTools in browser: {}", e))?;

    println!("[COMMAND] DevTools opened in system browser");
    Ok(())
}

#[tauri::command]
pub async fn open_devtools_window(url: String, app_handle: tauri::AppHandle) -> Result<(), String> {
    println!("[COMMAND] open_devtools_window called with URL: {}", url);

    // 启动或获取 DevTools 代理服务器
    let devtools_port = crate::devtools_server::get_or_start_server(&app_handle)
        .await
        .map_err(|e| format!("Failed to start DevTools server: {}", e))?;

    println!("[COMMAND] DevTools server running on port {}", devtools_port);

    // Extract WebSocket URL from the devtoolsFrontendUrl
    let ws_url = if let Some(ws_start) = url.find("ws=") {
        let ws_part = &url[ws_start + 3..];
        let ws_end = ws_part.find('&').unwrap_or(ws_part.len());
        ws_part[..ws_end].to_string()
    } else {
        return Err("No WebSocket URL found in devtoolsFrontendUrl".to_string());
    };

    println!("[COMMAND] Extracted WebSocket URL: {}", ws_url);

    // Determine the DevTools type based on the URL
    let devtools_page = if url.contains("js_app.html") || url.contains("v8only=true") {
        "js_app.html"
    } else {
        "inspector.html"
    };

    // 构建本地 DevTools URL
    let local_devtools_url = format!(
        "http://127.0.0.1:{}/front_end/{}?ws={}",
        devtools_port, devtools_page, ws_url
    );

    println!("[COMMAND] Creating DevTools window with URL: {}", local_devtools_url);

    // 创建一个新的 Tauri 窗口来显示 DevTools
    // 窗口标签只能包含字母数字、-、/、: 和 _，需要替换所有其他字符
    let window_label = format!(
        "devtools-{}",
        ws_url
            .replace("://", "-")
            .replace("/", "-")
            .replace(":", "-")
            .replace(".", "_")  // 替换点号为下划线
    );

    // 检查窗口是否已经存在
    if let Some(existing_window) = app_handle.get_window(&window_label) {
        println!("[COMMAND] DevTools window already exists, focusing it");
        existing_window.set_focus().map_err(|e| e.to_string())?;
        return Ok(());
    }

    // 创建新窗口
    tauri::WindowBuilder::new(
        &app_handle,
        &window_label,
        tauri::WindowUrl::External(local_devtools_url.parse().map_err(|e| format!("Invalid URL: {}", e))?),
    )
    .title("Chrome DevTools")
    .inner_size(1200.0, 800.0)
    .resizable(true)
    .build()
    .map_err(|e| format!("Failed to create DevTools window: {}", e))?;

    println!("[COMMAND] DevTools window created successfully");
    Ok(())
}

#[tauri::command]
pub async fn get_targets(state: State<'_, AppState>) -> Result<Vec<TargetInfo>, String> {
    println!("[COMMAND] get_targets called");
    let registry = state.registry.lock().await;
    let targets = registry.get_all();

    let target_infos: Vec<TargetInfo> = targets
        .iter()
        .map(|target| TargetInfo {
            id: target.get_id(),
            r#type: match target.get_type() {
                crate::targets::types::TargetType::Local => "local",
                crate::targets::types::TargetType::Remote => "remote",
            }
            .to_string(),
            name: target.get_name(),
            status: "connected".to_string(),
            last_discovery: Some(chrono::Utc::now().timestamp_millis() as u64),
        })
        .collect();

    println!("[COMMAND] Returning {} targets", target_infos.len());
    Ok(target_infos)
}

#[tauri::command]
pub async fn get_apps(state: State<'_, AppState>) -> Result<Vec<AppInfo>, String> {
    println!("[COMMAND] get_apps called");
    let apps = state.apps.lock().await;
    let app_list: Vec<AppInfo> = apps.values().cloned().collect();
    println!("[COMMAND] Returning {} apps", app_list.len());
    Ok(app_list)
}
