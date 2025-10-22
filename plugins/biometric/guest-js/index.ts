// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

import { invoke } from '@tauri-apps/api/core'

export enum AuthMode {
  PROMPT = 0,
  ENCRYPT = 1,
  DECRYPT = 2,
}

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

export interface DecryptedCipherData {
  data: string,
}

export interface EncryptedCipherData {
  data: string,
  initializationVector: string,
}

export type CipherData = DecryptedCipherData | EncryptedCipherData;

interface BaseAuthOptions {
    allowDeviceCredential?: boolean;
    cancelTitle?: string;
    fallbackTitle?: string;
    title?: string;
    subtitle?: string;
    confirmationRequired?: boolean;
    maxAttemps?: number;
}

interface PromptAuthOptions extends BaseAuthOptions {
    mode: AuthMode.PROMPT;
}

interface EncryptAuthOptions extends BaseAuthOptions {
    mode: AuthMode.ENCRYPT;
    cipherKey: string;
    cipherData: DecryptedCipherData;
}

interface DecryptAuthOptions extends BaseAuthOptions {
    mode: AuthMode.DECRYPT;
    cipherKey: string;
    cipherData: EncryptedCipherData;
}

export type AuthOptions = BaseAuthOptions | PromptAuthOptions | EncryptAuthOptions | DecryptAuthOptions;

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
 * @returns a promise resolving to an object containing the encrypted or decrypted data if any
 */
export async function authenticate(
  reason: string,
  options?: AuthOptions
): Promise<CipherData> {
  return await invoke('plugin:biometric|authenticate', {
    reason,
    ...options
  })
}
