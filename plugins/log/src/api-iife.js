if("__TAURI__"in window){var __TAURI_LOG__=function(e){"use strict";var n=Object.defineProperty,t=(e,t)=>{for(var r in t)n(e,r,{get:t[r],enumerable:!0})},r=(e,n,t)=>{if(!n.has(e))throw TypeError("Cannot "+t)},a=(e,n,t)=>(r(e,n,"read from private field"),t?t.call(e):n.get(e)),i=(e,n,t,a)=>(r(e,n,"write to private field"),a?a.call(e,t):n.set(e,t),t);function o(e,n=!1){let t=window.crypto.getRandomValues(new Uint32Array(1))[0],r=`_${t}`;return Object.defineProperty(window,r,{value:t=>(n&&Reflect.deleteProperty(window,r),e?.(t)),writable:!1,configurable:!0}),t}t({},{Channel:()=>c,PluginListener:()=>s,addPluginListener:()=>u,convertFileSrc:()=>_,invoke:()=>d,transformCallback:()=>o});var l,c=class{constructor(){this.__TAURI_CHANNEL_MARKER__=!0,((e,n,t)=>{if(n.has(e))throw TypeError("Cannot add the same private member more than once");n instanceof WeakSet?n.add(e):n.set(e,t)})(this,l,(()=>{})),this.id=o((e=>{a(this,l).call(this,e)}))}set onmessage(e){i(this,l,e)}get onmessage(){return a(this,l)}toJSON(){return`__CHANNEL__:${this.id}`}};l=new WeakMap;var s=class{constructor(e,n,t){this.plugin=e,this.event=n,this.channelId=t}async unregister(){return d(`plugin:${this.plugin}|remove_listener`,{event:this.event,channelId:this.channelId})}};async function u(e,n,t){let r=new c;return r.onmessage=t,d(`plugin:${e}|register_listener`,{event:n,handler:r}).then((()=>new s(e,n,r.id)))}async function d(e,n={}){return new Promise(((t,r)=>{let a=o((e=>{t(e),Reflect.deleteProperty(window,`_${i}`)}),!0),i=o((e=>{r(e),Reflect.deleteProperty(window,`_${a}`)}),!0);window.__TAURI_IPC__({cmd:e,callback:a,error:i,...n})}))}function _(e,n="asset"){let t=encodeURIComponent(e);return navigator.userAgent.includes("Windows")?`https://${n}.localhost/${t}`:`${n}://localhost/${t}`}async function f(e,n){await d("plugin:event|unlisten",{event:e,eventId:n})}async function w(e,n,t){return d("plugin:event|listen",{event:e,windowLabel:n,handler:o(t)}).then((n=>async()=>f(e,n)))}t({},{TauriEvent:()=>h,emit:()=>p,listen:()=>E,once:()=>y});var v,g,h=((v=h||{}).WINDOW_RESIZED="tauri://resize",v.WINDOW_MOVED="tauri://move",v.WINDOW_CLOSE_REQUESTED="tauri://close-requested",v.WINDOW_CREATED="tauri://window-created",v.WINDOW_DESTROYED="tauri://destroyed",v.WINDOW_FOCUS="tauri://focus",v.WINDOW_BLUR="tauri://blur",v.WINDOW_SCALE_FACTOR_CHANGED="tauri://scale-change",v.WINDOW_THEME_CHANGED="tauri://theme-changed",v.WINDOW_FILE_DROP="tauri://file-drop",v.WINDOW_FILE_DROP_HOVER="tauri://file-drop-hover",v.WINDOW_FILE_DROP_CANCELLED="tauri://file-drop-cancelled",v.MENU="tauri://menu",v);async function E(e,n){return w(e,null,n)}async function y(e,n){return async function(e,n,t){return w(e,n,(n=>{t(n),f(e,n.id).catch((()=>{}))}))}(e,null,n)}async function p(e,n){return async function(e,n,t){await d("plugin:event|emit",{event:e,windowLabel:n,payload:t})}(e,void 0,n)}async function I(e,n,t){var r,a;const i=null===(r=(new Error).stack)||void 0===r?void 0:r.split("\n").map((e=>e.split("@"))),o=null==i?void 0:i.filter((([e,n])=>e.length>0&&"[native code]"!==n)),{file:l,line:c,...s}=null!=t?t:{};let u=null===(a=null==o?void 0:o[0])||void 0===a?void 0:a.filter((e=>e.length>0)).join("@");"Error"===u&&(u="webview::unknown"),await d("plugin:log|log",{level:e,message:n,location:u,file:l,line:c,keyValues:s})}return function(e){e[e.Trace=1]="Trace",e[e.Debug=2]="Debug",e[e.Info=3]="Info",e[e.Warn=4]="Warn",e[e.Error=5]="Error"}(g||(g={})),e.attachConsole=async function(){return await E("log://log",(e=>{const n=e.payload,t=n.message.replace(/[\u001b\u009b][[()#;?]*(?:[0-9]{1,4}(?:;[0-9]{0,4})*)?[0-9A-ORZcf-nqry=><]/g,"");switch(n.level){case g.Trace:console.log(t);break;case g.Debug:console.debug(t);break;case g.Info:console.info(t);break;case g.Warn:console.warn(t);break;case g.Error:console.error(t);break;default:throw new Error(`unknown log level ${n.level}`)}}))},e.debug=async function(e,n){await I(g.Debug,e,n)},e.error=async function(e,n){await I(g.Error,e,n)},e.info=async function(e,n){await I(g.Info,e,n)},e.trace=async function(e,n){await I(g.Trace,e,n)},e.warn=async function(e,n){await I(g.Warn,e,n)},e}({});Object.defineProperty(window.__TAURI__,"log",{value:__TAURI_LOG__})}
