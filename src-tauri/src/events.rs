use tauri::Manager;

use crate::{
    ipc::{
        IPCAddPoolFileOffers, IPCAddPoolNode, IPCAddPoolUser, IPCAppendPoolMessage,
        IPCCompletePoolFileDownload, IPCInitPool, IPCInitPoolFileSeeders,
        IPCLatestPoolMessages, IPCPoolNode, IPCReconnectPool, IPCRemovePoolFileOffer,
        IPCRemovePoolNode, IPCRemovePoolUser, IPCStateUpdate, IPCRefreshAuthToken,
    },
    poolpb::{PoolFileInfo, PoolFileSeeders, PoolMessage},
    sspb::PoolUserInfo,
    GLOBAL_APP_HANDLE, MESSAGES_DB, STORE_MANAGER,
};

const STATE_UPDATE_EVENT: &'static str = "state-update";
const REFRESH_AUTH_TOKEN_EVENT: &'static str = "refresh-auth-token";

const INIT_APP_EVENT: &'static str = "init-app";

const INIT_POOL_EVENT: &'static str = "init-pool";
const RECONNECT_POOL_EVENT: &'static str = "reconnect-pool";
const ADD_POOL_NODE_EVENT: &'static str = "add-pool-node";
const REMOVE_POOL_NODE_EVENT: &'static str = "remove-pool-node";
const ADD_POOL_USER_EVENT: &'static str = "add-pool-user";
const REMOVE_POOL_USER_EVENT: &'static str = "remove-pool-user";

const ADD_POOL_FILE_OFFERS_EVENT: &'static str = "add-pool-file-offers";
const REMOVE_POOL_FILE_OFFER_EVENT: &'static str = "remove-pool-file-offer";
const INIT_POOL_FILE_SEEDERS_EVENT: &'static str = "init-pool-file-seeders";

const COMPLETE_POOL_FILE_DOWNLOAD_EVENT: &'static str = "complete-pool-file-download";

const LATEST_POOL_MESSAGES_EVENT: &'static str = "latest-pool-messages";
const APPEND_POOL_MESSAGE_EVENT: &'static str = "append-pool-message";

pub fn state_update_event(state: IPCStateUpdate) {
    if let Some(app_handle) = &*GLOBAL_APP_HANDLE.load() {
        let _ = app_handle.emit_all(STATE_UPDATE_EVENT, state);
    }
}

pub fn refresh_auth_token_event() {
    if let Some(app_handle) = &*GLOBAL_APP_HANDLE.load() {
        let _ = app_handle.emit_all(REFRESH_AUTH_TOKEN_EVENT, IPCRefreshAuthToken {
            auth_token: STORE_MANAGER.auth_token(),
        });
    }
}

pub fn init_app_event() {
    if let Some(app_handle) = &*GLOBAL_APP_HANDLE.load() {
        let _ = app_handle.emit_all(INIT_APP_EVENT, STORE_MANAGER.ipc_init_app());
    }
}

pub fn init_pool_event(init_pool: IPCInitPool) {
    if let Some(app_handle) = &*GLOBAL_APP_HANDLE.load() {
        let _ = app_handle.emit_all(INIT_POOL_EVENT, init_pool);
    }
}

pub fn reconnect_pool_event(pool_id: &String, reauth: bool) {
    if let Some(app_handle) = &*GLOBAL_APP_HANDLE.load() {
        let _ = app_handle.emit_all(
            RECONNECT_POOL_EVENT,
            IPCReconnectPool {
                pool_id: pool_id.clone(),
                reauth,
            },
        );
    }
}

pub fn add_pool_node_event(pool_id: &String, node: IPCPoolNode) {
    if let Some(app_handle) = &*GLOBAL_APP_HANDLE.load() {
        let _ = app_handle.emit_all(
            ADD_POOL_NODE_EVENT,
            IPCAddPoolNode {
                pool_id: pool_id.clone(),
                node,
            },
        );
    }
}

