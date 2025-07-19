<script>
  import { appDataDir, resolve } from '@tauri-apps/api/path'
  import { LazyStore } from '@tauri-apps/plugin-store'
  import { onMount } from 'svelte'

  let { onMessage } = $props()

  let key = $state()
  let value = $state()

  const storeName = 'cache.json'

  let store = new LazyStore(storeName)
  let path = $state('')
  let cache = $state({})

  async function refreshEntries() {
    try {
      const values = await store.entries()
      cache = {}
      for (const [key, value] of values) {
        cache[key] = value
      }
    } catch (error) {
      onMessage(error)
    }
  }

  onMount(async () => {
    path = await resolve(await appDataDir(), storeName)
    await refreshEntries()
  })

  async function write(key, value) {
    try {
      if (value) {
        await store.set(key, value)
      } else {
        await store.delete(key)
      }
      const v = await store.get(key)
      if (v === undefined) {
        delete cache[key]
        cache = cache
      } else {
        cache[key] = v
      }
    } catch (error) {
      onMessage(error)
    }
  }

  async function reset() {
    try {
      await store.reset()
    } catch (error) {
      onMessage(error)
    }
    await refreshEntries()
  }

  async function reload() {
    try {
      await store.reload({ overrideDefaults: true })
    } catch (error) {
      onMessage(error)
    }
    await refreshEntries()
  }

  async function close() {
    try {
      await store.close()
      onMessage('Store is now closed, any new operations will error out')
    } catch (error) {
      onMessage(error)
    }
  }

  function reopen() {
    store = new LazyStore(storeName)
    onMessage('We made a new `LazyStore` instance, operations will now work')
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
      <button class="btn" onclick={() => write(key, value)}>Write</button>
      <button class="btn" onclick={() => reset()}>Reset</button>
      <button class="btn" onclick={() => reload()}>Reload</button>
      <button class="btn" onclick={() => close()}>Close</button>
      <button class="btn" onclick={() => reopen()}>Re-open</button>
    </div>
    <div>Store at <code>{path}</code> on disk</div>
  </div>

  <div>
    <h2>Store Values</h2>
    {#each Object.entries(cache) as [k, v]}
      <div>{k} = {v}</div>
    {/each}
  </div>
</div>
