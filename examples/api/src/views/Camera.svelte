<script>
  import { getPhoto, ResultType, Source } from "@tauri-apps/plugin-camera";

  let source = Source.Camera;
  let imageSrc = "";

  const sources = [
    {
      value: Source.Camera,
      label: "Camera",
    },
    {
      value: Source.Photo,
      label: "Photo",
    },
    {
      value: Source.Prompt,
      label: "Prompt",
    },
  ];

  async function get() {
    try {
      const image = await getPhoto({
        resultType: ResultType.Base64,
        source,
      });
      imageSrc = `data:image/png;base64, ${image.data}`;
    } catch (e) {
      console.error(e);
    }
  }
</script>

<div>
  {#if imageSrc}
    <img src={imageSrc} alt="Selected" />
  {/if}
  <div class="flex">
    <select class="input" id="dir" bind:value={source}>
      {#each sources as source}
        <option value={source.value}>{source.label}</option>
      {/each}
    </select>
    <button class="btn" on:click={get}> Get photo </button>
  </div>
</div>

<style>
  img {
    max-width: 100vw;
    max-height: 80vh;
  }
</style>
