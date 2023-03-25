export interface BeginRegisterRequest {
	DisplayName: string;
}

export interface BeginRegisterResponse {
	DeviceID: string;
	CredentialCreation: CredentialCreationOptions;
}

export interface FinishRegisterRequest {
	DeviceID: string;
	DeviceName: string;
	DeviceType: number;
	CredentialData: Credential;
}

export interface FinishRegisterResponse {
	Token: string;
}

export interface BeginAuthenticateRequest {
	UserID: string;
	DeviceID: string;
}

export interface BeginAuthenticateResponse {
	CredentialAssertion: CredentialRequestOptions;
}

export interface FinishAuthenticateRequest {
	UserID: string;
	DeviceID: string;
	CredentialData: Credential;
}

export interface FinishAuthenticateResponse {
	Token: string;
}