import { invoke } from "@tauri-apps/api/core"

export type PushTokenResponse = {
  value?: string;
}

export async function pushToken(): Promise<string | null> {
  return await invoke<{value?: string}>('plugin:push-notifications|push_token', {
    payload: {},
  }).then((r: PushTokenResponse) => (r.value ? r.value : null));
}
 