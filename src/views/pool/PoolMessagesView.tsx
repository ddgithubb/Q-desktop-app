import { useState, useRef, useMemo, useEffect, memo } from 'react';
import sanitizeHtml from 'sanitize-html';
import { fileSizeToString } from '../../utils/file-size';
import { formatDate, formatTime, minutesToMillisecond } from '../../utils/time';
// import { PoolFileInfo, PoolMessageType, PoolMessage, PoolNodeState, PoolUpdateNodeState, PoolFileOffer, PoolImageOffer } from '../../pool/pool.model';
import './PoolMessagesView.css';

import DownloadIcon from '../../assets/download.png';
import ArrowIcon from '../../assets/arrow.png';
import FileIcon from '../../assets/file.png';
import { PoolMessage, PoolFileInfo, PoolMessage_Type, PoolMediaType, PoolImageData, PoolMessage_MediaOfferData } from '../../types/pool.v1';
import { FeedMessage, HistoryFeed, NodeStatus, PoolFileDownload, PoolNodeStatus, PoolUserStatus, UserStatus } from '../../types/pool.model';
import { Backend, getTempAssetURL } from '../../backend/global';
import { PoolStore, store } from '../../store/store';
import { motion } from 'framer-motion';
import { poolAction } from '../../store/slices/pool.slice';

const CONSISTENT_MESSAGE_INTERVAL: number = minutesToMillisecond(5);

export interface PoolMessagesViewParams {
    poolID: string;
    poolKey: number;
    feed: FeedMessage[];
    historyFeed?: HistoryFeed;
}

export const PoolMessagesView = memo(PoolMessagesViewComponent);

