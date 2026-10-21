<script>
  import WebSocket from '@tauri-apps/plugin-websocket'
  import { onDestroy } from 'svelte'

  export let onMessage

  let url = 'wss://echo.websocket.org'
  let message = ''
  let ws

  async function connect() {
    try {
      ws = await WebSocket.connect(url)
      ws.addListener((received) => {
        onMessage(received)
        if (received.type === 'Close') {
          ws = null
        }
      })
      onMessage(`Connected to ${url}`)
    } catch (error) {
      onMessage(error)
    }
  }

  function send() {
    ws.send(message)
      .then(() => (message = ''))
      .catch(onMessage)
  }

  function disconnect() {
    ws.disconnect().catch(onMessage)
    ws = null
  }

  onDestroy(() => {
    ws?.disconnect().catch(() => {})
  })
</script>

<div class="flex flex-col gap-2">
  {#if ws}
    <form class="flex flex-row gap-2" on:submit|preventDefault={send}>
      <input class="input grow" placeholder="Message" bind:value={message} />
      <button class="btn" type="submit">Send</button>
      <button class="btn" type="button" on:click={disconnect}>Disconnect</button>
    </form>
  {:else}
    <form class="flex flex-row gap-2" on:submit|preventDefault={connect}>
      <input class="input grow" placeholder="ws:// or wss:// URL" bind:value={url} />
      <button class="btn" type="submit">Connect</button>
    </form>
  {/if}
</div>
