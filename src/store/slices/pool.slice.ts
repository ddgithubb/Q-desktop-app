import { createSlice, PayloadAction } from "@reduxjs/toolkit";
import { IPCInitPool } from "../../backend/backend.model";
import { Backend } from "../../backend/global";
import { Pool, PoolConnectionState, DownloadProgressStatus, FeedMessage, NodeStatus, UserStatus } from "../../types/pool.model";
import { PoolMessage, PoolFileInfo } from "../../types/pool.v1";
import { PoolDeviceInfo, PoolUserInfo } from "../../types/sync_server.v1";
import { PoolInfo, SSMessage_InitPoolData } from "../../types/sync_server.v1";
import { PoolStore, store } from "../store";
import { PoolsState, PoolAction, UpdateConnectionStateAction, AddUserAction, RemoveUserAction, InitMessageAction, AppendMessageAction, AddNodeAction, RemoveNodeAction, RemoveFileOfferAction, AddDownloadAction, RemoveDownloadAction, InitPoolAction, InitFileSeedersAction, AddFileOffersAction, UpdateDownloadStatus, CompleteDownloadAction, ClearPoolAction, SetSavedPoolDataAction, SetPoolsAction } from "./pool.action";

const MAX_FEED_SIZE = 50;

const initialState: PoolsState = {
    pools: [],
}

