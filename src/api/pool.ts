import { fetch, Body, ResponseType } from '@tauri-apps/api/http';
import { PoolInfo } from '../types/sync_server.v1';
import { API_ENDPOINT, fetchPostJSONWithAuthToken } from "./api";
import { CreateInviteToPoolResponse, CreatePoolRequest, CreatePoolResponse, JoinPoolRequest, JoinPoolResponse } from "./pool.schemas";

const POOL_ENDPOINT = API_ENDPOINT + "/pool";

function fetchPostJSONPool(path: String, body: any, responseType: ResponseType = ResponseType.JSON) {
    return fetchPostJSONWithAuthToken(POOL_ENDPOINT + path, body, responseType);
}

function fetchPostJSONPoolID(poolID: string, path: String, body: any, responseType: ResponseType = ResponseType.JSON) {
    return fetchPostJSONWithAuthToken(POOL_ENDPOINT + "/" + poolID + path, body, responseType);
}

export async function CreatePool(poolName: string): Promise<PoolInfo> {
    let createPoolRequest: CreatePoolRequest = {
        PoolName: poolName,
    };

    let res = await fetchPostJSONPool("/create", createPoolRequest);

    if (!res.ok) {
        throw "Failed to create pool";
    }

    let createPoolResponse: CreatePoolResponse = res.data as any;

    return createPoolResponse.PoolInfo;
}

export async function JoinPool(inviteLink: string): Promise<PoolInfo> {
    let joinPoolRequest: JoinPoolRequest = {
        InviteLink: inviteLink,
    };

    let res = await fetchPostJSONPool("/join", joinPoolRequest);

    if (!res.ok) {
        throw "Failed to join pool";
    }

    let joinPoolResponse: JoinPoolResponse = res.data as any;
    return joinPoolResponse.PoolInfo;
}

export async function LeavePool(poolID: string) {
    let res = await fetchPostJSONPoolID(poolID, "/leave", null);

    if (!res.ok) {
        throw "Failed to leave pool";
    }
}

export async function CreateInviteToPool(poolID: string): Promise<string> {
    let res = await fetchPostJSONPoolID(poolID, "/create-invite", null);

    if (!res.ok) {
        throw "Failed to create invite";
    }

    let createInviteToPoolResponse: CreateInviteToPoolResponse = res.data as any;
    return createInviteToPoolResponse.InviteLink;
}