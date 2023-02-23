import { useState, useRef, useMemo, useEffect, memo } from 'react';
import sanitizeHtml from 'sanitize-html';
import { fileSizeToString } from '../../utils/file-size';
import { formatDate, formatTime, minutesToMillisecond } from '../../utils/time';
// import { PoolFileInfo, PoolMessageType, PoolMessage, PoolNodeState, PoolUpdateNodeState, PoolFileOffer, PoolImageOffer } from '../../pool/pool.model';
import './PoolMessagesView.css';

import DownloadIcon from '../../assets/download.png';
import FileIcon from '../../assets/file.png';
import { PoolMessage, PoolFileInfo, PoolMessage_Type, PoolMediaType, PoolImageData, PoolMessage_MediaOfferData } from '../../types/pool.v1';
import { FeedMessage, NodeStatus, PoolFileDownload, PoolNodeStatus, PoolUserStatus, UserStatus } from '../../types/pool.model';
import { Backend, getTempAssetURL } from '../../backend/global';
import { PoolStore } from '../../store/store';


const CONSISTENT_MESSAGE_INTERVAL: number = minutesToMillisecond(5);
const MIN_MESSAGE_HEIGHT: number = 28;
const MESSAGES_VIEWPORT: number = 2;
const EXTRA_MESSAGES_VIEWPORT: number = 1;
var SCROLL_THRESHOLD_FOR_CONTENT: number;
var MESSAGES_PER_VIEWPORT: number;
var MAIN_MESSAGES_TO_RENDER: number;
var MAX_MESSAGES_TO_RENDER: number;
var EXTRA_MESSAGES_TO_RENDER: number;

const calcMessageBounds = () => {
    MESSAGES_PER_VIEWPORT = window.innerHeight / MIN_MESSAGE_HEIGHT;
    SCROLL_THRESHOLD_FOR_CONTENT = window.innerHeight / 2;
    MAIN_MESSAGES_TO_RENDER = MESSAGES_VIEWPORT * MESSAGES_PER_VIEWPORT
    EXTRA_MESSAGES_TO_RENDER = EXTRA_MESSAGES_VIEWPORT * MESSAGES_PER_VIEWPORT
    MAX_MESSAGES_TO_RENDER = 2 * EXTRA_MESSAGES_TO_RENDER + MAIN_MESSAGES_TO_RENDER;
    //console.log(MESSAGES_PER_VIEWPORT, MAIN_MESSAGES_TO_RENDER, EXTRA_MESSAGES_TO_RENDER, MAX_MESSAGES_TO_RENDER);
};

calcMessageBounds();
window.addEventListener("resize", calcMessageBounds);

export interface PoolMessagesViewParams {
    poolID: string;
    feed: FeedMessage[];
}

export const PoolMessagesView = memo(PoolMessagesViewComponent);

