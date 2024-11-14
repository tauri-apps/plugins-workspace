<script>
  import * as opener from '@tauri-apps/plugin-opener'

  export let onMessage

  const programs = [
    'firefox',
    'google chrome',
    'chromium',
    'safari',
    'open',
    'start',
    'xdg-open',
    'gio',
    'gnome-open',
    'kde-open',
    'wslview'
  ]

  let url = ''
  let path = ''
  let revealPath = ''

  let urlProgram = 'Default'
  let pathProgram = 'Default'

  function openUrl() {
    opener
      .openUrl(url, urlProgram === 'Default' ? undefined : urlProgram)
      .catch(onMessage)
  }

  function openPath() {
    opener
      .openPath(path, pathProgram === 'Default' ? undefined : pathProgram)
      .catch(onMessage)
  }

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
    <select class="input" bind:value={urlProgram}>
      <option value="Default">Default</option>
      {#each programs as p}
        <option value={p}>{p}</option>
      {/each}
    </select>
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
    <select class="input" bind:value={pathProgram}>
      <option value="Default">Default</option>
      {#each programs as p}
        <option value={p}>{p}</option>
      {/each}
    </select>
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
