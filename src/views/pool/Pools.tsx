import { useEffect, useRef, useState } from "react";
import { useSelector } from "react-redux";
import { Outlet, useNavigate } from "react-router-dom";
import { GlobalState, PoolStore } from "../../store/store";
import { IndicatorDot } from "../components/IndicatorDot";

import './Pools.css';

import ErrorIcon from '../../assets/error.png';
import ArrowIcon from '../../assets/arrow.png';

import AddNoOutlineIcon from '../../assets/add-no-outline.png';
import { InputOverlay } from "../components/InputOverlay";
import { Tooltip } from "react-tooltip";
import { CreateInviteToPool, CreatePool } from "../../api/pool";
import { Backend } from "../../backend/global";
import { motion } from "framer-motion";

export const DEFAULT_TEST_POOL_NAME: string = "main";

enum OverlayMode {
    NONE,
    CREATE_POOL,
    JOIN_POOL,
    INVITE_LINK,
}

export function Pools() {

    const navigate = useNavigate();
    const profile = useSelector((globalState: GlobalState) => globalState.profile);
    const pools = useSelector((globalState: GlobalState) => globalState.pool.pools);
    const [ overlayMode, setOverlayMode ] = useState<OverlayMode>(OverlayMode.NONE);
    const [ errorMessage, setErrorMessage ] = useState<string>("");
    const errorMessageClearTimeout = useRef<NodeJS.Timeout | null>(null);
    const [ inviteLink, setInviteLink ] = useState<string | undefined>(undefined);

    useEffect(() => {
        // navigate("/pool/main?displayName=TEST");
        // navigate("/join-pool");
    }, []);

    const startErrorMessageClearTimeout = () => {
        if (errorMessageClearTimeout.current) {
            clearTimeout(errorMessageClearTimeout.current);
        }
        errorMessageClearTimeout.current = setTimeout(() => {
            setErrorMessage("");
        }, 3000);
    }

    const showErrorMessage = (errorMsg: string) => {
        setErrorMessage(errorMsg);
        startErrorMessageClearTimeout();
    }

    const overlayOnClose = () => {
        setOverlayMode(OverlayMode.NONE);
    }

    const createPoolOverlay = () => {
        setOverlayMode(OverlayMode.CREATE_POOL);
    }

    const joinPoolOverlay = () => {
        setOverlayMode(OverlayMode.JOIN_POOL);
    }

    const inviteLinkOverlay = (poolID: string) => {
        setInviteLink(undefined);
        Backend.createInviteLink(poolID).then((invite) => {
            console.log("INVITE LINK", invite);
            setInviteLink(invite);
            console.log(inviteLink);
        }).catch((err) => {
            showErrorMessage("Error creating invite link");
        });
        setOverlayMode(OverlayMode.INVITE_LINK);
    }

    const createPool = (poolName: string) => {
        console.log("CREATE POOL WITH NAME", poolName);
        Backend.createPool(poolName).catch((err) => {
            console.log("ERROR CREATING POOL", err);
            showErrorMessage("Error creating pool");
        });
        setOverlayMode(OverlayMode.NONE);
    }

    const joinPool = (inviteLink: string) => {
        console.log("JOIN POOL WITH LINK", inviteLink);
        Backend.joinPool(inviteLink).catch((err) => {
            console.log("ERROR JOINING POOL", err);
            showErrorMessage("Error joining pool");
        });
        setOverlayMode(OverlayMode.NONE);
    }

    const connectToPool = (poolID: string) => {
        navigate("/pool/" + poolID);
    }

    const leavePool = (poolID: string) => {
        Backend.leavePool(poolID).catch((err) => {
            console.log("ERROR LEAVING POOL", err);
            showErrorMessage("Error leaving pool");
        });
    }

    return (
        <>
        <div className="pools-container">
            <div className="pools-profile-container">
                Hello, {profile.userInfo.displayName}
            </div>

            {/* TESTING */}
            {/* <div className="pools-pool-container">
                <div className="pools-pool-info-container">
                    <div className="pools-pool-info-name">
                        POOL_NAME    
                    </div>
                    <div className="pools-pool-info-stats">
                        <div className="pools-pool-info-stats-bar-container">
                            <IndicatorDot type="offline" />
                            <div className="pools-pool-info-stats-bar-subtitle">
                                TOTAL_USERS
                            </div>
                        </div>
                    </div>
                </div>
                <div className="pools-pool-action-container">
                    <div className="pools-pool-button pools-pool-connect-button">
                        Connect
                    </div>
                    <div id={"more_button_" + "POOL_ID"} className="pools-pool-button pools-pool-more-button">
                        More <img src={ArrowIcon} style={{ rotate: "90deg", marginLeft: 8, marginTop: 2 }} height="15" />
                    </div>
                    <Tooltip anchorSelect="#more_button_POOL_ID" className="pools-more-button-tooltip" clickable noArrow>
                        <div className="pools-pool-button pools-pool-invite-button" onClick={() => inviteLinkOverlay("test")}>
                            Invite
                        </div>
                        <div className="pools-pool-button pools-pool-leave-button">
                            Leave
                        </div>
                    </Tooltip>
                </div>
            </div> */}
            {/* TESTING */}

            {
                pools.map(pool => (
                    <div className="pools-pool-container" key={pool.poolID}>
                        <div className="pools-pool-info-container">
                            <div className="pools-pool-info-name">
                                {pool.poolName}  
                            </div>
                            <div className="pools-pool-info-stats">
                                <div className="pools-pool-info-stats-bar-container">
                                    <IndicatorDot type="offline" />
                                    <div className="pools-pool-info-stats-bar-subtitle">
                                    {pool.users.length} Total User{pool.users.length > 1 ? "s" : ""}
                                    </div>
                                </div>
                            </div>
                        </div>
                        <div className="pools-pool-action-container">
                            <div className="pools-pool-button pools-pool-connect-button" onClick={() => connectToPool(pool.poolID)}>
                                Connect
                            </div>
                            <div id={"more_button_" + pool.poolID} className="pools-pool-button pools-pool-more-button">
                                More <img src={ArrowIcon} style={{ rotate: "90deg", marginLeft: 8, marginTop: 2 }} height="15" />
                            </div>
                            <Tooltip anchorSelect={"#more_button_" + pool.poolID} className="pools-more-button-tooltip" clickable noArrow>
                                <div className="pools-pool-button pools-pool-invite-button" onClick={() => inviteLinkOverlay(pool.poolID)}>
                                    Invite
                                </div>
                                <div className="pools-pool-button pools-pool-leave-button" onClick={() => leavePool(pool.poolID)}>
                                    Leave
                                </div>
                            </Tooltip>
                        </div>
                    </div>
                ))
            }
            
            <div className="pools-add-button-container">
                <div className="pools-add-button" id="pool-add-button">
                    <img className="pools-add-button-icon" src={AddNoOutlineIcon} />
                </div>
                <Tooltip anchorSelect="#pool-add-button" className="pools-add-button-tooltip" clickable noArrow>
                    <div className="pools-add-button-tooltip-button" onClick={createPoolOverlay}>
                        Create Pool
                    </div>
                    <div className="pools-add-button-tooltip-sep"/>
                    <div className="pools-add-button-tooltip-button" onClick={joinPoolOverlay}>
                        Join Pool
                    </div>
                </Tooltip>
            </div>
        </div>
        {
            overlayMode == OverlayMode.CREATE_POOL ? (
                <InputOverlay
                    placeholder="Pool Name"
                    editable={true}
                    onSubmit={createPool}
                    onClose={overlayOnClose}
                    buttonContent="Create"
                />
            ) : overlayMode == OverlayMode.JOIN_POOL ? (
                <InputOverlay
                    placeholder="Invite Link"
                    editable={true}
                    onSubmit={joinPool}
                    onClose={overlayOnClose}
                    buttonContent="Join"
                    extraInfo='Invite link that starts with "invite:"'
                />
            ) : overlayMode == OverlayMode.INVITE_LINK ? (
                <InputOverlay
                    initValue={inviteLink || "Generating link..."}
                    editable={false}
                    onSubmit={(value) => {navigator.clipboard.writeText(value)}}
                    onClose={overlayOnClose}
                    buttonContent="Copy"
                    extraInfo="Send this link to your friends to invite them to your pool. Note that this link will expire after 24 hours."
                />
            ) : null
        }
        <motion.div className="pools-status-container" initial={{ y: -100 }} animate={{ y: (errorMessage != "" ? 20 : -100) }}> 
            <div className="pools-status pools-status-error">
                <img className="pools-status-img" src={ErrorIcon} />
                { errorMessage }
            </div>
        </motion.div>
        </>
    )
}