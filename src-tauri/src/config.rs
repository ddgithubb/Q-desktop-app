use std::time::Duration;

pub const PRODUCTION_MODE: bool = false;

pub const CHUNK_SIZE: usize = 32 * 1024;

pub const MAX_DC_BUFFER_SIZE: usize = 16 * 1024 * 1024;
pub const MAX_DC_BUFFER_CHUNK_AMOUNT: usize = MAX_DC_BUFFER_SIZE / CHUNK_SIZE;
pub const DC_REFILL_RATE_SIZE: usize = 1 * 1024 * 1024;
pub const DC_REFILL_RATE_CHUNK_AMOUNT: usize = DC_REFILL_RATE_SIZE / CHUNK_SIZE;

pub const BUFFERED_AMOUNT_LOW_THRESHOLD: usize = MAX_DC_BUFFER_SIZE - DC_REFILL_RATE_SIZE; 
pub const DC_INIT_BUFFER_MAX_FILL_RATE_TIMEOUT: u64 = Duration::from_secs(1).as_millis() as u64;
pub const DC_INIT_BUFFER_MIN_FILL_RATE_TIMEOUT: u64 = Duration::from_millis(1).as_millis() as u64;

pub const CACHE_CHUNK_SIZE: usize = 1 * 1024 * 1024;
pub const CACHE_CHUNK_TO_CHUNK_SIZE_FACTOR: usize = CACHE_CHUNK_SIZE / CHUNK_SIZE;

pub const CACHE_CHUNK_BUFFER_SIZE: usize = 16 * 1024 * 1024;
pub const CACHE_CHUNK_BUFFER_AMOUNT: usize = CACHE_CHUNK_BUFFER_SIZE / CHUNK_SIZE;
pub const CACHE_FILE_SIZE: usize = 256 * 1024 * 1024;
pub const MAX_CACHE_CHUNKS_AMOUNT: usize = CACHE_FILE_SIZE / CACHE_CHUNK_SIZE; 

pub const MAX_TEMP_FILE_SIZE: u64 = 16 * 1024 * 1024;
// pub const MAX_TEMP_FILES_PER_POOL: usize = 10;
pub const MAX_TEMP_FILES_SIZE_PER_POOL: u64 = 128 * 1024 * 1024;
// pub const TOTAL_TEMP_FILE_THRESHOLD: u64 = 512 * 1024 * 1024;

pub const MAX_SEND_CHUNK_BUFFER_SIZE: usize = 16 * 1024 * 1024;
pub const MAX_SEND_CHUNK_BUFFER_LENGTH: usize = MAX_SEND_CHUNK_BUFFER_SIZE / CHUNK_SIZE;

pub const CHUNKS_MISSING_POLLING_INTERVAL_IN_SEC: usize = 1;
pub const CHUNKS_MISSING_POLLING_INTERVAL: Duration = Duration::from_secs(CHUNKS_MISSING_POLLING_INTERVAL_IN_SEC as u64);
pub const CHUNKS_MISSING_SEND_INTERVAL_IN_SEC: usize = 5;
pub const CHUNKS_MISSING_SEND_INTERVAL: Duration = Duration::from_secs(CHUNKS_MISSING_SEND_INTERVAL_IN_SEC as u64);
pub const MAX_CHUNKS_MISSING_RETRY: usize = 3;
pub const MAX_POLL_COUNT_BEFORE_SEND: usize = CHUNKS_MISSING_SEND_INTERVAL_IN_SEC / CHUNKS_MISSING_POLLING_INTERVAL_IN_SEC; 

pub const MESSAGES_DB_CHUNK_SIZE: u64 = 16 * 1024;
pub const RECEIVED_MESSAGES_SIZE: usize = 100;
pub const LATEST_MESSAGES_SIZE: usize = 50;

pub const MIN_MESSAGE_HIEGHT: u32 = 28;
pub const MESSAGE_VIEWPORT_SIZE: u32 = 3;

pub const MESSAGE_ID_LENGTH: usize = 10;
pub const FILE_ID_LENGTH: usize = 10;

pub const PREVIEW_IMAGE_DIMENSION: u32 = 10;

pub const HEARTBEAT_INTERVAL_SECONDS: u64 = 30;
pub const HEARTBEAT_TIMEOUT_SECONDS: u64 = 10;

pub const SYNC_SERVER_TEST_DOMAIN: &'static str = "192.168.0.18:80";
pub const SYNC_SERVER_PROD_DOMAIN: &'static str = "ec2-99-79-191-205.ca-central-1.compute.amazonaws.com";
pub const SYNC_SERVER_IS_SECURE: bool = false;
pub const SYNC_SERVER_VERSION: &'static str = "v1";
pub const SYNC_SERVER_DOMAIN: &'static str = if PRODUCTION_MODE { SYNC_SERVER_PROD_DOMAIN } else { SYNC_SERVER_TEST_DOMAIN };

pub const MAIN_TEST_POOL_ID: &'static str = "MAIN_TEST_POOL_ID";

fn sync_server_ws_host(path: String) -> String {
    format!("ws{}://{}{}",
        if SYNC_SERVER_IS_SECURE { "s" } else { "" },
        SYNC_SERVER_DOMAIN,
        path
    )
}

fn sync_server_api_host(path: String) -> String {
    format!("http{}://{}{}",
        if SYNC_SERVER_IS_SECURE { "s" } else { "" },
        SYNC_SERVER_DOMAIN,
        path
    )
}

pub fn sync_server_connect_endpoint(pool_id: &str, device_id: String) -> String {
    sync_server_ws_host(format!("/ss/{}/connect?poolid={}&deviceid={}&test={}",
        SYNC_SERVER_VERSION,
        pool_id,
        device_id,
        if pool_id != MAIN_TEST_POOL_ID { "false" } else { "true" }
    ))
}

pub fn sync_server_api_get_version_endpoint() -> String {
    sync_server_api_host(format!("/ss/version"))
}