// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

import type { invoke } from "@tauri-apps/api/primitives";

/** @ignore */
declare global {
  interface Window {
    __TAURI_INTERNALS__: {
      invoke: typeof invoke;
    };
  }
}

export class Authenticator {
  async init(): Promise<void> {
    return await window.__TAURI_INTERNALS__.invoke(
      "plugin:authenticator|init_auth",
    );
  }

  async register(challenge: string, application: string): Promise<string> {
    return await window.__TAURI_INTERNALS__.invoke(
      "plugin:authenticator|register",
      {
        timeout: 10000,
        challenge,
        application,
      },
    );
  }

  async verifyRegistration(
    challenge: string,
    application: string,
    registerData: string,
    clientData: string,
  ): Promise<string> {
    return await window.__TAURI_INTERNALS__.invoke(
      "plugin:authenticator|verify_registration",
      {
        challenge,
        application,
        registerData,
        clientData,
      },
    );
  }

  async sign(
    challenge: string,
    application: string,
    keyHandle: string,
  ): Promise<string> {
    return await window.__TAURI_INTERNALS__.invoke(
      "plugin:authenticator|sign",
      {
        timeout: 10000,
        challenge,
        application,
        keyHandle,
      },
    );
  }

  async verifySignature(
    challenge: string,
    application: string,
    signData: string,
    clientData: string,
    keyHandle: string,
    pubkey: string,
  ): Promise<number> {
    return await window.__TAURI_INTERNALS__.invoke(
      "plugin:authenticator|verify_signature",
      {
        challenge,
        application,
        signData,
        clientData,
        keyHandle,
        pubkey,
      },
    );
  }
}
