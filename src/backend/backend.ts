import { invoke } from "@tauri-apps/api";
import EventEmitter from "events";
import { AddDownloadAction, ClearPoolAction, RemoveDownloadAction, UpdateConnectionStateAction } from "../store/slices/pool.action";
import { poolAction } from "../store/slices/pool.slice";
import { store } from "../store/store";
import { DownloadProgressStatus, PoolConnectionState, PoolFileDownload } from "../types/pool.model";
import { PoolFileInfo } from "../types/pool.v1";
import { open } from '@tauri-apps/api/dialog';

export class BackendCommands {

    poolKeyMap: Map<string, number> = new Map();
    events: EventEmitter;

    constructor() {
        this.events = new EventEmitter();
    }

    getPoolKey(poolId: string): number | undefined {
        return this.poolKeyMap.get(poolId);
    }

    connectToPool(poolId: string, poolKey: number, displayName: string) {
        this.poolKeyMap.set(poolId, poolKey);

        store.dispatch(poolAction.updateConnectionState({
            key: poolKey,
            state: PoolConnectionState.CONNECTING,
        } as UpdateConnectionStateAction));

        invoke('connect_to_pool', { poolId, displayName });
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

    sendTextMessage(poolId: string, text: String) {
        invoke('send_text_message', { poolId, text: text.trim() });
    }

    async addFileOffer(poolId: string) {
        let key = this.getPoolKey(poolId);
        if (key == undefined) return;

        let filePath = await open({
            multiple: false,
            directory: false,
            title: "Add File",
        });

        if (!filePath || typeof filePath != 'string') {
            return;
        }

        invoke('add_file_offer', { poolId, filePath });
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
                extensions: ['png', 'jpeg'],
            }]
        });

        if (!filePath || typeof filePath != 'string') {
            return;
        }

        invoke('add_image_offer', { poolId, filePath });
    }

    async downloadFile(poolId: string, fileInfo: PoolFileInfo) {
        let key = this.getPoolKey(poolId);
        if (key == undefined) return;

        for (const download of store.getState().pool.pools[key].downloadQueue) {
            if (download.fileInfo.fileId == fileInfo.fileId) {
                return;
            }
        }

        let dirPath = await open({
            multiple: false,
            directory: true,
            title: "Choose Save Directory",
        });
        
        if (!dirPath || typeof dirPath != 'string') {
            return;
        }

        invoke('download_file', { poolId, fileInfo, dirPath });
    }

    retractFileOffer(poolId: string, fileId: string) {
        let key = this.getPoolKey(poolId);
        if (key == undefined) return;

        invoke('retract_file_offer', { poolId, fileId });
    }

    removeFileDownload(poolId: string, fileDownload: PoolFileDownload) {
        let key = this.getPoolKey(poolId);
        if (key == undefined) return;

        if (fileDownload.status == DownloadProgressStatus.DOWNLOADING) {
            invoke('remove_file_download', { poolId, fileId: fileDownload.fileInfo.fileId });
        } else {
            let removeDownloadAction: RemoveDownloadAction = {
                key,
                fileID: fileDownload.fileInfo.fileId,
            };
            store.dispatch(poolAction.removeDownload(removeDownloadAction));
        }
    }
}
