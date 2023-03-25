import { fetch, Body, ResponseType } from '@tauri-apps/api/http';
import { Backend } from '../backend/global';
import { getStoreState } from '../store/store';
import { DeviceType, PoolDeviceInfo, PoolUserInfo } from '../types/sync_server.v1';
import { BeginAuthenticateRequest, BeginAuthenticateResponse, BeginRegisterRequest, BeginRegisterResponse, FinishAuthenticateRequest, FinishAuthenticateResponse, FinishRegisterRequest, FinishRegisterResponse } from './auth.schemas';
import { startRegistration, startAuthentication } from '@simplewebauthn/browser';
import { API_ENDPOINT, fetchPostJSON, setAuthToken } from './api';

// TEMP
const AUTH_ENDPOINT = API_ENDPOINT + "/auth";
const DEVICE_TYPE: DeviceType = DeviceType.DESKTOP;

function fetchPostJSONAuth(path: string, body: any, responseType: ResponseType = ResponseType.JSON) {
    return fetchPostJSON(AUTH_ENDPOINT + path, body, responseType);
}

export async function RegisterNewUser(displayName: string, deviceName: string): Promise<[PoolUserInfo, PoolDeviceInfo]> {
    let beginRegisterRequest: BeginRegisterRequest = {
        DisplayName: displayName,
    };

    let res = await fetchPostJSONAuth("/begin-register", beginRegisterRequest);

    if (!res.ok) {
        throw "Failed to begin registration";
    }

    let beginRegisterResponse: BeginRegisterResponse = res.data as any;
    let userID = beginRegisterResponse.CredentialCreation.publicKey!.user.id as any as string;
    
    let credential = await startRegistration(beginRegisterResponse.CredentialCreation.publicKey as any)

    if (!credential) {
        throw "Failed to get credential";
    }

    let deviceInfo: PoolDeviceInfo = {
        deviceId: beginRegisterResponse.DeviceID,
        deviceName: deviceName,
        deviceType: DEVICE_TYPE,
    };

    let finishRegisterRequest: FinishRegisterRequest = {
        DeviceID: deviceInfo.deviceId,
        DeviceName: deviceInfo.deviceName,
        DeviceType: deviceInfo.deviceType,
        CredentialData: credential,
    }

    res = await fetchPostJSONAuth("/finish-register", finishRegisterRequest);

    if (!res.ok) {
        throw "Failed to finish registration";
    }

    let finishRegisterResponse: FinishRegisterResponse = res.data as any;
    setAuthToken(finishRegisterResponse.Token);

    let userInfo: PoolUserInfo = {
        userId: userID,
        displayName,
        devices: [deviceInfo],
    };

    return [userInfo, deviceInfo];
}

export async function AuthenticateDevice() {
    let profile = getStoreState().profile;
    let userID = profile.userInfo.userId;
    let deviceID = profile.device.deviceId;

    let beginAuthenticateRequest: BeginAuthenticateRequest = {
        UserID: userID,
        DeviceID: deviceID,
    };

    let res = await fetchPostJSONAuth("/begin-auth", beginAuthenticateRequest);

    if (!res.ok) {
        throw "Failed to begin authentication";
    }

    let beginAuthenticateResponse: BeginAuthenticateResponse = res.data as any;

    let credential = await startAuthentication(beginAuthenticateResponse.CredentialAssertion.publicKey as any);

    if (!credential) {
        throw "Failed to get credential";
    }

    credential.response.userHandle = undefined;

    let finishAuthenticateRequest: FinishAuthenticateRequest = {
        UserID: userID,
        DeviceID: deviceID,
        CredentialData: credential,
    };

    res = await fetchPostJSONAuth("/finish-auth", finishAuthenticateRequest);

    if (!res.ok) {
        throw "Failed to finish authentication";
    }

    let finishAuthenticateResponse: FinishAuthenticateResponse = res.data as any;
    setAuthToken(finishAuthenticateResponse.Token);
}

export async function RegisterNewDevice() {
    
}