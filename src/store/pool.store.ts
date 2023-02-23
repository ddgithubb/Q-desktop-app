import { EventEmitter } from "events";
import { IPCFileDownloadProgress } from "../backend/backend.model";

export class PoolStoreClass {
    
    private displayNameMap: Map<string, string>; // userID -> displayName
    private poolActiveDevices: Map<string, Map<string, string>>; // poolID -> deviceID -> userID
    private downloadsProgress: Map<string, number>; // fileID -> progress
    completedDownloadEvents: EventEmitter;
    
    constructor() {
        this.displayNameMap = new Map();
        this.poolActiveDevices = new Map();
        this.downloadsProgress = new Map();
        this.completedDownloadEvents = new EventEmitter();
    }

    getDisplayName(userID: string): string | undefined {
        return this.displayNameMap.get(userID);
    }

    getActiveDeviceUserID(poolID: string, deviceID: string): string | undefined {
        return this.poolActiveDevices.get(poolID)?.get(deviceID);
    }

    getActiveDevicesCount(poolID: string): number {
        return this.poolActiveDevices.get(poolID)?.size || 0;
    }

    hasActiveDevice(poolID: string, deviceID: string): boolean {
        return this.poolActiveDevices.get(poolID)?.has(deviceID) || false;
    }

    setDisplayName(userID: string, displayName: string) {
        this.displayNameMap.set(userID, displayName);
    }

    addActiveDevice(poolID: string, deviceID: string, userID: string) {
        let activeDevices = this.poolActiveDevices.get(poolID);
        if (!activeDevices) {
            activeDevices = new Map();
            this.poolActiveDevices.set(poolID, activeDevices);
        };

        activeDevices.set(deviceID, userID);
    }

    removeActiveDevice(poolID: string, deviceID: string) {
        let activeDevices = this.poolActiveDevices.get(poolID);
        if (!activeDevices) return;

        activeDevices.delete(deviceID);
    }

    clearActiveDevices(poolID: string) {
        let activeDevices = this.poolActiveDevices.get(poolID);
        if (!activeDevices) return;

        activeDevices.clear();
    }

    updateDownloadProgress(downloads_progress: IPCFileDownloadProgress[]) {
        this.downloadsProgress.clear();
        for (const progress of downloads_progress) {
            this.downloadsProgress.set(progress.file_id, progress.progress);
        }
        console.log("Updated downloads progress:", this.downloadsProgress);
    }

    getDownloadProgress(fileID: string): number {
        return this.downloadsProgress.get(fileID) || 0;
    }
}  