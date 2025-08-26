<script>
  import * as fs from '@tauri-apps/plugin-fs'
  import * as os from '@tauri-apps/plugin-os'
  import { convertFileSrc } from '@tauri-apps/api/core'
  import { arrayBufferToBase64 } from '../lib/utils'
  import { onDestroy, onMount } from 'svelte'

  const { onMessage, insecureRenderHtml } = $props()

  let path = $state('')
  let img
  /** @type {fs.FileHandle} */
  let file = $state()
  let renameTo = $state()
  let watchPath = $state('')
  let watchDebounceDelay = $state(0)
  let watchRecursive = $state(false)
  /** @type {fs.BaseDirectory | undefined} */
  let baseDir = $state()
  let unwatchFn
  let unwatchPath = ''
  let isMobile = $state(false)

  onMount(() => {
    let platform = os.platform()
    isMobile = platform === 'android' || platform === 'ios'
  })

  const dirOptions = Object.keys(fs.BaseDirectory).filter((key) =>
    isNaN(parseInt(key))
  )

  function open() {
    fs.open(path, {
      baseDir,
      read: true,
      write: true,
      create: true
    })
      .then((f) => {
        file = f
        onMessage(`Opened ${path}`)
      })
      .catch(onMessage)
  }

  function mkdir() {
    fs.mkdir(path, { baseDir, recursive: true })
      .then(() => {
        onMessage(`Created dir ${path}`)
      })
      .catch(onMessage)
  }

  function remove() {
    fs.remove(path, { baseDir })
      .then(() => {
        onMessage(`Removed ${path}`)
      })
      .catch(onMessage)
  }

  function rename() {
    fs.rename(path, renameTo, {
      oldPathBaseDir,
      newPathBaseDir
    })
      .then(() => {
        onMessage(`Renamed ${path} to ${renameTo}`)
      })
      .catch(onMessage)
  }

  function truncate() {
    file
      .truncate(0)
      .then(() => {
        onMessage(`Truncated file`)
      })
      .catch(onMessage)
  }

  function write() {
    const encoder = new TextEncoder()
    file
      .write(encoder.encode('Hello from Tauri :)'))
      .then(() => {
        onMessage(`wrote to file`)
      })
      .catch(onMessage)
  }

  function stat() {
    file
      .stat()
      .then((stat) => {
        onMessage(`File stat ${JSON.stringify(stat)}`)
      })
      .catch(onMessage)
  }

  function read() {
    const opts = {
      baseDir
    }
    fs.stat(path, opts)
      .then((stat) => {
        const isFile = stat.isFile

        const promise = isFile
          ? fs.readFile(path, opts)
          : fs.readDir(path, opts)
        promise
          .then(function (response) {
            if (isFile) {
              if (path.includes('.png') || path.includes('.jpg')) {
                arrayBufferToBase64(
                  new Uint8Array(response),
                  function (base64) {
                    const src = 'data:image/png;base64,' + base64
                    insecureRenderHtml('<img src="' + src + '"></img>')
                  }
                )
              } else {
                const value = String.fromCharCode.apply(null, response)
                insecureRenderHtml(
                  '<textarea id="file-response"></textarea><button id="file-save">Save</button>'
                )
                setTimeout(() => {
                  const fileInput = document.getElementById('file-response')
                  fileInput.value = value
                  document
                    .getElementById('file-save')
                    .addEventListener('click', function () {
                      fs.writeTextFile(path, fileInput.value, {
                        baseDir
                      }).catch(onMessage)
                    })
                })
              }
            } else {
              onMessage(response)
            }
          })
          .catch(onMessage)
      })
      .catch(onMessage)
  }

  function setSrc() {
    img.src = convertFileSrc(path)
  }

  function watch() {
    unwatch()
    if (watchPath) {
      onMessage(`Watching ${watchPath} for changes`)
      let options = {
        recursive: watchRecursive,
        delayMs: watchDebounceDelay
      }
      if (options.delayMs === 0) {
        fs.watchImmediate(watchPath, onMessage, options)
          .then((fn) => {
            unwatchFn = fn
            unwatchPath = watchPath
          })
          .catch(onMessage)
      } else {
        fs.watch(watchPath, onMessage, options)
          .then((fn) => {
            unwatchFn = fn
            unwatchPath = watchPath
          })
          .catch(onMessage)
      }
    }
  }

  function unwatch() {
    if (unwatchFn) {
      onMessage(`Stopped watching ${unwatchPath} for changes`)
      unwatchFn()
    }
    unwatchFn = undefined
    unwatchPath = undefined
  }

  onDestroy(() => {
    if (file) {
      file.close()
    }
    if (unwatchFn) {
      unwatchFn()
    }
  })
</script>

<div class="flex flex-col">
  {#if isMobile}
    <div>
      On mobile, paths outside of App* paths require the use of dialogs
      regardless of Tauri's scope mechanism.
    </div>
    <br />
  {/if}
  <div class="flex gap-1">
    <select class="input" bind:value={baseDir}>
      <option value={undefined} selected>None</option>
      {#each dirOptions as dir}
        <option value={fs.BaseDirectory[dir]}>{dir}</option>
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
    <button class="btn" onclick={open}>Open</button>
    <button class="btn" onclick={read}>Read</button>
    <button class="btn" onclick={mkdir}>Mkdir</button>
    <button class="btn" onclick={remove}>Remove</button>
    <div class="flex flex-row">
      <button class="btn" onclick={rename}>Rename</button>
      <input class="input" bind:value={renameTo} placeholder="To" />
    </div>
    <button class="btn" type="button" onclick={setSrc}>Use as img src</button>
  </div>
  {#if file}
    <div>
      <button class="btn" onclick={write}>Write</button>
      <button class="btn" onclick={truncate}>Truncate</button>
      <button class="btn" onclick={stat}>Stat</button>
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
    <button class="btn" onclick={watch}>Watch</button>
    <button class="btn" onclick={unwatch}>Unwatch</button>
  </div>
</div>

<br />

<img alt="" bind:this={img} />
