<script lang="ts">
  import {
    open,
    save,
    confirm,
    message,
    type PickerMode,
    type FileAccessMode
  } from '@tauri-apps/plugin-dialog'
  import { readFile } from '@tauri-apps/plugin-fs'
  import type { ViewProps } from '../App.svelte'

  let { onMessage }: ViewProps = $props()

  let defaultPath: string | undefined = $state()
  let filter = $state('')
  let multiple = $state(false)
  let directory = $state(false)
  let pickerMode: PickerMode = $state('document')
  let fileAccessMode: FileAccessMode = $state('scoped')

  function arrayBufferToBase64(
    buffer: ArrayBuffer,
    callback: (base64: string) => void
  ) {
    var blob = new Blob([buffer], {
      type: 'application/octet-binary'
    })
    var reader = new FileReader()
    reader.onload = function (evt) {
      var dataurl = evt.target!.result as string
      callback(dataurl.split(',')[1])
    }
    reader.readAsDataURL(blob)
  }

  async function prompt() {
    confirm('Do you want to do something?')
      .then((res) => onMessage(res ? 'Yes' : 'No'))
      .catch(onMessage)
  }

  async function promptCustom() {
    confirm('Is Tauri awesome?', {
      okLabel: 'Absolutely',
      cancelLabel: 'Totally'
    })
      .then((res) =>
        onMessage(
          res ? 'Tauri is absolutely awesome' : 'Tauri is totally awesome'
        )
      )
      .catch(onMessage)
  }

  async function msg() {
    await message('Tauri is awesome!').then((res) => onMessage(res))
  }

  async function msgCustom() {
    const buttons = { yes: 'awesome', no: 'amazing', cancel: 'stunning' }
    await message(`Tauri is: `, { buttons })
      .then((res) => onMessage(`Tauri is ${res}`))
      .catch(onMessage)
  }

  async function openDialog() {
    try {
      const result = await open({
        title: 'My wonderful open dialog',
        defaultPath,
        filters: filter
          ? [
              {
                name: 'Tauri Example',
                extensions: filter.split(',').map((f) => f.trim())
              }
            ]
          : [],
        multiple,
        directory,
        pickerMode,
        fileAccessMode
      })

      if (Array.isArray(result)) {
        onMessage(result)
      } else if (result === null) {
        onMessage('user cancelled the selection')
      } else {
        const pathToRead = result
        const isFile = pathToRead.match(/\S+\.\S+$/g)

        await readFile(pathToRead).then(function (res) {
          if (isFile) {
            if (
              pathToRead.includes('.png')
              || pathToRead.includes('.jpg')
              || pathToRead.includes('.jpeg')
            ) {
              arrayBufferToBase64(res.buffer, function (base64) {
                const src = 'data:image/png;base64,' + base64
                onMessage('<img src="' + src + '"></img>')
              })
            } else {
              // Convert byte array to UTF-8 string
              const decoder = new TextDecoder('utf-8')
              const text = decoder.decode(res)
              onMessage(text)
            }
          } else {
            onMessage(res)
          }
        })
      }
    } catch (exception) {
      onMessage(exception)
    }
  }

  function saveDialog() {
    save({
      title: 'My wonderful save dialog',
      defaultPath,
      filters: filter
        ? [
            {
              name: 'Tauri Example',
              extensions: filter.split(',').map((f) => f.trim())
            }
          ]
        : []
    })
      .then(onMessage)
      .catch(onMessage)
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
  <button class="btn" id="open-dialog" onclick={openDialog}>Open dialog</button>
  <button class="btn" id="save-dialog" onclick={saveDialog}
    >Open save dialog</button
  >
  <button class="btn" id="prompt-dialog" onclick={prompt}>Prompt</button>
  <button class="btn" id="custom-prompt-dialog" onclick={promptCustom}
    >Prompt (custom)</button
  >
  <button class="btn" id="message-dialog" onclick={msg}>Message</button>
  <button class="btn" id="message-dialog" onclick={msgCustom}
    >Message (custom)</button
  >
</div>
