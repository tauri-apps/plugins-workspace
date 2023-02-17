<script>
  import { getPhoto, ResultType, Source } from "tauri-plugin-camera-api";

  let imageSrc = "";

  async function get() {
    try {
      const image = await getPhoto({
        resultType: ResultType.Base64,
        source: Source.Camera,
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
  <button on:click={get}> Get photo </button>
</div>

<style>
  img {
    max-width: 100vw;
    max-height: 80vh;
  }
</style>
