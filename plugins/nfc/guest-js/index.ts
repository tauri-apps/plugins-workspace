// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

import { invoke } from "@tauri-apps/api/primitives";

export const RTD_TEXT = [0x54]; // "T"
export const RTD_URI = [0x55]; // "U"

export enum ScanKind {
  Ndef,
  Tag,
}

export interface ScanOptions {
  keepSessionAlive?: boolean;
}

export enum NFCTypeNameFormat {
  Empty = 0,
  NfcWellKnown = 1,
  Media = 2,
  AbsoluteURI = 3,
  NfcExternal = 4,
  Unknown = 5,
  Unchanged = 6,
}

export interface TagRecord {
  tnf: NFCTypeNameFormat;
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

export interface NFCRecord {
  format: NFCTypeNameFormat;
  kind: number[];
  id: number[];
  payload: number[];
}

export function record(
  format: NFCTypeNameFormat,
  kind: string | number[],
  id: string | number[],
  payload: string | number[]
): NFCRecord {
  return {
    format,
    kind:
      typeof kind === "string"
        ? Array.from(new TextEncoder().encode(kind))
        : kind,
    id: typeof id === "string" ? Array.from(new TextEncoder().encode(id)) : id,
    payload:
      typeof payload === "string"
        ? Array.from(new TextEncoder().encode(payload))
        : payload,
  };
}

export function textRecord(text: string, id?: string | number[], language: string = "en"): NFCRecord {
  const payload = Array.from(new TextEncoder().encode(language + text))
  payload.unshift(language.length)
  return record(NFCTypeNameFormat.NfcWellKnown, RTD_TEXT, id || [], payload);
}

const protocols = [
  "",
  "http://www.",
  "https://www.",
  "http://",
  "https://",
  "tel:",
  "mailto:",
  "ftp://anonymous:anonymous@",
  "ftp://ftp.",
  "ftps://",
  "sftp://",
  "smb://",
  "nfs://",
  "ftp://",
  "dav://",
  "news:",
  "telnet://",
  "imap:",
  "rtsp://",
  "urn:",
  "pop:",
  "sip:",
  "sips:",
  "tftp:",
  "btspp://",
  "btl2cap://",
  "btgoep://",
  "tcpobex://",
  "irdaobex://",
  "file://",
  "urn:epc:id:",
  "urn:epc:tag:",
  "urn:epc:pat:",
  "urn:epc:raw:",
  "urn:epc:",
  "urn:nfc:",
];

function encodeURI(uri: string): number[] {
  let prefix = "";

  protocols.slice(1).forEach(function (protocol) {
    if ((!prefix || prefix === "urn:") && uri.indexOf(protocol) === 0) {
      prefix = protocol;
    }
  });

  if (!prefix) {
    prefix = "";
  }

  const encoded = Array.from(
    new TextEncoder().encode(uri.slice(prefix.length))
  );
  const protocolCode = protocols.indexOf(prefix);
  // prepend protocol code
  encoded.unshift(protocolCode);

  return encoded;
}

export function uriRecord(uri: string, id?: string | number[]): NFCRecord {
  return record(
    NFCTypeNameFormat.NfcWellKnown,
    RTD_URI,
    id || [],
    encodeURI(uri)
  );
}

export async function scan(
  kind: ScanKind,
  options?: ScanOptions
): Promise<Scan> {
  return await invoke("plugin:nfc|scan", {
    kind: kind === ScanKind.Ndef ? "ndef" : "tag",
    ...options,
  });
}

export async function write(records: NFCRecord[]): Promise<void> {
  return await invoke("plugin:nfc|write", {
    records,
  });
}

export async function isAvailable(): Promise<boolean> {
  return await invoke("plugin:nfc|isAvailable");
}
