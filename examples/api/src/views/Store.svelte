<script>
  import { Store } from "@tauri-apps/plugin-store";
  import { onMount } from "svelte";

  export let onMessage;

  let key;
  let value;

  const store = new Store("cache.json");
  let cache = {};

  onMount(async () => {
    await store.load();
    const values = await store.entries();
    for (const [key, value] of values) {
      cache[key] = value;
    }
    cache = cache;
  });

  function write(key, value) {
    store
      .set(key, value)
      .then(() => store.get(key))
      .then((v) => {
        cache[key] = v;
        cache = cache;
      })
      .then(() => store.save())
      .catch(onMessage);
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

    <button class="btn" on:click={() => write(key, value)}> Write </button>
  </div>

  <div>
    {#each Object.entries(cache) as [k, v]}
      <div>{k} = {v}</div>
    {/each}
  </div>
</div>
