import { createSlice, PayloadAction } from "@reduxjs/toolkit";

export interface SettingState {
}

export interface StorageSettings {
}

const initialState: SettingState = {
    storageSettings: {
    }
}

const settingSlice = createSlice({
    name: "setting",
    initialState: initialState,
    reducers: {
    }
});

export const settingReducer = settingSlice.reducer;
export const settingAction = settingSlice.actions;

