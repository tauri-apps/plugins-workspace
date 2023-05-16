window.hotkeys(navigator.appVersion.includes('Mac') ? 'command+option+i' : 'ctrl+shift+i', () => {
  window.__TAURI_INVOKE__('plugin:window|internal_toggle_devtools');
});
