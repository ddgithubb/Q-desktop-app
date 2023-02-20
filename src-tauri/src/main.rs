#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use std::{sync::Arc, env};

use app::{
    __cmd__connect_to_pool, __cmd__disconnect_from_pool, __cmd__download_file,
    __cmd__remove_file_download, __cmd__retract_file_offer, __cmd__add_file_offer,
    __cmd__add_image_offer, __cmd__send_text_message,
    commands::{
        connect_to_pool, disconnect_from_pool, download_file, remove_file_download,
        retract_file_offer, add_file_offer, add_image_offer, send_text_message,
    },
    GLOBAL_APP_HANDLE, MESSAGES_DB, POOL_MANAGER, STORE_MANAGER, config::PRODUCTION_MODE,
};
use log::info;
use tauri::Manager;

#[tokio::main]
async fn main() {
    if !PRODUCTION_MODE {
      env::set_var("RUST_LOG", "app=trace");
    }

    env_logger::init();
    tauri::async_runtime::set(tokio::runtime::Handle::current());
    tauri::Builder::default()
        .setup(|app| {
            // https://github.com/tauri-apps/tauri/blob/dev/examples/splashscreen/tauri.conf.json
            // let splashscreen_window = app.get_window("splashscreen").unwrap(); need to specify splashscreen in tauri.conf.json
            let main_window = app.get_window("main").unwrap();

            tokio::spawn(async move {
                init_app().await;
                main_window.show().unwrap();
            });

            GLOBAL_APP_HANDLE.store(Some(Arc::new(app.app_handle())));

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            connect_to_pool,
            disconnect_from_pool,
            send_text_message,
            add_file_offer,
            add_image_offer,
            download_file,
            retract_file_offer,
            remove_file_download,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

async fn init_app() {
    info!("Initializing App...");
    lazy_static::initialize(&POOL_MANAGER);
    lazy_static::initialize(&STORE_MANAGER);
    lazy_static::initialize(&MESSAGES_DB);
}
