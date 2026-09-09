<script>
  import {
    checkPermissions,
    requestPermissions,
    getCurrentPosition,

    watchPosition,
    clearWatch

  } from '@tauri-apps/plugin-geolocation'

  export let onMessage

  let pos = null
  let watchId = null

  async function getPosition() {
    let permissions = await checkPermissions()
    if (
      permissions.location === 'prompt' ||
      permissions.location === 'prompt-with-rationale'
    ) {
      permissions = await requestPermissions(['location'])
    }

    if (permissions.location === 'granted') {
      getCurrentPosition().then((position) => {
        pos = position
        onMessage(position)
      }).catch((err) => {
        pos = null
        onMessage(err)
      })
    } else {
      onMessage('permission denied')
    }
  }

  async function watchPos() {
    let permissions = await checkPermissions()
    if (
      permissions.location === 'prompt' ||
      permissions.location === 'prompt-with-rationale'
    ) {
      permissions = await requestPermissions(['location'])
    }

    if (permissions.location === 'granted') {
      watchId = await watchPosition({
        enableHighAccuracy: true,
        timeout: 5000,
        maximumAge: 0
      }, (position) => {
        pos = position
        onMessage(position)
      })
      onMessage('watchId: ' + watchId)
    } else {
      onMessage('permission denied')
    }
  }

  async function stopWatching() {
    await clearWatch(watchId)
    watchId = null
    pos = null
  }

</script>

<button class="btn" id="cli-matches" on:click={getPosition}>
  Get Position
</button>

<button class="btn" on:click={watchPos}>
  Watch Position
</button>

<button class="btn" on:click={stopWatching}>
  Stop Watching
</button>

{#if watchId}
  <span>Watch ID: {watchId}</span>
{/if}

{#if pos}
  <pre>{JSON.stringify(pos, null, 2)}</pre>
{/if}
