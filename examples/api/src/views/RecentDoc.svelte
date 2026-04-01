<script>
  import {
    addRecentDocument,
    getRecentDocuments,
    clearRecentDocuments
  } from '@tauri-apps/plugin-recent-doc'
  import { open } from '@tauri-apps/plugin-dialog'

  let { onMessage } = $props()
</script>

<div class="flex flex-col gap-2">
  <button
    class="btn"
    onclick={async () => {
      try {
        const file = await open({
          multiple: false,
          title: 'Select a file to add to recent documents'
        })
        if (file) {
          await addRecentDocument(file)
          onMessage(`Added "${file}" to recent documents.`)
        } else {
          onMessage('No file selected.')
        }
      } catch (e) {
        onMessage(e)
      }
    }}>Add document to recent list</button
  >
  <button
    class="btn"
    onclick={async () => {
      try {
        const files = await getRecentDocuments()
        onMessage(files)
      } catch (e) {
        onMessage(e)
      }
    }}>Get recent documents</button
  >
  <button
    class="btn"
    onclick={async () => {
      try {
        await clearRecentDocuments()
        onMessage('Cleared recent documents.')
      } catch (e) {
        onMessage(e)
      }
    }}>Clear recent documents</button
  >
</div>
