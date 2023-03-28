import { PoolInfo } from "../types/sync_server.v1";

export interface CreatePoolRequest {
    PoolName: string;
}

export interface CreatePoolResponse {
    PoolInfo: PoolInfo;
}

export interface JoinPoolRequest {
    InviteLink: string;
}

export interface JoinPoolResponse {
    PoolInfo: PoolInfo;
}

export interface LeavePoolResponse {
    Success: boolean;
}

export interface CreateInviteToPoolResponse {
    InviteLink: string;
}