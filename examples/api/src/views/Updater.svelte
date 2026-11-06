<script lang="ts">
  import { check, Update, type CacheMode } from '@tauri-apps/plugin-updater'
  import { relaunch } from '@tauri-apps/plugin-process'
  import { onDestroy } from 'svelte'

  let { onMessage } = $props()

  let cacheMode = $state<CacheMode>('checkNow')
  let isChecking = $state(false)
  let isInstalling = $state(false)
  let newUpdate = $state<Update | undefined>()
  let totalSize = $state(0)
  let downloadedSize = $state(0)
  let progress = $derived(
    totalSize ? Math.round((downloadedSize / totalSize) * 100) : 0
  )

  async function checkUpdate() {
    isChecking = true
    try {
      const update = await check({ cacheMode })
      if (update) {
        onMessage(`Should update: ${update.available}`)
        onMessage(update)

        newUpdate = update
      } else {
        onMessage('No update available')
      }
    } catch (e) {
      onMessage(e)
    } finally {
      isChecking = false
    }
  }

  async function install() {
    isInstalling = true
    downloadedSize = 0
    try {
      await newUpdate!.downloadAndInstall((downloadProgress) => {
        switch (downloadProgress.event) {
          case 'Started':
            totalSize = downloadProgress.data.contentLength!
            break
          case 'Progress':
            downloadedSize += downloadProgress.data.chunkLength
            break
          case 'Finished':
            break
        }
      })
      onMessage('Installation complete, restarting...')
      await new Promise((resolve) => setTimeout(resolve, 2000))
      await relaunch()
    } catch (e) {
      console.error(e)
      onMessage(e)
    } finally {
      isInstalling = false
    }
  }

  onDestroy(() => {
    newUpdate?.close()
  })
</script>

{#if !isChecking && !newUpdate}
  <div class="flex flex-col gap-2">
    <button class="btn h10" onclick={checkUpdate}>Check update</button>
    <label for="updater-cache-mode" class="mt-4 font-semibold">Cache mode</label
    >
    <select class="input h10" id="updater-cache-mode" bind:value={cacheMode}>
      <option value="default">Default</option>
      <option value="bypass">Bypass</option>
      <option value="checkNow">Check now</option>
    </select>
  </div>
{:else}
  <div class="flex children:grow children:h10">
    {#if !isInstalling && newUpdate}
      <button class="btn" onclick={install}>Install update</button>
    {:else}
      <div class="progress">
        <span>{progress}%</span>
        <div class="progress-bar" style="width: {progress}%"></div>
      </div>
    {/if}
  </div>
{/if}

<style>
  .progress {
    width: 100%;
    height: 50px;
    position: relative;
    margin-top: 5%;
  }

  .progress > span {
    font-size: 1.2rem;
  }

  .progress-bar {
    height: 30px;
    background-color: hsl(32, 94%, 46%);
    border: 1px solid #333;
  }
</style>
