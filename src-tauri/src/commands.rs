use crate::{
    events::{latest_pool_messages_event, init_app_event}, poolpb::PoolFileInfo, POOL_MANAGER, STORE_MANAGER, ipc::IPCPoolMessageHistory, MESSAGES_DB, sspb::{PoolDeviceInfo, PoolUserInfo, PoolInfo},
};

#[tauri::command]
pub fn register_device(user_info: PoolUserInfo, device_info: PoolDeviceInfo) {
    STORE_MANAGER.new_profile(user_info, device_info);
    init_app_event();
}

#[tauri::command]
pub fn set_auth_token(auth_token: String) {
    STORE_MANAGER.set_auth_token(auth_token);
}

#[tauri::command]
pub fn add_pool(pool_info: PoolInfo) {
    STORE_MANAGER.update_pool(pool_info);
}

#[tauri::command]
pub fn remove_pool(pool_id: String) {
    STORE_MANAGER.remove_pool(&pool_id);
}

#[tauri::command]
pub async fn connect_to_pool(pool_id: String) {
    latest_pool_messages_event(&pool_id);
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
pub async fn add_file_offer(pool_id: String, file_path: String) {
    POOL_MANAGER.add_file_offer(&pool_id, file_path).await;
}

#[tauri::command]
pub async fn add_image_offer(pool_id: String, file_path: String) {
    POOL_MANAGER.add_image_offer(&pool_id, file_path).await;
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

#[tauri::command]
pub async fn request_message_history(pool_id: String, msg_id: String, chunk_number: u64) -> IPCPoolMessageHistory {
    if msg_id.is_empty() {
        MESSAGES_DB.messages_history_chunk(&pool_id, chunk_number)
    } else {
        MESSAGES_DB.messages_history_chunk_by_id(&pool_id, &msg_id)
    }
}