if("__TAURI__"in window){var __TAURI_PLUGIN_CLIPBOARDMANAGER__=function(r){"use strict";async function e(r,e={},a){return window.__TAURI_INTERNALS__.invoke(r,e,a)}return"function"==typeof SuppressedError&&SuppressedError,r.readImage=async function(){const r=await e("plugin:clipboard|read_image");return Uint8Array.from(r.image.buffer)},r.readText=async function(){return(await e("plugin:clipboard|read_text")).plainText.text},r.writeImage=async function(r){return e("plugin:clipboard|write_image",{data:{image:{buffer:Array.isArray(r)?r:Array.from(r)}}})},r.writeText=async function(r,a){return e("plugin:clipboard|write_text",{data:{plainText:{label:a?.label,text:r}}})},r}({});Object.defineProperty(window.__TAURI__,"clipboardManager",{value:__TAURI_PLUGIN_CLIPBOARDMANAGER__})}
