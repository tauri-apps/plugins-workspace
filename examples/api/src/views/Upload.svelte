<script>
  import { download, upload, HttpMethod } from '@tauri-apps/plugin-upload'
  import { open } from '@tauri-apps/plugin-dialog'
  import { JsonView } from '@zerodevx/svelte-json-view'
  import { appDataDir } from '@tauri-apps/api/path'
  import { onMount } from 'svelte'

  export let onMessage

  let downloadUrl = 'https://httpbin.org/json'
  let downloadFolder = ''
  let downloadPath = ''
  let downloadProgress = null
  let downloadResult = null
  let isDownloading = false

  let uploadUrl = 'https://httpbin.org/post'
  let uploadFilePath = ''
  let uploadMethod = HttpMethod.Post

  // Update URL when method changes
  $: {
    switch (uploadMethod) {
      case HttpMethod.Post:
        uploadUrl = 'https://httpbin.org/post'
        break
      case HttpMethod.Put:
        uploadUrl = 'https://httpbin.org/put'
        break
      case HttpMethod.Patch:
        uploadUrl = 'https://httpbin.org/patch'
        break
    }
  }
  let uploadProgress = null
  let uploadResult = null
  let isUploading = false

  onMount(async () => {
    try {
      const defaultDir = await appDataDir()
      if (!downloadFolder) {
        downloadFolder = defaultDir
        updateDownloadPath()
      }
    } catch (error) {
      onMessage({ error: `Failed to get default directory: ${error.toString()}` })
    }
  })

  async function selectDownloadFolder() {
    try {
      const selected = await open({
        directory: true,
        multiple: false,
        defaultPath: downloadFolder || undefined
      })
      if (selected) {
        downloadFolder = selected
        updateDownloadPath()
      }
    } catch (error) {
      onMessage({ error: error.toString() })
    }
  }

  function getFilenameFromUrl(url) {
    try {
      const urlObj = new URL(url)
      let pathname = urlObj.pathname
      
      // Remove leading slash
      if (pathname.startsWith('/')) {
        pathname = pathname.substring(1)
      }
      
      // If pathname is empty or ends with slash, use a default name
      if (!pathname || pathname.endsWith('/')) {
        return 'downloaded-file.json'
      }
      
      // Extract filename from pathname
      const segments = pathname.split('/')
      let filename = segments[segments.length - 1]
      
      // If no extension, try to infer from URL or use default
      if (!filename.includes('.')) {
        // Check if URL suggests a file type
        if (url.includes('json') || urlObj.searchParams.has('format') && urlObj.searchParams.get('format') === 'json') {
          filename += '.json'
        } else if (url.includes('xml')) {
          filename += '.xml'
        } else if (url.includes('csv')) {
          filename += '.csv'
        } else {
          filename += '.txt'
        }
      }
      
      return filename
    } catch (error) {
      return 'downloaded-file.json'
    }
  }

  function updateDownloadPath() {
    if (downloadFolder && downloadUrl) {
      const filename = getFilenameFromUrl(downloadUrl)
      downloadPath = `${downloadFolder}/${filename}`
    } else {
      downloadPath = ''
    }
  }

  // Update download path when URL changes
  $: if (downloadUrl) {
    updateDownloadPath()
  }

  async function selectUploadFile() {
    try {
      const selected = await open({
        directory: false,
        multiple: false
      })
      if (selected) {
        uploadFilePath = selected
      }
    } catch (error) {
      onMessage({ error: error.toString() })
    }
  }

  async function startDownload() {
    if (!downloadUrl || !downloadFolder) {
      onMessage({ error: 'Please provide both URL and download folder' })
      return
    }
    
    // Ensure download path is updated
    updateDownloadPath()
    
    if (!downloadPath) {
      onMessage({ error: 'Could not generate download path' })
      return
    }

    isDownloading = true
    downloadProgress = null
    downloadResult = null

    try {
      await download(
        downloadUrl,
        downloadPath,
        (progress) => {
          downloadProgress = {
            progress: progress.progress,
            progressTotal: progress.progressTotal,
            total: progress.total,
            transferSpeed: progress.transferSpeed,
            percentage: progress.total > 0 ? Math.round((progress.progressTotal / progress.total) * 100) : 0
          }
        },
        new Map([
          ['User-Agent', 'Tauri Upload Plugin Demo']
        ])
      )

      downloadResult = {
        success: true,
        message: `File downloaded successfully to: ${downloadPath}`,
        finalProgress: downloadProgress
      }

      onMessage({
        type: 'download',
        result: downloadResult
      })
    } catch (error) {
      downloadResult = {
        success: false,
        error: error.toString()
      }
      onMessage({ error: error.toString() })
    } finally {
      isDownloading = false
    }
  }

  async function startUpload() {
    if (!uploadUrl || !uploadFilePath) {
      onMessage({ error: 'Please provide both URL and file path' })
      return
    }

    isUploading = true
    uploadProgress = null
    uploadResult = null

    try {
      const response = await upload(
        uploadUrl,
        uploadFilePath,
        (progress) => {
          uploadProgress = {
            progress: progress.progress,
            progressTotal: progress.progressTotal,
            total: progress.total,
            transferSpeed: progress.transferSpeed,
            percentage: progress.total > 0 ? Math.round((progress.progressTotal / progress.total) * 100) : 0
          }
        },
        new Map([
          ['User-Agent', 'Tauri Upload Plugin Demo']
        ]),
        uploadMethod
      )

      uploadResult = {
        success: true,
        response: response,
        finalProgress: uploadProgress
      }

      onMessage({
        type: 'upload',
        result: uploadResult
      })
    } catch (error) {
      uploadResult = {
        success: false,
        error: error.toString()
      }
      onMessage({ error: error.toString() })
    } finally {
      isUploading = false
    }
  }
