import { createSlice, PayloadAction } from "@reduxjs/toolkit";
import { Pool, PoolConnectionState, DownloadProgressStatus, FeedMessage, NodeStatus, UserStatus } from "../../types/pool.model";
import { PoolFileInfo, PoolMessage } from "../../types/pool.v1";
import { PoolDeviceInfo } from "../../types/sync_server.v1";
import { PoolStore } from "../store";
import { PoolsState, PoolAction, UpdateConnectionStateAction, AddUserAction, RemoveUserAction, LatestMessagesAction, AppendMessageAction, AddNodeAction, RemoveNodeAction, RemoveFileOfferAction, AddDownloadAction, RemoveDownloadAction, InitPoolAction, InitFileSeedersAction, AddFileOffersAction, UpdateDownloadStatus, CompleteDownloadAction, ClearPoolAction, SetPoolsAction, AddMessageHistoryAction, AddPoolAction } from "./pool.action";

// const MAX_FEED_SIZE = 100;
// const MAX_FEED_SIZE = 50;
var MAX_FEED_SIZE = 100; // DEFAULT
var HISTORY_OVERLAP_THRESHOLD = Math.trunc(MAX_FEED_SIZE / 2);

export function setMaxFeedSize(maxFeedSize: number) {
    MAX_FEED_SIZE = maxFeedSize;
    HISTORY_OVERLAP_THRESHOLD = Math.trunc(MAX_FEED_SIZE / 2);
}

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
                    nodeID: "",
                    connectionState: PoolConnectionState.CLOSED,
                    users: poolsInfo[i].users,
                    fileOffers: [],
                    availableFiles: [],
                    downloadQueue: [],
                    feed: [],
                };
                state.pools.push(pool);

                PoolStore.setDisplayNames(pool.users);
            }
        },
        addPool(state: PoolsState, action: PayloadAction<AddPoolAction>) {
            let poolInfo = action.payload.poolInfo;
            let pool: Pool = {
                poolID: poolInfo.poolId,
                poolName: poolInfo.poolName,
                nodeID: "",
                connectionState: PoolConnectionState.CLOSED,
                users: poolInfo.users,
                fileOffers: [],
                availableFiles: [],
                downloadQueue: [],
                feed: [],
            };
            state.pools.unshift(pool);

            PoolStore.setDisplayNames(pool.users);
        },
        removePool(state: PoolsState, action: PayloadAction<PoolAction>) {
            let poolID = action.payload.poolID;
            let index = state.pools.findIndex((pool) => pool.poolID == poolID);
            if (index >= 0) {
                state.pools.splice(index, 1);
            }

            PoolStore.clearActiveDevices(action.payload.poolID);
        },
        initPool(state: PoolsState, action: PayloadAction<InitPoolAction>) {
            let initPool = action.payload.initPool;
            let index = state.pools.findIndex(pool => pool.poolID == initPool.pool_info.poolId);
            if (index < 0) {
                return;
            }

            let pool = state.pools[index];
            state.pools.splice(index, 1);
            state.pools.unshift(pool);

            pool.poolName = initPool.pool_info.poolName;
            pool.connectionState = PoolConnectionState.CONNECTED;
            pool.nodeID = initPool.node_id;
            pool.users = initPool.pool_info.users;

            PoolStore.setDisplayNames(pool.users);

            PoolStore.clearActiveDevices(pool.poolID);
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
            pool.historyFeed = undefined;

            PoolStore.clearActiveDevices(pool.poolID);
        },
        updateConnectionState(state: PoolsState, action: PayloadAction<UpdateConnectionStateAction>) {
            let pool = getPool(state, action);
            pool.connectionState = action.payload.state;
        },
        addUser(state: PoolsState, action: PayloadAction<AddUserAction>) {
            let pool = getPool(state, action);
            let userInfo = action.payload.userInfo;
            let newUser = true;
            for (let i = 0; i < pool.users.length; i++) {
                if (pool.users[i].userId == userInfo.userId) {
                    pool.users[i] = userInfo
                    newUser = false;
                    break;
                }
            }

            if (newUser) {
                addToFeed(pool.feed, {
                    userStatus: {
                        userID: userInfo.userId,
                        status: UserStatus.JOINED,
                        created: Date.now(),
                    }
                });
    
                pool.users.push(userInfo);
            }

            PoolStore.setDisplayName(userInfo.userId, userInfo.displayName);
        },
        removeUser(state: PoolsState, action: PayloadAction<RemoveUserAction>) {
            let pool = getPool(state, action);
            let userID = action.payload.userID;

            addToFeed(pool.feed, {
                userStatus: {
                    userID,
                    status: UserStatus.LEFT,
                    created: Date.now(),
                }
            });

            for (let i = 0; i < pool.users.length; i++) {
                if (pool.users[i].userId == action.payload.userID) {
                    pool.users.splice(i, 1);
                    break;
                }
            }
        },
        addNode(state: PoolsState, action: PayloadAction<AddNodeAction>) {
            let pool = getPool(state, action);
            let nodeID = action.payload.node.node_id;
            let userID = action.payload.node.user_id;
            let device = action.payload.node.device;

            for (const user of pool.users) {
                if (user.userId == userID) {
                    let found = false;
                    for (const d of user.devices) {
                        if (d.deviceId == device.deviceId) {
                            found = true;
                            break;
                        }
                    }
                    if (!found) {
                        user.devices.push(device);
                    }
                    break;
                }
            }
            
            addToFeed(pool.feed, {
                nodeStatus: {
                    nodeID,
                    userID,
                    device, 
                    status: NodeStatus.ACTIVE,
                    created: Date.now(),
                },
            });

            PoolStore.addActiveDevice(pool.poolID, nodeID, userID);
        },
        removeNode(state: PoolsState, action: PayloadAction<RemoveNodeAction>) {
            let pool = getPool(state, action);
            let nodeID = action.payload.nodeID;
            let userID = PoolStore.getActiveDeviceUserID(pool.poolID, nodeID);
            if (!userID) return;

            let device: PoolDeviceInfo | undefined = undefined;
            for (const user of pool.users) {
                if (user.userId == userID) {
                    for (const d of user.devices) {
                        if (d.deviceId == nodeID) {
                            device = d                            
                            break;
                        }
                    }
                    break;
                }
            }

            if (!device) {
                return;
            }

            addToFeed(pool.feed, {
                nodeStatus: {
                    nodeID: action.payload.nodeID,
                    userID: userID,
                    device: device,
                    status: NodeStatus.INACTIVE,
                    created: Date.now(),
                },
            });

            for (let i = 0; i < pool.availableFiles.length; i++) {
                let fileOffer = pool.availableFiles[i];
                for (let j = 0; j < fileOffer.seederNodeIds.length; j++) {
                    if (fileOffer.seederNodeIds[j] == nodeID) {
                        fileOffer.seederNodeIds.splice(j, 1);
                        break;
                    }
                }

                if (fileOffer.seederNodeIds.length == 0) {
                    pool.availableFiles.splice(i, 1);
                }
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
                    break;
                }
            }
        },
        completeDownload(state: PoolsState, action: PayloadAction<CompleteDownloadAction>) {
            let pool = getPool(state, action);
            for (let i = 0; i < pool.downloadQueue.length; i++) {
                if (pool.downloadQueue[i].fileInfo!.fileId == action.payload.fileID) {
                    pool.downloadQueue[i].status = action.payload.success ? DownloadProgressStatus.SUCCESS : DownloadProgressStatus.FAILURE;
                    break;
                }
            }
            PoolStore.completedDownloadEvents.emit(action.payload.fileID, action.payload.success);
        },
        removeDownload(state: PoolsState, action: PayloadAction<RemoveDownloadAction>) {
            let pool = getPool(state, action);
            for (let i = 0; i < pool.downloadQueue.length; i++) {
                if (pool.downloadQueue[i].fileInfo!.fileId == action.payload.fileID) {
                    pool.downloadQueue.splice(i, 1);
                    break;
                }
            }
            PoolStore.removeDownloadProgress(action.payload.fileID);
        },
        latestMessages(state: PoolsState, action: PayloadAction<LatestMessagesAction>) {
            let pool = getPool(state, action);
            pool.feed = messagesToFeed(action.payload.messages);
        },
        appendMessage(state: PoolsState, action: PayloadAction<AppendMessageAction>) {
            let pool = getPool(state, action);
            
            addToFeed(pool.feed, {
                msg: action.payload.message,
            });
        },
        addMessageHistory(state: PoolsState, action: PayloadAction<AddMessageHistoryAction>) {
            let pool = getPool(state, action);
            let messageHistory = action.payload.messageHistory;
            let newFeed = messagesToFeed(messageHistory.messages);

            if (!pool.historyFeed) {
                // desc, transition from regular feed
                pool.historyFeed = {
                    feed: newFeed,
                    historyChunkLens: messageHistory.chunk_lens,
                    historyChunkNumber: messageHistory.chunk_number,
                    wasLatest: messageHistory.is_latest,
                };
                attatchOverlappingHistoryFeed(pool);
            } else if (messageHistory.chunk_number < pool.historyFeed.historyChunkNumber) {
                // desc
                newFeed.push(...pool.historyFeed.feed);
                pool.historyFeed.feed = newFeed;
                pool.historyFeed.historyChunkLens.unshift(messageHistory.messages.length);
                pool.historyFeed.historyChunkNumber = messageHistory.chunk_number;
                pool.historyFeed.wasLatest = false;

                let newFeedLen = pool.historyFeed.feed.length; 
                while (newFeedLen > MAX_FEED_SIZE && pool.historyFeed.historyChunkLens.length > 2) {
                    let chunkLen = pool.historyFeed.historyChunkLens.pop()!;
                    newFeedLen -= chunkLen; 
                }

                if (newFeedLen != pool.historyFeed.feed.length) {
                    pool.historyFeed.feed.splice(newFeedLen);
                }
            } else {
                // asc
                if (pool.historyFeed.wasLatest) { // Garaunteed to be the same chunkNumber as last history chunk
                    let newFeedLen =
                            pool.historyFeed.feed.length -
                            pool.historyFeed.historyChunkLens[pool.historyFeed.historyChunkLens.length - 1];
                    pool.historyFeed.feed.splice(newFeedLen);
                    pool.historyFeed.feed.push(...newFeed);
                } else {
                    pool.historyFeed.feed.push(...newFeed);
                    pool.historyFeed.historyChunkLens.push(messageHistory.messages.length);

                    let newFeedLen = pool.historyFeed.feed.length; 
                    while (newFeedLen > MAX_FEED_SIZE && pool.historyFeed.historyChunkLens.length > 2) {
                        let chunkLen = pool.historyFeed.historyChunkLens.shift()!;
                        newFeedLen -= chunkLen;
                        pool.historyFeed.historyChunkNumber += 1;
                    }

                    if (newFeedLen != pool.historyFeed.feed.length) {
                        let deleteCount = pool.historyFeed.feed.length - newFeedLen;
                        pool.historyFeed.feed.splice(0, deleteCount);
                    }
                }

                if (messageHistory.is_latest) {
                    attatchOverlappingHistoryFeed(pool);
                }

                pool.historyFeed.wasLatest = messageHistory.is_latest;
            }
        },
        switchToLatestFeed(state: PoolsState, action: PayloadAction<PoolAction>) {
            let pool = getPool(state, action);
            pool.historyFeed = undefined;
        }
    }
});

