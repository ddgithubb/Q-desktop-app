#![cfg_attr(
  all(not(debug_assertions), target_os = "windows"),
  windows_subsystem = "windows"
)]

use app::{STORE_MANAGER, POOL_MANAGER, MESSAGES_DB};
use log::info;
use tauri::Manager;

#[tokio::main]
async fn main() {
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
      
      // let local_window = tauri::WindowBuilder::new(
      //   app,
      //   "local",
      //   tauri::WindowUrl::App("index.html".into())
      // ).build()?;

      Ok(())
    })
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}

async fn init_app() {
  info!("Initializing App...");
  lazy_static::initialize(&POOL_MANAGER);
  lazy_static::initialize(&STORE_MANAGER);
  lazy_static::initialize(&MESSAGES_DB);
}