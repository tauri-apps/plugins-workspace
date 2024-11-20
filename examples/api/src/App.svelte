<script>
  import { writable } from 'svelte/store'
  import { getCurrentWindow } from '@tauri-apps/api/window'
  import { getCurrentWebview } from '@tauri-apps/api/webview'
  import * as os from '@tauri-apps/plugin-os'

  import Welcome from './views/Welcome.svelte'
  import Cli from './views/Cli.svelte'
  import Communication from './views/Communication.svelte'
  import Dialog from './views/Dialog.svelte'
  import FileSystem from './views/FileSystem.svelte'
  import Http from './views/Http.svelte'
  import Notifications from './views/Notifications.svelte'
  import Shortcuts from './views/Shortcuts.svelte'
  import Shell from './views/Shell.svelte'
  import Opener from './views/Opener.svelte'
  import Store from './views/Store.svelte'
  import Updater from './views/Updater.svelte'
  import Clipboard from './views/Clipboard.svelte'
  import WebRTC from './views/WebRTC.svelte'
  import Scanner from './views/Scanner.svelte'
  import Biometric from './views/Biometric.svelte'
  import Geolocation from './views/Geolocation.svelte'
  import Haptics from './views/Haptics.svelte'

  import { onMount, tick } from 'svelte'
  import { ask } from '@tauri-apps/plugin-dialog'
  import Nfc from './views/Nfc.svelte'

  const appWindow = getCurrentWindow()

  if (appWindow.label !== 'main') {
    appWindow.onCloseRequested(async (event) => {
      const confirmed = await confirm('Are you sure?')
      if (!confirmed) {
        // user did not confirm closing the window; let's prevent it
        event.preventDefault()
      }
    })
  }

  getCurrentWebview().onDragDropEvent((event) => {
    onMessage(`File drop: ${JSON.stringify(event.payload)}`)
  })

  const userAgent = navigator.userAgent.toLowerCase()
  const isMobile = userAgent.includes('android') || userAgent.includes('iphone')

  const views = [
    {
      label: 'Welcome',
      component: Welcome,
      icon: 'i-ph-hand-waving'
    },
    {
      label: 'Communication',
      component: Communication,
      icon: 'i-codicon-radio-tower'
    },
    !isMobile && {
      label: 'CLI',
      component: Cli,
      icon: 'i-codicon-terminal'
    },
    {
      label: 'Dialog',
      component: Dialog,
      icon: 'i-codicon-multiple-windows'
    },
    {
      label: 'File system',
      component: FileSystem,
      icon: 'i-codicon-files'
    },
    {
      label: 'HTTP',
      component: Http,
      icon: 'i-ph-globe-hemisphere-west'
    },
    {
      label: 'Notifications',
      component: Notifications,
      icon: 'i-codicon-bell-dot'
    },
    !isMobile && {
      label: 'Shortcuts',
      component: Shortcuts,
      icon: 'i-codicon-record-keys'
    },
    {
      label: 'Shell',
      component: Shell,
      icon: 'i-codicon-terminal-bash'
    },
    {
      label: 'Opener',
      component: Opener,
      icon: 'i-codicon-link-external'
    },
    {
      label: 'Store',
      component: Store,
      icon: 'i-codicon-file-code'
    },
    !isMobile && {
      label: 'Updater',
      component: Updater,
      icon: 'i-codicon-cloud-download'
    },
    {
      label: 'Clipboard',
      component: Clipboard,
      icon: 'i-codicon-clippy'
    },
    {
      label: 'WebRTC',
      component: WebRTC,
      icon: 'i-ph-broadcast'
    },
    isMobile && {
      label: 'Scanner',
      component: Scanner,
      icon: 'i-ph-scan'
    },
    isMobile && {
      label: 'NFC',
      component: Nfc,
      icon: 'i-ph-nfc'
    },
    isMobile && {
      label: 'Biometric',
      component: Biometric,
      icon: 'i-ph-scan'
    },
    isMobile && {
      label: 'Geolocation',
      component: Geolocation,
      icon: 'i-ph-map-pin'
    },
    isMobile && {
      label: 'Haptics',
      component: Haptics,
      icon: 'i-ph-vibrate'
    }
  ]

  let selected = views[0]
  function select(view) {
    selected = view
  }

  // Window controls
  let isWindowMaximized
  onMount(async () => {
    isWindowMaximized = await appWindow.isMaximized()
    appWindow.onResized(async () => {
      isWindowMaximized = await appWindow.isMaximized()
    })
  })

  function minimize() {
    appWindow.minimize()
  }

  async function toggleMaximize() {
    ;(await appWindow.isMaximized())
      ? appWindow.unmaximize()
      : appWindow.maximize()
  }

  let confirmed_close = false
  async function close() {
    if (!confirmed_close) {
      confirmed_close = await ask(
        'Are you sure that you want to close this window?',
        {
          title: 'Tauri API'
        }
      )
      if (confirmed_close) {
        appWindow.close()
      }
    }
  }

  // dark/light
  let isDark
  onMount(() => {
    isDark = localStorage && localStorage.getItem('theme') == 'dark'
    applyTheme(isDark)
  })
  function applyTheme(isDark) {
    const html = document.querySelector('html')
    isDark ? html.classList.add('dark') : html.classList.remove('dark')
    localStorage && localStorage.setItem('theme', isDark ? 'dark' : '')
  }
  function toggleDark() {
    isDark = !isDark
    applyTheme(isDark)
  }

  // Console
  let messages = writable([])
  let consoleTextEl
  async function onMessage(value) {
    messages.update((r) => [
      ...r,
      {
        html:
          `<pre><strong class="text-accent dark:text-darkAccent">[${new Date().toLocaleTimeString()}]:</strong> ` +
          (typeof value === 'string' ? value : JSON.stringify(value, null, 1)) +
          '</pre>'
      }
    ])
    await tick()
    if (consoleTextEl) consoleTextEl.scrollTop = consoleTextEl.scrollHeight
  }

  // this function renders HTML without sanitizing it so it's insecure
  // we only use it with our own input data
  async function insecureRenderHtml(html) {
    messages.update((r) => [
      ...r,
      {
        html:
          `<pre><strong class="text-accent dark:text-darkAccent">[${new Date().toLocaleTimeString()}]:</strong> ` +
          html +
          '</pre>'
      }
    ])
    await tick()
    if (consoleTextEl) consoleTextEl.scrollTop = consoleTextEl.scrollHeight
  }

  function clear() {
    messages.update(() => [])
  }

  let consoleEl, consoleH, cStartY
  let minConsoleHeight = 50
  function startResizingConsole(e) {
    cStartY = e.clientY

    const styles = window.getComputedStyle(consoleEl)
    consoleH = parseInt(styles.height, 10)

    const moveHandler = (e) => {
      const dy = e.clientY - cStartY
      const newH = consoleH - dy
      consoleEl.style.height = `${
        newH < minConsoleHeight ? minConsoleHeight : newH
      }px`
    }
    const upHandler = () => {
      document.removeEventListener('mouseup', upHandler)
      document.removeEventListener('mousemove', moveHandler)
    }
    document.addEventListener('mouseup', upHandler)
    document.addEventListener('mousemove', moveHandler)
  }

  let isWindows
  onMount(async () => {
    isWindows = (await os.platform()) === 'windows'
  })

  // mobile
  let isSideBarOpen = false
  let sidebar
  let sidebarToggle
  let isDraggingSideBar = false
  let draggingStartPosX = 0
  let draggingEndPosX = 0
  const clamp = (min, num, max) => Math.min(Math.max(num, min), max)

  function toggleSidebar(sidebar, isSideBarOpen) {
    sidebar.style.setProperty(
      '--translate-x',
      `${isSideBarOpen ? '0' : '-18.75'}rem`
    )
  }

  onMount(() => {
    sidebar = document.querySelector('#sidebar')
    sidebarToggle = document.querySelector('#sidebarToggle')

    document.addEventListener('click', (e) => {
      if (sidebarToggle.contains(e.target)) {
        isSideBarOpen = !isSideBarOpen
      } else if (isSideBarOpen && !sidebar.contains(e.target)) {
        isSideBarOpen = false
      }
    })

    document.addEventListener('touchstart', (e) => {
      if (sidebarToggle.contains(e.target)) return

      const x = e.touches[0].clientX
      if ((0 < x && x < 20 && !isSideBarOpen) || isSideBarOpen) {
        isDraggingSideBar = true
        draggingStartPosX = x
      }
    })

    document.addEventListener('touchmove', (e) => {
      if (isDraggingSideBar) {
        const x = e.touches[0].clientX
        draggingEndPosX = x
        const delta = (x - draggingStartPosX) / 10
        sidebar.style.setProperty(
          '--translate-x',
          `-${clamp(0, isSideBarOpen ? 0 - delta : 18.75 - delta, 18.75)}rem`
        )
      }
    })

    document.addEventListener('touchend', () => {
      if (isDraggingSideBar) {
        const delta = (draggingEndPosX - draggingStartPosX) / 10
        isSideBarOpen = isSideBarOpen ? delta > -(18.75 / 2) : delta > 18.75 / 2
      }

      isDraggingSideBar = false
    })
  })

  $: {
    const sidebar = document.querySelector('#sidebar')
    if (sidebar) {
      toggleSidebar(sidebar, isSideBarOpen)
    }
  }
