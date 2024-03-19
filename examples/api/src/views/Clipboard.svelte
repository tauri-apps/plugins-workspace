<script>
  import * as clipboard from "@tauri-apps/plugin-clipboard-manager";
  import { open } from "@tauri-apps/plugin-dialog";
  import { arrayBufferToBase64 } from "../lib/utils";
  import { readFile } from "@tauri-apps/plugin-fs";

  export let onMessage;
  export let insecureRenderHtml;
  let text = "clipboard message";

  function writeText() {
    clipboard
      .writeText(text)
      .then(() => {
        onMessage("Wrote to the clipboard");
      })
      .catch(onMessage);
  }

  async function writeImage() {
    try {
      const res = await open({
        title: "Image to write to clipboard",
        filters: [
          {
            name: "Clipboard IMG",
            extensions: ["png", "jpg", "jpeg"],
          },
        ],
      });
      const bytes = await readFile(res.path);
      await clipboard.writeImage(bytes);
      onMessage("wrote image");
    } catch (e) {
      onMessage(e);
    }
  }

  async function read() {
    try {
      const image = await clipboard.readImage();
      arrayBufferToBase64(await image.rgba(), function (base64) {
        const src = "data:image/png;base64," + base64;
        insecureRenderHtml('<img src="' + src + '"></img>');
      });
      return;
    } catch (_) {}

    clipboard
      .readText()
      .then((contents) => {
        onMessage(`Clipboard contents: ${contents}`);
      })
      .catch((e) => {
        onMessage(e);
      });
  }
</script>

<div class="flex gap-1">
  <input
    class="grow input"
    placeholder="Text to write to the clipboard"
    bind:value={text}
  />
  <button class="btn" type="button" on:click={writeText}>Write</button>
  <button class="btn" type="button" on:click={writeImage}>Pick Image</button>

  <button class="btn" type="button" on:click={read}>Read</button>
</div>
