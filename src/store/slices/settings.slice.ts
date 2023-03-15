import { createSlice, PayloadAction } from "@reduxjs/toolkit";

export interface SettingState {
}

const initialState: SettingState = {
}

const settingSlice = createSlice({
    name: "setting",
    initialState: initialState,
    reducers: {
    }
});

export const settingReducer = settingSlice.reducer;
export const settingAction = settingSlice.actions;

