import { fetch, Body, ResponseType } from '@tauri-apps/api/http';
import { Backend } from '../backend/global';
import { getStoreState } from '../store/store';
import { DeviceType, PoolDeviceInfo, PoolUserInfo } from '../types/sync_server.v1';
import { BeginAuthenticateRequest, BeginAuthenticateResponse, BeginRegisterRequest, BeginRegisterResponse, FinishAuthenticateRequest, FinishAuthenticateResponse, FinishRegisterRequest } from './schemas';
import { startRegistration, startAuthentication } from '@simplewebauthn/browser';

// TEMP
const AUTH_ENDPOINT = "http://192.168.0.18:80/auth";
const DEVICE_TYPE: DeviceType = DeviceType.DESKTOP;

export async function RegisterNewUser(displayName: string, deviceName: string) {
    let beginRegisterRequest: BeginRegisterRequest = {
        DisplayName: displayName,
    };

    let res = await fetch(AUTH_ENDPOINT + "/begin-register", {
        method: 'POST',
        headers: {
            'Content-Type': 'application/json',
        },
        body: Body.json(beginRegisterRequest),
        responseType: ResponseType.JSON,
    });

    if (!res.ok) {
        throw null;
    }

    let beginRegisterResponse: BeginRegisterResponse = res.data as any;
    let userID = beginRegisterResponse.CredentialCreation.publicKey!.user.id as any as string;
    
    let credential = await startRegistration(beginRegisterResponse.CredentialCreation.publicKey as any)

    if (!credential) {
        throw null;
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

    res = await fetch(AUTH_ENDPOINT + "/finish-register", {
        method: 'POST',
        headers: {
            'Content-Type': 'application/json',
        },
        body: Body.json(finishRegisterRequest),
        responseType: ResponseType.Text,
    });

    if (!res.ok) {
        throw null;
    }

    let userInfo: PoolUserInfo = {
        userId: userID,
        displayName,
        devices: [deviceInfo],
    };

    Backend.registerDevice(userInfo, deviceInfo);
}

export async function AuthenticateDevice(): Promise<string> {
    let profile = getStoreState().profile;
    let userID = profile.userInfo.userId;
    let deviceID = profile.device.deviceId;

    let beginAuthenticateRequest: BeginAuthenticateRequest = {
        UserID: userID,
        DeviceID: deviceID,
    };

    let res = await fetch(AUTH_ENDPOINT + "/begin-auth", {
        method: 'POST',
        headers: {
            'Content-Type': 'application/json',
        },
        body: Body.json(beginAuthenticateRequest),
        responseType: ResponseType.JSON,
    });

    if (!res.ok) {
        throw null;
    }

    let beginAuthenticateResponse: BeginAuthenticateResponse = res.data as any;

    let credential = await startAuthentication(beginAuthenticateResponse.CredentialAssertion.publicKey as any);

    if (!credential) {
        throw null;
    }

    credential.response.userHandle = undefined;

    let finishAuthenticateRequest: FinishAuthenticateRequest = {
        UserID: userID,
        DeviceID: deviceID,
        CredentialData: credential,
    };

    res = await fetch(AUTH_ENDPOINT + "/finish-auth", {
        method: 'POST',
        headers: {
            'Content-Type': 'application/json',
        },
        body: Body.json(finishAuthenticateRequest),
        responseType: ResponseType.JSON,
    });

    if (!res.ok) {
        throw null;
    }

    let finishAuthenticateResponse: FinishAuthenticateResponse = res.data as any;
    return finishAuthenticateResponse.Token
}

export async function RegisterNewDevice() {
    
}