
import { useState, useRef, memo, useEffect } from 'react'
import { CircularProgressbar } from 'react-circular-progressbar'
import { fileSizeToString } from '../../utils/file-size'
import { Pool, PoolConnectionState, DownloadProgressStatus, PoolFileDownload, PoolNode } from '../../types/pool.model'
import { IndicatorDot } from '../components/IndicatorDot'
import { PoolDisplayUsersView } from './PoolDisplayUsersView'
import { PoolMessageMode, UserMapType } from './PoolView'

import './PoolDisplayView.css'
import AddIcon from '../../assets/add.png'
import AddImageIcon from '../../assets/add-image.png'
import FileIcon from '../../assets/file.png'
import CancelIcon from '../../assets/trash.png'
import SendIcon from '../../assets/send.png'
import { STATE_UPDATE_EVENT } from '../../backend/events'
import { listen } from '@tauri-apps/api/event'
import { IPCFileDownloadProgress, IPCStateUpdate } from '../../backend/backend.model'
import { Backend } from '../../backend/global'

export interface PoolDisplayViewParams {
    pool: Pool;
    myNode: PoolNode;
    messageMode: PoolMessageMode;
    userMap: UserMapType;
}

export const PoolDisplayView = memo(PoolDisplayViewComponent);

function PoolDisplayViewComponent({ pool, myNode, messageMode, userMap }: PoolDisplayViewParams) {

    const [textAreaElement, setTextAreaElement] = useState<HTMLDivElement | null>(null);

    const cachedTextMessage = useRef<string>("");

    const shiftKeyDown = useRef<boolean>(false);
    const enterKeyDown = useRef<boolean>(false);

    const sendTextMessage = () => {
        if (!textAreaElement) return;
        if (textAreaElement.innerHTML == "") return;
        Backend.sendTextMessage(pool.poolID, textAreaElement.innerHTML);
        cachedTextMessage.current = "";
        textAreaElement.innerHTML = "";
    }

    const addFile = () => {
        Backend.addFileOffer(pool.poolID);
    }

    const addImage = () => {
        Backend.addImageOffer(pool.poolID);
    }

    const retractFileOffer = (fileID: string) => {
        Backend.retractFileOffer(pool.poolID, fileID)
    }

    const textAreaKeyDown = (e: React.KeyboardEvent<HTMLDivElement>) => {
        if (e.key == 'Enter') {
            enterKeyDown.current = true;
            if (!shiftKeyDown.current) {
                e.preventDefault();
                sendTextMessage();
                return
            }
        } else if (e.key == 'Shift') {
            shiftKeyDown.current = true;
        }
        if (!textAreaElement) return
        if (textAreaElement?.innerHTML.length >= 5000) {
            e.preventDefault();
        }
        textAreaElement?.scrollTo({ top: textAreaElement.scrollHeight });
    }

    const textAreaKeyUp = (e: React.KeyboardEvent<HTMLDivElement>) => {
        if (e.key == 'Enter') {
            enterKeyDown.current = false;
        } else if (e.key == 'Shift') {
            shiftKeyDown.current = false;
        }
    }

    const textAreaBlur = () => {
        if (!textAreaElement) return;
        cachedTextMessage.current = textAreaElement?.innerHTML
    }

    return (
        <div className="display-container">
            <div className="display-overlay-container">
                <DownloadQueue poolID={pool.poolID} downloadQueue={pool.downloadQueue} />
            </div>
            <div className="display-info-bar">
                <div className="display-info-bar-pool-name">
                    {pool.poolName}
                </div>
                {
                    pool.connectionState == PoolConnectionState.CONNECTED ? (
                        <div className="display-info-bar-status">
                            <IndicatorDot type="online" />
                            <div className="display-info-bar-subtitle">
                                {pool.activeNodes.length || 0} Device{(pool.activeNodes.length || 0) == 1 ? "" : "s"} Active
                            </div>
                        </div>
                    ) : (
                        <div className="display-info-bar-status">
                            <IndicatorDot type="danger" />
                            <div className="display-info-bar-subtitle">
                                Connecting...
                            </div>
                        </div>
                    )
                }
                <div className="display-info-bar-status">
                    <IndicatorDot type="offline" />
                    <div className="display-info-bar-subtitle">
                        {pool.users.length} Total User{(pool.users.length || 1) > 1 ? "s" : ""}
                    </div>
                </div>
            </div>
            <div className="display-toggle-hide display-message-input" aria-hidden={messageMode != PoolMessageMode.TEXT}>
                <div
                    className="display-component-container display-text-input"
                    data-placeholder='Send Text Message'
                    contentEditable
                    ref={(e) => setTextAreaElement(e)}
                    onKeyDown={textAreaKeyDown}
                    onKeyUp={textAreaKeyUp}
                    onBlur={textAreaBlur}
                    dangerouslySetInnerHTML={{ __html: cachedTextMessage.current }}
                    spellCheck="false"
                />
                <div className="display-message-input-icons">
                    <img className="display-message-input-icon" src={AddImageIcon} onClick={addImage} />
                </div>
            </div>
            <div className="display-toggle-hide display-component-container display-files-container" aria-hidden={messageMode != PoolMessageMode.FILE}>
                <div className="display-file-container display-file-container-add-button" onClick={addFile}>
                    <img src={AddIcon} height={28} width={28} />
                    Add File
                </div>
                {/* delete file(s) button with function to delete multiple files by just clicking*/}
                {
                    myNode.fileOffers.map((fileInfo) => (
                        <div className="display-cancel-button-container" key={fileInfo.fileId} onClick={() => retractFileOffer(fileInfo.fileId)}>
                            <div className="display-file-container display-cancel-button-child elipsify-container">
                                <img src={FileIcon} height={22} width={22} />
                                <span className="display-file-name elipsify-content">{fileInfo.fileName}</span>
                                <span className="display-file-size elipsify-extra">{fileSizeToString(fileInfo.totalSize)}</span>
                            </div>
                            <img className="display-cancel-button-icon" src={CancelIcon} />
                        </div>
                    ))
                }
            </div>
            <PoolDisplayUsersView poolID={pool.poolID || ""} users={pool.users || []} userMap={userMap} hidden={messageMode != PoolMessageMode.USERS} />
        </div>
    )
}

