import { listen } from '@tauri-apps/api/event';
import { AddFileOffersAction, AddNodeAction, AppendMessageAction, CompleteDownloadAction, InitFileSeedersAction, LatestMessagesAction, InitPoolAction, RemoveDownloadAction, RemoveFileOfferAction, RemoveNodeAction, RemoveUserAction, UpdateConnectionStateAction, AddUserAction } from '../store/slices/pool.action';
import { poolAction, setMaxFeedSize } from '../store/slices/pool.slice';
import { profileAction, ProfileState } from '../store/slices/profile.slice';
import { PoolStore, store } from '../store/store';
import { PoolConnectionState } from '../types/pool.model';
import { IPCAddPoolFileOffers, IPCAddPoolNode, IPCAppendPoolMessage, IPCCompletePoolFileDownload, IPCInitPool, IPCInitPoolFileSeeders, IPCLatestPoolMessages, IPCInitProfile, IPCReconnectPool, IPCRemovePoolFileOffer, IPCRemovePoolNode, IPCRemovePoolUser, IPCAddPoolUser, IPCStateUpdate } from './backend.model';
import { Backend } from './global';

const STATE_UPDATE_EVENT: string = "state-update";

const INIT_PROFILE_EVENT: string = "init-profile";

const INIT_POOL_EVENT: string = "init-pool";
const RECONNECT_POOL_EVENT: string = "reconnect-pool";
const ADD_POOL_NODE_EVENT: string = "add-pool-node";
const REMOVE_POOL_NODE_EVENT: string = "remove-pool-node";
const ADD_POOL_USER_EVENT: string = "add-pool-user";
const REMOVE_POOL_USER_EVENT: string = "remove-pool-user";

const ADD_POOL_FILE_OFFERS_EVENT: string = "add-pool-file-offers";
const REMOVE_POOL_FILE_OFFER_EVENT: string = "remove-pool-file-offer";
const INIT_POOL_FILE_SEEDERS_EVENT: string = "init-pool-file-seeders";

const COMPLETE_POOL_FILE_DOWNLOAD_EVENT: string = "complete-pool-file-download";

const LATEST_POOL_MESSAGES_EVENT: string = "latest-pool-messages";
const APPEND_POOL_MESSAGE_EVENT: string = "append-pool-message";

listen(STATE_UPDATE_EVENT, (event) => {
    let state: IPCStateUpdate = event.payload as any;
    PoolStore.updateDownloadProgress(state.file_downloads_progress);
});

listen(INIT_PROFILE_EVENT, (event) => {
    let initProfile: IPCInitProfile = event.payload as any;
    let profileState: ProfileState = {
        userInfo: initProfile.user_info,
        device: initProfile.device,
    };
    store.dispatch(profileAction.initProfile(profileState));
});

listen(INIT_POOL_EVENT, (event) => {
    let initPool: IPCInitPool = event.payload as any;
    let key = Backend.getPoolKey(initPool.pool_info.poolId);
    if (key == undefined) return;

    let initPoolAction: InitPoolAction = {
        key,
        initPool,
    };
    store.dispatch(poolAction.initPool(initPoolAction));
});

listen(RECONNECT_POOL_EVENT, (event) => {
    let reconnectPool: IPCReconnectPool = event.payload as any;
    let key = Backend.getPoolKey(reconnectPool.pool_id);
    if (key == undefined) return;

    let updateConnectionStateAction: UpdateConnectionStateAction = {
        key,
        state: PoolConnectionState.RECONNECTING,
    };
    store.dispatch(poolAction.updateConnectionState(updateConnectionStateAction));
});

listen(ADD_POOL_NODE_EVENT, (event) => {
    let addPoolNode: IPCAddPoolNode = event.payload as any;
    let key = Backend.getPoolKey(addPoolNode.pool_id);
    if (key == undefined) return;
    
    let addNodeAction: AddNodeAction = {
        key,
        nodeID: addPoolNode.node.node_id,
        userID: addPoolNode.node.user_id,
    };
    store.dispatch(poolAction.addNode(addNodeAction));
});

