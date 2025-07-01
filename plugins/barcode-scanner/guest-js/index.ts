// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

import {
  invoke,
  requestPermissions as requestPermissions_,
  checkPermissions as checkPermissions_
} from '@tauri-apps/api/core'

export type { PermissionState } from '@tauri-apps/api/core'

export enum Format {
  QRCode = 'QR_CODE',
  /**
   * Not supported on iOS.
   */
  UPC_A = 'UPC_A',
  UPC_E = 'UPC_E',
  EAN8 = 'EAN_8',
  EAN13 = 'EAN_13',
  Code39 = 'CODE_39',
  Code93 = 'CODE_93',
  Code128 = 'CODE_128',
  /**
   * Not supported on iOS.
   */
  Codabar = 'CODABAR',
  ITF = 'ITF',
  Aztec = 'AZTEC',
  DataMatrix = 'DATA_MATRIX',
  PDF417 = 'PDF_417',
  /**
   * Not supported on Android. Requires iOS 15.4+
   */
  GS1DataBar = 'GS1_DATA_BAR',
  /**
   * Not supported on Android. Requires iOS 15.4+
   */
  GS1DataBarLimited = 'GS1_DATA_BAR_LIMITED',
  /**
   * Not supported on Android. Requires iOS 15.4+
   */
  GS1DataBarExpanded = 'GS1_DATA_BAR_EXPANDED'
}

export interface ScanOptions {
  cameraDirection?: 'back' | 'front'
  formats?: Format[]
  windowed?: boolean
}

export interface Scanned {
  content: string
  format: Format
  bounds: unknown
}

/**
 * Start scanning.
 * @param options
 */
export async function scan(options?: ScanOptions): Promise<Scanned> {
  return await invoke('plugin:barcode-scanner|scan', { ...options })
}

/**
 * Cancel the current scan process.
 */
export async function cancel(): Promise<void> {
  await invoke('plugin:barcode-scanner|cancel')
}

/**
 * Get permission state.
 */
export async function checkPermissions(): Promise<PermissionState> {
  return await checkPermissions_<{ camera: PermissionState }>(
    'barcode-scanner'
  ).then((r) => r.camera)
}

/**
 * Request permissions to use the camera.
 */
export async function requestPermissions(): Promise<PermissionState> {
  return await requestPermissions_<{ camera: PermissionState }>(
    'barcode-scanner'
  ).then((r) => r.camera)
}

/**
 * Open application settings. Useful if permission was denied and the user must manually enable it.
 */
export async function openAppSettings(): Promise<void> {
  await invoke('plugin:barcode-scanner|open_app_settings')
}