pub fn remove_pool_node_event(pool_id: &String, node_id: String) {
    if let Some(app_handle) = &*GLOBAL_APP_HANDLE.load() {
        let _ = app_handle.emit_all(
            REMOVE_POOL_NODE_EVENT,
            IPCRemovePoolNode {
                pool_id: pool_id.clone(),
                node_id,
            },
        );
    }
}

pub fn add_pool_user_event(pool_id: &String, user_info: PoolUserInfo) {
    if let Some(app_handle) = &*GLOBAL_APP_HANDLE.load() {
        let _ = app_handle.emit_all(
            ADD_POOL_USER_EVENT,
            IPCAddPoolUser {
                pool_id: pool_id.clone(),
                user_info,
            },
        );
    }
}

pub fn remove_pool_user_event(pool_id: &String, user_id: String) {
    if let Some(app_handle) = &*GLOBAL_APP_HANDLE.load() {
        let _ = app_handle.emit_all(
            REMOVE_POOL_USER_EVENT,
            IPCRemovePoolUser {
                pool_id: pool_id.clone(),
                user_id,
            },
        );
    }
}

pub fn add_pool_file_offers_event(
    pool_id: &String,
    node_id: String,
    file_offers: Vec<PoolFileInfo>,
) {
    if let Some(app_handle) = &*GLOBAL_APP_HANDLE.load() {
        let _ = app_handle.emit_all(
            ADD_POOL_FILE_OFFERS_EVENT,
            IPCAddPoolFileOffers {
                pool_id: pool_id.clone(),
                node_id,
                file_offers,
            },
        );
    }
}

pub fn remove_pool_file_offer_event(pool_id: &String, node_id: String, file_id: String) {
    if let Some(app_handle) = &*GLOBAL_APP_HANDLE.load() {
        let _ = app_handle.emit_all(
            REMOVE_POOL_FILE_OFFER_EVENT,
            IPCRemovePoolFileOffer {
                pool_id: pool_id.clone(),
                node_id,
                file_id,
            },
        );
    }
}

pub fn init_pool_file_seeders_event(pool_id: &String, file_seeders: Vec<PoolFileSeeders>) {
    if let Some(app_handle) = &*GLOBAL_APP_HANDLE.load() {
        let _ = app_handle.emit_all(
            INIT_POOL_FILE_SEEDERS_EVENT,
            IPCInitPoolFileSeeders {
                pool_id: pool_id.clone(),
                file_seeders,
            },
        );
    }
}

pub fn complete_pool_file_download_event(pool_id: &String, file_id: String, success: bool) {
    if let Some(app_handle) = &*GLOBAL_APP_HANDLE.load() {
        let _ = app_handle.emit_all(
            COMPLETE_POOL_FILE_DOWNLOAD_EVENT,
            IPCCompletePoolFileDownload {
                pool_id: pool_id.clone(),
                file_id,
                success,
            },
        );
    }
}

pub fn latest_pool_messages_event(pool_id: &String) {
    if let Some(app_handle) = &*GLOBAL_APP_HANDLE.load() {
        let max_messages_render = STORE_MANAGER.max_messages_render();
        let _ = app_handle.emit_all(
            LATEST_POOL_MESSAGES_EVENT,
            IPCLatestPoolMessages {
                pool_id: pool_id.clone(),
                messages: MESSAGES_DB.last_messages(&pool_id, max_messages_render),
                max_messages_render,
            },
        );
    }
}

pub fn append_pool_message_event(pool_id: &String, message: PoolMessage) {
    if let Some(app_handle) = &*GLOBAL_APP_HANDLE.load() {
        let _ = app_handle.emit_all(
            APPEND_POOL_MESSAGE_EVENT,
            IPCAppendPoolMessage {
                pool_id: pool_id.clone(),
                message,
            },
        );
    }
}
