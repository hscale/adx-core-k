// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use tauri::Manager;

// Custom commands for Tauri
#[tauri::command]
fn get_platform_info() -> serde_json::Value {
    serde_json::json!({
        "platform": std::env::consts::OS,
        "arch": std::env::consts::ARCH,
        "version": env!("CARGO_PKG_VERSION"),
        "is_dev": cfg!(debug_assertions)
    })
}

#[tauri::command]
async fn show_notification(title: String, body: String) -> Result<(), String> {
    // This would integrate with the notification plugin
    println!("Notification: {} - {}", title, body);
    Ok(())
}

#[tauri::command]
async fn save_file(path: String, contents: String) -> Result<(), String> {
    std::fs::write(&path, contents)
        .map_err(|e| format!("Failed to save file: {}", e))?;
    Ok(())
}

#[tauri::command]
async fn read_file(path: String) -> Result<String, String> {
    std::fs::read_to_string(&path)
        .map_err(|e| format!("Failed to read file: {}", e))
}

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_window_state::Builder::default().build())
        .invoke_handler(tauri::generate_handler![
            get_platform_info,
            show_notification,
            save_file,
            read_file
        ])
        .setup(|app| {
            // Set up the application
            let window = app.get_webview_window("main").unwrap();
            
            // Configure window
            #[cfg(debug_assertions)]
            {
                window.open_devtools();
            }
            
            // Set minimum size
            let _ = window.set_min_size(Some(tauri::LogicalSize::new(800.0, 600.0)));
            
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}