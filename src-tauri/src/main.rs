mod connection;
mod native;
mod service;

use std::{os::unix::fs::MetadataExt, time::Duration};
use tauri::{WebviewUrl, WebviewWindowBuilder};

fn main() {
    let mut socket_identity = None;
    let app = tauri::Builder::default()
        // This process renders suggestions only. The CLI controls its lifecycle.
        .plugin(tauri_plugin_single_instance::init(|_, _, _| {}))
        .invoke_handler(tauri::generate_handler![service::get_completion])
        .setup(|app| {
            #[cfg(target_os = "macos")]
            app.set_activation_policy(tauri::ActivationPolicy::Accessory);
            let service = service::Service::new()?;
            use tauri::Manager;
            app.manage(service.clone());
            WebviewWindowBuilder::new(app, "popup", WebviewUrl::App("index.html".into()))
                .title("figfig suggestions")
                .inner_size(service::POPUP_WIDTH, service::popup_height(8))
                .decorations(false)
                .resizable(false)
                .always_on_top(true)
                .focused(false)
                .focusable(false)
                .skip_taskbar(true)
                .visible(false)
                .visible_on_all_workspaces(true)
                .build()?;
            let listener = connection::bind_socket()?;
            let handle = app.handle().clone();
            let server_service = service.clone();
            tauri::async_runtime::spawn(async move {
                if let Err(error) = connection::serve(listener, server_service, handle).await {
                    eprintln!("figfig: socket service failed: {error}");
                }
            });
            let handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                let mut interval = tokio::time::interval(Duration::from_millis(120));
                loop {
                    interval.tick().await;
                    let service = service.clone();
                    let callback_app = handle.clone();
                    if handle
                        .run_on_main_thread(move || service.track(&callback_app))
                        .is_err()
                    {
                        break;
                    }
                }
            });
            Ok(())
        })
        .build(tauri::generate_context!());
    match app {
        Ok(app) => {
            if let Ok(path) = figfig_core::runtime::socket_path() {
                if let Ok(meta) = std::fs::metadata(&path) {
                    socket_identity = Some((path, meta.ino()));
                }
            }
            app.run(|_, _| {});
        }
        Err(error) => {
            eprintln!("figfig: {error}");
            std::process::exit(1);
        }
    }
    if let Some((path, inode)) = socket_identity {
        if std::fs::metadata(&path).is_ok_and(|m| m.ino() == inode) {
            let _ = std::fs::remove_file(path);
        }
    }
}
