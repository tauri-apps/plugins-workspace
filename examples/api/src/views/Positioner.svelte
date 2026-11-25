<script>
  import {
    moveWindow,
    moveWindowConstrained,
    Position
  } from '@tauri-apps/plugin-positioner'

  export let onMessage

  // `Position` is a numeric enum, so it also maps the values back to their names
  const positions = Object.keys(Position).filter((key) => isNaN(Number(key)))

  let position = 'Center'
  let constrained = false

  function move() {
    ;(constrained ? moveWindowConstrained : moveWindow)(Position[position])
      .then(() => onMessage(`Moved the window to ${position}`))
      .catch(onMessage)
  }
</script>

<div class="flex flex-col gap-2">
  <p>
    The tray positions only resolve once the tray icon has been clicked, which
    reports its location to the plugin.
  </p>
  <div class="flex flex-row gap-2 items-center">
    <select class="input" bind:value={position}>
      {#each positions as name}
        <option value={name}>{name}</option>
      {/each}
    </select>
    <label>
      <input type="checkbox" bind:checked={constrained} />
      Constrain to the tray icon's monitor
    </label>
    <button class="btn" on:click={move}>Move window</button>
  </div>
</div>