function getPool(state: PoolsState, action: PayloadAction<PoolAction>): Pool {
    return state.pools.find(pool => pool.poolID == action.payload.poolID)!;
}

function messagesToFeed(messages: PoolMessage[]): FeedMessage[] {
    return messages.map((msg) => {
        let feedMsg: FeedMessage = {
            msg,
        };
        return feedMsg;
    });
}

function addToFeed(feed: FeedMessage[], feedMsg: FeedMessage) {
    feed.push(feedMsg);

    if (feed.length > MAX_FEED_SIZE) {
        if (feed[1].msg) {
            feed.shift();
        } else {
            feed.splice(1, 1);
        }
    }
}

// pool feed first message must be a valid message with msgID
// Returns NEW lastChunkLen
function attatchOverlappingHistoryFeed(pool: Pool) {
    if (!pool.historyFeed) {
        return;
    }

    let historyFeed = pool.historyFeed.feed;
    let lastChunkLen = pool.historyFeed.historyChunkLens[pool.historyFeed.historyChunkLens.length - 1];
    let firstOverlapMsgID = pool.feed[0].msg!.msgId;
    let lastOverlapMsgID = historyFeed[historyFeed.length - 1].msg!.msgId;
    let overlapMessages: FeedMessage[] | undefined = undefined;

    // console.log("BEFORE HISTORY CHUNKS", historyFeed.slice(), lastChunkLen, firstOverlapMsgID, lastOverlapMsgID);

    for (let i = 0; i < pool.feed.length; i++) {
        if (pool.feed[i].msg?.msgId == lastOverlapMsgID) {
            overlapMessages = pool.feed.slice(0, i + 1);
            break;
        }
    }

    // console.log("Overlapping messages len", overlapMessages?.length);

    if (overlapMessages != undefined) {
        for (let i = historyFeed.length - lastChunkLen; i < historyFeed.length; i++) {
            if (historyFeed[i].msg?.msgId == firstOverlapMsgID) {
                historyFeed.splice(i);
                lastChunkLen -= historyFeed.length - i;
                historyFeed.push(...overlapMessages);
                lastChunkLen += overlapMessages.length;
                break;
            }
        }
    }

    // console.log("AFTER HISTORY CHUNKS", historyFeed.slice(), lastChunkLen);
    
    pool.historyFeed.historyChunkLens[pool.historyFeed.historyChunkLens.length - 1] = lastChunkLen;
}

export function checkMajorityMessageOverlap(msgs1: FeedMessage[], msgs2: FeedMessage[]): boolean {
    let msgID = msgs1[msgs1.length - 1].msg!.msgId;

    for (let i = HISTORY_OVERLAP_THRESHOLD; i < msgs2.length; i++) {
        if (msgs2[i].msg?.msgId == msgID) {
            return true;
        }
    }
    return false;
}

export const poolReducer = poolSlice.reducer;
export const poolAction = poolSlice.actions;

