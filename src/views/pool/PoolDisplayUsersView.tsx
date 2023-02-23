import { memo, useRef, useState } from "react";
import { IndicatorDot } from "../components/IndicatorDot";

import AccordionArrowIcon from '../../assets/accordion-arrow.png';
import { fileSizeToString } from "../../utils/file-size";

import BrowserIcon from '../../assets/browser.png';
import DesktopIcon from '../../assets/desktop.png';
import MobilePhoneIcon from '../../assets/mobile-phone.png';
import { DeviceType, PoolUserInfo } from "../../types/sync_server.v1";
import { Backend } from "../../backend/global";
import { PoolStore } from "../../store/store";


export interface PoolDisplayUsersViewParams {
    poolID: string;
    users: PoolUserInfo[],
    hidden: boolean;
}

export const PoolDisplayUsersView = memo(PoolDisplayUsersViewComponent);

function PoolDisplayUsersViewComponent({ poolID, users, hidden }: PoolDisplayUsersViewParams) {

    const openAccordionUsersMap = useRef<Map<string, boolean>>(new Map<string, boolean>()).current;
    const [ accordionsOpened, setAccordionsOpened ] = useState<number>(0);

    const toggleAccordion = (userID: string) => {
        let opened = !(openAccordionUsersMap.get(userID));
        openAccordionUsersMap.set(userID, opened);
        setAccordionsOpened(accordionsOpened + (opened ? 1 : -1));
    }

    return (
        <div className="display-toggle-hide display-component-container display-users-container" aria-hidden={hidden}>
            {/* Use react virtualized (since it is a static set of data, define height and specifically specify height in DOM insetad of in CSS)*/}
            {
                users.map((user) => {
                    let activeDevicesCount = 0;
                    for (const device of user.devices) {
                        if (PoolStore.hasActiveDevice(poolID, device.deviceId)) {
                            activeDevicesCount++;
                        }
                    }
                    return (
                        <div className="display-user-accordion-container" key={user.userId}>
                            <div className="display-user-header elipsify-container" onClick={() => toggleAccordion(user.userId)}>
                                <img className="display-user-accordion-arrow" src={AccordionArrowIcon} aria-expanded={openAccordionUsersMap.get(user.userId)} />
                                <div className="display-user-display-name elipsify-content">{user.displayName}</div>
                                <div className="display-user-info-point elipsify-extra">
                                    <IndicatorDot type={activeDevicesCount ? "online" : "offline"} size="small" />
                                    {activeDevicesCount ? activeDevicesCount + " Device" + (activeDevicesCount > 1 ? "s" : "") + " Active" : " Offline"}
                                </div>
                            </div>
                                {
                                    openAccordionUsersMap.get(user.userId) && (
                                        <div className="display-user-devices-container">
                                            {
                                                user.devices.slice().sort((a, b) => (PoolStore.hasActiveDevice(poolID, a.deviceId) ? 0 : 1) - (PoolStore.hasActiveDevice(poolID, b.deviceId) ? 0 : 1)).map((node) => (
                                                    <div className={"display-user-device-container " + "display-user-device-container" + (PoolStore.hasActiveDevice(poolID, node.deviceId) ? "-online" : "-offline")} key={node.deviceId}>
                                                        <div className="display-user-device-header">
                                                            <img src={
                                                                node.deviceType == DeviceType.DESKTOP ? DesktopIcon :
                                                                node.deviceType == DeviceType.MOBILE ? MobilePhoneIcon : BrowserIcon
                                                            } height="20" width="20" />
                                                            <span>{node.deviceName}</span>
                                                        </div>
                                                    </div>
                                                ))
                                            }
                                        </div>
                                    )
                                }
                        </div>
                    )
                })
            }
        </div>
    )
}