if("__TAURI__"in window){var __TAURI_WEBSOCKET__=function(){"use strict";var e=Object.defineProperty,t=(e,t,n)=>{if(!t.has(e))throw TypeError("Cannot "+n)},n=(e,n,r)=>(t(e,n,"read from private field"),r?r.call(e):n.get(e)),r=(e,n,r,i)=>(t(e,n,"write to private field"),i?i.call(e,r):n.set(e,r),r);function i(e,t=!1){let n=window.crypto.getRandomValues(new Uint32Array(1))[0],r=`_${n}`;return Object.defineProperty(window,r,{value:n=>(t&&Reflect.deleteProperty(window,r),e?.(n)),writable:!1,configurable:!0}),n}((t,n)=>{for(var r in n)e(t,r,{get:n[r],enumerable:!0})})({},{Channel:()=>a,PluginListener:()=>o,addPluginListener:()=>c,convertFileSrc:()=>d,invoke:()=>l,transformCallback:()=>i});var s,a=class{constructor(){this.__TAURI_CHANNEL_MARKER__=!0,((e,t,n)=>{if(t.has(e))throw TypeError("Cannot add the same private member more than once");t instanceof WeakSet?t.add(e):t.set(e,n)})(this,s,(()=>{})),this.id=i((e=>{n(this,s).call(this,e)}))}set onmessage(e){r(this,s,e)}get onmessage(){return n(this,s)}toJSON(){return`__CHANNEL__:${this.id}`}};s=new WeakMap;var o=class{constructor(e,t,n){this.plugin=e,this.event=t,this.channelId=n}async unregister(){return l(`plugin:${this.plugin}|remove_listener`,{event:this.event,channelId:this.channelId})}};async function c(e,t,n){let r=new a;return r.onmessage=n,l(`plugin:${e}|register_listener`,{event:t,handler:r}).then((()=>new o(e,t,r.id)))}async function l(e,t={}){return new Promise(((n,r)=>{let s=i((e=>{n(e),Reflect.deleteProperty(window,`_${a}`)}),!0),a=i((e=>{r(e),Reflect.deleteProperty(window,`_${s}`)}),!0);window.__TAURI_IPC__({cmd:e,callback:s,error:a,...t})}))}function d(e,t="asset"){let n=encodeURIComponent(e);return navigator.userAgent.includes("Windows")?`https://${t}.localhost/${n}`:`${t}://localhost/${n}`}class u{constructor(e,t){this.id=e,this.listeners=t}static async connect(e,t){const n=[];return await l("plugin:websocket|connect",{url:e,callbackFunction:i((e=>{n.forEach((t=>t(e)))})),options:t}).then((e=>new u(e,n)))}addListener(e){this.listeners.push(e)}async send(e){let t;if("string"==typeof e)t={type:"Text",data:e};else if("object"==typeof e&&"type"in e)t=e;else{if(!Array.isArray(e))throw new Error("invalid `message` type, expected a `{ type: string, data: any }` object, a string or a numeric array");t={type:"Binary",data:e}}return await l("plugin:websocket|send",{id:this.id,message:t})}async disconnect(){return await this.send({type:"Close",data:{code:1e3,reason:"Disconnected by client"}})}}return u}();Object.defineProperty(window.__TAURI__,"websocket",{value:__TAURI_WEBSOCKET__})}
