// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

/**
 * Read and write NFC tags on Android and iOS.
 *
 * This plugin is mobile only, the APIs reject on desktop platforms.
 *
 * @module
 */

import { invoke } from '@tauri-apps/api/core'

/**
 * Record Type Definition (RTD) of an NDEF well known text record, the `"T"` (`0x54`) byte.
 *
 * @since 2.0.0
 */
export const RTD_TEXT = [0x54] // "T"
/**
 * Record Type Definition (RTD) of an NDEF well known URI record, the `"U"` (`0x55`) byte.
 *
 * @since 2.0.0
 */
export const RTD_URI = [0x55] // "U"

/**
 * Filters the tags to scan by the URI of their payload.
 *
 * Every property is optional and only the ones that are set take part in the filter.
 * **Android only**, the iOS implementation ignores this filter.
 */
export interface UriFilter {
  /** Only match URIs with this scheme, e.g. `https`. */
  scheme?: string
  /** Only match URIs with this authority (host), e.g. `tauri.app`. */
  host?: string
  /** Only match URIs whose path starts with this prefix, e.g. `/docs`. */
  pathPrefix?: string
}

/**
 * The NFC technologies a tag can support, mirroring the `android.nfc.tech` classes.
 *
 * **Android only**, used by the `techLists` filter of an `ndef` {@link ScanKind}.
 */
export enum TechKind {
  /** ISO-DEP (ISO 14443-4) properties and I/O operations. */
  IsoDep,
  /** MIFARE Classic properties and I/O operations. */
  MifareClassic,
  /** MIFARE Ultralight and MIFARE Ultralight C properties and I/O operations. */
  MifareUltralight,
  /** NDEF data and operations on tags that are already formatted as NDEF. */
  Ndef,
  /** Formatting operations on tags that can be formatted as NDEF but are not yet. */
  NdefFormatable,
  /** NFC-A (ISO 14443-3A) properties and I/O operations. */
  NfcA,
  /** NFC-B (ISO 14443-3B) properties and I/O operations. */
  NfcB,
  /** NFC Barcode (Kovio NFC Barcode) properties and I/O operations. */
  NfcBarcode,
  /** NFC-F (JIS 6319-4) properties and I/O operations. */
  NfcF,
  /** NFC-V (ISO 15693) properties and I/O operations. */
  NfcV
}

/**
 * The kind of scan to perform, which defines which tags are matched.
 *
 * Use `tag` to match any discovered tag and `ndef` to only match tags carrying an NDEF message.
 */
export type ScanKind =
  | {
      /** Matches any tag that is discovered, whether it carries an NDEF message or not. */
      type: 'tag'
      /** Only match tags whose payload URI matches this filter. **Android only**. */
      uri?: UriFilter
      /** Only match tags whose payload has this MIME type, e.g. `text/plain`. **Android only**. */
      mimeType?: string
    }
  | {
      /** Only matches tags that carry an NDEF message. */
      type: 'ndef'
      /** Only match tags whose NDEF payload URI matches this filter. **Android only**. */
      uri?: UriFilter
      /**
       * Only match tags whose NDEF payload has this MIME type, e.g. `text/plain`.
       * **Android only**.
       */
      mimeType?: string
      /**
       *  Each of the tech-lists is considered independently and the activity is considered a match if
       * any single tech-list matches the tag that was discovered.
       * This provides AND and OR semantics for filtering desired techs.
       *
       * See <https://developer.android.com/reference/android/nfc/NfcAdapter#ACTION_TECH_DISCOVERED> for more information.
       *
       * Examples
       *
       * ```ts
       * import type { TechKind } from "@tauri-apps/plugin-nfc"
       *
       * const techLists = [
       *  // capture anything using NfcF
       *  [TechKind.NfcF],
       *  // capture all MIFARE Classics with NDEF payloads
       *  [TechKind.NfcA, TechKind.MifareClassic, TechKind.Ndef]
       * ]
       * ```
       */
      techLists?: TechKind[][]
    }

/** Options for the {@link scan} function. */
export interface ScanOptions {
  /**
   * Whether the connection to the scanned tag must be kept open after the scan resolves,
   * so that a following {@link write} call writes to that tag instead of scanning a new one.
   * Defaults to `false`.
   */
  keepSessionAlive?: boolean
  /** Message displayed in the UI. iOS only. */
  message?: string
  /** Message displayed in the UI when the message has been read. iOS only. */
  successMessage?: string
}

/** Options for the {@link write} function. */
export interface WriteOptions {
  /**
   * The kind of scan to perform to find the tag to write to.
   * Required on Android unless a {@link scan} session is kept alive.
   */
  kind?: ScanKind
  /** Message displayed in the UI when reading the tag. iOS only. */
  message?: string
  /** Message displayed in the UI when the tag has been read. iOS only. */
  successfulReadMessage?: string
  /** Message displayed in the UI when the message has been written. iOS only. */
  successMessage?: string
}

