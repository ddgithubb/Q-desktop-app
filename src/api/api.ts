import { fetch, Body, ResponseType } from "@tauri-apps/api/http";
import { Backend } from "../backend/global";
import { getStoreState } from "../store/store";
import { AuthenticateDevice } from "./auth";

export const API_ENDPOINT = "http://192.168.0.18:80";
var AUTH_TOKEN = "";

export function setAuthToken(authToken: string) {
    AUTH_TOKEN = authToken;
    Backend.setAuthToken(authToken)
}

export async function fetchPostJSON(endpoint: string, body: any, responseType: ResponseType = ResponseType.JSON) {
    return await fetch(endpoint, {
        method: 'POST',
        headers: {
            'Content-Type': 'application/json',
        },
        body: Body.json(body),
        responseType,
    });
}

export async function fetchPostJSONWithAuthToken(endpoint: string, body: any, responseType: ResponseType = ResponseType.JSON) {
    let deviceID = getStoreState().profile.device.deviceId;
    let res = await fetch(endpoint, {
        method: 'POST',
        headers: {
            'Content-Type': 'application/json',
            'x-device-id': deviceID,
            'x-auth-token': AUTH_TOKEN,
        },
        body: Body.json(body),
        responseType,
    });

    if (res.status == 401) {
        window.location.href = "/authenticating";
        throw "Unauthorized";
    }

    // console.log("Refreshed token: ", res.headers["x-auth-token"]);
    setAuthToken(res.headers["x-auth-token"]);

    return res;
}