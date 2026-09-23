<script>
  import { Stronghold } from '@tauri-apps/plugin-stronghold'
  import { appLocalDataDir, join } from '@tauri-apps/api/path'

  export let onMessage

  const clientName = 'api-example'
  const storeKey = 'secret'

  let password = 'password'
  let stronghold
  let store
  let record = ''

  async function load() {
    try {
      const path = await join(await appLocalDataDir(), 'api.stronghold')
      stronghold = await Stronghold.load(path, password)
      let client
      try {
        client = await stronghold.loadClient(clientName)
      } catch {
        client = await stronghold.createClient(clientName)
      }
      store = client.getStore()
      const value = await store.get(storeKey)
      record = value ? new TextDecoder().decode(value) : ''
      onMessage('Stronghold loaded')
    } catch (error) {
      stronghold = null
      onMessage(error)
    }
  }

  async function save() {
    try {
      await store.insert(storeKey, Array.from(new TextEncoder().encode(record)))
      await stronghold.save()
      onMessage('Record saved')
    } catch (error) {
      onMessage(error)
    }
  }

  async function unload() {
    await stronghold.unload().catch(onMessage)
    stronghold = null
    store = null
    record = ''
  }
</script>

<div class="flex flex-col gap-2">
  {#if stronghold}
    <div class="flex flex-row gap-2">
      <input class="input grow" placeholder="Secret" bind:value={record} />
      <button class="btn" on:click={save}>Save</button>
      <button class="btn" on:click={unload}>Lock</button>
    </div>
  {:else}
    <form class="flex flex-row gap-2" on:submit|preventDefault={load}>
      <input
        class="input grow"
        type="password"
        placeholder="Password"
        bind:value={password}
      />
      <button class="btn" type="submit">Unlock</button>
    </form>
  {/if}
</div>
