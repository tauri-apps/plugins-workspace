if("__TAURI__"in window){var __TAURI_PLUGIN_PROCESS__=function(_){"use strict";async function n(_,n={},e){return window.__TAURI_INTERNALS__.invoke(_,n,e)}return"function"==typeof SuppressedError&&SuppressedError,_.exit=async function(_=0){await n("plugin:process|exit",{code:_})},_.relaunch=async function(){await n("plugin:process|restart")},_}({});Object.defineProperty(window.__TAURI__,"process",{value:__TAURI_PLUGIN_PROCESS__})}
