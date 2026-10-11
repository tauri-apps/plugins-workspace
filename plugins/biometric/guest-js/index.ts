// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

/**
 * Prompt the user for biometric authentication on Android and iOS.
 *
 * @module
 */

import { invoke } from '@tauri-apps/api/core'

/**
 * The kind of biometry hardware detected on the device.
 */
export enum BiometryType {
  /** No biometry hardware is available, or it is not enrolled with the operating system. */
  None = 0,
  /** Apple TouchID or Android fingerprint. */
  TouchID = 1,
  /** Apple FaceID or Android face authentication. */
  FaceID = 2,
  /** Android iris authentication. */
  Iris = 3
}

/**
 * The result of {@linkcode checkStatus}, describing whether biometric authentication can
 * currently be used.
 */
export interface Status {
  /** Whether the device can currently authenticate using biometrics. */
  isAvailable: boolean
  /** The kind of biometry hardware detected on the device, even when {@linkcode isAvailable} is `false`. */
  biometryType: BiometryType
  /** A human-readable reason why biometric authentication is unavailable. Only set when {@linkcode isAvailable} is `false`. */
  error?: string
  /** A platform-specific error code describing why biometric authentication is unavailable. Only set when {@linkcode isAvailable} is `false`. */
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

/**
 * Options for the {@linkcode authenticate} biometric prompt.
 */
export interface AuthOptions {
  /** Enables authentication using the device's password or PIN. Available on both Android and iOS. */
  allowDeviceCredential?: boolean
  /** Label for the cancel button. Available on both Android and iOS. */
  cancelTitle?: string

  // iOS options
  /** Text displayed on the fallback button if biometric authentication fails. **iOS only.** */
  fallbackTitle?: string

  // android options
  /** Title indicating the purpose of the biometric verification. **Android only.** */
  title?: string
  /** Subtitle providing contextual information of the biometric verification. **Android only.** */
  subtitle?: string
  /** Whether additional user confirmation is required, such as pressing a button, after successful biometric authentication. **Android only.** */
  confirmationRequired?: boolean
  /** Maximum number of attempts allowed before the prompt is dismissed. Defaults to `3`. **Android only.** */
  maxAttemps?: number
}

/**
 * Checks if the biometric authentication is available.
 * @example
 * ```typescript
 * import { checkStatus } from '@tauri-apps/plugin-biometric';
 *
 * const status = await checkStatus();
 * if (status.isAvailable) {
 *   // do something
 * }
 * ```
 * @returns a promise resolving to an object containing all the information about the status of the biometry.
 * @since 2.0.0
 */
export async function checkStatus(): Promise<Status> {
  return await invoke('plugin:biometric|status')
}

/**
 * Prompts the user for authentication using the system interface (touchID, faceID or Android Iris).
 * Rejects if the authentication fails.
 *
 * @example
 * ```typescript
 * import { authenticate } from "@tauri-apps/plugin-biometric";
 * await authenticate('Open your wallet');
 * ```
 * @param reason A message shown to the user explaining why authentication is requested.
 * @param options Configuration for the biometric prompt.
 * @returns a promise resolving to `void` once the user is successfully authenticated.
 * @since 2.0.0
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
