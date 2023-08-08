import { invoke } from "@tauri-apps/api/tauri";

export class Authenticator {
  async init(): Promise<void> {
    return await invoke("plugin:authenticator|init_auth");
  }

  async register(challenge: string, application: string): Promise<string> {
    return await invoke("plugin:authenticator|register", {
      timeout: 10000,
      challenge,
      application,
    });
  }

  async verifyRegistration(
    challenge: string,
    application: string,
    registerData: string,
    clientData: string,
  ): Promise<string> {
    return await invoke("plugin:authenticator|verify_registration", {
      challenge,
      application,
      registerData,
      clientData,
    });
  }

  async sign(
    challenge: string,
    application: string,
    keyHandle: string,
  ): Promise<string> {
    return await invoke("plugin:authenticator|sign", {
      timeout: 10000,
      challenge,
      application,
      keyHandle,
    });
  }

  async verifySignature(
    challenge: string,
    application: string,
    signData: string,
    clientData: string,
    keyHandle: string,
    pubkey: string,
  ): Promise<number> {
    return await invoke("plugin:authenticator|verify_signature", {
      challenge,
      application,
      signData,
      clientData,
      keyHandle,
      pubkey,
    });
  }
}
