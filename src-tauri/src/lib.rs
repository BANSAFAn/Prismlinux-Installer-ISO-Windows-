mod commands;

use tauri::Emitter;
use commands::{get_usb_drives, download_iso, flash_usb_drive, setup_direct_boot, select_local_iso, reboot_system, exit_app};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                api.prevent_close();
                let _ = window.emit("close-requested", ());
            }
        })
        .invoke_handler(tauri::generate_handler![
            get_usb_drives,
            download_iso,
            flash_usb_drive,
            setup_direct_boot,
            select_local_iso,
            reboot_system,
            exit_app
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
