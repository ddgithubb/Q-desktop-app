import { listen } from '@tauri-apps/api/event';
import { AddFileOffersAction, AddNodeAction, AppendMessageAction, CompleteDownloadAction, InitFileSeedersAction, LatestMessagesAction, InitPoolAction, RemoveDownloadAction, RemoveFileOfferAction, RemoveNodeAction, RemoveUserAction, UpdateConnectionStateAction, AddUserAction, SetPoolsAction } from '../store/slices/pool.action';
import { poolAction, setMaxFeedSize } from '../store/slices/pool.slice';
import { profileAction, ProfileState } from '../store/slices/profile.slice';
import { PoolStore, store } from '../store/store';
import { PoolConnectionState } from '../types/pool.model';
import { IPCAddPoolFileOffers, IPCAddPoolNode, IPCAppendPoolMessage, IPCCompletePoolFileDownload, IPCInitPool, IPCInitPoolFileSeeders, IPCLatestPoolMessages, IPCReconnectPool, IPCRemovePoolFileOffer, IPCRemovePoolNode, IPCRemovePoolUser, IPCAddPoolUser, IPCStateUpdate, IPCInitApp, IPCRefreshAuthToken } from './ipc';
import { Backend } from './global';
import { setAuthToken } from '../api/api';

const STATE_UPDATE_EVENT: string = "state-update";
const REFRESH_AUTH_TOKEN_EVENT: string = "refresh-auth-token";

const INIT_APP_EVENT: string = "init-app";

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

listen(REFRESH_AUTH_TOKEN_EVENT, (event) => {
    let refreshAuthToken: IPCRefreshAuthToken = event.payload as any;
    setAuthToken(refreshAuthToken.auth_token);
});

listen(INIT_APP_EVENT, (event) => {
    let initApp: IPCInitApp = event.payload as any;
    let profileState: ProfileState = {
        registered: initApp.registered,
        userInfo: initApp.user_info,
        device: initApp.device,
    };
    store.dispatch(profileAction.initProfile(profileState));

    let setPoolsAction: SetPoolsAction = {
        poolInfos: initApp.pools,
    }
    store.dispatch(poolAction.setPools(setPoolsAction));
});

listen(INIT_POOL_EVENT, (event) => {
    let initPool: IPCInitPool = event.payload as any;
    let initPoolAction: InitPoolAction = {
        poolID: initPool.pool_info.poolId,
        initPool,
    };
    store.dispatch(poolAction.initPool(initPoolAction));
});

listen(RECONNECT_POOL_EVENT, (event) => {
    let reconnectPool: IPCReconnectPool = event.payload as any;
    let updateConnectionStateAction: UpdateConnectionStateAction = {
        poolID: reconnectPool.pool_id,
        state: PoolConnectionState.RECONNECTING,
    };
    store.dispatch(poolAction.updateConnectionState(updateConnectionStateAction));

    Backend.connectToPool(reconnectPool.pool_id, reconnectPool.reauth);
});

listen(ADD_POOL_NODE_EVENT, (event) => {
    let addPoolNode: IPCAddPoolNode = event.payload as any;
    let addNodeAction: AddNodeAction = {
        poolID: addPoolNode.pool_id,
        node: addPoolNode.node,
    };
    store.dispatch(poolAction.addNode(addNodeAction));
});

listen(REMOVE_POOL_NODE_EVENT, (event) => {
    let removePoolNode: IPCRemovePoolNode = event.payload as any;
    let removeNodeAction: RemoveNodeAction = {
        poolID: removePoolNode.pool_id,
        nodeID: removePoolNode.node_id,
    };
    store.dispatch(poolAction.removeNode(removeNodeAction));
});

listen(ADD_POOL_USER_EVENT, (event) => {
    let addPoolUser: IPCAddPoolUser = event.payload as any;
    let addUserAction: AddUserAction = {
        poolID: addPoolUser.pool_id,
        userInfo: addPoolUser.user_info,
    };
    store.dispatch(poolAction.addUser(addUserAction));
});

listen(REMOVE_POOL_USER_EVENT, (event) => {
    let removePoolUser: IPCRemovePoolUser = event.payload as any;
    let removeUserAction: RemoveUserAction = {
        poolID: removePoolUser.pool_id,
        userID: removePoolUser.user_id,
    };
    store.dispatch(poolAction.removeUser(removeUserAction));
});

listen(ADD_POOL_FILE_OFFERS_EVENT, (event) => {
    let addFileOffers: IPCAddPoolFileOffers = event.payload as any;
    let addFileOffersAction: AddFileOffersAction = {
        poolID: addFileOffers.pool_id,
        nodeID: addFileOffers.node_id,
        fileInfos: addFileOffers.file_offers,
    };
    store.dispatch(poolAction.addFileOffers(addFileOffersAction));
});

listen(REMOVE_POOL_FILE_OFFER_EVENT, (event) => {
    let removeFileOffer: IPCRemovePoolFileOffer = event.payload as any;
    let removeFileOfferAction: RemoveFileOfferAction = {
        poolID: removeFileOffer.pool_id,
        nodeID: removeFileOffer.node_id,
        fileID: removeFileOffer.file_id,
    };
    store.dispatch(poolAction.removeFileOffer(removeFileOfferAction));
});

listen(INIT_POOL_FILE_SEEDERS_EVENT, (event) => {
    let initFileSeeders: IPCInitPoolFileSeeders = event.payload as any;
    let InitFileSeedersAction: InitFileSeedersAction = {
        poolID: initFileSeeders.pool_id,
        fileSeeders: initFileSeeders.file_seeders,
    };
    store.dispatch(poolAction.initFileSeeders(InitFileSeedersAction));
});

listen(COMPLETE_POOL_FILE_DOWNLOAD_EVENT, (event) => {
    let completeFileDownload: IPCCompletePoolFileDownload = event.payload as any;
    let completeDownloadAction: CompleteDownloadAction = {
        poolID: completeFileDownload.pool_id,
        fileID: completeFileDownload.file_id,
        success: completeFileDownload.success,
    };
    store.dispatch(poolAction.completeDownload(completeDownloadAction));
});

listen(LATEST_POOL_MESSAGES_EVENT, (event) => {
    let latestMessages: IPCLatestPoolMessages = event.payload as any;
    setMaxFeedSize(latestMessages.max_messages_render);
    let latestMessagesAction: LatestMessagesAction = {
        poolID: latestMessages.pool_id,
        messages: latestMessages.messages,
    };
    store.dispatch(poolAction.latestMessages(latestMessagesAction));
});

listen(APPEND_POOL_MESSAGE_EVENT, (event) => {
    let appendMessage: IPCAppendPoolMessage = event.payload as any;
    let appendMessageAction: AppendMessageAction = {
        poolID: appendMessage.pool_id,
        message: appendMessage.message,
    };
    store.dispatch(poolAction.appendMessage(appendMessageAction));
});