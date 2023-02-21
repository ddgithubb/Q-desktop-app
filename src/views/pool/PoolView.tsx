import React, { memo, useEffect, useMemo, useRef, useState } from 'react';
import { useSelector } from 'react-redux';
import { useNavigate, useParams, useSearchParams } from 'react-router-dom';
import { PoolConnectionState, PoolNode } from '../../types/pool.model';
import { getStoreState, GlobalState, store } from '../../store/store';
import { motion } from 'framer-motion';
import { PoolMessagesView } from './PoolMessagesView';

import './PoolView.css';
import TextMessageIcon from '../../assets/text-message.png';
import FileIcon from '../../assets/file.png';
import UserGroupIcon from '../../assets/user-group.png';
import SettingsIcon from '../../assets/settings.png';
import DisconnectedIcon from '../../assets/disconnected.png';
import DisconnectIcon from '../../assets/disconnect.png';
import { PoolDisplayView } from './PoolDisplayView';
import { poolAction } from '../../store/slices/pool.slice';
import { PoolInfo, PoolUserInfo } from '../../types/sync_server.v1';
import { Backend } from '../../backend/global';

export enum PoolMessageMode {
    DISCONNECT,
    TEXT,
    FILE,
    USERS,
    SETTINGS,
}

type ActionBarButtonType = 'feature' | 'function' | 'utility' | 'danger';
interface ActionBarButtonProps {
    buttonType: ActionBarButtonType;
    icon: string;
    mode: PoolMessageMode;
    messageMode: PoolMessageMode;
    setMessageMode: React.Dispatch<React.SetStateAction<PoolMessageMode>>;
}

export function PoolContainerView() {
    const navigate = useNavigate();
    const { poolID } = useParams();
    const [ searchParams ] = useSearchParams();
    const [ poolKey, setPoolKey ] = useState<number>(0);

    useEffect(() => {
        if (!poolID) {
            navigate('/pool');
            return;
        }

        // TEMP 
        let displayName = searchParams.get("displayName");
        if (displayName == null) {
            navigate('/join-pool?poolid=' + poolID);
            return;
        }
        let poolInfo: PoolInfo = {
            poolId: poolID,
            poolName: poolID,
            users: []
        };
        store.dispatch(poolAction.setPools([poolInfo]));
        // TEMP

        let pools = getStoreState().pool.pools;
        for (const pool of pools) {
            if (pool.poolID == poolID) {
                setPoolKey(pool.key);
                Backend.connectToPool(poolID, poolKey, displayName);
                return;
            }
        }

        console.log("Going to pool");
        navigate('/pool');
    }, [])

    if (!poolID && !poolKey) {
        return null
    } else {
        return <PoolView poolID={poolID!} poolKey={poolKey} />
    }
}

export type UserMapType = Map<string, PoolUserActiveDevices>;

export interface PoolUserActiveDevices {
    user: PoolUserInfo;
    activeDevices: Map<String, PoolNode> | undefined;
}

export function PoolView({ poolID, poolKey }: { poolID: string, poolKey: number }) {

    const [ messageMode, setMessageMode ] = useState<PoolMessageMode>(PoolMessageMode.TEXT);
    const pool = useSelector((state: GlobalState) => state.pool.pools.at(poolKey));
    const navigate = useNavigate();
    const [ userMap, myNode ] = useMemo<[UserMapType, PoolNode]>(() => {
        if (!pool) return [new Map<string, PoolUserActiveDevices>(), {} as PoolNode];

        let userMap = new Map<string, PoolUserActiveDevices>();
        for (const user of pool.users) {
            let userAndDevices: PoolUserActiveDevices = {
                user: user,
                activeDevices: undefined,
            };
            userMap.set(user.userId, userAndDevices);
        }

        let myNodeID = store.getState().profile.device.deviceId;
        let myNode: PoolNode = {} as PoolNode;
        for (const activeNode of pool.activeNodes) {
            let userAndDevices = userMap.get(activeNode.userID);
            if (!userAndDevices) continue;
            if (!userAndDevices.activeDevices) {
                userAndDevices.activeDevices = new Map<String, PoolNode>();
            }
            if (activeNode.nodeID == myNodeID) {
                myNode = activeNode;
            }
            userAndDevices.activeDevices.set(activeNode.nodeID, activeNode);
        }

        console.log(userMap, myNode, pool.activeNodes, myNodeID);

        return [ userMap, myNode ];
    }, [pool?.users, pool?.activeNodes]);

    useEffect(() => {
        if (messageMode == PoolMessageMode.DISCONNECT) {
            Backend.disconnectFromPool(poolID);
            navigate('/');
        }
    }, [messageMode]);

    return (
        <div className="pool-view">
            {/* TODO: add fixed siaply of pool name along with # of active devices, # of active users, and # of users in general */}
            <PoolMessagesView poolID={poolID} feed={pool?.feed || []} userMap={userMap} />
            {
                pool ? (
                    <PoolDisplayView pool={pool} myNode={myNode} messageMode={messageMode} userMap={userMap} />
                ) : null
            }
            <ActionBar connectionState={pool?.connectionState || PoolConnectionState.CLOSED } messageMode={messageMode} setMessageMode={setMessageMode} />
            <motion.div className="pool-status-container" initial={{ y: -100 }} animate={{ y: (pool?.connectionState == PoolConnectionState.RECONNECTING ? 20 : -100) }}> 
                <div className="pool-status pool-status-disconnected">
                    <img className="pool-status-img" src={DisconnectedIcon} />
                    Lost Connection. Reconnecting...
                </div>
            </motion.div>
        </div>
    )
}

const ActionBar = memo(ActionBarComponent);

function ActionBarComponent({ connectionState, messageMode, setMessageMode }: { connectionState: PoolConnectionState, messageMode: PoolMessageMode, setMessageMode: React.Dispatch<React.SetStateAction<PoolMessageMode>> }) {
    return (
        <motion.div 
            className="action-bar" 
            initial={{ x: 150, opacity: 1 }}
            animate={{ x: connectionState == PoolConnectionState.CONNECTED ? 0 : 150 }} 
            transition={{ type: "spring", duration: 0.5 }} 
        >
            <ActionBarButton buttonType='danger' mode={PoolMessageMode.DISCONNECT} icon={DisconnectIcon} messageMode={messageMode} setMessageMode={setMessageMode} />
            <ActionBarButton buttonType='utility' mode={PoolMessageMode.SETTINGS} icon={SettingsIcon} messageMode={messageMode} setMessageMode={setMessageMode} />
            <ActionBarButton buttonType='utility' mode={PoolMessageMode.USERS} icon={UserGroupIcon} messageMode={messageMode} setMessageMode={setMessageMode}/>
            <div className="action-bar-button-spacer"/>
            <ActionBarButton buttonType='feature' mode={PoolMessageMode.FILE} icon={FileIcon} messageMode={messageMode} setMessageMode={setMessageMode}/>
            <ActionBarButton buttonType='feature' mode={PoolMessageMode.TEXT} icon={TextMessageIcon} messageMode={messageMode} setMessageMode={setMessageMode}/>
        </motion.div>
    )
}

const ActionBarButton = memo(ActionBarButtonComponent);

function ActionBarButtonComponent(props: ActionBarButtonProps) {
    return (
        <div 
            className={"action-bar-button action-bar-button-" + props.buttonType + (props.messageMode == props.mode ? " action-bar-button-selected" : "")} 
            onClick={() => {
                if (props.messageMode != props.mode) {
                    props.setMessageMode(props.mode);
                }
            }}
        >
            <img className="action-bar-icon" src={props.icon} />
        </div>
    )
}