<script>
  import { LazyStore } from "@tauri-apps/plugin-store";
  import { onMount } from "svelte";

  export let onMessage;

  let key;
  let value;

  const store = new LazyStore("cache.json");
  let cache = {};

  onMount(async () => {
    const values = await store.entries();
    for (const [key, value] of values) {
      cache[key] = value;
    }
    cache = cache;
  });

  async function write(key, value) {
    try {
      await store.set(key, value);
      const v = await store.get(key);
      cache[key] = v;
      cache = cache;
    } catch (error) {
      onMessage(error);
    }
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
