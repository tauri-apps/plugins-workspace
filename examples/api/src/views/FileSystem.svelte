<script lang="ts">
  import * as fs from '@tauri-apps/plugin-fs'
  import * as os from '@tauri-apps/plugin-os'
  import { convertFileSrc } from '@tauri-apps/api/core'
  import { arrayBufferToBase64 } from '../lib/utils'
  import { onDestroy, onMount } from 'svelte'
  import type { ViewProps } from '../App.svelte'

  const { onMessage }: ViewProps = $props()

  let path = $state('')
  let img: HTMLImageElement
  let file: fs.FileHandle | undefined = $state()
  let renameTo = $state('')
  let watchPath = $state('')
  let watchDebounceDelay = $state(0)
  let watchRecursive = $state(false)
  let baseDir: fs.BaseDirectory | undefined = $state()
  let unwatchFn: (() => void) | undefined
  let unwatchPath = ''
  let isMobile = $state(false)

  const dirOptions = Object.entries(fs.BaseDirectory).filter(([key]) =>
    isNaN(parseInt(key))
  )

  onMount(() => {
    let platform = os.platform()
    isMobile = platform === 'android' || platform === 'ios'
  })

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
      oldPathBaseDir: baseDir
      // newPathBaseDir
    })
      .then(() => {
        onMessage(`Renamed ${path} to ${renameTo}`)
      })
      .catch(onMessage)
  }

  function truncate(file: fs.FileHandle) {
    file
      .truncate(0)
      .then(() => {
        onMessage(`Truncated file`)
      })
      .catch(onMessage)
  }

  function write(file: fs.FileHandle) {
    const encoder = new TextEncoder()
    file
      .write(encoder.encode('Hello from Tauri :)'))
      .then(() => {
        onMessage(`wrote to file`)
      })
      .catch(onMessage)
  }

  function stat(file: fs.FileHandle) {
    file
      .stat()
      .then((stat) => {
        onMessage(`File stat ${JSON.stringify(stat)}`)
      })
      .catch(onMessage)
  }

  async function read() {
    try {
      const opts = { baseDir }
      const pathStat = await fs.stat(path, opts)

      if (!pathStat.isFile) {
        onMessage(await fs.readDir(path, opts))
        return
      }

      const response = await fs.readFile(path, opts)
      if (path.includes('.png') || path.includes('.jpg')) {
        arrayBufferToBase64(response, function (base64: string) {
          const src = 'data:image/png;base64,' + base64
          onMessage('<img src="' + src + '"></img>')
        })
        return
      }

      const value = new TextDecoder().decode(response)
      onMessage(
        '<textarea id="file-response"></textarea><button id="file-save">Save</button>'
      )
      setTimeout(() => {
        const fileInput = document.getElementById(
          'file-response'
        ) as HTMLTextAreaElement | null
        if (!fileInput) return

        fileInput.value = value
        document
          .getElementById('file-save')
          ?.addEventListener('click', function () {
            fs.writeTextFile(path, fileInput.value, { baseDir }).catch(
              onMessage
            )
          })
      })
    } catch (error) {
      onMessage(error)
    }
  }

  function setSrc() {
    img.src = convertFileSrc(path)
  }

  function watch() {
    unwatch()
    if (watchPath) {
      onMessage(`Watching ${watchPath} for changes`)
      const options = {
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
    unwatchPath = ''
  }

  onDestroy(() => {
    file?.close()
    unwatchFn?.()
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
      {#each dirOptions as [dirName, dirValue]}
        <option value={dirValue}>{dirName}</option>
      {/each}
    </select>
    <input
      class="input grow"
      placeholder="Type the path to read..."
      bind:value={path}
    />
  </div>
  <br />

  <div class="grid gap-2 justify-start">
    <div class="flex gap-2">
      <button class="btn" onclick={open}>Open</button>
      <button class="btn" onclick={read}>Read</button>
      <button class="btn" onclick={mkdir}>Mkdir</button>
      <button class="btn" onclick={remove}>Remove</button>
    </div>
    <div class="flex gap-1">
      <input class="input" bind:value={renameTo} placeholder="To" />
      <button class="btn" onclick={rename}>Rename</button>
    </div>
    <button class="btn" type="button" onclick={setSrc}>Use as img src</button>
  </div>

  {#if file}
    <div>
      <button class="btn" onclick={() => write(file!)}>Write</button>
      <button class="btn" onclick={() => truncate(file!)}>Truncate</button>
      <button class="btn" onclick={() => stat(file!)}>Stat</button>
    </div>
  {/if}

  <h3>Watch</h3>

  <input
    class="input grow"
    placeholder="Type the path to watch..."
    bind:value={watchPath}
  />
  <br />
  <div class="grid grid-cols-2 gap-2 items-center">
    <label for="watch-debounce-delay" class="col-span-2">Debounce delay in milliseconds (<code>0</code> disables the debouncer)</label
    >
    <input
      class="input"
      id="watch-debounce-delay"
      bind:value={watchDebounceDelay}
    />
    <div>
      <input
        type="checkbox"
        id="watch-recursive"
        bind:checked={watchRecursive}
      />
      <label for="watch-recursive">Recursive</label>
    </div>
  </div>
  <br />
  <div class="flex gap-2">
    <button class="btn" onclick={watch}>Watch</button>
    <button class="btn" onclick={unwatch}>Unwatch</button>
  </div>
</div>

<br />

<img alt="" bind:this={img} />
