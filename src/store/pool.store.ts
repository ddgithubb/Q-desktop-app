import { EventEmitter } from "events";
import { IPCFileDownloadProgress } from "../backend/ipc";
import { PoolUserInfo } from "../types/sync_server.v1";

const INVITE_LINK_CACHE_DURATION = 30 * 1000 * 60; // 30 minutes

export class PoolStoreClass {
    
    private displayNameMap: Map<string, string>; // userID -> displayName
    private poolActiveDevices: Map<string, Map<string, string>>; // poolID -> deviceID -> userID
    private downloadsProgress: Map<string, number>; // fileID -> progress
    completedDownloadEvents: EventEmitter;

    private inviteLinks: Map<string, string>; // poolID -> inviteLink
    
    constructor() {
        this.displayNameMap = new Map();
        this.poolActiveDevices = new Map();
        this.downloadsProgress = new Map();
        this.completedDownloadEvents = new EventEmitter();

        this.inviteLinks = new Map();
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

    setDisplayNames(poolUsers: PoolUserInfo[]) {
        for (const user of poolUsers) {
            this.setDisplayName(user.userId, user.displayName);
        }
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
    }

    removeDownloadProgress(fileID: string) {
        this.downloadsProgress.delete(fileID);
    }

    getDownloadProgress(fileID: string): number {
        return this.downloadsProgress.get(fileID) || 0;
    }

    hasDownload(fileID: string): boolean {
        return this.downloadsProgress.has(fileID);
    }

    startInviteLinkCleaner() {
        if (this.inviteLinks.size == 0) return;
        let interval = setInterval(() => {
            this.inviteLinks.forEach((inviteLink, poolID) => {
                this.inviteLinks.delete(poolID);
            });
            if (this.inviteLinks.size == 0) clearInterval(interval);
        }, INVITE_LINK_CACHE_DURATION);
    }

    setInviteLink(poolID: string, inviteLink: string) {
        this.inviteLinks.set(poolID, inviteLink);
        if (this.inviteLinks.size == 1) this.startInviteLinkCleaner();
    }

    getInviteLink(poolID: string): string | undefined {
        return this.inviteLinks.get(poolID);
    }
}