function PoolMessagesViewComponent({ poolID, poolKey, feed, historyFeed }: PoolMessagesViewParams) {

    const [messagesElement, setMessagesElement] = useState<HTMLDivElement | null>(null);
    const [atNewestMessage, setAtNewestMessage] = useState<boolean>(true);

    const atTopThreshold = useRef<boolean>(false);
    const atBottomThreshold = useRef<boolean>(false);
    const [ hasMoreTop, setHasMoreTop ] = useState<boolean>(false);
    const [ hasMoreBottom, setHasMoreBottom ] = useState<boolean>(false);

    const feedView = useMemo(() => {
        // console.log("Feed View:", historyFeed, feed);
        if (historyFeed) {
            return historyFeed.feed;
        }
        return feed;
    }, [feed, historyFeed]);

    useEffect(() => {
        if (!messagesElement) return;
        if (atNewestMessage) {
            messagesElement.lastElementChild?.scrollIntoView();
        }
    }, [feedView]);

    const onMessagesScroll = (e: React.UIEvent<HTMLDivElement, UIEvent>) => {
        // console.log("SCROLLING", e.currentTarget.scrollTop, e.currentTarget.offsetHeight, e.currentTarget.scrollHeight);
        let scrollOffsetThreshold = window.innerHeight;
        if (e.currentTarget.scrollTop + e.currentTarget.offsetHeight >= e.currentTarget.scrollHeight - scrollOffsetThreshold) {
            if (!atBottomThreshold.current) {
                atBottomThreshold.current = true;
                Backend.requestMessageHistory(poolID, true).then((hasMore) => {
                    console.log("Has more bottom:", hasMore);
                    setHasMoreBottom(hasMore);

                    if (hasMore) {
                        setHasMoreTop(true);
                    }
                });
            }
        } else if (atBottomThreshold.current) {
            atBottomThreshold.current = false;
        } else {
            if (e.currentTarget.scrollTop <= scrollOffsetThreshold) {
                if (!atTopThreshold.current) {
                    atTopThreshold.current = true;
                    Backend.requestMessageHistory(poolID, false).then((hasMore) => {
                        console.log("Has more top:", hasMore);
                        setHasMoreTop(hasMore);

                        if (hasMore) {
                            setHasMoreBottom(true);
                        }
                    });
                }
            } else if (atTopThreshold.current) {
                atTopThreshold.current = false;
            }
        }

        if (!hasMoreBottom) {
            if (e.currentTarget.scrollTop + e.currentTarget.offsetHeight + 10 >= e.currentTarget.scrollHeight) {
                if (!atNewestMessage) {
                    console.log("AT LATEST");
                    setAtNewestMessage(true);
                }
            } else {
                if (atNewestMessage) {
                    console.log("NOT AT LATEST");
                    setAtNewestMessage(false);
                }
            }
        }
    }

    const downloadFile = (fileInfo: PoolFileInfo) => {
        Backend.downloadFile(poolID, fileInfo);
    }

    const goToLatest = () => {
        if (historyFeed) {
            setAtNewestMessage(true);
            store.dispatch(poolAction.switchToLatestFeed({ key: poolKey }));
        } else {
            messagesElement!.lastElementChild?.scrollIntoView();
        }
    }

    return (
        <div className="pool-messages-container" ref={(e) => setMessagesElement(e)} onScroll={onMessagesScroll}>
            {
                !atNewestMessage ? (
                    <motion.div 
                        className="pool-message-latest-container"
                        initial={{ y: -20, opacity: 0 }}
                        animate={{ y: 0, opacity: 1 }}
                        transition={{ type: "spring", duration: 0.2 }} 
                        onClick={goToLatest}
                    >
                        <img src={ArrowIcon} className="pool-message-latest-arrow" />
                    </motion.div>
                ) : null
            }
            <div className="pool-start-spacer">
                {
                    hasMoreTop ? (
                        <div className="pool-message-status">Retrieving History...</div>
                    ) : (
                        <div className="pool-message-status">No saved messages beyond this point</div>
                    )
                }
            </div>
            {
                feedView.map((feedMsg, index) => {
                    if (feedMsg.msg) {
                        let messageContentElement: JSX.Element = <></>;
                        let msg = feedMsg.msg;
                        let data = msg.data as any;

                        switch (msg.type) {
                            case PoolMessage_Type.TEXT:
                                if (!data.textData) return;
                                let text: string = data.textData.text;
                                messageContentElement =
                                    <div className="pool-message-text" dangerouslySetInnerHTML={{ __html: sanitizeHtml(text, { allowedTags: ['br'] }) }} />
                                break;
                            case PoolMessage_Type.FILE_OFFER:
                                if (!data.fileOfferData) return;
                                let fileInfo: PoolFileInfo = data.fileOfferData;
                                messageContentElement =
                                    <div className="pool-message-file-data-container">
                                        <div className="pool-message-file-container elipsify-container" onClick={() => downloadFile(fileInfo)}>
                                            <img src={FileIcon} width={35} height={35} />
                                            <span className="pool-message-file-name elipsify-content">{fileInfo.fileName}</span>
                                            <span className="pool-message-file-size elipsify-extra">{fileSizeToString(fileInfo.totalSize)}</span>
                                        </div>
                                    </div>
                                break;
                            case PoolMessage_Type.MEDIA_OFFER:
                                if (!data.mediaOfferData) return;
                                let mediaOfferData: PoolMessage_MediaOfferData = data.mediaOfferData;
                                let mediaData = mediaOfferData.mediaData as any;
                                switch (mediaOfferData.mediaType) {
                                    case PoolMediaType.IMAGE:
                                        if (!mediaData.imageData || !mediaOfferData.fileInfo) return;
                                        let imageData: PoolImageData = mediaData.imageData;
                                        let fileInfo: PoolFileInfo = mediaOfferData.fileInfo;
                                        messageContentElement =
                                            <>
                                                <AsyncImage poolID={poolID} fileInfo={fileInfo} imageData={imageData} />
                                                <div className="pool-message-image-download-container">
                                                    <img className="pool-message-image-download-icon" src={DownloadIcon} />
                                                    <div className="pool-message-image-download-filename" onClick={() => downloadFile(fileInfo)}>
                                                        {fileInfo.fileName} {" (" + fileSizeToString(fileInfo.totalSize) + ")"}
                                                    </div>
                                                </div>
                                            </>
                                        break;
                                    default:
                                        return;
                                }
                                break;
                            default:
                                return;
                        }

                        let hasHeader = (
                            index == 0 ||
                            !feedView[index - 1].msg ||
                            msg.userId != feedView[index - 1].msg!.userId ||  // should be deviceID
                            msg.created - feedView[index - 1].msg!.created > CONSISTENT_MESSAGE_INTERVAL
                        )

                        return (
                            <div className={"pool-message-container" + (hasHeader ? " pool-header-spacer" : "")} key={msg.msgId}>
                                {
                                    hasHeader ? (
                                        <HeaderComponent displayName={PoolStore.getDisplayName(msg.userId)} msg={msg} />
                                    ) : (
                                        <div className="pool-message-date pool-message-portable-date">{formatTime(msg.created)}</div>
                                    )
                                }
                                {messageContentElement}
                            </div>
                        )

                    } else if (feedMsg.nodeStatus) {
                        let nodeStatus: PoolNodeStatus = feedMsg.nodeStatus;
                            
                        return (
                            <div className={"pool-message-container"} key={nodeStatus.nodeID + nodeStatus.status.toString() + nodeStatus.created.toString()}>
                                <div className="pool-message-date pool-message-portable-date">{formatTime(nodeStatus.created)}</div>
                                <NodeStatusComponent nodeStatus={nodeStatus} />
                            </div>
                        )
                    } else if (feedMsg.userStatus) {
                        let userStatus: PoolUserStatus = feedMsg.userStatus;

                        return (
                            <div className={"pool-message-container"} key={userStatus.userID + userStatus.status.toString() + userStatus.created.toString()}>
                                <div className="pool-message-date pool-message-portable-date">{formatTime(userStatus.created)}</div>
                                <UserStatusComponent displayName={PoolStore.getDisplayName(userStatus.userID)} userStatus={userStatus} />
                            </div>
                        )
                    }
                })
            }
        </div>
    )
}