const poolSlice = createSlice({
    name: "pool",
    initialState: initialState,
    reducers: {
        setPools(state: PoolsState, action: PayloadAction<SetPoolsAction>) {
            let poolsInfo = action.payload.poolInfos;
            for (let i = 0; i < poolsInfo.length; i++) {
                let pool: Pool = {
                    poolID: poolsInfo[i].poolId,
                    poolName: poolsInfo[i].poolName,
                    key: i,
                    nodeID: "",
                    connectionState: PoolConnectionState.CLOSED,
                    users: poolsInfo[i].users,
                    fileOffers: [],
                    availableFiles: [],
                    downloadQueue: [],
                    feed: [],
                };
                state.pools.push(pool);

                for (const user of pool.users) {
                    PoolStore.setDisplayName(user.userId, user.displayName);
                }
            }
        },
        setSavedPoolData(state: PoolsState, action: PayloadAction<SetSavedPoolDataAction>) {
            let pool = getPool(state, action);
            let offlinePoolData = action.payload.offlinePoolData;
            pool.feed = offlinePoolData.messages.map((msg) => {
                let feedMsg: FeedMessage = {
                    msg,
                };
                return feedMsg;
            });
        },
        initPool(state: PoolsState, action: PayloadAction<InitPoolAction>) {
            let pool = getPool(state, action);
            let initPool = action.payload.initPool;
            pool.poolName = initPool.pool_info.poolName;
            pool.connectionState = PoolConnectionState.CONNECTED;
            pool.nodeID = initPool.node_id;
            pool.users = initPool.pool_info.users;

            PoolStore.clearActiveDevices(pool.poolID);

            for (const user of pool.users) {
                PoolStore.setDisplayName(user.userId, user.displayName);
            }
            
            for (const node of initPool.init_nodes) {
                PoolStore.addActiveDevice(pool.poolID, node.node_id, node.user_id);
            }
        },
        clearPool(state: PoolsState, action: PayloadAction<ClearPoolAction>) {
            let pool = getPool(state, action);
            pool.connectionState = PoolConnectionState.CLOSED;
            pool.fileOffers = [];
            pool.availableFiles = [];
            pool.downloadQueue = [];
            pool.feed = [];

            PoolStore.clearActiveDevices(pool.poolID);
        },
        updateConnectionState(state: PoolsState, action: PayloadAction<UpdateConnectionStateAction>) {
            let pool = getPool(state, action);
            pool.connectionState = action.payload.state;
        },
        addUser(state: PoolsState, action: PayloadAction<AddUserAction>) {
            let pool = getPool(state, action);
            let userInfo = action.payload.userInfo;

            pool.feed.push({
                userStatus: {
                    userID: userInfo.userId,
                    status: UserStatus.JOINED,
                    created: Date.now(),
                }
            });

            if (pool.feed.length > MAX_FEED_SIZE) {
                pool.feed.shift();
            }

            pool.users.push(userInfo);

            PoolStore.setDisplayName(userInfo.userId, userInfo.displayName);
        },
        removeUser(state: PoolsState, action: PayloadAction<RemoveUserAction>) {
            let pool = getPool(state, action);
            let userID = action.payload.userID;

            pool.feed.push({
                userStatus: {
                    userID,
                    status: UserStatus.LEFT,
                    created: Date.now(),
                }
            });

            if (pool.feed.length > MAX_FEED_SIZE) {
                pool.feed.shift();
            }

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
        },
        appendMessage(state: PoolsState, action: PayloadAction<AppendMessageAction>) {
            let pool = getPool(state, action);
            
            pool.feed.push({
                msg: action.payload.message,
            });

            if (pool.feed.length > MAX_FEED_SIZE) {
                pool.feed.shift();
            }
        },
        addNode(state: PoolsState, action: PayloadAction<AddNodeAction>) {
            let pool = getPool(state, action);
            let nodeID = action.payload.nodeID;
            let userID = action.payload.userID;
            
            pool.feed.push({
                nodeStatus: {
                    nodeID,
                    userID,
                    status: NodeStatus.ACTIVE,
                    created: Date.now(),
                },
            });

            if (pool.feed.length > MAX_FEED_SIZE) {
                pool.feed.shift();
            }

            PoolStore.addActiveDevice(pool.poolID, nodeID, userID);
        },
        removeNode(state: PoolsState, action: PayloadAction<RemoveNodeAction>) {
            let pool = getPool(state, action);
            let nodeID = action.payload.nodeID;
            let userID = PoolStore.getActiveDeviceUserID(pool.poolID, nodeID);
            if (!userID) return;

            pool.feed.push({
                nodeStatus: {
                    nodeID: action.payload.nodeID,
                    userID: userID,
                    status: NodeStatus.INACTIVE,
                    created: Date.now(),
                },
            });

            if (pool.feed.length > MAX_FEED_SIZE) {
                pool.feed.shift();
            }

            PoolStore.removeActiveDevice(pool.poolID, nodeID);
        },
        initFileSeeders(state: PoolsState, action: PayloadAction<InitFileSeedersAction>) {
            let pool = getPool(state, action);
            pool.availableFiles = action.payload.fileSeeders;
        },
        addFileOffers(state: PoolsState, action: PayloadAction<AddFileOffersAction>) {
            let pool = getPool(state, action);
            let nodeID = action.payload.nodeID;

            if (pool.nodeID == nodeID) {
                pool.fileOffers.unshift(...action.payload.fileInfos);
            }
            
            let fileOffersMap = new Map<string, PoolFileInfo>();
            for (const fileInfo of action.payload.fileInfos) {
                fileOffersMap.set(fileInfo.fileId, fileInfo);
            }

            for (const fileOffer of pool.availableFiles) {
                if (fileOffersMap.has(fileOffer.fileInfo!.fileId)) {
                    fileOffer.seederNodeIds.push(nodeID);
                    fileOffersMap.delete(fileOffer.fileInfo!.fileId);
                }
            }

            fileOffersMap.forEach((fileInfo) => {
                pool.availableFiles.unshift({
                    fileInfo,
                    seederNodeIds: [nodeID],
                });
            });
        },
        removeFileOffer(state: PoolsState, action: PayloadAction<RemoveFileOfferAction>) {
            let pool = getPool(state, action);
            let fileID = action.payload.fileID;
            let nodeID = action.payload.nodeID;

            if (pool.nodeID == nodeID) {
                for (let i = 0; i < pool.fileOffers.length; i++) {
                    if (pool.fileOffers[i].fileId == fileID) {
                        pool.fileOffers.splice(i, 1);
                        break;
                    }
                }
            }

            for (let i = 0; i < pool.availableFiles.length; i++) {
                let fileOffer = pool.availableFiles[i];
                if (fileOffer.fileInfo!.fileId == fileID) {
                    for (let j = 0; j < fileOffer.seederNodeIds.length; j++) {
                        if (fileOffer.seederNodeIds[j] == nodeID) {
                            fileOffer.seederNodeIds.splice(j, 1);
                            break;
                        }
                    }

                    if (fileOffer.seederNodeIds.length == 0) {
                        pool.availableFiles.splice(i, 1);
                    }

                    break;
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
                    PoolStore.completedDownloadEvents.emit(action.payload.fileID, action.payload.success);
                    return;
                }
            }
        },
        removeDownload(state: PoolsState, action: PayloadAction<RemoveDownloadAction>) {
            let pool = getPool(state, action);
            for (let i = 0; i < pool.downloadQueue.length; i++) {
                if (pool.downloadQueue[i].fileInfo!.fileId == action.payload.fileID) {
                    pool.downloadQueue.splice(i, 1);
                    PoolStore.removeDownloadProgress(action.payload.fileID);
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

