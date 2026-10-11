<script>
  import { enable, disable, isEnabled } from '@tauri-apps/plugin-autostart'
  import { onMount } from 'svelte'

  export let onMessage

  let enabled = false

  async function refresh() {
    enabled = await isEnabled()
  }

  function toggle() {
    ;(enabled ? disable() : enable())
      .then(refresh)
      .then(() => onMessage(`Autostart ${enabled ? 'enabled' : 'disabled'}`))
      .catch(onMessage)
  }

  onMount(() => {
    refresh().catch(onMessage)
  })
</script>

<div class="flex flex-row gap-2 items-center">
  <button class="btn" on:click={toggle}>
    {enabled ? 'Disable' : 'Enable'} autostart
  </button>
  <span>Launching at login is {enabled ? 'enabled' : 'disabled'}</span>
</div>
