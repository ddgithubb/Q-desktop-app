import { listen } from '@tauri-apps/api/event';
import { AddDownloadAction, AddFileOffersAction, AddNodeAction, AppendMessageAction, CompleteDownloadAction, InitFileSeedersAction, InitMessageAction, InitPoolAction, RemoveDownloadAction, RemoveFileOfferAction, RemoveNodeAction, RemoveUserAction, UpdateConnectionStateAction, UpdateUserAction } from '../store/slices/pool.action';
import { poolAction } from '../store/slices/pool.slice';
import { profileAction, ProfileState } from '../store/slices/profile.slice';
import { store } from '../store/store';
import { PoolConnectionState } from '../types/pool.model';
import { IPCAddPoolFileDownload, IPCAddPoolFileOffers, IPCAddPoolNode, IPCAppendPoolMessage, IPCCompletePoolFileDownload, IPCInitPool, IPCInitPoolFileSeeders, IPCInitPoolMessages, IPCInitProfile, IPCReconnectPool, IPCRemovePoolFileOffer, IPCRemovePoolNode, IPCRemovePoolUser, IPCUpdatePoolUser } from './backend.model';
import { Backend } from './global';

export const STATE_UPDATE_EVENT: string = "state-update";

const INIT_PROFILE_EVENT: string = "init-profile";

const INIT_POOL_EVENT: string = "init-pool";
const RECONNECT_POOL_EVENT: string = "reconnect-pool";
const ADD_POOL_NODE_EVENT: string = "add-pool-node";
const REMOVE_POOL_NODE_EVENT: string = "remove-pool-node";
const UPDATE_POOL_USER_EVENT: string = "update-pool-user";
const REMOVE_POOL_USER_EVENT: string = "remove-pool-user";

const ADD_POOL_FILE_OFFERS_EVENT: string = "add-pool-file-offers";
const REMOVE_POOL_FILE_OFFER_EVENT: string = "remove-pool-file-offer";
const INIT_POOL_FILE_SEEDERS_EVENT: string = "init-pool-file-seeders";

const ADD_FILE_DOWNLOAD_EVENT: string = "add-file-download-event";
const COMPLETE_POOL_FILE_DOWNLOAD_EVENT: string = "complete-pool-file-download";

const INIT_POOL_MESSAGES_EVENT: string = "init-pool-messages";
const APPEND_POOL_MESSAGE_EVENT: string = "append-pool-message";

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
    if (!key) return;

    let initPoolAction: InitPoolAction = {
        key,
        initPool,
    };
    store.dispatch(poolAction.initPool(initPoolAction));
});

listen(RECONNECT_POOL_EVENT, (event) => {
    let reconnectPool: IPCReconnectPool = event.payload as any;
    let key = Backend.getPoolKey(reconnectPool.pool_id);
    if (!key) return;

    let updateConnectionStateAction: UpdateConnectionStateAction = {
        key,
        state: PoolConnectionState.RECONNECTING,
    };
    store.dispatch(poolAction.updateConnectionState(updateConnectionStateAction));
});

listen(ADD_POOL_NODE_EVENT, (event) => {
    let addPoolNode: IPCAddPoolNode = event.payload as any;
    let key = Backend.getPoolKey(addPoolNode.pool_id);
    if (!key) return;
    
    let addNodeAction: AddNodeAction = {
        key,
        node: {
            nodeID: addPoolNode.node.node_id,
            userID: addPoolNode.node.user_id,
            fileOffers: [],
        },
    };
    store.dispatch(poolAction.addNode(addNodeAction));
});

listen(REMOVE_POOL_NODE_EVENT, (event) => {
    let removePoolNode: IPCRemovePoolNode = event.payload as any;
    let key = Backend.getPoolKey(removePoolNode.pool_id);
    if (!key) return;

    let removeNodeAction: RemoveNodeAction = {
        key,
        nodeID: removePoolNode.node_id,
    };
    store.dispatch(poolAction.removeNode(removeNodeAction));
});

listen(UPDATE_POOL_USER_EVENT, (event) => {
    let updatePoolUser: IPCUpdatePoolUser = event.payload as any;
    let key = Backend.getPoolKey(updatePoolUser.pool_id);
    if (!key) return;

    let updateUserAction: UpdateUserAction = {
        key,
        userInfo: updatePoolUser.user_info,
    };
    store.dispatch(poolAction.updateUser(updateUserAction));
});

listen(REMOVE_POOL_USER_EVENT, (event) => {
    let removePoolUser: IPCRemovePoolUser = event.payload as any;
    let key = Backend.getPoolKey(removePoolUser.pool_id);
    if (!key) return;

    let removeUserAction: RemoveUserAction = {
        key,
        userID: removePoolUser.user_id,
    };
    store.dispatch(poolAction.removeUser(removeUserAction));
});

listen(ADD_POOL_FILE_OFFERS_EVENT, (event) => {
    let addFileOffers: IPCAddPoolFileOffers = event.payload as any;
    let key = Backend.getPoolKey(addFileOffers.pool_id);
    if (!key) return;

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
    if (!key) return;

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
    if (!key) return;

    let InitFileSeedersAction: InitFileSeedersAction = {
        key,
        fileSeeders: initFileSeeders.file_seeders,
    };
    store.dispatch(poolAction.initFileSeeders(InitFileSeedersAction));
});

listen(ADD_FILE_DOWNLOAD_EVENT, (event) => {
    let addFileDownload: IPCAddPoolFileDownload = event.payload as any;
    let key = Backend.getPoolKey(addFileDownload.pool_id);
    if (!key) return;

    let addDownloadAction: AddDownloadAction = {
        key,
        fileInfo: addFileDownload.file_info,
    };
    store.dispatch(poolAction.addDownload(addDownloadAction));
});

listen(COMPLETE_POOL_FILE_DOWNLOAD_EVENT, (event) => {
    let completeFileDownload: IPCCompletePoolFileDownload = event.payload as any;
    let key = Backend.getPoolKey(completeFileDownload.pool_id);
    if (!key) return;

    let completeDownloadAction: CompleteDownloadAction = {
        key,
        fileID: completeFileDownload.file_id,
        success: completeFileDownload.success,
    };
    store.dispatch(poolAction.completeDownload(completeDownloadAction));

    Backend.events.emit(completeFileDownload.file_id, completeFileDownload.success);
});

listen(INIT_POOL_MESSAGES_EVENT, (event) => {
    let initMessages: IPCInitPoolMessages = event.payload as any;
    let key = Backend.getPoolKey(initMessages.pool_id);
    if (!key) return;

    let initMessagesAction: InitMessageAction = {
        key,
        messages: initMessages.messages,
    };
    store.dispatch(poolAction.initMessages(initMessagesAction));
});

listen(APPEND_POOL_MESSAGE_EVENT, (event) => {
    let appendMessage: IPCAppendPoolMessage = event.payload as any;
    let key = Backend.getPoolKey(appendMessage.pool_id);
    if (!key) return;

    let appendMessageAction: AppendMessageAction = {
        key,
        message: appendMessage.message,
    };
    store.dispatch(poolAction.appendMessage(appendMessageAction));
});