if("__TAURI__"in window){var __TAURI_APP__=function(e){"use strict";var n=Object.defineProperty,t=(e,n,t)=>{if(!n.has(e))throw TypeError("Cannot "+t)},r=(e,n,r)=>(t(e,n,"read from private field"),r?r.call(e):n.get(e)),i=(e,n,r,i)=>(t(e,n,"write to private field"),i?i.call(e,r):n.set(e,r),r);function a(e,n=!1){let t=window.crypto.getRandomValues(new Uint32Array(1))[0],r=`_${t}`;return Object.defineProperty(window,r,{value:t=>(n&&Reflect.deleteProperty(window,r),e?.(t)),writable:!1,configurable:!0}),t}((e,t)=>{for(var r in t)n(e,r,{get:t[r],enumerable:!0})})({},{Channel:()=>s,PluginListener:()=>l,addPluginListener:()=>c,convertFileSrc:()=>_,invoke:()=>u,transformCallback:()=>a});var o,s=class{constructor(){this.__TAURI_CHANNEL_MARKER__=!0,((e,n,t)=>{if(n.has(e))throw TypeError("Cannot add the same private member more than once");n instanceof WeakSet?n.add(e):n.set(e,t)})(this,o,(()=>{})),this.id=a((e=>{r(this,o).call(this,e)}))}set onmessage(e){i(this,o,e)}get onmessage(){return r(this,o)}toJSON(){return`__CHANNEL__:${this.id}`}};o=new WeakMap;var l=class{constructor(e,n,t){this.plugin=e,this.event=n,this.channelId=t}async unregister(){return u(`plugin:${this.plugin}|remove_listener`,{event:this.event,channelId:this.channelId})}};async function c(e,n,t){let r=new s;return r.onmessage=t,u(`plugin:${e}|register_listener`,{event:n,handler:r}).then((()=>new l(e,n,r.id)))}async function u(e,n={}){return new Promise(((t,r)=>{let i=a((e=>{t(e),Reflect.deleteProperty(window,`_${o}`)}),!0),o=a((e=>{r(e),Reflect.deleteProperty(window,`_${i}`)}),!0);window.__TAURI_IPC__({cmd:e,callback:i,error:o,...n})}))}function _(e,n="asset"){let t=encodeURIComponent(e);return navigator.userAgent.includes("Windows")?`https://${n}.localhost/${t}`:`${n}://localhost/${t}`}return e.getName=async function(){return u("plugin:app|name")},e.getTauriVersion=async function(){return u("plugin:app|tauri_version")},e.getVersion=async function(){return u("plugin:app|version")},e.hide=async function(){return u("plugin:app|hide")},e.show=async function(){return u("plugin:app|show")},e}({});Object.defineProperty(window.__TAURI__,"app",{value:__TAURI_APP__})}
