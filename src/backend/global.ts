import { BackendCommands } from "./backend";
import { convertFileSrc } from '@tauri-apps/api/tauri';
import {} from './events'; // DO NOT REMOVE

export const Backend = new BackendCommands();

export async function getTempAssetURL(poolID: string, fileID: string): Promise<string> {
    return convertFileSrc(poolID + "/" + fileID, "media");
}