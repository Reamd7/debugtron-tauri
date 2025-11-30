// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod devtools_server;
mod state;
mod targets;

use state::AppState;
use tauri::Manager;

fn main() {
    tauri::Builder::default()
        .setup(|app| {
            // Clean up DevTools temp directory on startup
            println!("[MAIN] Cleaning up DevTools temp directory...");
            if let Err(e) = devtools_server::cleanup_devtools_temp() {
                eprintln!("[MAIN] Failed to clean DevTools temp: {}", e);
            }

            // Initialize app state
            let app_handle = app.handle();
            let state = AppState::new(app_handle.clone());

            // Store state in app
            app.manage(state.clone());

            // Initialize and discover apps
            tauri::async_runtime::spawn(async move {
                println!("[MAIN] Starting app initialization...");
                match state.initialize().await {
                    Ok(_) => println!("[MAIN] App initialization completed successfully"),
                    Err(e) => eprintln!("[MAIN] Failed to initialize app state: {}", e),
                }
            });

            Ok(())
        })
        .on_window_event(|event| {
            // Handle file drop events
            if let tauri::WindowEvent::FileDrop(dropped) = event.event() {
                println!("[MAIN] FileDropEvent: {:?}", dropped);
            }

            // Handle window destroyed event
            if let tauri::WindowEvent::Destroyed = event.event() {
                // Only clean up when the main window is destroyed
                if event.window().label() == "main" {
                    println!("[MAIN] Main window destroyed, cleaning up DevTools temp...");
                    if let Err(e) = devtools_server::cleanup_devtools_temp() {
                        eprintln!("[MAIN] Failed to clean DevTools temp on exit: {}", e);
                    }
                } else {
                    println!("[MAIN] DevTools window '{}' closed", event.window().label());
                }
            }
        })
        .invoke_handler(tauri::generate_handler![
            commands::debug,
            commands::debug_path,
            commands::add_remote_device,
            commands::remove_device,
            commands::refresh_device_apps,
            commands::open_devtools,
            commands::open_devtools_window,
            commands::get_targets,
            commands::get_apps,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
