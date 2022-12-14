import { invoke } from '@tauri-apps/api/tauri';

class Authenticator {
    async init() {
        return await invoke("plugin:authenticator|init");
    }
    async register(challenge, application) {
        return await invoke("plugin:authenticator|register", {
            timeout: 10000,
            challenge,
            application,
        });
    }
    async verifyRegistration(challenge, application, registerData, clientData) {
        return await invoke("plugin:authenticator|verify_registration", {
            challenge,
            application,
            registerData,
            clientData,
        });
    }
    async sign(challenge, application, keyHandle) {
        return await invoke("plugin:authenticator|sign", {
            timeout: 10000,
            challenge,
            application,
            keyHandle,
        });
    }
    async verifySignature(challenge, application, signData, clientData, keyHandle, pubkey) {
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

export { Authenticator };
//# sourceMappingURL=index.mjs.map
