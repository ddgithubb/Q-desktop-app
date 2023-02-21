import { PoolFileInfo, PoolMessage } from "./pool.v1";
import { PoolUserInfo } from "./sync_server.v1";

export const NODE_ID_LENGTH = 10;
export const MESSAGE_ID_LENGTH = 21;
export const FILE_ID_LENGTH = 21;

export enum PoolConnectionState {
    CLOSED,
    CONNECTED,
    CONNECTING,
    RECONNECTING,
}

export enum DownloadProgressStatus {
    DOWNLOADING,
    SUCCESS,
    FAILURE,
}

export enum NodeStatus {
    INACTIVE,
    ACTIVE,
}

export interface Pool {
    poolID: string;
    poolName: string;
    key: number;
    connectionState: PoolConnectionState;
    users: PoolUserInfo[];
    activeNodes: PoolNode[];
    downloadQueue: PoolFileDownload[];
    feed: FeedMessage[];
}

export interface FeedMessage {
    msg?: PoolMessage;
    nodeStatus?: PoolNodeStatus;
}

export interface PoolNodeStatus {
    nodeID: string,
    userID: string,
    status: NodeStatus;
    created: number;
}

export interface PoolFileDownload {
    fileInfo: PoolFileInfo;
    status: DownloadProgressStatus;
    progress?: number;
}

export interface PoolNode {
    nodeID: string;
    userID: string;
    fileOffers: PoolFileInfo[];
}