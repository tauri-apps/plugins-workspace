<script>
  import { Update, check } from "tauri-plugin-updater-api";
  import { relaunch } from "tauri-plugin-process-api";

  export let onMessage;

  let isChecking, isInstalling, newUpdate;

  async function checkUpdate() {
    isChecking = true;
    try {
      const update = await check();
      onMessage(`Should update: ${update.response.available}`);
      onMessage(update.response);

      newUpdate = update;
    } catch (e) {
      onMessage(e);
    } finally {
      isChecking = false;
    }
  }

  async function install() {
    isInstalling = true;
    try {
      await newUpdate.downloadAndInstall((res) => {
        console.log("event", res);
      });
      onMessage("Installation complete, restarting...");
      await new Promise((resolve) => setTimeout(resolve, 2000));
      await relaunch();
    } catch (e) {
      console.error(e);
      onMessage(e);
    } finally {
      isInstalling = false;
    }
  }
</script>

<div class="flex children:grow children:h10">
  {#if !isChecking && !newUpdate}
    <button class="btn" on:click={checkUpdate}>Check update</button>
  {:else if !isInstalling && newUpdate}
    <button class="btn" on:click={install}>Install update</button>
  {:else}
    <button
      class="btn text-accentText dark:text-darkAccentText flex items-center justify-center"
      ><div class="spinner animate-spin" /></button
    >
  {/if}
</div>

<style>
  .spinner {
    height: 1.2rem;
    width: 1.2rem;
    border-radius: 50rem;
    color: currentColor;
    border: 2px dashed currentColor;
  }
</style>
