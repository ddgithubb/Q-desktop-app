import { PoolFileInfo, PoolFileSeeders, PoolMessage } from "./pool.v1";
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

export enum UserStatus {
    JOINED,
    LEFT,
}

export interface Pool {
    poolID: string;
    poolName: string;
    key: number;
    nodeID: string;
    connectionState: PoolConnectionState;
    users: PoolUserInfo[];
    fileOffers: PoolFileInfo[];
    availableFiles: PoolFileSeeders[];
    downloadQueue: PoolFileDownload[];
    feed: FeedMessage[];
    historyFeed?: HistoryFeed;
}

export interface HistoryFeed {
    feed: FeedMessage[];
    historyChunkLens: number[];
    historyChunkNumber: number;
    wasLatest: boolean;
}

export interface FeedMessage {
    msg?: PoolMessage;
    nodeStatus?: PoolNodeStatus;
    userStatus?: PoolUserStatus;
}

export interface PoolNodeStatus {
    nodeID: string,
    userID: string,
    status: NodeStatus;
    created: number;
}

export interface PoolUserStatus {
    userID: string,
    status: UserStatus;
    created: number;
}

export interface PoolFileDownload {
    fileInfo: PoolFileInfo;
    status: DownloadProgressStatus;
}