</script>

<!-- custom titlebar for Windows -->
{#if isWindows}
  <div
    class="w-screen select-none h-8 pl-2 flex justify-between items-center absolute text-primaryText dark:text-darkPrimaryText"
    data-tauri-drag-region
  >
    <span class="lt-sm:pl-10 text-darkPrimaryText">Tauri API Validation</span>
    <span
      class="
      h-100%
      children:h-100% children:w-12 children:inline-flex
      children:items-center children:justify-center"
    >
      <button
        aria-label="Toggle dark mode"
        title={isDark ? 'Switch to Light mode' : 'Switch to Dark mode'}
        class="bg-inherit border-none hover:bg-hoverOverlay active:bg-hoverOverlayDarker dark:hover:bg-darkHoverOverlay dark:active:bg-darkHoverOverlayDarker"
        on:click={toggleDark}
      >
        {#if isDark}
          <div class="i-ph-sun"></div>
        {:else}
          <div class="i-ph-moon"></div>
        {/if}
      </button>
      <button
        aria-label="Minimize window"
        title="Minimize"
        class="bg-inherit border-none hover:bg-hoverOverlay active:bg-hoverOverlayDarker dark:hover:bg-darkHoverOverlay dark:active:bg-darkHoverOverlayDarker"
        on:click={minimize}
      >
        <div class="i-codicon-chrome-minimize"></div>
      </button>
      <button
        aria-label="Maximize window"
        title={isWindowMaximized ? 'Restore' : 'Maximize'}
        class="bg-inherit border-none hover:bg-hoverOverlay active:bg-hoverOverlayDarker dark:hover:bg-darkHoverOverlay dark:active:bg-darkHoverOverlayDarker"
        on:click={toggleMaximize}
      >
        {#if isWindowMaximized}
          <div class="i-codicon-chrome-restore"></div>
        {:else}
          <div class="i-codicon-chrome-maximize"></div>
        {/if}
      </button>
      <button
        aria-label="Close window"
        title="Close"
        class="bg-inherit border-none hover:bg-red-700 dark:hover:bg-red-700 hover:text-darkPrimaryText active:bg-red-700/90 dark:active:bg-red-700/90 active:text-darkPrimaryText"
        on:click={close}
      >
        <div class="i-codicon-chrome-close"></div>
      </button>
    </span>
  </div>
{/if}

<!-- Sidebar toggle, only visible on small screens -->
<div
  id="sidebarToggle"
  class="z-2000 sidebar-toggle hidden lt-sm:flex justify-center absolute items-center w-8 h-8 rd-8
            bg-accent dark:bg-darkAccent active:bg-accentDark dark:active:bg-darkAccentDark"
>
  {#if isSideBarOpen}
    <span class="i-codicon-close animate-duration-300ms animate-fade-in"></span>
  {:else}
    <span class="i-codicon-menu animate-duration-300ms animate-fade-in"></span>
  {/if}
</div>

<div
  class="flex h-screen w-screen overflow-hidden children-pt8 children-pb-2 text-primaryText dark:text-darkPrimaryText"
>
  <aside
    id="sidebar"
    class="lt-sm:h-screen lt-sm:shadow-lg lt-sm:shadow lt-sm:transition-transform lt-sm:absolute lt-sm:z-1999
      bg-darkPrimaryLighter transition-colors-250 overflow-hidden grid select-none px-2"
  >
    <a href="https://tauri.app" target="_blank">
      <img class="p-7" src="tauri_logo.png" alt="Tauri logo" />
    </a>
    {#if !isWindows}
      <a href="##" class="nv justify-between h-8" on:click={toggleDark}>
        {#if isDark}
          Switch to Light mode
          <div class="i-ph-sun"></div>
        {:else}
          Switch to Dark mode
          <div class="i-ph-moon"></div>
        {/if}
      </a>
      <br />
      <div class="bg-white/5 h-2px"></div>
      <br />
    {/if}

    <a
      class="nv justify-between h-8"
      target="_blank"
      href="https://tauri.app/v1/guides/"
    >
      Documentation
      <span class="i-codicon-link-external"></span>
    </a>
    <a
      class="nv justify-between h-8"
      target="_blank"
      href="https://github.com/tauri-apps/tauri"
    >
      GitHub
      <span class="i-codicon-link-external"></span>
    </a>
    <a
      class="nv justify-between h-8"
      target="_blank"
      href="https://github.com/tauri-apps/tauri/tree/dev/examples/api"
    >
      Source
      <span class="i-codicon-link-external"></span>
    </a>
    <br />
    <div class="bg-white/5 h-2px"></div>
    <br />
    <div
      class="flex flex-col overflow-y-auto children-h-10 children-flex-none gap-1"
    >
      {#each views as view}
        {#if view}
          <a
            href="##"
            class="nv {selected === view ? 'nv_selected' : ''}"
            on:click={() => {
              select(view)
              isSideBarOpen = false
            }}
          >
            <div class="{view.icon} mr-2"></div>
            <p>{view.label}</p></a
          >
        {/if}
      {/each}
    </div>
  </aside>
  <main
    class="flex-1 bg-primary dark:bg-darkPrimary transition-transform transition-colors-250 grid grid-rows-[2fr_auto]"
    class:transparent={isMobile}
  >
    <div class="px-5 overflow-hidden grid grid-rows-[auto_1fr]">
      <h1>{selected.label}</h1>
      <div class="overflow-y-auto">
        <div class="mr-2">
          <svelte:component
            this={selected.component}
            {onMessage}
            {insecureRenderHtml}
          />
        </div>
      </div>
    </div>

    <div
      bind:this={consoleEl}
      id="console"
      class="select-none h-15rem grid grid-rows-[2px_2rem_1fr] gap-1 overflow-hidden"
    >
      <!-- svelte-ignore a11y_no_static_element_interactions -->
      <div
        on:mousedown={startResizingConsole}
        class="bg-black/20 h-2px cursor-ns-resize"
      ></div>
      <div class="flex justify-between items-center px-2">
        <p class="font-semibold">Console</p>
        <button
          aria-label="Clear Console"
          class="cursor-pointer h-85% rd-1 p-1 flex justify-center items-center border-none bg-inherit
                hover:bg-hoverOverlay dark:hover:bg-darkHoverOverlay
                active:bg-hoverOverlay/25 dark:active:bg-darkHoverOverlay/25
          "
          on:click={clear}
        >
          <div class="i-codicon-clear-all"></div>
        </button>
      </div>
      <div
        bind:this={consoleTextEl}
        class="px-2 overflow-y-auto all:font-mono code-block all:text-xs select-text mr-2"
      >
        {#each $messages as r}
          {@html r.html}
        {/each}
      </div>
    </div>
  </main>
</div>
