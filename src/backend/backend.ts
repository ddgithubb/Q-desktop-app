import { invoke } from "@tauri-apps/api";
import EventEmitter from "events";
import { AddDownloadAction, ClearPoolAction, UpdateConnectionStateAction } from "../store/slices/pool.action";
import { poolAction } from "../store/slices/pool.slice";
import { store } from "../store/store";
import { PoolConnectionState } from "../types/pool.model";
import { PoolFileInfo } from "../types/pool.v1";
import { open } from '@tauri-apps/api/dialog';

export class BackendCommands {

    poolKeyMap: Map<string, number> = new Map();
    events: EventEmitter;

    constructor() {
        this.events = new EventEmitter();
    }

    getPoolKey(poolID: string): number | undefined {
        return this.poolKeyMap.get(poolID);
    }

    connectToPool(poolID: string, poolKey: number, _display_name: string) {
        this.poolKeyMap.set(poolID, poolKey);

        store.dispatch(poolAction.updateConnectionState({
            key: poolKey,
            state: PoolConnectionState.CONNECTING,
        } as UpdateConnectionStateAction));

        invoke('connect_to_pool', { pool_id: poolID, _display_name });
    }

    disconnectFromPool(poolID: string) {
        let key = this.getPoolKey(poolID);
        if (!key) return;
        this.poolKeyMap.delete(poolID);

        store.dispatch(poolAction.clearPool({
            key,
        } as ClearPoolAction));

        invoke('disconnect_from_pool', { pool_id: poolID });
    }

    sendTextMessage(poolID: string, text: String) {
        invoke('send_text_message', { pool_id: poolID, text: text.trim() });
    }

    async addFileOffer(poolID: string) {
        let file_path = await open({
            multiple: false,
            directory: false,
            title: "Add File",
        });

        if (!file_path || typeof file_path != 'string') {
            return;
        }

        invoke('add_file_offer', { pool_id: poolID, file_path });
    }

    async addImageOffer(poolID: string) {
        let file_path = await open({
            multiple: false,
            directory: false,
            title: "Add Image",
            filters: [{
                name: 'Image',
                extensions: ['png', 'jpeg'],
            }]
        });

        if (!file_path || typeof file_path != 'string') {
            return;
        }

        invoke('add_image_offer', { pool_id: poolID, file_path });
    }

    async downloadFile(poolID: string, fileInfo: PoolFileInfo) {
        let key = this.getPoolKey(poolID);
        if (!key) return;

        for (const download of store.getState().pool.pools[key].downloadQueue) {
            if (download.fileInfo.fileId == fileInfo.fileId) {
                return;
            }
        }

        let dir_path = await open({
            multiple: false,
            directory: true,
            title: "Choose Save Directory",
        });

        invoke('download_file', { pool_id: poolID, file_info: fileInfo, dir_path });
    }

    retractFileOffer(poolID: string, fileID: string) {
        invoke('retract_file_offer', { pool_id: poolID, file_id: fileID });
    }

    removeFileDownload(poolID: string, fileID: string) {
        invoke('remove_file_download', { pool_id: poolID, file_id: fileID });
    }
}
