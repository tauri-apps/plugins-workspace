<script>
  import { onMount } from 'svelte'
  import { getString, setString } from '@tauri-apps/plugin-secure-storage'

  export let onMessage

  let key
  let value
  let output

  async function read(key) {
    try {
      output = await getString(key)
    } catch (error) {
      onMessage(error)
    }
  }

  async function write(key, value) {
    try {
      await setString(key, value)
    } catch (error) {
      onMessage(error)
    }
  }

  function reset() {
    output = ''
  }
</script>

<div class="flex flex-col childre:grow gap-1">
  <div class="flex flex-col flex-row-md gap-4">
    <div class="flex items-center gap-1">
      Key:
      <input class="grow input" bind:value={key} />
    </div>

    <div class="flex items-center gap-1">
      Value:
      <input class="grow input" bind:value />
    </div>

    <div>
      <button class="btn" on:click={() => write(key, value)}>Write</button>
      <button class="btn" on:click={() => read(key)}>Read</button>
      <button class="btn" on:click={() => reset()}>Reset Output</button>
    </div>
  </div>

  <div>
    Output: {output}
  </div>
</div>
