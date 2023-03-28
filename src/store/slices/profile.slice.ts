import { createSlice, PayloadAction } from "@reduxjs/toolkit";
import { DeviceType, PoolDeviceInfo, PoolUserInfo } from "../../types/sync_server.v1";
import { PoolStore } from "../store";

export interface ProfileState {
    registered: boolean,
    userInfo: PoolUserInfo,
    device: PoolDeviceInfo;
}

const initialState: ProfileState = {
    registered: false,
    userInfo: {
        userId: "TEST_USER",
        displayName: "TEST_NAME",
        devices: [{
            deviceId: "DEVICE_ID",
            deviceType: DeviceType.DESKTOP,
            deviceName: "Main device",
        }],
    },
    device: {
        deviceId: "DEVICE_ID",
        deviceType: DeviceType.DESKTOP,
        deviceName: "Main device",
    }
}

const profileSlice = createSlice({
    name: "profile",
    initialState: initialState,
    reducers: {
        initProfile(state: ProfileState, action: PayloadAction<ProfileState>) {
            state.registered = action.payload.registered;
            state.userInfo = action.payload.userInfo;
            state.device = action.payload.device;
        },
    }
});

export const profileReducer = profileSlice.reducer;
export const profileAction = profileSlice.actions;