listen(REMOVE_POOL_NODE_EVENT, (event) => {
    let removePoolNode: IPCRemovePoolNode = event.payload as any;
    let key = Backend.getPoolKey(removePoolNode.pool_id);
    if (key == undefined) return;

    let removeNodeAction: RemoveNodeAction = {
        key,
        nodeID: removePoolNode.node_id,
    };
    store.dispatch(poolAction.removeNode(removeNodeAction));
});

listen(ADD_POOL_USER_EVENT, (event) => {
    let addPoolUser: IPCAddPoolUser = event.payload as any;
    let key = Backend.getPoolKey(addPoolUser.pool_id);
    if (key == undefined) return;

    let addUserAction: AddUserAction = {
        key,
        userInfo: addPoolUser.user_info,
    };
    store.dispatch(poolAction.addUser(addUserAction));
});

listen(REMOVE_POOL_USER_EVENT, (event) => {
    let removePoolUser: IPCRemovePoolUser = event.payload as any;
    let key = Backend.getPoolKey(removePoolUser.pool_id);
    if (key == undefined) return;

    let removeUserAction: RemoveUserAction = {
        key,
        userID: removePoolUser.user_id,
    };
    store.dispatch(poolAction.removeUser(removeUserAction));
});

listen(ADD_POOL_FILE_OFFERS_EVENT, (event) => {
    let addFileOffers: IPCAddPoolFileOffers = event.payload as any;
    let key = Backend.getPoolKey(addFileOffers.pool_id);
    if (key == undefined) return;

    let addFileOffersAction: AddFileOffersAction = {
        key,
        nodeID: addFileOffers.node_id,
        fileInfos: addFileOffers.file_offers,
    };
    store.dispatch(poolAction.addFileOffers(addFileOffersAction));
});

listen(REMOVE_POOL_FILE_OFFER_EVENT, (event) => {
    let removeFileOffer: IPCRemovePoolFileOffer = event.payload as any;
    let key = Backend.getPoolKey(removeFileOffer.pool_id);
    if (key == undefined) return;

    let removeFileOfferAction: RemoveFileOfferAction = {
        key,
        nodeID: removeFileOffer.node_id,
        fileID: removeFileOffer.file_id,
    };
    store.dispatch(poolAction.removeFileOffer(removeFileOfferAction));
});

listen(INIT_POOL_FILE_SEEDERS_EVENT, (event) => {
    let initFileSeeders: IPCInitPoolFileSeeders = event.payload as any;
    let key = Backend.getPoolKey(initFileSeeders.pool_id);
    if (key == undefined) return;

    let InitFileSeedersAction: InitFileSeedersAction = {
        key,
        fileSeeders: initFileSeeders.file_seeders,
    };
    store.dispatch(poolAction.initFileSeeders(InitFileSeedersAction));
});

listen(COMPLETE_POOL_FILE_DOWNLOAD_EVENT, (event) => {
    let completeFileDownload: IPCCompletePoolFileDownload = event.payload as any;
    let key = Backend.getPoolKey(completeFileDownload.pool_id);
    if (key == undefined) return;

    let completeDownloadAction: CompleteDownloadAction = {
        key,
        fileID: completeFileDownload.file_id,
        success: completeFileDownload.success,
    };
    store.dispatch(poolAction.completeDownload(completeDownloadAction));
});

listen(LATEST_POOL_MESSAGES_EVENT, (event) => {
    let latestMessages: IPCLatestPoolMessages = event.payload as any;
    let key = Backend.getPoolKey(latestMessages.pool_id);
    if (key == undefined) return;

    setMaxFeedSize(latestMessages.max_messages_render);
    let latestMessagesAction: LatestMessagesAction = {
        key,
        messages: latestMessages.messages,
    };
    store.dispatch(poolAction.latestMessages(latestMessagesAction));
});

listen(APPEND_POOL_MESSAGE_EVENT, (event) => {
    let appendMessage: IPCAppendPoolMessage = event.payload as any;
    let key = Backend.getPoolKey(appendMessage.pool_id);
    if (key == undefined) return;

    let appendMessageAction: AppendMessageAction = {
        key,
        message: appendMessage.message,
    };
    store.dispatch(poolAction.appendMessage(appendMessageAction));
});