const HeaderComponent = memo(({ displayName, msg }: { displayName: string | undefined, msg: PoolMessage }) => (
    <div className="pool-message-info-bar elipsify-container">
        <div className="pool-message-name elipsify-content">
            {displayName || msg.userId}
        </div>
        <div className="pool-message-date elipsify-extra">
            {formatDate(msg.created)}
        </div>
    </div>
), (prev, next) => {
    return !next.displayName || prev.displayName == next.displayName
});

const NodeStatusComponent = memo(({ nodeStatus }: { nodeStatus: PoolNodeStatus }) => (
    <div className="pool-message-node-status">
        Device {nodeStatus.nodeID} {"(" + nodeStatus.nodeID + ")"} has {nodeStatus.status == NodeStatus.ACTIVE ? "joined" : "left"}
    </div>
), (prev, next) => {
    return !next.nodeStatus.nodeID || prev.nodeStatus.nodeID == next.nodeStatus.nodeID;
});

const UserStatusComponent = memo(({ displayName, userStatus }: { displayName: string | undefined, userStatus: PoolUserStatus }) => (
    <div className="pool-message-node-status">
        User {displayName} {"(" + userStatus.userID + ")"} has {userStatus.status == UserStatus.JOINED ? "joined" : "left"} the pool
    </div>
), (prev, next) => {
    return !next.displayName || prev.displayName == next.displayName;
});

const AsyncImage = memo(({ poolID, fileInfo, imageData }: { poolID: string, fileInfo: PoolFileInfo, imageData: PoolImageData }) => {

    let [requesting, setRequesting] = useState<boolean>(false);
    let [src, setSrc] = useState<string>("");

    useEffect(() => {
        updateSrc();
    }, []);

    useEffect(() => {
        let callback = (success: boolean) => {
            console.log("File complete success: ", success);
            if (success) {
                updateSrc();
            }
            setRequesting(false);
        };
        PoolStore.completedDownloadEvents.once(fileInfo.fileId, callback);
        return () => {
            PoolStore.completedDownloadEvents.removeListener(fileInfo.fileId, callback);
        }
    });

    const hasMedia = (): boolean => {
        return src != "";
    }

    const updateSrc = async () => {
        try {
            let src = await getTempAssetURL(poolID, fileInfo.fileId);
            // console.log("Media Source: ", src);
            setSrc(src);
            setRequesting(false);
        } catch (e) {
            setSrc("");
        }
    }

    const requestImage = () => {
        if (requesting) return;
        Backend.downloadFile(poolID, fileInfo, true).then((success) => {
            if (success) {
                setRequesting(true);
            }
        });
    }

    return (
        <div className="pool-message-image-container">
            <div className="pool-message-image-sub-container">
                <img
                    className={"pool-message-image" + (!hasMedia() ? " pool-message-image-preview-blur" : "")}
                    src={src == "" ? imageData.previewImageBase64 : src}
                    onError={() => setSrc("")}
                    height={Math.min(400, (imageData.height / imageData.width) * Math.min(400, imageData.width, window.innerWidth - 80))} />
                {
                    !hasMedia() ? (
                        <div className="pool-message-image-missing-container" onClick={() => requestImage()}>
                            {requesting ? "Requesting..." : "Request Image"}
                        </div>
                    ) : undefined
                }
            </div>
        </div>
    )
});