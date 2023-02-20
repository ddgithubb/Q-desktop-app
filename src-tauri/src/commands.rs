use crate::{poolpb::PoolFileInfo, POOL_MANAGER, STORE_MANAGER};

#[tauri::command]
pub async fn connect_to_pool(pool_id: String, display_name: String) {
    STORE_MANAGER._set_display_name(display_name);
    POOL_MANAGER.connect_to_pool(pool_id).await;
}

#[tauri::command]
pub async fn disconnect_from_pool(pool_id: String) {
    POOL_MANAGER.disconnect_from_pool(pool_id).await;
}

#[tauri::command]
pub async fn send_text_message(pool_id: String, text: String) {
    POOL_MANAGER.send_text_message(&pool_id, text).await;
}

#[tauri::command]
pub async fn add_file_offer(
    pool_id: String,
    file_path: String,
) {
    POOL_MANAGER
        .add_file_offer(&pool_id, file_path)
        .await;
}

#[tauri::command]
pub async fn add_image_offer(
    pool_id: String,
    file_path: String,
) {
    POOL_MANAGER
        .add_image_offer(&pool_id, file_path)
        .await;
}

#[tauri::command]
pub async fn download_file(pool_id: String, file_info: PoolFileInfo, dir_path: String) {
    POOL_MANAGER
        .download_file(&pool_id, file_info, dir_path)
        .await;
}

#[tauri::command]
pub async fn retract_file_offer(pool_id: String, file_id: String) {
    POOL_MANAGER.retract_file_offer(&pool_id, file_id).await;
}

#[tauri::command]
pub async fn remove_file_download(pool_id: String, file_id: String) {
    POOL_MANAGER.remove_file_download(&pool_id, file_id).await;
}
