<script>
  import {
    getPhoto,
    pickImages,
    checkPermissions,
    requestPermissions,
    ResultType,
    Source
  } from '@tauri-apps/plugin-camera'

  export let onMessage

  const sources = [
    { value: Source.Prompt, label: 'Prompt' },
    { value: Source.Camera, label: 'Camera' },
    { value: Source.Photos, label: 'Photos' }
  ]

  let source = Source.Prompt
  let saveToGallery = false
  let allowEditing = false
  let imageSrc = ''

  async function get() {
    try {
      const image = await getPhoto({
        resultType: ResultType.DataUrl,
        source,
        saveToGallery,
        allowEditing,
        width: 1024
      })
      if (image) {
        imageSrc = image.data
        onMessage({ format: image.format, saved: image.saved, exif: image.exif })
      } else {
        onMessage('cancelled')
      }
    } catch (e) {
      onMessage(e)
    }
  }

  async function pick() {
    try {
      const images = await pickImages({ limit: 5, width: 1024 })
      onMessage(
        images.map(({ data, assetUrl, format }) => ({ data, assetUrl, format }))
      )
    } catch (e) {
      onMessage(e)
    }
  }
</script>

<div>
  <div class="flex gap-2 items-center">
    <select class="input" id="camera-source" bind:value={source}>
      {#each sources as { value, label }}
        <option {value}>{label}</option>
      {/each}
    </select>
    <label>
      <input type="checkbox" bind:checked={saveToGallery} />
      Save to gallery
    </label>
    <label>
      <input type="checkbox" bind:checked={allowEditing} />
      Allow editing
    </label>
  </div>

  <br />

  <button class="btn" id="camera-get-photo" on:click={get}>Get photo</button>
  <button class="btn" id="camera-pick-images" on:click={pick}
    >Pick images</button
  >
  <button
    class="btn"
    id="camera-check-permissions"
    on:click={() => checkPermissions().then(onMessage).catch(onMessage)}
    >Check permissions</button
  >
  <button
    class="btn"
    id="camera-request-permissions"
    on:click={() => requestPermissions().then(onMessage).catch(onMessage)}
    >Request permissions</button
  >

  {#if imageSrc}
    <br />
    <img src={imageSrc} alt="Selected" />
  {/if}
</div>

<style>
  img {
    max-width: 100%;
    max-height: 60vh;
  }
</style>