/**
 * The Type Name Format (TNF) of an NDEF record,
 * which defines how the record type is interpreted.
 */
export enum NFCTypeNameFormat {
  /** The record is empty: type, identifier and payload must be empty. */
  Empty = 0,
  /**
   * The record type is an NFC Forum well known type, defined by a Record Type Definition (RTD)
   * such as {@link RTD_TEXT} or {@link RTD_URI}.
   */
  NfcWellKnown = 1,
  /** The record type is a MIME media type as defined in RFC 2046, e.g. `text/plain`. */
  Media = 2,
  /** The record type is an absolute URI as defined in RFC 3986. */
  AbsoluteURI = 3,
  /** The record type is an NFC Forum external type, i.e. a type namespaced by its issuer. */
  NfcExternal = 4,
  /**
   * The record type is unknown: the type must be empty and the payload interpretation
   * is left to the application.
   */
  Unknown = 5,
  /**
   * The record is a middle or last chunk of a chunked record and inherits the type of the
   * first chunk, so its own type must be empty.
   */
  Unchanged = 6
}

/** An NDEF record read from a scanned tag. */
export interface TagRecord {
  /**
   * The Type Name Format (TNF) of the record,
   * which defines how {@link TagRecord.kind} is interpreted.
   */
  tnf: NFCTypeNameFormat
  /** The record type bytes. */
  kind: number[]
  /** The record identifier bytes. Can be empty. */
  id: number[]
  /** The record payload bytes. */
  payload: number[]
}

/** An NFC tag that has been scanned. */
export interface Tag {
  /** The tag identifier bytes, as reported by the operating system. */
  id: number[]
  /** The technologies the tag supports, e.g. `["android.nfc.tech.Ndef"]` on Android. */
  kind: string[]
  /** The NDEF records stored on the tag. Empty when the tag holds no NDEF message. */
  records: TagRecord[]
}

/**
 * An NDEF record to be written to a tag.
 *
 * Use {@link record}, {@link textRecord} or {@link uriRecord} to create one.
 */
export interface NFCRecord {
  /** The Type Name Format (TNF) of the record. */
  format: NFCTypeNameFormat
  /**
   * The record type, interpreted according to {@link NFCRecord.format}.
   * For well known records this is a Record Type Definition (RTD) value
   * such as {@link RTD_TEXT} or {@link RTD_URI}.
   */
  kind: number[]
  /** The record identifier. Can be empty. */
  id: number[]
  /** The record payload bytes. */
  payload: number[]
}

/**
 * Creates an NDEF record with the given type name format, type, identifier and payload.
 *
 * Strings are encoded as UTF-8 byte arrays.
 *
 * @example
 * ```typescript
 * import { record, NFCTypeNameFormat, write } from '@tauri-apps/plugin-nfc';
 * const mimeRecord = record(NFCTypeNameFormat.Media, 'text/plain', '', 'hello world');
 * await write([mimeRecord], { kind: { type: 'ndef' } });
 * ```
 *
 * @param format The Type Name Format (TNF) of the record.
 * @param kind The record type, interpreted according to `format`.
 * @param id The record identifier. Use an empty string or array when the record has none.
 * @param payload The record payload.
 * @returns The NDEF record, ready to be written with {@link write}.
 *
 * @since 2.0.0
 */
export function record(
  format: NFCTypeNameFormat,
  kind: string | number[],
  id: string | number[],
  payload: string | number[]
): NFCRecord {
  return {
    format,
    kind:
      typeof kind === 'string'
        ? Array.from(new TextEncoder().encode(kind))
        : kind,
    id: typeof id === 'string' ? Array.from(new TextEncoder().encode(id)) : id,
    payload:
      typeof payload === 'string'
        ? Array.from(new TextEncoder().encode(payload))
        : payload
  }
}

/**
 * Creates an NDEF well known text record ({@link RTD_TEXT}).
 *
 * The payload is the UTF-8 encoded text prefixed by the language code
 * and by a status byte holding the length of that language code.
 *
 * @example
 * ```typescript
 * import { textRecord, write } from '@tauri-apps/plugin-nfc';
 * await write([textRecord('hello world')], { kind: { type: 'ndef' } });
 * ```
 *
 * @param text The text to store in the record.
 * @param id The record identifier. Defaults to an empty identifier.
 * @param language The IANA language code of the text. Defaults to `en`.
 * @returns The NDEF record, ready to be written with {@link write}.
 *
 * @since 2.0.0
 */
export function textRecord(
  text: string,
  id?: string | number[],
  language: string = 'en'
): NFCRecord {
  const payload = Array.from(new TextEncoder().encode(language + text))
  payload.unshift(language.length)
  return record(NFCTypeNameFormat.NfcWellKnown, RTD_TEXT, id ?? [], payload)
}

