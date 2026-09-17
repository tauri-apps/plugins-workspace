// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

import { invoke } from '@tauri-apps/api/core'

export enum BiometryType {
  None = 0,
  // Apple TouchID or Android fingerprint
  TouchID = 1,
  // Apple FaceID or Android face authentication
  FaceID = 2,
  // Android iris authentication
  Iris = 3
}

export interface Status {
  isAvailable: boolean
  biometryType: BiometryType
  error?: string
  errorCode?:
    | 'appCancel'
    | 'authenticationFailed'
    | 'invalidContext'
    | 'notInteractive'
    | 'passcodeNotSet'
    | 'systemCancel'
    | 'userCancel'
    | 'userFallback'
    | 'biometryLockout'
    | 'biometryNotAvailable'
    | 'biometryNotEnrolled'
}

export interface AuthOptions {
  /** Enables authentication using the device's password. This feature is available on both Android and iOS. */
  allowDeviceCredential?: boolean
  /** Label for the Cancel button. This feature is available on both Android and iOS. */
  cancelTitle?: string
  /** The plain data that must be encrypted after successfull biometric authentication */
  dataToEncrypt?: string
  /** The encrypted data that must be decrypted after successfull biometric authentication */
  dataToDecrypt?: string


  // iOS options
  /** Specifies the text displayed on the fallback button if biometric authentication fails. This feature is available iOS only. */
  fallbackTitle?: string

  // android options
  /** Title indicating the purpose of biometric verification. This feature is available Android only. */
  title?: string
  /** SubTitle providing contextual information of biometric verification. This feature is available Android only. */
  subtitle?: string
  /**  Specifies whether additional user confirmation is required, such as pressing a button after successful biometric authentication. This feature is available Android only. */
  confirmationRequired?: boolean
  maxAttemps?: number
}

/**
 * Checks if the biometric authentication is available.
 * @returns a promise resolving to an object containing all the information about the status of the biometry.
 */
export async function checkStatus(): Promise<Status> {
  return await invoke('plugin:biometric|status')
}

/**
 * Prompts the user for authentication using the system interface (touchID, faceID or Android Iris).
 * Rejects if the authentication fails.
 *
 * ```javascript
 * import { authenticate } from "@tauri-apps/plugin-biometric";
 * await authenticate('Open your wallet');
 * ```
 * @param reason
 * @param options
 * @returns
 */
export async function authenticate(
  reason: string,
  options?: AuthOptions
): Promise<void> {
  await invoke('plugin:biometric|authenticate', {
    reason,
    ...options
  })
}


/**
 * Encrypts/Decrypts some payload using biometric authentication. It will Prompt the user to authenticate
 * using the system interface (touchID, faceID or Android Iris).
 * Rejects if the authentication fails.
 * 
 * Warning: consider that if the data is encrypted and the user changes the biometric settings, the key needed 
 * to decrypt the data will be lost. So, it's recommended to use this only to data that the user can get back 
 * by other means (i.e. storing their password in a secure way so they can login passwordless with biometric authentication).
 *
 * ```javascript
 * import { biometricCipher } from "@tauri-apps/plugin-biometric";
 * 
 * // how to encrypt some data
 * const options = {
 *   dataToEncrypt: "...", // if not empty, will encrypt this data
 * };
 * const encryptedData = await biometricCipher('Open your wallet', options);
 * 
 * // how to decrypt the encrypted data
 * const options = {
 *   dataToDecrypt: encryptedData // if not empty, will decrypt the data (that was previously encrypted by this method)
 * };
 * const decryptedData = await biometricCipher('Open your wallet', options);
 * ```
 * @param reason
 * @param options
 * @returns
 */
export async function biometricCipher(
  reason: string,
  options?: AuthOptions
): Promise<{data: string}> {
  return await invoke<{data: string}>('plugin:biometric|biometric_cipher', {
    reason,
    ...options
  });
}

