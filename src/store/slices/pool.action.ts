import { IPCInitPool, IPCPoolMessageHistory, IPCPoolNode, IPCSavedPoolData } from "../../backend/ipc";
import { Pool, PoolConnectionState, DownloadProgressStatus } from "../../types/pool.model";
import { PoolMessage, PoolFileInfo, PoolFileSeeders } from "../../types/pool.v1";
import { PoolInfo, PoolUserInfo } from "../../types/sync_server.v1";

export interface PoolsState {
    pools: Pool[];
}

export interface PoolAction {
    key: number;
}

export interface SetPoolsAction {
    poolInfos: PoolInfo[];
}

export interface AddPoolAction extends PoolAction {
    poolInfo: PoolInfo;
}

export interface InitPoolAction extends PoolAction {
    initPool: IPCInitPool
}

export interface ClearPoolAction extends PoolAction { }

export interface UpdateConnectionStateAction extends PoolAction {
    state: PoolConnectionState;
}

export interface AddUserAction extends PoolAction {
    userInfo: PoolUserInfo;
}

export interface RemoveUserAction extends PoolAction {
    userID: string;
}

export interface AddFileOffersAction extends PoolAction {
    nodeID: string;
    fileInfos: PoolFileInfo[];
}

export interface RemoveFileOfferAction extends PoolAction {
    nodeID: string;
    fileID: string;
} 

export interface InitFileSeedersAction extends PoolAction {
    fileSeeders: PoolFileSeeders[];
}

export interface AddNodeAction extends PoolAction {
    node: IPCPoolNode;
}

export interface RemoveNodeAction extends PoolAction {
    nodeID: string;
}

export interface AddDownloadAction extends PoolAction {
    fileInfo: PoolFileInfo;
}

export interface UpdateDownloadStatus extends PoolAction {
    fileID: string;
    status: DownloadProgressStatus
}

export interface CompleteDownloadAction extends PoolAction {
    fileID: string;
    success: boolean;
}

export interface RemoveDownloadAction extends PoolAction {
    fileID: string;
}

export interface LatestMessagesAction extends PoolAction {
    messages: PoolMessage[];
}

export interface AppendMessageAction extends PoolAction {
    message: PoolMessage;
}

export interface AddMessageHistoryAction extends PoolAction {
    messageHistory: IPCPoolMessageHistory;
}