import { createSlice, PayloadAction } from "@reduxjs/toolkit";
import { IPCInitPool } from "../../backend/backend.model";
import { Pool, PoolConnectionState, DownloadProgressStatus, PoolNode, FeedMessage, NodeStatus } from "../../types/pool.model";
import { PoolMessage, PoolFileInfo } from "../../types/pool.v1";
import { PoolDeviceInfo, PoolUserInfo } from "../../types/sync_server.v1";
import { PoolInfo, SSMessage_InitPoolData } from "../../types/sync_server.v1";
import { PoolsState, PoolAction, UpdateConnectionStateAction, UpdateUserAction, RemoveUserAction, InitMessageAction, AppendMessageAction, AddNodeAction, RemoveNodeAction, RemoveFileOfferAction, AddDownloadAction, RemoveDownloadAction, InitPoolAction, InitFileSeedersAction, AddFileOffersAction, UpdateDownloadStatus, CompleteDownloadAction, ClearPoolAction } from "./pool.action";

const LATEST_MESSAGES_SIZE = 50;

const initialState: PoolsState = {
    pools: [],
}

const poolSlice = createSlice({
    name: "pool",
    initialState: initialState,
    reducers: {
        setPools(state: PoolsState, action: PayloadAction<PoolInfo[]>) {
            let poolsInfo = action.payload
            for (let i = 0; i < action.payload.length; i++) {
                state.pools.push({
                    poolID: poolsInfo[i].poolId,
                    poolName: poolsInfo[i].poolName,
                    key: i,
                    connectionState: PoolConnectionState.CLOSED,
                    users: poolsInfo[i].users,
                    activeNodes: [],
                    downloadQueue: [],
                    feed: [],
                    messagesLength: 0,
                } as Pool)
            }
        },
        initPool(state: PoolsState, action: PayloadAction<InitPoolAction>) {
            let pool = getPool(state, action);
            let initPool = action.payload.initPool;
            pool.poolName = initPool.pool_info.poolName;
            pool.connectionState = PoolConnectionState.CONNECTED;
                pool.users = initPool.pool_info.users;
            pool.activeNodes = initPool.init_nodes.map((node) => {
                return {
                    nodeID: node.node_id,
                    userID: node.user_id,
                    fileOffers: [],
                }
            });
            pool.downloadQueue = [];
            pool.feed = [];
            pool.messagesLength = 0;
        },
        clearPool(state: PoolsState, action: PayloadAction<ClearPoolAction>) {
            let pool = getPool(state, action);
            pool.connectionState = PoolConnectionState.CLOSED;
            pool.activeNodes = [];
            pool.downloadQueue = [];
            pool.feed = [];
            pool.messagesLength = 0;
        },
        updateConnectionState(state: PoolsState, action: PayloadAction<UpdateConnectionStateAction>) {
            let pool = getPool(state, action);
            pool.connectionState = action.payload.state;

            if (action.payload.state != PoolConnectionState.CONNECTED) {
                pool.activeNodes = [];
            }
        },
        updateUser(state: PoolsState, action: PayloadAction<UpdateUserAction>) {
            let pool = getPool(state, action);
            let userInfo = action.payload.userInfo;
            for (const user of pool.users) {
                if (user.userId == user.userId) {
                    user.devices = user.devices;
                    return;
                }
            }
            pool.users.push(userInfo);
        },
        removeUser(state: PoolsState, action: PayloadAction<RemoveUserAction>) {
            let pool = getPool(state, action);
            for (let i = 0; i < pool.users.length; i++) {
                if (pool.users[i].userId == action.payload.userID) {
                    pool.users.splice(i, 1);
                    break;
                }
            }
        },
        initMessages(state: PoolsState, action: PayloadAction<InitMessageAction>) {
            let pool = getPool(state, action);
            pool.feed = action.payload.messages.map((msg) => {
                let feedMsg: FeedMessage = {
                    msg,
                };
                return feedMsg;
            });
            pool.messagesLength = pool.feed.length;
        },
        appendMessage(state: PoolsState, action: PayloadAction<AppendMessageAction>) {
            let pool = getPool(state, action);
            pool.feed.push({
                msg: action.payload.message,
            });
            pool.messagesLength++;
            if (pool.messagesLength > LATEST_MESSAGES_SIZE) {
                pool.feed.shift();
            }
        },
        addNode(state: PoolsState, action: PayloadAction<AddNodeAction>) {
            let pool = getPool(state, action);
            pool.activeNodes.push(action.payload.node);
            pool.feed.push({
                nodeStatus: {
                    nodeID: action.payload.node.nodeID,
                    userID: action.payload.node.userID,
                    status: NodeStatus.ACTIVE,
                    created: Date.now(),
                },
            });
        },
        removeNode(state: PoolsState, action: PayloadAction<RemoveNodeAction>) {
            let pool = getPool(state, action);
            let userID = "";
            for (let i = 0; i < pool.activeNodes.length; i++) {
                if (pool.activeNodes[i].nodeID == action.payload.nodeID) {
                    userID = pool.activeNodes[i].userID;
                    pool.activeNodes.splice(i, 1);
                    pool.feed.push({
                        nodeStatus: {
                            nodeID: action.payload.nodeID,
                            userID: userID,
                            status: NodeStatus.INACTIVE,
                            created: Date.now(),
                        },
                    });
                    break;
                }
            }
        },
        addFileOffers(state: PoolsState, action: PayloadAction<AddFileOffersAction>) {
            let pool = getPool(state, action);
            for (const node of pool.activeNodes) {
                if (node.nodeID == action.payload.nodeID) {
                    node.fileOffers.unshift(...action.payload.fileInfos);
                    break;
                }
            }
        },
        removeFileOffer(state: PoolsState, action: PayloadAction<RemoveFileOfferAction>) {
            let pool = getPool(state, action);
            for (const node of pool.activeNodes) {
                if (node.nodeID == action.payload.nodeID) {
                    for (let i = 0; i < node.fileOffers.length; i++) {
                        if (node.fileOffers[i].fileId == action.payload.fileID) {
                            node.fileOffers.splice(i, 1);
                        }
                    }
                    break;
                }
            }
        },
        initFileSeeders(state: PoolsState, action: PayloadAction<InitFileSeedersAction>) {
            let pool = getPool(state, action);
            let fileOffersMap = new Map<string, PoolFileInfo[]>();
            for (const fileSeeders of action.payload.fileSeeders) {
                for (const nodeID of fileSeeders.seederNodeIds) {
                    let existingFileOffers = fileOffersMap.get(nodeID);
                    if (!existingFileOffers) {
                        existingFileOffers = [];
                        fileOffersMap.set(nodeID, existingFileOffers);
                    }
                    existingFileOffers.push(fileSeeders.fileInfo!);
                }
            }

            for (const node of pool.activeNodes) {
                let fileOffers = fileOffersMap.get(node.nodeID);
                if (fileOffers) {
                    node.fileOffers = fileOffers;
                }
            }
        },
        addDownload(state: PoolsState, action: PayloadAction<AddDownloadAction>) {
            let pool = getPool(state, action);
            pool.downloadQueue.push({ 
                fileInfo: action.payload.fileInfo,
                status: DownloadProgressStatus.DOWNLOADING
            });
        },
        updateDownloadStatus(state: PoolsState, action: PayloadAction<UpdateDownloadStatus>) {
            let pool = getPool(state, action);
            for (let i = 0; i < pool.downloadQueue.length; i++) {
                if (pool.downloadQueue[i].fileInfo!.fileId == action.payload.fileID) {
                    pool.downloadQueue[i].status = action.payload.status;
                    return;
                }
            }
        },
        completeDownload(state: PoolsState, action: PayloadAction<CompleteDownloadAction>) {
            let pool = getPool(state, action);
            for (let i = 0; i < pool.downloadQueue.length; i++) {
                if (pool.downloadQueue[i].fileInfo!.fileId == action.payload.fileID) {
                    pool.downloadQueue[i].status = action.payload.success ? DownloadProgressStatus.SUCCESS : DownloadProgressStatus.FAILURE;
                    return;
                }
            }
        },
        removeDownload(state: PoolsState, action: PayloadAction<RemoveDownloadAction>) {
            let pool = getPool(state, action);
            for (let i = 0; i < pool.downloadQueue.length; i++) {
                if (pool.downloadQueue[i].fileInfo!.fileId == action.payload.fileID) {
                    pool.downloadQueue.splice(i, 1);
                    return;
                }
            }
        }
    }
});

function getPool(state: PoolsState, action: PayloadAction<PoolAction>): Pool {
    return state.pools[action.payload.key];
}

export const poolReducer = poolSlice.reducer;
export const poolAction = poolSlice.actions;

