<script lang="ts">
	import WebSocket from 'tauri-plugin-websocket-api'
	import { onMount } from 'svelte'

	let ws
	let response = ''
	let message = ''

	onMount(async () => {
		ws = await WebSocket.connect('ws://127.0.0.1:8080').then(r => {
			_updateResponse('Connected')
			return r
		}).catch(_updateResponse)
		ws.addListener(_updateResponse)
	})

	function _updateResponse(returnValue) {
		response += (typeof returnValue === 'string' ? returnValue : JSON.stringify(returnValue)) + '<br>'
	}

	function send() {
		ws.send(message).then(() => _updateResponse('Message sent')).catch(_updateResponse)
	}

	function disconnect() {
		ws.disconnect().then(() => _updateResponse('Disconnected')).catch(_updateResponse)
	}
</script>

<div>
	<input bind:value={message}>
	<button on:click={send}>Send</button>
	<button on:click={disconnect}>Disconnect</button>
</div>
<div>{@html response}</div>
