<script>
  import * as dialog from "tauri-plugin-dialog-api";

  let response = "";

  async function open() {
    try {
      const res = await dialog.open({
        filters: [
          {
            name: "test",
            extensions: ["image/jpg"],
          },
        ],
        multiple: false,
      });
      response = JSON.stringify(res, null, 2);
    } catch (e) {
      console.error(e);
    }
  }
</script>

<div>
  <div>{response}</div>
  <button on:click={open}> Open </button>
</div>
