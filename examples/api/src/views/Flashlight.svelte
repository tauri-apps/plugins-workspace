<script>
  import { onMount } from 'svelte'
  import {
    enable,
    disable,
    toggle,
    isAvailable,
    isEnabled
  } from '@tauri-apps/plugin-flashlight'

  export let onMessage

  let available = null
  let enabled = null

  async function refresh() {
    try {
      available = await isAvailable()
      enabled = await isEnabled()
    } catch (e) {
      onMessage(e)
    }
  }

  function run(action) {
    return () =>
      action()
        .then((res) => onMessage(res ?? 'ok'))
        .catch(onMessage)
        .finally(refresh)
  }

  onMount(refresh)
</script>

<div>
  <p>
    Available: <b>{available ?? '...'}</b> / Enabled:
    <b>{enabled ?? '...'}</b>
  </p>

  <button class="btn" id="flashlight-enable" on:click={run(enable)}>enable</button>
  <button class="btn" id="flashlight-disable" on:click={run(disable)}
    >disable</button
  >
  <button class="btn" id="flashlight-toggle" on:click={run(toggle)}>toggle</button>
  <button class="btn" id="flashlight-refresh" on:click={refresh}>refresh state</button
  >
</div>

<br />

<p>
  The flashlight can also be toggled while scanning from the Scanner page, which on
  Android goes through the barcode scanner's camera.
</p>
