import { invoke } from "@tauri-apps/api";
import { AddDownloadAction, ClearPoolAction, SetSavedPoolDataAction, RemoveDownloadAction, UpdateConnectionStateAction } from "../store/slices/pool.action";
import { poolAction } from "../store/slices/pool.slice";
import { store } from "../store/store";
import { DownloadProgressStatus, PoolConnectionState, PoolFileDownload } from "../types/pool.model";
import { PoolFileInfo } from "../types/pool.v1";
import { open } from '@tauri-apps/api/dialog';
import { IPCSavedPoolData } from "./backend.model";

export class BackendCommands {

    poolKeyMap: Map<string, number> = new Map();

    constructor() {}

    getPoolKey(poolId: string): number | undefined {
        return this.poolKeyMap.get(poolId);
    }

    async connectToPool(poolId: string, poolKey: number, displayName: string) {
        this.poolKeyMap.set(poolId, poolKey);

        store.dispatch(poolAction.updateConnectionState({
            key: poolKey,
            state: PoolConnectionState.CONNECTING,
        } as UpdateConnectionStateAction));

        let savedPoolData: IPCSavedPoolData = await invoke('connect_to_pool', { poolId, displayName });
        let setSavedPoolDataAction: SetSavedPoolDataAction = {
            key: poolKey,
            offlinePoolData: savedPoolData,
        };
        store.dispatch(poolAction.setSavedPoolData(setSavedPoolDataAction));
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
}
