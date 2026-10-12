<script>
  import {
    getCurrent,
    onOpenUrl,
    register,
    unregister,
    isRegistered
  } from '@tauri-apps/plugin-deep-link'
  import { onMount, onDestroy } from 'svelte'

  export let onMessage

  let current = null
  let protocol = 'tauri-api'
  let unlisten

  onMount(async () => {
    try {
      current = await getCurrent()
      unlisten = await onOpenUrl((urls) => {
        current = urls
        onMessage(`Opened with ${urls.join(', ')}`)
      })
    } catch (error) {
      onMessage(error)
    }
  })

  onDestroy(() => {
    unlisten?.()
  })

  function registerProtocol() {
    register(protocol)
      .then(() => onMessage(`Registered ${protocol}://`))
      .catch(onMessage)
  }

  function unregisterProtocol() {
    unregister(protocol)
      .then(() => onMessage(`Unregistered ${protocol}://`))
      .catch(onMessage)
  }

  function checkProtocol() {
    isRegistered(protocol)
      .then((registered) =>
        onMessage(
          `${protocol}:// is ${registered ? '' : 'not '}handled by this app`
        )
      )
      .catch(onMessage)
  }
</script>

<div class="flex flex-col gap-2">
  <div>
    Current deep link: <code>{current ? current.join(', ') : 'none'}</code>
  </div>

  <div class="flex flex-row gap-2 items-center">
    <input class="input grow" placeholder="Scheme" bind:value={protocol} />
    <button class="btn" on:click={registerProtocol}>Register</button>
    <button class="btn" on:click={unregisterProtocol}>Unregister</button>
    <button class="btn" on:click={checkProtocol}>Is registered?</button>
  </div>
</div>
