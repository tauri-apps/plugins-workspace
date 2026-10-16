<script>
  import { authenticate, biometricCipher } from "@tauri-apps/plugin-biometric";
  import * as os from "@tauri-apps/plugin-os";

  export let onMessage;
  let allowDeviceCredential = true;

  const isAndroid = os.platform() === "android";
  let plainText = "";
  let encryptedData = "";

  function auth() {
    authenticate("Tauri API wants to show it is awesome :)", {
      allowDeviceCredential,
      cancelTitle: "Cancel request",
      fallbackTitle: "Trying the fallback option",
      title: "Tauri API Auth",
      subtitle: "Please authenticate :)",
      confirmationRequired: false,
      maxAttemps: 1,
    })
      .then(onMessage)
      .catch(onMessage);
  }

  function encrypt() {
    biometricCipher("Authenticate to encrypt the data", {
      title: "Tauri API Encrypt",
      cancelTitle: "Cancel",
      dataToEncrypt: plainText,
    })
      .then(({ data }) => {
        encryptedData = data;
        onMessage(`Encrypted: ${data}`);
      })
      .catch(onMessage);
  }

  function decrypt() {
    biometricCipher("Authenticate to decrypt the data", {
      title: "Tauri API Decrypt",
      cancelTitle: "Cancel",
      dataToDecrypt: encryptedData,
    })
      .then(({ data }) => onMessage(`Decrypted: ${data}`))
      .catch(onMessage);
  }
</script>

<div>
  <input
    type="checkbox"
    id="allowDeviceCredential"
    bind:checked={allowDeviceCredential}
  />
  <label for="allowDeviceCredential">Allow device credential</label>
</div>
<button class="btn" id="biometric-authenticate" on:click={auth}>
  Authenticate
</button>

{#if isAndroid}
  <div class="flex flex-col gap-2 mt-4">
    <h3>Biometric cipher</h3>
    <input class="input" placeholder="Data to encrypt" bind:value={plainText} />
    <button class="btn" id="biometric-encrypt" on:click={encrypt}>
      Encrypt
    </button>
    <input
      class="input"
      placeholder="Encrypted data"
      bind:value={encryptedData}
    />
    <button
      class="btn"
      id="biometric-decrypt"
      disabled={!encryptedData}
      on:click={decrypt}
    >
      Decrypt
    </button>
  </div>
{/if}
