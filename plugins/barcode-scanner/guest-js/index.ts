// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

/**
 * Scan QR codes, EAN-13 and other kinds of barcodes with the device's camera on Android and iOS.
 *
 * @module
 */

import {
  invoke,
  requestPermissions as requestPermissions_,
  checkPermissions as checkPermissions_
} from '@tauri-apps/api/core'

export type { PermissionState } from '@tauri-apps/api/core'

/**
 * The barcode symbologies that can be scanned, or used to restrict a scan via {@link ScanOptions.formats}.
 */
export enum Format {
  /**
   * QR code, a two-dimensional matrix barcode.
   */
  QRCode = 'QR_CODE',
  /**
   * UPC-A, a 12-digit numeric barcode commonly used on retail products in North America.
   *
   * Not supported on iOS.
   */
  UPC_A = 'UPC_A',
  /**
   * UPC-E, a compressed 6-digit variant of UPC-A used on small packaging.
   */
  UPC_E = 'UPC_E',
  /**
   * EAN-8, an 8-digit numeric barcode used on small retail packaging.
   */
  EAN8 = 'EAN_8',
  /**
   * EAN-13, a 13-digit numeric barcode used worldwide on retail products.
   */
  EAN13 = 'EAN_13',
  /**
   * Code 39, an alphanumeric barcode used in logistics and inventory tracking.
   */
  Code39 = 'CODE_39',
  /**
   * Code 93, a compact alphanumeric barcode similar to Code 39.
   */
  Code93 = 'CODE_93',
  /**
   * Code 128, a high-density alphanumeric barcode used in shipping and packaging.
   */
  Code128 = 'CODE_128',
  /**
   * Codabar, a numeric barcode commonly used by libraries and blood banks.
   *
   * Not supported on iOS.
   */
  Codabar = 'CODABAR',
  /**
   * ITF (Interleaved 2 of 5), a numeric barcode encoding an even number of digits.
   */
  ITF = 'ITF',
  /**
   * Aztec code, a two-dimensional matrix barcode often used on tickets and boarding passes.
   */
  Aztec = 'AZTEC',
  /**
   * Data Matrix, a two-dimensional matrix barcode used to encode small amounts of data.
   */
  DataMatrix = 'DATA_MATRIX',
  /**
   * PDF417, a stacked linear barcode used on IDs, boarding passes and shipping labels.
   */
  PDF417 = 'PDF_417',
  /**
   * GS1 DataBar, a compact barcode used to mark variable-measure items such as fresh food.
   *
   * Not supported on Android. Requires iOS 15.4+
   */
  GS1DataBar = 'GS1_DATA_BAR',
  /**
   * The limited variant of {@link Format.GS1DataBar}, encoding fewer digits in a smaller symbol.
   *
   * Not supported on Android. Requires iOS 15.4+
   */
  GS1DataBarLimited = 'GS1_DATA_BAR_LIMITED',
  /**
   * The expanded variant of {@link Format.GS1DataBar}, capable of encoding additional data such as weight.
   *
   * Not supported on Android. Requires iOS 15.4+
   */
  GS1DataBarExpanded = 'GS1_DATA_BAR_EXPANDED'
}

/**
 * Options to configure a {@link scan} call.
 */
export interface ScanOptions {
  /**
   * Which camera to use for scanning. Defaults to `back`.
   */
  cameraDirection?: 'back' | 'front'
  /**
   * The barcode formats to scan for. Defaults to all supported formats.
   */
  formats?: Format[]
  /**
   * Whether to show the camera in a small window instead of taking over the whole screen. Defaults to `false`.
   */
  windowed?: boolean
}

/**
 * The result of a successful {@link scan} call.
 */
export interface Scanned {
  /**
   * The decoded content of the barcode.
   */
  content: string
  /**
   * The format of the scanned barcode.
   */
  format: Format
  /**
   * The bounding box of the scanned barcode within the camera frame, when reported by the platform.
   */
  bounds: unknown
}

/**
 * Start scanning, opening the device's camera. The returned promise resolves once a barcode
 * matching the given options has been scanned, or rejects if the scan is cancelled.
 *
 * @example
 * ```typescript
 * import { scan, Format } from '@tauri-apps/plugin-barcode-scanner';
 *
 * const scanned = await scan({ windowed: true, formats: [Format.QRCode] });
 * ```
 *
 * @param options Configuration for the scan.
 * @returns A promise resolving to the scanned barcode.
 * @since 2.0.0
 */
export async function scan(options?: ScanOptions): Promise<Scanned> {
  return await invoke('plugin:barcode-scanner|scan', { ...options })
}

/**
 * Cancel the current scan process.
 *
 * @example
 * ```typescript
 * import { cancel } from '@tauri-apps/plugin-barcode-scanner';
 *
 * await cancel();
 * ```
 *
 * @since 2.0.0
 */
export async function cancel(): Promise<void> {
  await invoke('plugin:barcode-scanner|cancel')
}

/**
 * Get the current state of the camera permission.
 *
 * @example
 * ```typescript
 * import { checkPermissions } from '@tauri-apps/plugin-barcode-scanner';
 *
 * const permissionState = await checkPermissions();
 * ```
 *
 * @returns A promise resolving to the current state of the camera permission.
 * @since 2.0.0
 */
export async function checkPermissions(): Promise<PermissionState> {
  return await checkPermissions_<{ camera: PermissionState }>(
    'barcode-scanner'
  ).then((r) => r.camera)
}

/**
 * Request permissions to use the camera.
 *
 * @example
 * ```typescript
 * import { requestPermissions } from '@tauri-apps/plugin-barcode-scanner';
 *
 * const permissionState = await requestPermissions();
 * ```
 *
 * @returns A promise resolving to the new state of the camera permission.
 * @since 2.0.0
 */
export async function requestPermissions(): Promise<PermissionState> {
  return await requestPermissions_<{ camera: PermissionState }>(
    'barcode-scanner'
  ).then((r) => r.camera)
}

/**
 * Open application settings. Useful if permission was denied and the user must manually enable it.
 *
 * @example
 * ```typescript
 * import { openAppSettings } from '@tauri-apps/plugin-barcode-scanner';
 *
 * await openAppSettings();
 * ```
 *
 * @since 2.0.0
 */
export async function openAppSettings(): Promise<void> {
  await invoke('plugin:barcode-scanner|open_app_settings')
}
