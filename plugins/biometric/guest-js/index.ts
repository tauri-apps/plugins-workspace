// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

declare global {
  interface Window {
    __TAURI_INVOKE__: <T>(cmd: string, args?: unknown) => Promise<T>;
  }
}

export enum BiometryType {
  None = 0,
  // Apple TouchID or Android fingerprint
  TouchID = 1,
  // Apple FaceID or Android face authentication
  FaceID = 2,
  // Android iris authentication
  Iris = 3,
}

export interface Status {
  isAvailable: boolean;
  biometryType: BiometryType;
  error?: string;
  errorCode?:
    | "appCancel"
    | "authenticationFailed"
    | "invalidContext"
    | "notInteractive"
    | "passcodeNotSet"
    | "systemCancel"
    | "userCancel"
    | "userFallback"
    | "biometryLockout"
    | "biometryNotAvailable"
    | "biometryNotEnrolled";
}

export interface AuthOptions {
  allowDeviceCredential?: boolean;

  // iOS options
  fallbackTitle?: string;
  cancelTitle?: string;
  
  // android options
  title?: string;
  subtitle?: string;
  confirmationRequired?: boolean;
}

export async function checkStatus(): Promise<Status> {
  return window.__TAURI_INVOKE__("plugin:biometry|status");
}

export async function authenticate(
  reason: string,
  options?: AuthOptions,
): Promise<void> {
  return window.__TAURI_INVOKE__("plugin:biometry|authenticate", {
    reason,
    ...options,
  });
}
