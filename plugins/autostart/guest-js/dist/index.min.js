var d=Object.defineProperty;var e=(c,a)=>{for(var b in a)d(c,b,{get:a[b],enumerable:!0});};

var f={};e(f,{convertFileSrc:()=>w,invoke:()=>c,transformCallback:()=>s});function u(){return window.crypto.getRandomValues(new Uint32Array(1))[0]}function s(e,r=!1){let n=u(),t=`_${n}`;return Object.defineProperty(window,t,{value:o=>(r&&Reflect.deleteProperty(window,t),e==null?void 0:e(o)),writable:!1,configurable:!0}),n}async function c(e,r={}){return new Promise((n,t)=>{let o=s(i=>{n(i),Reflect.deleteProperty(window,`_${a}`);},!0),a=s(i=>{t(i),Reflect.deleteProperty(window,`_${o}`);},!0);window.__TAURI_IPC__({cmd:e,callback:o,error:a,...r});})}function w(e,r="asset"){let n=encodeURIComponent(e);return navigator.userAgent.includes("Windows")?`https://${r}.localhost/${n}`:`${r}://localhost/${n}`}

// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
async function isEnabled() {
    return await c("plugin:autostart|is_enabled");
}
async function enable() {
    await c("plugin:autostart|enable");
}
async function disable() {
    await c("plugin:autostart|disable");
}

export { disable, enable, isEnabled };
//# sourceMappingURL=index.min.js.map
