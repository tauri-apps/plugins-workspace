<script>
  import * as fs from "@tauri-apps/plugin-fs";
  import { convertFileSrc } from "@tauri-apps/api/core";

  export let onMessage;
  export let insecureRenderHtml;

  let path = "";
  let img;
  let file;
  let renameTo;

  function getDir() {
    const dirSelect = document.getElementById("dir");
    return dirSelect.value ? parseInt(dir.value) : null;
  }

  function arrayBufferToBase64(buffer, callback) {
    const blob = new Blob([buffer], {
      type: "application/octet-binary",
    });
    const reader = new FileReader();
    reader.onload = function (evt) {
      const dataurl = evt.target.result;
      callback(dataurl.substr(dataurl.indexOf(",") + 1));
    };
    reader.readAsDataURL(blob);
  }

  const DirOptions = Object.keys(fs.BaseDirectory)
    .filter((key) => isNaN(parseInt(key)))
    .map((dir) => [dir, fs.BaseDirectory[dir]]);

  function open() {
    fs.open(path, {
      baseDir: getDir(),
      read: true,
      write: true,
      create: true,
    })
      .then((f) => {
        file = f;
        onMessage(`Opened ${path}`);
      })
      .catch(onMessage);
  }

  function mkdir() {
    fs.mkdir(path, { baseDir: getDir() })
      .then(() => {
        onMessage(`Created dir ${path}`);
      })
      .catch(onMessage);
  }

  function remove() {
    fs.remove(path, { baseDir: getDir() })
      .then(() => {
        onMessage(`Removed ${path}`);
      })
      .catch(onMessage);
  }

  function rename() {
    fs.rename(path, renameTo, {
      oldPathBaseDir: getDir(),
      newPathBaseDir: getDir(),
    })
      .then(() => {
        onMessage(`Renamed ${path} to ${renameTo}`);
      })
      .catch(onMessage);
  }

  function truncate() {
    file
      .truncate(0)
      .then(() => {
        onMessage(`Truncated file`);
      })
      .catch(onMessage);
  }

  function stat() {
    file
      .stat()
      .then((stat) => {
        onMessage(`File stat ${JSON.stringify(stat)}`);
      })
      .catch(onMessage);
  }

  function read() {
    const opts = {
      baseDir: getDir(),
    };
    fs.stat(path, opts)
      .then((stat) => {
        const isFile = stat.isFile;

        const promise = isFile
          ? fs.readFile(path, opts)
          : fs.readDir(path, opts);
        promise
          .then(function (response) {
            if (isFile) {
              if (path.includes(".png") || path.includes(".jpg")) {
                arrayBufferToBase64(
                  new Uint8Array(response),
                  function (base64) {
                    const src = "data:image/png;base64," + base64;
                    insecureRenderHtml('<img src="' + src + '"></img>');
                  }
                );
              } else {
                const value = String.fromCharCode.apply(null, response);
                insecureRenderHtml(
                  '<textarea id="file-response"></textarea><button id="file-save">Save</button>'
                );
                setTimeout(() => {
                  const fileInput = document.getElementById("file-response");
                  fileInput.value = value;
                  document
                    .getElementById("file-save")
                    .addEventListener("click", function () {
                      fs.writeTextFile(path, fileInput.value, {
                        dir: getDir(),
                      }).catch(onMessage);
                    });
                });
              }
            } else {
              onMessage(response);
            }
          })
          .catch(onMessage);
      })
      .catch(onMessage);
  }

  function setSrc() {
    img.src = convertFileSrc(path);
  }
</script>

<div class="flex flex-col">
  <div class="flex gap-1">
    <select class="input" id="dir">
      <option value="">None</option>
      {#each DirOptions as dir}
        <option value={dir[1]}>{dir[0]}</option>
      {/each}
    </select>
    <input
      class="input grow"
      placeholder="Type the path to read..."
      bind:value={path}
    />
  </div>
  <br />
  <div>
    <button class="btn" on:click={open}>Open</button>
    <button class="btn" on:click={read}>Read</button>
    <button class="btn" on:click={mkdir}>Mkdir</button>
    <button class="btn" on:click={remove}>Remove</button>
    <div class="flex flex-row">
      <button class="btn" on:click={rename}>Rename</button>
      <input class="input" bind:value={renameTo} placeholder="To" />
    </div>
    <button class="btn" type="button" on:click={setSrc}>Use as img src</button>
  </div>
  {#if file}
    <div>
      <button class="btn" on:click={truncate}>Truncate</button>
      <button class="btn" on:click={stat}>Stat</button>
    </div>
  {/if}
</div>

<br />

<img alt="" bind:this={img} />
