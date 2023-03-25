import { invoke } from "@tauri-apps/api";
import { AddDownloadAction, AddMessageHistoryAction, ClearPoolAction, RemoveDownloadAction, UpdateConnectionStateAction } from "../store/slices/pool.action";
import { checkMajorityMessageOverlap, poolAction } from "../store/slices/pool.slice";
import { getStoreState, store } from "../store/store";
import { DownloadProgressStatus, PoolConnectionState, PoolFileDownload } from "../types/pool.model";
import { PoolFileInfo } from "../types/pool.v1";
import { open } from '@tauri-apps/api/dialog';
import { IPCPoolMessageHistory } from "./ipc";
import sanitizeHTML from "sanitize-html";
import { AuthenticateDevice, RegisterNewUser } from "../api/auth";
import { CreateInviteToPool, CreatePool, JoinPool, LeavePool } from "../api/pool";

const BR_TAG: string = "<br />";

export class BackendCommands {

    poolKeyMap: Map<string, number> = new Map();

    constructor() { }

    getPoolKey(poolId: string): number | undefined {
        return this.poolKeyMap.get(poolId);
    }

    setAuthToken(authToken: string) {
        invoke('set_auth_token', { authToken });
    }

    async registerNewUser(displayName: string, deviceName: string) {
        let [ userInfo, deviceInfo ] = await RegisterNewUser(displayName, deviceName);
        invoke('register_device', {
            userInfo,
            deviceInfo,
        });
    }

    async createPool(poolName: string) {
        let poolInfo = await CreatePool(poolName);
        invoke('add_pool', { poolInfo });
    }

    async joinPool(inviteLink: string) {
        let poolInfo = await JoinPool(inviteLink);
        invoke('add_pool', { poolInfo });
    }

    async leavePool(poolId: string) {
        await LeavePool(poolId);
        invoke('remove_pool', { poolId });
    }

    async createInviteLink(poolId: string): Promise<string> {
        return await CreateInviteToPool(poolId);
    }

    async connectToPool(poolId: string, poolKey: number, authenticate: boolean = false) {
        this.poolKeyMap.set(poolId, poolKey);

        store.dispatch(poolAction.updateConnectionState({
            key: poolKey,
            state: PoolConnectionState.CONNECTING,
        } as UpdateConnectionStateAction));

        if (authenticate) {
            try {
                await AuthenticateDevice();
            } catch (e) {
                this.connectToPool(poolId, poolKey, authenticate);
                return;
            }
        }

        invoke('connect_to_pool', { poolId });
    }

    disconnectFromPool(poolId: string) {
        let key = this.getPoolKey(poolId);
        if (key == undefined) return;

        this.poolKeyMap.delete(poolId);

        store.dispatch(poolAction.clearPool({
            key,
        } as ClearPoolAction));

        invoke('disconnect_from_pool', { poolId });
    }

    sendTextMessage(poolId: string, text: string) {
        text = sanitizeHTML(text.trim(), { allowedTags: ['br'] });

        let i = text.length - BR_TAG.length;
        while (i >= 0) {
            if (text.substring(i, i + BR_TAG.length) != BR_TAG) {
                break;
            }
            text = text.slice(0, i);
            i -= BR_TAG.length;
        }

        if (text.length == 0) {
            return;
        }

        invoke('send_text_message', { poolId, text });
    }

    async addFileOffer(poolId: string) {
        let key = this.getPoolKey(poolId);
        if (key == undefined) return;

        let filePath = await open({
            multiple: false, // Not supported for now
            directory: false,
            title: "Add File",
        });

        if (!filePath) {
            return;
        }

        if (typeof filePath == 'string') {
            invoke('add_file_offer', { poolId, filePath });
        } else {
            // for (const path of filePath) {
            //     invoke('add_file_offer', { poolId, filePath: path });
            // }
        }
    }

    async addImageOffer(poolId: string) {
        let key = this.getPoolKey(poolId);
        if (key == undefined) return;

        let filePath = await open({
            multiple: false,
            directory: false,
            title: "Add Image",
            filters: [{
                name: 'Image',
                extensions: ['png', 'jpg', 'jpeg'],
            }]
        });

        if (!filePath || typeof filePath != 'string') {
            return;
        }

        invoke('add_image_offer', { poolId, filePath });
    }

    async downloadFile(poolId: string, fileInfo: PoolFileInfo, isTemp: boolean = false): Promise<boolean> {
        let key = this.getPoolKey(poolId);
        if (key == undefined) return false;

        let dirPath = "";
        if (!isTemp) {
            let path = await open({
                multiple: false,
                directory: true,
                title: "Choose Save Directory",
            });
            if (!path || typeof path != 'string') {
                return false;
            }

            dirPath = path;
        }

        let addDownloadAction: AddDownloadAction = {
            key,
            fileInfo,
        };
        store.dispatch(poolAction.addDownload(addDownloadAction));

        await invoke('download_file', { poolId, fileInfo, dirPath });

        return true;
    }

    retractFileOffer(poolId: string, fileId: string) {
        let key = this.getPoolKey(poolId);
        if (key == undefined) return;

        invoke('retract_file_offer', { poolId, fileId });
    }

    removeFileDownload(poolId: string, fileDownload: PoolFileDownload) {
        let key = this.getPoolKey(poolId);
        if (key == undefined) return;

        let removeDownloadAction: RemoveDownloadAction = {
            key,
            fileID: fileDownload.fileInfo.fileId,
        };
        store.dispatch(poolAction.removeDownload(removeDownloadAction));

        if (fileDownload.status == DownloadProgressStatus.DOWNLOADING) {
            invoke('remove_file_download', { poolId, fileId: fileDownload.fileInfo.fileId });
        }
    }

    async requestMessageHistory(poolId: string, asc: boolean = false): Promise<boolean> {
        let key = this.getPoolKey(poolId);
        if (key == undefined) return false;
        let messageHistory: IPCPoolMessageHistory;

        let pool = getStoreState().pool.pools[key];
        if (pool.historyFeed) {
            let chunkNumber: number;
            if (asc) {
                if (checkMajorityMessageOverlap(pool.historyFeed.feed, pool.feed)) {
                    store.dispatch(poolAction.switchToLatestFeed({ key }));
                    return false;
                }

                chunkNumber =
                    pool.historyFeed.historyChunkNumber + pool.historyFeed.historyChunkLens.length;

                if (pool.historyFeed.wasLatest) {
                    chunkNumber -= 1;
                }
            } else {
                if (pool.historyFeed.historyChunkNumber == 0) {
                    return false;
                }

                chunkNumber = pool.historyFeed.historyChunkNumber - 1;
            }

            // console.log("Requesting Message History", chunkNumber);

            messageHistory = await invoke('request_message_history', { poolId, msgId: "", chunkNumber });
        } else {
            if (asc) {
                return false;
            }

            let msgId = "";
            for (const feedMsg of pool.feed) {
                if (feedMsg.msg) {
                    console.log(feedMsg.msg);
                    msgId = feedMsg.msg.msgId;
                    break;
                }
            }

            if (msgId == "") {
                return false;
            }

            // console.log("Requesting Message History", msgId);

            messageHistory = await invoke('request_message_history', { poolId, msgId, chunkNumber: 0 });
        }

        if (messageHistory.messages.length == 0) {
            return false;
        }

        let addMessageHistoryAction: AddMessageHistoryAction = {
            key,
            messageHistory,
        };
        store.dispatch(poolAction.addMessageHistory(addMessageHistoryAction));

        return true;
    }

}