function DownloadQueue({ poolID, downloadQueue }: { poolID: string, downloadQueue: PoolFileDownload[] }) {

    let dqMap = useRef<Map<string, PoolFileDownload>>(new Map());

    useEffect(() => {
        let newDqMap = new Map<string, PoolFileDownload>();
        
        for (const download of downloadQueue) {
            newDqMap.set(download.fileInfo.fileId, download);
        }
        
        dqMap.current = newDqMap;
    }, [downloadQueue]);

    useEffect(() => {
        listen(STATE_UPDATE_EVENT, ({ event, payload }) => {
            let state: IPCStateUpdate = payload as any;
            for (const download_progress of state.file_downloads_progress) {
                let download = dqMap.current.get(download_progress.file_id);
                if (!download) continue;
                download.progress = download_progress.progress
            }
        });
    }, []);

    const removeFileDownload = (fileDownload: PoolFileDownload) => {
        Backend.removeFileDownload(poolID, fileDownload);
    }

    return (
        <div className="display-downloading-files-container">
            {
                downloadQueue.map((fileDownload) => (
                    <div
                        className="display-cancel-button-container display-downloading-file-container"
                        onClick={() => removeFileDownload(fileDownload)}
                        key={fileDownload.fileInfo!.fileId}>
                        <div className="display-downloading-file display-cancel-button-child">
                            <div className="display-downloading-file-progress">
                                <CircularProgressbar
                                    value={fileDownload.progress || 0}
                                    strokeWidth={15}
                                    styles={{
                                        path: {
                                            stroke: `rgb(${getRGBFromDownloadProgressStatus(fileDownload.status)})`,
                                        },
                                        trail: {
                                            stroke: `rgba(${getRGBFromDownloadProgressStatus(fileDownload.status)}, 0.1)`
                                        }
                                    }} />
                            </div>
                            <div className="display-downloading-file-name">
                                {fileDownload.fileInfo!.fileName}
                            </div>
                        </div>
                        <img className="display-cancel-button-icon" src={CancelIcon} />
                    </div>
                ))
            }
        </div>
    )
}

function getRGBFromDownloadProgressStatus(status: DownloadProgressStatus): string {
    if (status == DownloadProgressStatus.DOWNLOADING) {
        return "84, 140, 230";
    }
    else if (status == DownloadProgressStatus.SUCCESS) {
        return "48, 198, 76";
    }
    else if (status == DownloadProgressStatus.FAILURE) {
        return "212, 51, 51";
    }
    return "";
}