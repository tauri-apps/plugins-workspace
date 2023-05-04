import { invoke } from '@tauri-apps/api/tauri'

type PermissionState = 'granted' | 'denied'

type Format = 'QR_CODE' | 'UPC_A' | 'UPC_E' | 'EAN_8' | 'EAN_13' | 'CODE_39' | 'CODE_93' | 'CODE_128' | 'CODABAR' | 'ITF' | 'AZTEC' | 'DATA_MATRIX' | 'PDF_417'

interface ScanOptions {
  cameraDirection?: 'back' | 'front'
  formats?: Format[]
}

interface PrepareOptions {
  cameraDirection?: 'back' | 'front'
}

/**
 * Prepare the camera before starting scanning.
 * This is optional, only use it if you need a small performance improvement when preparing a scan in advance.
 * @param options 
 */
export async function prepare(options?: PrepareOptions) {
  invoke('plugin:barcodeScanner|prepare', { ...options })
}

/**
 * Start scanning.
 * @param options 
 */
export async function scan(options?: ScanOptions) {
  invoke('plugin:barcodeScanner|scan', { ...options })
}

/**
 * Cancel the current scan process.
 */
export async function cancel() {
  invoke('plugin:barcodeScanner|cancel')
}

/**
 * Get permission state.
 */
export async function checkPermissions(): Promise<PermissionState> {
  return invoke<{ camera: PermissionState }>('plugin:barcodeScanner|check_permissions').then(r => r.camera)
}

/**
 * Request permissions to use the camera.
 */
export async function requestPermissions(): Promise<PermissionState> {
  return invoke<{ camera: PermissionState }>('plugin:barcodeScanner|request_permissions').then(r => r.camera)
}

/**
 * Open application settings. Useful if permission was denied and the user must manually enable it.
 */
export async function openAppSettings() {
  invoke('plugin:barcodeScanner|open_app_settings')
}
