<script>
  import * as opener from '@tauri-apps/plugin-opener'

  export let onMessage

  let url = ''
  let urlProgram = ''
  function openUrl() {
    opener.openUrl(url, urlProgram ? urlProgram : undefined).catch(onMessage)
  }

  let path = ''
  let pathProgram = ''
  function openPath() {
    opener
      .openPath(path, pathProgram ? pathProgram : undefined)
      .catch(onMessage)
  }

  let revealPath = ''
  function revealItemInDir() {
    opener.revealItemInDir(revealPath).catch(onMessage)
  }
</script>

<div class="flex flex-col gap-2">
  <form
    class="flex flex-row gap-2 items-center"
    on:submit|preventDefault={openUrl}
  >
    <button class="btn" type="submit">Open URL</button>
    <input
      class="input grow"
      placeholder="Type the URL to open..."
      bind:value={url}
    />
    <span> with </span>
    <input class="input" bind:value={urlProgram} />
  </form>

  <form
    class="flex flex-row gap-2 items-center"
    on:submit|preventDefault={openPath}
  >
    <button class="btn" type="submit">Open Path</button>
    <input
      class="input grow"
      placeholder="Type the path to open..."
      bind:value={path}
    />
    <span> with </span>
    <input class="input" bind:value={pathProgram} />
  </form>

  <form
    class="flex flex-row gap-2 items-center"
    on:submit|preventDefault={revealItemInDir}
  >
    <button class="btn" type="submit">Reveal</button>
    <input
      class="input grow"
      placeholder="Type the path to reveal..."
      bind:value={revealPath}
    />
  </form>
</div>
