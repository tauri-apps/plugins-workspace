<script>
  import { authenticate } from "@tauri-apps/plugin-biometric";

  export let onMessage;
  let allowDeviceCredential = true;

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
</script>

<div>
  <input
    type="checkbox"
    id="dllowDeviceCredential"
    bind:checked={allowDeviceCredential}
  />
  <label for="allowDeviceCredentiale">Allow device credential</label>
</div>
<button class="btn" id="cli-matches" on:click={auth}> Authenticate </button>