function PoolMessagesViewComponent({ poolID, feed }: PoolMessagesViewParams) {

    const [messagesElement, setMessagesElement] = useState<HTMLDivElement | null>(null);
    const [atNewestMessage, setAtNewestMessage] = useState<boolean>(true);
    const lastFirstMessageElement = useRef<Element | null>();
    const lastFirstMessageScrollTop = useRef<number>(0);
    const lastLastMessageElement = useRef<Element | null>();
    const lastLastMessageScrollTop = useRef<number>(0);

    const [messageIndexThreshold, setMessageIndexThreshold] = useState<number>(MAIN_MESSAGES_TO_RENDER);
    const atTopThreshold = useRef<boolean>(false);
    const atBottomThreshold = useRef<boolean>(false);
    const poolMessagesView = useMemo(() => {
        //console.log(messages.length, messageIndexThreshold, Math.max(0, messages.length - messageIndexThreshold), messages.length - messageIndexThreshold + MAX_MESSAGES_TO_RENDER);
        return feed.slice(Math.max(0, feed.length - messageIndexThreshold), feed.length - messageIndexThreshold + MAX_MESSAGES_TO_RENDER);
    }, [feed, messageIndexThreshold]);

    useEffect(() => {
        adjustScroll();
    }, [poolMessagesView]);

    const adjustScroll = () => {
        if (!messagesElement) return;
        if (atNewestMessage) {
            messagesElement?.scrollTo({ top: messagesElement.scrollHeight + 1000, behavior: "auto" })
        } else {
            let deltaY = 0;
            if (!messagesElement?.contains(lastLastMessageElement.current || null)) {
                deltaY = lastFirstMessageScrollTop.current - (lastFirstMessageElement.current?.scrollTop || 0)
            } else {
                deltaY = (lastLastMessageElement.current?.scrollTop || 0) - lastLastMessageScrollTop.current
            }
            if (deltaY != 0) {
                messagesElement.scrollTo({ top: messagesElement.scrollTop + deltaY });
            }
        }
        lastFirstMessageElement.current = messagesElement.childNodes[1] as Element || null;
        lastFirstMessageScrollTop.current = (messagesElement.childNodes[1] as Element)?.scrollTop || 0;
        lastLastMessageElement.current = messagesElement.lastElementChild;
        lastLastMessageScrollTop.current = messagesElement.lastElementChild?.scrollTop || 0;
    }

    const onMessagesScroll = (e: React.UIEvent<HTMLDivElement, UIEvent>) => {
        //console.log("SCROLLING", e.currentTarget.scrollTop, e.currentTarget.offsetHeight, e.currentTarget.scrollHeight, SCROLL_THRESHOLD_FOR_CONTENT);
        if (e.currentTarget.scrollTop + e.currentTarget.offsetHeight >= e.currentTarget.scrollHeight - SCROLL_THRESHOLD_FOR_CONTENT) {
            if (!atBottomThreshold.current && messageIndexThreshold > MAX_MESSAGES_TO_RENDER) {
                atBottomThreshold.current = true;
                // GET EXTRA BOTTOM (if needed/from indexedDB)
                //console.log("GET EXTRA BOTTOM");
                setMessageIndexThreshold(messageIndexThreshold - EXTRA_MESSAGES_TO_RENDER);
            } else {
                if (e.currentTarget.scrollTop + e.currentTarget.offsetHeight + 10 >= e.currentTarget.scrollHeight) {
                    if (!atNewestMessage) {
                        setAtNewestMessage(true);
                    }
                } else {
                    if (atNewestMessage) {
                        setAtNewestMessage(false);
                    }
                }
            }
        } else if (atBottomThreshold.current) {
            atBottomThreshold.current = false;
        }
        if (e.currentTarget.scrollTop <= SCROLL_THRESHOLD_FOR_CONTENT) {
            if (!atTopThreshold.current) {
                atTopThreshold.current = true;
                // GET EXTRA TOP
                // ONLY if there is extra top, or else don't (SOLUTION RIGHT NOW DOESN"T COUNT FOR THAT)
                // NEGATIVE MESSAGEINDEXTHRESHOLD IS FINE, because there is a Math.max in the slice
                // So the only thing to add if using stored messages, is to have an EXTRA condition if there is extra top
                //console.log(messages.length, messageIndexThreshold);
                //console.log("GET EXTRA TOP", messages.length - messageIndexThreshold, messages.length - (messageIndexThreshold + EXTRA_MESSAGES_TO_RENDER), messages.length - messageIndexThreshold + MAX_MESSAGES_TO_RENDER);
                if (feed.length - messageIndexThreshold >= 0) {
                    setMessageIndexThreshold(messageIndexThreshold + EXTRA_MESSAGES_TO_RENDER);
                }
                //setMessageIndexThreshold(messageIndexThreshold + EXTRA_MESSAGES_TO_RENDER); // maybe should only do that when messages actually render????
            }
        } else if (atTopThreshold.current) {
            atTopThreshold.current = false;
        }
    }

    const downloadFile = (fileInfo: PoolFileInfo) => {
        Backend.downloadFile(poolID, fileInfo);
    }

    return (
        <div className="pool-messages-container" ref={(e) => setMessagesElement(e)} onScroll={onMessagesScroll}>
            <div className="pool-start-spacer">
                <div className="pool-message-status">No saved messages beyond this point</div>
            </div>
            {
                poolMessagesView.map((feedMsg, index) => {
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
                            !poolMessagesView[index - 1].msg ||
                            msg.userId != poolMessagesView[index - 1].msg!.userId ||  // should be deviceID
                            msg.created - poolMessagesView[index - 1].msg!.created > CONSISTENT_MESSAGE_INTERVAL
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
            <div className="pool-end-spacer" />
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

    const hasMedia = (): boolean => {
        return src != "";
    }

    const updateSrc = async () => {
        let src = await getTempAssetURL(poolID, fileInfo.fileId);
        console.log("Media Source: ", src);
        if (src != "") {
            setSrc(src);
            setRequesting(false);
        }
    }

    const requestImage = () => {
        if (requesting) return;
        PoolStore.completedDownloadEvents.once(fileInfo.fileId, (success: boolean) => {
            updateSrc();
            setRequesting(false);
        });
        setRequesting(true);
        Backend.downloadFile(poolID, fileInfo);
    }

    return (
        <div className="pool-message-image-container">
            <div className="pool-message-image-sub-container">
                <img
                    loading="lazy"
                    className={"pool-message-image" + (!hasMedia() ? " pool-message-image-preview-blur" : "")}
                    src={src == "" ? imageData.previewImageBase64 : src}
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