import { createSlice, PayloadAction } from "@reduxjs/toolkit";
import { DeviceType, PoolDeviceInfo, PoolUserInfo } from "../../types/sync_server.v1";

export interface ProfileState {
    userInfo: PoolUserInfo,
    device: PoolDeviceInfo;
}

const initialState: ProfileState = {
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
            state = action.payload;
        },
    }
});

export const profileReducer = profileSlice.reducer;
export const profileAction = profileSlice.actions;