const protocols = [
  '',
  'http://www.',
  'https://www.',
  'http://',
  'https://',
  'tel:',
  'mailto:',
  'ftp://anonymous:anonymous@',
  'ftp://ftp.',
  'ftps://',
  'sftp://',
  'smb://',
  'nfs://',
  'ftp://',
  'dav://',
  'news:',
  'telnet://',
  'imap:',
  'rtsp://',
  'urn:',
  'pop:',
  'sip:',
  'sips:',
  'tftp:',
  'btspp://',
  'btl2cap://',
  'btgoep://',
  'tcpobex://',
  'irdaobex://',
  'file://',
  'urn:epc:id:',
  'urn:epc:tag:',
  'urn:epc:pat:',
  'urn:epc:raw:',
  'urn:epc:',
  'urn:nfc:'
]

function encodeURI(uri: string): number[] {
  let prefix = ''

  protocols.slice(1).forEach(function (protocol) {
    if (
      (prefix.length === 0 || prefix === 'urn:')
      && uri.indexOf(protocol) === 0
    ) {
      prefix = protocol
    }
  })

  if (prefix.length === 0) {
    prefix = ''
  }

  const encoded = Array.from(new TextEncoder().encode(uri.slice(prefix.length)))
  const protocolCode = protocols.indexOf(prefix)
  // prepend protocol code
  encoded.unshift(protocolCode)

  return encoded
}

/**
 * Creates an NDEF well known URI record ({@link RTD_URI}).
 *
 * The URI is encoded with the NDEF URI abbreviation scheme: a known prefix such as
 * `https://` is replaced by the identifier code byte that starts the payload.
 *
 * @example
 * ```typescript
 * import { uriRecord, write } from '@tauri-apps/plugin-nfc';
 * await write([uriRecord('https://tauri.app')], { kind: { type: 'ndef' } });
 * ```
 *
 * @param uri The URI to store in the record.
 * @param id The record identifier. Defaults to an empty identifier.
 * @returns The NDEF record, ready to be written with {@link write}.
 *
 * @since 2.0.0
 */
export function uriRecord(uri: string, id?: string | number[]): NFCRecord {
  return record(
    NFCTypeNameFormat.NfcWellKnown,
    RTD_URI,
    id ?? [],
    encodeURI(uri)
  )
}

function mapScanKind(kind: ScanKind): Record<string, unknown> {
  const { type: scanKind, ...kindOptions } = kind
  return { [scanKind]: kindOptions }
}

/**
 * Scans an NFC tag, resolving when a tag matching the given filters is read.
 *
 * See <https://developer.android.com/develop/connectivity/nfc/nfc#ndef> for more information.
 *
 * @example
 * ```typescript
 * import { scan } from '@tauri-apps/plugin-nfc';
 * const tag = await scan({ type: 'tag' });
 * ```
 *
 * @param kind The kind of scan to perform, which defines how tags are matched.
 * @param options Additional scan options such as the iOS UI messages and whether the
 * session must be kept alive for a following {@link write} call.
 * @returns A promise resolving to the tag that has been scanned.
 *
 * @since 2.0.0
 */
export async function scan(
  kind: ScanKind,
  options?: ScanOptions
): Promise<Tag> {
  return await invoke('plugin:nfc|scan', {
    kind: mapScanKind(kind),
    ...options
  })
}

/**
 * Write to an NFC tag.
 *
 * If you did not previously call {@link scan} with {@link ScanOptions.keepSessionAlive} set to true,
 * it will first scan the tag then write to it.
 *
 * @example
 * ```typescript
 * import { uriRecord, write } from '@tauri-apps/plugin-nfc';
 * await write([uriRecord('https://tauri.app')], { kind: { type: 'ndef' } });
 * ```
 *
 * @param records The NDEF records to write to the tag.
 * @param options Additional write options such as the kind of scan used to find the tag
 * and the iOS UI messages.
 *
 * @since 2.0.0
 */
export async function write(
  records: NFCRecord[],
  options?: WriteOptions
): Promise<void> {
  const { kind, ...opts } = options ?? {}
  if (kind) {
    // @ts-expect-error map the property
    opts.kind = mapScanKind(kind)
  }
  await invoke('plugin:nfc|write', {
    records,
    ...opts
  })
}

/**
 * Checks whether NFC is supported by the device and currently usable by the app.
 *
 * Resolves to `false` on Android when the device has no NFC adapter or NFC is disabled in the
 * device settings, and on iOS when the `NFCReaderUsageDescription` entry is missing from the
 * `Info.plist` file or NFC tag reading is unavailable.
 *
 * @example
 * ```typescript
 * import { isAvailable } from '@tauri-apps/plugin-nfc';
 * const canScan = await isAvailable();
 * ```
 *
 * @returns A promise resolving to whether NFC is available on the device.
 *
 * @since 2.0.0
 */
export async function isAvailable(): Promise<boolean> {
  const { available }: { available: boolean } = await invoke(
    'plugin:nfc|is_available'
  )
  return available
}
