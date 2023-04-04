<script>
  import * as dialog from "tauri-plugin-dialog-api";

  let response = "";

  async function prompt() {
    try {
      const res = await dialog.confirm("Is Tauri awesome?", {
        okLabel: "Absolutely",
        cancelLabel: "Totally",
      });
      response = res
        ? "Tauri is absolutely awesome"
        : "Tauri is totally awesome";
    } catch (e) {
      console.error(e);
    }
  }
</script>

<div>
  <div>{response}</div>
  <button on:click={prompt}> Prompt </button>
</div>
