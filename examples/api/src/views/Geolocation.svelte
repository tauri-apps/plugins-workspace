<script>
  import {
    checkPermissions,
    requestPermissions,
    getCurrentPosition
  } from '@tauri-apps/plugin-geolocation'

  export let onMessage

  async function getPosition() {
    let permissions = await checkPermissions()
    if (
      permissions.location === 'prompt' ||
      permissions.location === 'prompt-with-rationale'
    ) {
      permissions = await requestPermissions(['location'])
    }

    if (permissions.location === 'granted') {
      getCurrentPosition().then(onMessage).catch(onMessage)
    } else {
      onMessage('permission denied')
    }
  }
</script>

<button class="btn" id="cli-matches" on:click={getPosition}>
  Get Position
</button>
