
import { useState, useRef, memo, useEffect } from 'react'
import { CircularProgressbar } from 'react-circular-progressbar'
import { fileSizeToString } from '../../utils/file-size'
import { Pool, PoolConnectionState, DownloadProgressStatus, PoolFileDownload } from '../../types/pool.model'
import { IndicatorDot } from '../components/IndicatorDot'
import { PoolDisplayUsersView } from './PoolDisplayUsersView'
import { PoolMessageMode } from './PoolView'

import './PoolDisplayView.css'
import AddIcon from '../../assets/add.png'
import AddImageIcon from '../../assets/add-image.png'
import FileIcon from '../../assets/file.png'
import CancelIcon from '../../assets/trash.png'
import SendIcon from '../../assets/send.png'
import { Backend } from '../../backend/global'
import { PoolStore } from '../../store/store'

export interface PoolDisplayViewParams {
    pool: Pool;
    messageMode: PoolMessageMode;
}

export const PoolDisplayView = memo(PoolDisplayViewComponent);

function PoolDisplayViewComponent({ pool, messageMode }: PoolDisplayViewParams) {

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
                                {PoolStore.getActiveDevicesCount(pool.poolID)} Device{(PoolStore.getActiveDevicesCount(pool.poolID)) == 1 ? "" : "s"} Active
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
                        {pool.users.length} Total User{pool.users.length > 1 ? "s" : ""}
                    </div>
                </div>
            </div>
            <div aria-hidden={messageMode != PoolMessageMode.TEXT} className="display-toggle-hide display-component-container display-message-input">
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
            <div aria-hidden={messageMode != PoolMessageMode.FILE} className="display-toggle-hide display-component-container display-files-container">
                <div className="display-file-container display-file-container-add-button" onClick={addFile}>
                    <img src={AddIcon} height={28} width={28} />
                    Add File
                </div>
                {
                    pool.fileOffers.map((fileInfo) => (
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
            <div aria-hidden={messageMode != PoolMessageMode.AVAILABLE_FILES} className="display-toggle-hide display-component-container display-available-files">
                {
                    pool.availableFiles.length ? 
                        pool.availableFiles.map((file) => (
                            <div className="display-available-file-offer" key={file.fileInfo!.fileId}>
                                <span className="display-available-file-name" onClick={() => Backend.downloadFile(pool.poolID, file.fileInfo!)}>{file.fileInfo!.fileName}</span> 
                                <span className="display-available-file-size">{fileSizeToString(file.fileInfo!.totalSize)}</span>
                                <span className="display-available-file-seeders-count">{file.seederNodeIds.length} seeder{file.seederNodeIds.length > 1 ? "s" : ""}</span>
                            </div>
                        ))
                    : (
                        <div className="display-no-available-files">
                            No available files
                        </div>
                    )
                }
            </div>
            <PoolDisplayUsersView hidden={messageMode != PoolMessageMode.USERS} poolID={pool.poolID} users={pool.users} />
        </div>
    )
}

function DownloadQueue({ poolID, downloadQueue }: { poolID: string, downloadQueue: PoolFileDownload[] }) {

    const refreshTimer = useRef<NodeJS.Timer | undefined>(undefined);
    const [, updateState] = useState<{}>();

    useEffect(() => {
        if (downloadQueue.length != 0) {
            if (refreshTimer.current == undefined) {
                refreshTimer.current = setInterval(() => {
                    updateState({});
                }, 500);
            }
        } else {
            clearInterval(refreshTimer.current);
            refreshTimer.current = undefined;
        }
    }, [downloadQueue]);

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
                                    value={fileDownload.status == DownloadProgressStatus.SUCCESS ? 100 : PoolStore.getDownloadProgress(fileDownload.fileInfo.fileId)}
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