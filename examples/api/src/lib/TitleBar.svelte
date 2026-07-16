<script lang="ts">
  import type { Window } from '@tauri-apps/api/window'
  import { ask } from '@tauri-apps/plugin-dialog'
  import { onMount } from 'svelte'

  let {
    appWindow,
    toggleDark,
    isDark
  }: { appWindow: Window; toggleDark: () => void; isDark: boolean } = $props()

  // Window controls
  let isWindowMaximized = $state(false)
  onMount(async () => {
    isWindowMaximized = await appWindow.isMaximized()
    appWindow.onResized(async () => {
      isWindowMaximized = await appWindow.isMaximized()
    })
  })

  async function minimize() {
    await appWindow.minimize()
  }

  async function toggleMaximize() {
    await appWindow.toggleMaximize()
  }

  let confirmedClose = false
  async function close() {
    if (!confirmedClose) {
      confirmedClose = await ask(
        'Are you sure that you want to close this window?',
        {
          title: 'Tauri API'
        }
      )
      if (confirmedClose) {
        appWindow.close()
      }
    }
  }
</script>

<div
  class="w-screen select-none h-8 flex justify-between items-center absolute text-primaryText dark:text-darkPrimaryText"
  data-tauri-drag-region
>
  <span
    class="h-100% pl-2 flex-1 flex items-center lt-sm:pl-10 lt-lg:text-darkPrimaryText [app-region:drag]"
    >Tauri API Validation</span
  >
  <span
    class="
      h-100%
      children:h-100% children:w-12 children:inline-flex
      children:items-center children:justify-center"
  >
    <button
      aria-label="Toggle dark mode"
      title={isDark ? 'Switch to Light mode' : 'Switch to Dark mode'}
      class="border-none hover:bg-hoverOverlay active:bg-hoverOverlayDarker dark:hover:bg-darkHoverOverlay dark:active:bg-darkHoverOverlayDarker"
      onclick={toggleDark}
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
      class="border-none hover:bg-hoverOverlay active:bg-hoverOverlayDarker dark:hover:bg-darkHoverOverlay dark:active:bg-darkHoverOverlayDarker"
      onclick={minimize}
    >
      <div class="i-codicon-chrome-minimize"></div>
    </button>
    <button
      aria-label="Maximize window"
      title={isWindowMaximized ? 'Restore' : 'Maximize'}
      class="border-none hover:bg-hoverOverlay active:bg-hoverOverlayDarker dark:hover:bg-darkHoverOverlay dark:active:bg-darkHoverOverlayDarker"
      onclick={toggleMaximize}
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
      class="border-none hover:bg-red-700 dark:hover:bg-red-700 hover:text-darkPrimaryText active:bg-red-700/90 dark:active:bg-red-700/90 active:text-darkPrimaryText"
      onclick={close}
    >
      <div class="i-codicon-chrome-close"></div>
    </button>
  </span>
</div>
