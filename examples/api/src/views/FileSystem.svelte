<script>
  import * as fs from "@tauri-apps/plugin-fs";
  import { convertFileSrc } from "@tauri-apps/api/core";
  import { arrayBufferToBase64 } from "../lib/utils";

  export let onMessage;
  export let insecureRenderHtml;

  let path = "";
  let img;
  let file;
  let renameTo;
  let watchPath = "";
  let watchDebounceDelay = 0;
  let watchRecursive = false;
  let unwatchFn;
  let unwatchPath = "";

  function getDir() {
    const dirSelect = document.getElementById("dir");
    return dirSelect.value ? parseInt(dir.value) : null;
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

  function watch() {
    unwatch();
    if (watchPath) {
      onMessage(`Watching ${watchPath} for changes`);
      let options = {
        recursive: watchRecursive,
        delayMs: parseInt(watchDebounceDelay),
      };
      if (options.delayMs === 0) {
        fs.watchImmediate(watchPath, onMessage, options)
          .then((fn) => {
            unwatchFn = fn;
            unwatchPath = watchPath;
          })
          .catch(onMessage);
      } else {
        fs.watch(watchPath, onMessage, options)
          .then((fn) => {
            unwatchFn = fn;
            unwatchPath = watchPath;
          })
          .catch(onMessage);
      }
    }
  }

  function unwatch() {
    if (unwatchFn) {
      onMessage(`Stopped watching ${unwatchPath} for changes`);
      unwatchFn();
    }
    unwatchFn = undefined;
    unwatchPath = undefined;
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

  <h3>Watch</h3>

  <input
    class="input grow"
    placeholder="Type the path to watch..."
    bind:value={watchPath}
  />
  <br />
  <div>
    <label for="watch-debounce-delay"
      >Debounce delay in milliseconds (`0` disables the debouncer)</label
    >
    <input
      class="input"
      id="watch-debounce-delay"
      bind:value={watchDebounceDelay}
    />
  </div>
  <br />
  <div>
    <input type="checkbox" id="watch-recursive" bind:checked={watchRecursive} />
    <label for="watch-recursive">Recursive</label>
  </div>
  <br />
  <div>
    <button class="btn" on:click={watch}>Watch</button>
    <button class="btn" on:click={unwatch}>Unwatch</button>
  </div>
</div>

<br />

<img alt="" bind:this={img} />
