import { profile } from "console";
import { PoolFileInfo, PoolFileSeeders, PoolMessage } from "../types/pool.v1";
import { PoolDeviceInfo, PoolInfo, PoolUserInfo } from "../types/sync_server.v1";

export interface IPCFileDownloadProgress {
    file_id: string,
    progress: number,
}

export interface IPCStateUpdate {
    file_downloads_progress: IPCFileDownloadProgress[],
}

export interface IPCInitProfile {
    user_info: PoolUserInfo,
    device: PoolDeviceInfo,
}

export interface IPCSavedPoolData {
    messages: PoolMessage[],
}

export interface IPCPoolNode {
    node_id: string,
    user_id: string,
}

export interface IPCInitPool {
    node_id: string,
    pool_info: PoolInfo,
    init_nodes: IPCPoolNode[], 
}

export interface IPCReconnectPool {
    pool_id: string,
}

export interface IPCAddPoolNode {
    pool_id: string,
    node: IPCPoolNode,
}

export interface IPCRemovePoolNode {
    pool_id: string,
    node_id: string,
}

export interface IPCAddPoolUser {
    pool_id: string,
    user_info: PoolUserInfo,
}

export interface IPCRemovePoolUser {
    pool_id: string,
    user_id: string,
}

export interface IPCAddPoolFileDownload {
    pool_id: string,
    file_info: PoolFileInfo,
}

export interface IPCAddPoolFileOffers {
    pool_id: string,
    node_id: string,
    file_offers: PoolFileInfo[],
}

export interface IPCRemovePoolFileOffer {
    pool_id: string,
    node_id: string,
    file_id: string,
}

export interface IPCInitPoolFileSeeders {
    pool_id: string,
    file_seeders: PoolFileSeeders[],
}

export interface IPCCompletePoolFileDownload {
    pool_id: string,
    file_id: string,
    success: boolean,
}

export interface IPCInitPoolMessages {
    pool_id: string,
    messages: PoolMessage[],
}

export interface IPCAppendPoolMessage {
    pool_id: string,
    message: PoolMessage,
}

