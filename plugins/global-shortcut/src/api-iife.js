if("__TAURI__"in window){var __TAURI_GLOBALSHORTCUT__=function(_){"use strict";return _.isRegistered=async function(_){return await window.__TAURI_INVOKE__("plugin:globalShortcut|is_registered",{shortcut:_})},_.register=async function(_,t){return await window.__TAURI_INVOKE__("plugin:globalShortcut|register",{shortcut:_,handler:window.__TAURI__.transformCallback(t)})},_.registerAll=async function(_,t){return await window.__TAURI_INVOKE__("plugin:globalShortcut|register_all",{shortcuts:_,handler:window.__TAURI__.transformCallback(t)})},_.unregister=async function(_){return await window.__TAURI_INVOKE__("plugin:globalShortcut|unregister",{shortcut:_})},_.unregisterAll=async function(){return await window.__TAURI_INVOKE__("plugin:globalShortcut|unregister_all")},_}({});Object.defineProperty(window.__TAURI__,"globalShortcut",{value:__TAURI_GLOBALSHORTCUT__})}