</script>

<div class="space-y-6">
  <div class="bg-gray-50 p-4 rounded-lg">
    <h3 class="text-lg font-semibold mb-4 text-gray-800">File Download</h3>
    
    <div class="space-y-3">
      <div>
        <label for="download-url" class="block text-sm font-medium text-gray-700 mb-1">Download URL:</label>
        <input
          id="download-url"
          bind:value={downloadUrl}
          type="url"
          placeholder="https://example.com/file.json"
          class="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500"
          disabled={isDownloading}
        />
      </div>

      <div>
        <label for="download-folder" class="block text-sm font-medium text-gray-700 mb-1">Download folder:</label>
        <div class="flex gap-2">
          <input
            id="download-folder"
            bind:value={downloadFolder}
            type="text"
            placeholder="Select download folder..."
            class="flex-1 px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500"
            disabled={isDownloading}
          />
          <button
            on:click={selectDownloadFolder}
            class="px-4 py-2 bg-gray-500 text-white rounded-md hover:bg-gray-600 disabled:opacity-50"
            disabled={isDownloading}
          >
            Browse
          </button>
        </div>
      </div>

      {#if downloadPath}
        <div class="bg-blue-50 border border-blue-200 p-3 rounded-md">
          <div class="text-sm text-blue-800">
            <strong>File will be saved as:</strong>
            <div class="font-mono text-xs mt-1 break-all">{downloadPath}</div>
          </div>
        </div>
      {/if}

      <button
        on:click={startDownload}
        class="w-full px-4 py-2 bg-blue-500 text-white rounded-md hover:bg-blue-600 disabled:opacity-50 disabled:cursor-not-allowed"
        disabled={isDownloading || !downloadUrl || !downloadFolder}
      >
        {isDownloading ? 'Downloading...' : 'Download File'}
      </button>

      {#if downloadProgress}
        <div class="bg-white p-3 rounded border">
          <div class="flex justify-between text-sm text-gray-600 mb-1">
            <span>Progress: {downloadProgress.percentage}%</span>
            <span>Speed: {Math.round(downloadProgress.transferSpeed / 1024)} KB/s</span>
          </div>
          <div class="w-full bg-gray-200 rounded-full h-2">
            <div 
              class="bg-blue-500 h-2 rounded-full transition-all duration-300"
              style="width: {downloadProgress.percentage}%"
            ></div>
          </div>
          <div class="text-xs text-gray-500 mt-1">
            {Math.round(downloadProgress.progressTotal / 1024)} KB / {Math.round(downloadProgress.total / 1024)} KB
          </div>
        </div>
      {/if}

      {#if downloadResult}
        <div class="bg-white p-3 rounded border">
          <JsonView json={downloadResult} />
        </div>
      {/if}
    </div>
  </div>

  <div class="bg-gray-50 p-4 rounded-lg">
    <h3 class="text-lg font-semibold mb-4 text-gray-800">File Upload</h3>
    
    <div class="space-y-3">
      <div>
        <label for="upload-url" class="block text-sm font-medium text-gray-700 mb-1">Upload URL:</label>
        <input
          id="upload-url"
          bind:value={uploadUrl}
          type="url"
          placeholder="https://httpbin.org/post"
          class="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-green-500"
          disabled={isUploading}
        />
      </div>

      <div>
        <label for="upload-file" class="block text-sm font-medium text-gray-700 mb-1">File to upload:</label>
        <div class="flex gap-2">
          <input
            id="upload-file"
            bind:value={uploadFilePath}
            type="text"
            placeholder="Select file to upload..."
            class="flex-1 px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-green-500"
            disabled={isUploading}
          />
          <button
            on:click={selectUploadFile}
            class="px-4 py-2 bg-gray-500 text-white rounded-md hover:bg-gray-600 disabled:opacity-50"
            disabled={isUploading}
          >
            Browse
          </button>
        </div>
      </div>

      <div>
        <label for="upload-method" class="block text-sm font-medium text-gray-700 mb-1">HTTP Method:</label>
        <select
          id="upload-method"
          bind:value={uploadMethod}
          class="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-green-500"
          disabled={isUploading}
        >
          <option value={HttpMethod.Post}>POST</option>
          <option value={HttpMethod.Put}>PUT</option>
          <option value={HttpMethod.Patch}>PATCH</option>
        </select>
        <p class="text-xs text-gray-500 mt-1">Choose the HTTP method for the upload request</p>
      </div>

      <div class="bg-blue-50 border border-blue-200 p-3 rounded-md">
        <div class="text-sm text-blue-800">
          <strong>Upload Configuration:</strong>
          <div class="font-mono text-xs mt-1">
            Method: {uploadMethod} | URL: {uploadUrl || 'Not set'}
          </div>
        </div>
      </div>

      <button
        on:click={startUpload}
        class="w-full px-4 py-2 bg-green-500 text-white rounded-md hover:bg-green-600 disabled:opacity-50 disabled:cursor-not-allowed"
        disabled={isUploading || !uploadUrl || !uploadFilePath}
      >
        {isUploading ? `Uploading (${uploadMethod})...` : `Upload File (${uploadMethod})`}
      </button>

      {#if uploadProgress}
        <div class="bg-white p-3 rounded border">
          <div class="flex justify-between text-sm text-gray-600 mb-1">
            <span>Progress: {uploadProgress.percentage}%</span>
            <span>Speed: {Math.round(uploadProgress.transferSpeed / 1024)} KB/s</span>
          </div>
          <div class="w-full bg-gray-200 rounded-full h-2">
            <div 
              class="bg-green-500 h-2 rounded-full transition-all duration-300"
              style="width: {uploadProgress.percentage}%"
            ></div>
          </div>
          <div class="text-xs text-gray-500 mt-1">
            {Math.round(uploadProgress.progressTotal / 1024)} KB / {Math.round(uploadProgress.total / 1024)} KB
          </div>
        </div>
      {/if}

      {#if uploadResult}
        <div class="bg-white p-3 rounded border">
          <JsonView json={uploadResult} />
        </div>
      {/if}
    </div>
  </div>
</div>
