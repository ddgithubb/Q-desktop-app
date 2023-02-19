import { BackendCommands } from "./backend";
import { appDataDir, join } from '@tauri-apps/api/path';
import { convertFileSrc } from '@tauri-apps/api/tauri';

export const Backend = new BackendCommands();

export async function getTempAssetURL(poolID: string, fileID: string): Promise<string> {
    let dir = await appDataDir();
    let tempURL = await join(dir, "temp", poolID, fileID);
    return convertFileSrc(tempURL);
}