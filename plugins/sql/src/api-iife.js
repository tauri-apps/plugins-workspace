if("__TAURI__"in window){var __TAURI_SQL__=function(){"use strict";var e=Object.defineProperty,t=(e,t,n)=>{if(!t.has(e))throw TypeError("Cannot "+n)},n=(e,n,r)=>(t(e,n,"read from private field"),r?r.call(e):n.get(e)),r=(e,n,r,s)=>(t(e,n,"write to private field"),s?s.call(e,r):n.set(e,r),r);function s(e,t=!1){let n=window.crypto.getRandomValues(new Uint32Array(1))[0],r=`_${n}`;return Object.defineProperty(window,r,{value:n=>(t&&Reflect.deleteProperty(window,r),e?.(n)),writable:!1,configurable:!0}),n}((t,n)=>{for(var r in n)e(t,r,{get:n[r],enumerable:!0})})({},{Channel:()=>a,PluginListener:()=>l,addPluginListener:()=>o,convertFileSrc:()=>u,invoke:()=>c,transformCallback:()=>s});var i,a=class{constructor(){this.__TAURI_CHANNEL_MARKER__=!0,((e,t,n)=>{if(t.has(e))throw TypeError("Cannot add the same private member more than once");t instanceof WeakSet?t.add(e):t.set(e,n)})(this,i,(()=>{})),this.id=s((e=>{n(this,i).call(this,e)}))}set onmessage(e){r(this,i,e)}get onmessage(){return n(this,i)}toJSON(){return`__CHANNEL__:${this.id}`}};i=new WeakMap;var l=class{constructor(e,t,n){this.plugin=e,this.event=t,this.channelId=n}async unregister(){return c(`plugin:${this.plugin}|remove_listener`,{event:this.event,channelId:this.channelId})}};async function o(e,t,n){let r=new a;return r.onmessage=n,c(`plugin:${e}|register_listener`,{event:t,handler:r}).then((()=>new l(e,t,r.id)))}async function c(e,t={}){return new Promise(((n,r)=>{let i=s((e=>{n(e),Reflect.deleteProperty(window,`_${a}`)}),!0),a=s((e=>{r(e),Reflect.deleteProperty(window,`_${i}`)}),!0);window.__TAURI_IPC__({cmd:e,callback:i,error:a,...t})}))}function u(e,t="asset"){let n=encodeURIComponent(e);return navigator.userAgent.includes("Windows")?`https://${t}.localhost/${n}`:`${t}://localhost/${n}`}class d{constructor(e){this.path=e}static async load(e){const t=await c("plugin:sql|load",{db:e});return new d(t)}static get(e){return new d(e)}async execute(e,t){const[n,r]=await c("plugin:sql|execute",{db:this.path,query:e,values:null!=t?t:[]});return{lastInsertId:r,rowsAffected:n}}async select(e,t){return await c("plugin:sql|select",{db:this.path,query:e,values:null!=t?t:[]})}async close(e){return await c("plugin:sql|close",{db:e})}}return d}();Object.defineProperty(window.__TAURI__,"sql",{value:__TAURI_SQL__})}
