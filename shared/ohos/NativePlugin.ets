// Copyright 2019-2026 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

export interface NativePlugin {
  invoke(command: string, payload: string): Promise<string>;
  close(): void;
}

export interface NativeModule {
  tauriOhosPluginInitialize(callback: (request: string) => void, files: string, cache: string, temp: string): void;
  tauriOhosPluginResponse(id: number, success: boolean, payload: string): void;
  tauriOhosPluginClose(): void;
}
