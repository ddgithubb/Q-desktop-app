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
    GLOBAL_APP_HANDLE, MESSAGES_DB, POOL_MANAGER, STORE_MANAGER, config::PRODUCTION_MODE, store::file_store::FileStore, sspb::{PoolUserInfo, PoolDeviceInfo, DeviceType}, ipc::IPCInitProfile, events::init_profile_event,
};
use log::info;
use tauri::{Manager, WindowEvent};

#[tokio::main]
async fn main() {
    if !PRODUCTION_MODE {
      env::set_var("RUST_LOG", "app=trace");
      env::set_var("RUST_BACKTRACE", "1");
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
                init_app_tests().await;
                main_window.show().unwrap();
            });

            GLOBAL_APP_HANDLE.store(Some(Arc::new(app.app_handle())));

            Ok(())
        })
        .register_uri_scheme_protocol("media", FileStore::register_media_protocol)
        .on_window_event(|event| {
            match event.event() {
                WindowEvent::Destroyed => {
                    let (destroyed_tx, destroyed_rx) = flume::bounded(0);
                    tokio::spawn(async move {
                        destroy_app().await;
                        let _ = destroyed_tx.send(());
                    });
                    let _ = destroyed_rx.recv();
                },
                _ => {}
            }
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

async fn init_app_tests() {
    let id = nanoid::nanoid!(21);
    log::info!("Generated ID: {}", id);

    let user_info = PoolUserInfo {
        user_id: id.clone(),
        display_name: "Test".into(),
        devices: vec![PoolDeviceInfo {
            device_id: id,
            device_type: DeviceType::Desktop.into(),
            device_name: "Test Device".into(),
        }],
    };
    
    STORE_MANAGER.new_profile(
        user_info.clone(),
        user_info.devices[0].clone(),
    );

    init_profile_event(IPCInitProfile {
        device: user_info.devices[0].clone(),
        user_info: user_info,
    });
}

async fn init_app() {
    info!("Initializing App...");
    lazy_static::initialize(&POOL_MANAGER);
    lazy_static::initialize(&STORE_MANAGER);
    lazy_static::initialize(&MESSAGES_DB);
    info!("Initialized App!");
}

async fn destroy_app() {
    info!("Destroying App...");
    POOL_MANAGER.clean_all().await;
    info!("Destroyed App!");
}