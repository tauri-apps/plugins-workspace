// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

declare global {
  interface Window {
    __TAURI_INVOKE__: <T>(cmd: string, args?: unknown) => Promise<T>;
  }
}

export enum ScanKind {
  Ndef,
  Tag,
}

export interface ScanOptions {
  keepAlive?: boolean;
}

export interface TagRecord {
  tnf: number;
  kind: number[];
  id: number[];
  payload: number[];
}

export interface Tag {
  id: string;
  kind: string;
  records: TagRecord[];
}

export interface Scan {
  id: string;
  kind: string;
  tag: Tag;
}

export async function scan(
  kind: ScanKind,
  options?: ScanOptions,
): Promise<Scan> {
  return await window.__TAURI_INVOKE__("plugin:nfc|scan", {
    kind: kind === ScanKind.Ndef ? "ndef" : "tag",
    ...options,
  });
}

export async function isAvailable(): Promise<boolean> {
  return await window.__TAURI_INVOKE__("plugin:nfc|isAvailable");
}
