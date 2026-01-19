<script>
  import { open, save, confirm, message } from "@tauri-apps/plugin-dialog";
  import { readFile } from "@tauri-apps/plugin-fs";

  export let onMessage;
  export let insecureRenderHtml;
  let defaultPath = null;
  let filter = null;
  let multiple = false;
  let directory = false;
  let pickerMode = "document";
  let fileAccessMode = "scoped";

  function arrayBufferToBase64(buffer, callback) {
    var blob = new Blob([buffer], {
      type: "application/octet-binary",
    });
    var reader = new FileReader();
    reader.onload = function (evt) {
      var dataurl = evt.target.result;
      callback(dataurl.substr(dataurl.indexOf(",") + 1));
    };
    reader.readAsDataURL(blob);
  }

  async function prompt() {
    confirm("Do you want to do something?")
      .then((res) => onMessage(res ? "Yes" : "No"))
      .catch(onMessage);
  }

  async function promptCustom() {
    confirm("Is Tauri awesome?", {
      okLabel: "Absolutely",
      cancelLabel: "Totally",
    })
      .then((res) =>
        onMessage(
          res ? "Tauri is absolutely awesome" : "Tauri is totally awesome"
        )
      )
      .catch(onMessage);
  }

  async function msg() {
    await message("Tauri is awesome!");
  }

  async function msgCustom(result) {
    const buttons = { yes: "awesome", no: "amazing", cancel: "stunning" };
    await message(`Tauri is: `, { buttons })
      .then((res) => onMessage(`Tauri is ${res}`))
      .catch(onMessage);
  }

  async function openDialog() {
    try {
      var result = await open({
        title: "My wonderful open dialog",
        defaultPath,
        filters: filter
          ? [
              {
                name: "Tauri Example",
                extensions: filter.split(",").map((f) => f.trim()),
              },
            ]
          : [],
        multiple,
        directory,
        pickerMode,
        fileAccessMode,
      })

      if (Array.isArray(result)) {
        onMessage(result);
      } else {
        var pathToRead = result;
        var isFile = pathToRead.match(/\S+\.\S+$/g);

        await readFile(pathToRead)
          .then(function (res) {
            if (isFile) {
              if (
                pathToRead.includes(".png") ||
                pathToRead.includes(".jpg") ||
                pathToRead.includes(".jpeg")
              ) {
                arrayBufferToBase64(
                  new Uint8Array(res),
                  function (base64) {
                    var src = "data:image/png;base64," + base64;
                    insecureRenderHtml('<img src="' + src + '"></img>');
                  }
                );
              } else {
                // Convert byte array to UTF-8 string
                const decoder = new TextDecoder('utf-8');
                const text = decoder.decode(new Uint8Array(res));
                onMessage(text);
              }
            } else {
              onMessage(res);
          }
        })
      }
    } catch(exception) {
      onMessage(exception)
    }
  }

  function saveDialog() {
    save({
      title: "My wonderful save dialog",
      defaultPath,
      filters: filter
        ? [
            {
              name: "Tauri Example",
              extensions: filter.split(",").map((f) => f.trim()),
            },
          ]
        : [],
      })
      .then(onMessage)
      .catch(onMessage);
  }
</script>

<div class="flex gap-2 children:grow">
  <input
    class="input"
    id="dialog-default-path"
    placeholder="Default path"
    bind:value={defaultPath}
  />
  <input
    class="input"
    id="dialog-filter"
    placeholder="Extensions filter, comma-separated"
    bind:value={filter}
  />
</div>

<br />
<div>
  <input type="checkbox" id="dialog-multiple" bind:checked={multiple} />
  <label for="dialog-multiple">Multiple</label>
</div>
<div>
  <input type="checkbox" id="dialog-directory" bind:checked={directory} />
  <label for="dialog-directory">Directory</label>
</div>
<div>
  <label for="dialog-picker-mode">Picker Mode:</label>
  <select id="dialog-picker-mode" bind:value={pickerMode}>
    <option value="">None</option>
    <option value="media">Media</option>
    <option value="image">Image</option>
    <option value="video">Video</option>
    <option value="document">Document</option>
  </select>
</div>
<div>
  <label for="dialog-file-access-mode">File Access Mode:</label>
  <select id="dialog-file-access-mode" bind:value={fileAccessMode}>
    <option value="copy">Copy</option>
    <option value="scoped">Scoped</option>
  </select>
</div>
<br />

<div class="flex flex-wrap flex-col md:flex-row gap-2 children:flex-shrink-0">
  <button class="btn" id="open-dialog" on:click={openDialog}>Open dialog</button>
  <button class="btn" id="save-dialog" on:click={saveDialog}
    >Open save dialog</button
  >
  <button class="btn" id="prompt-dialog" on:click={prompt}>Prompt</button>
  <button class="btn" id="custom-prompt-dialog" on:click={promptCustom}
    >Prompt (custom)</button
  >
  <button class="btn" id="message-dialog" on:click={msg}>Message</button>
  <button class="btn" id="message-dialog" on:click={msgCustom}>Message (custom)</button>

</div>
