if("__TAURI__"in window){var __TAURI_STRONGHOLD__=function(t){"use strict";var e=Object.defineProperty,r=(t,e,r)=>{if(!e.has(t))throw TypeError("Cannot "+r)},n=(t,e,n)=>(r(t,e,"read from private field"),n?n.call(t):e.get(t)),a=(t,e,n,a)=>(r(t,e,"write to private field"),a?a.call(t,n):e.set(t,n),n);function s(t,e=!1){let r=window.crypto.getRandomValues(new Uint32Array(1))[0],n=`_${r}`;return Object.defineProperty(window,n,{value:r=>(e&&Reflect.deleteProperty(window,n),t?.(r)),writable:!1,configurable:!0}),r}((t,r)=>{for(var n in r)e(t,n,{get:r[n],enumerable:!0})})({},{Channel:()=>o,PluginListener:()=>c,addPluginListener:()=>h,convertFileSrc:()=>u,invoke:()=>l,transformCallback:()=>s});var i,o=class{constructor(){this.__TAURI_CHANNEL_MARKER__=!0,((t,e,r)=>{if(e.has(t))throw TypeError("Cannot add the same private member more than once");e instanceof WeakSet?e.add(t):e.set(t,r)})(this,i,(()=>{})),this.id=s((t=>{n(this,i).call(this,t)}))}set onmessage(t){a(this,i,t)}get onmessage(){return n(this,i)}toJSON(){return`__CHANNEL__:${this.id}`}};i=new WeakMap;var c=class{constructor(t,e,r){this.plugin=t,this.event=e,this.channelId=r}async unregister(){return l(`plugin:${this.plugin}|remove_listener`,{event:this.event,channelId:this.channelId})}};async function h(t,e,r){let n=new o;return n.onmessage=r,l(`plugin:${t}|register_listener`,{event:e,handler:n}).then((()=>new c(t,e,n.id)))}async function l(t,e={}){return new Promise(((r,n)=>{let a=s((t=>{r(t),Reflect.deleteProperty(window,`_${i}`)}),!0),i=s((t=>{n(t),Reflect.deleteProperty(window,`_${a}`)}),!0);window.__TAURI_IPC__({cmd:t,callback:a,error:i,...e})}))}function u(t,e="asset"){let r=encodeURIComponent(t);return navigator.userAgent.includes("Windows")?`https://${e}.localhost/${r}`:`${e}://localhost/${r}`}function p(t){return"string"==typeof t?t:Array.from(t instanceof ArrayBuffer?new Uint8Array(t):t)}class d{constructor(t,e){this.type=t,this.payload=e}static generic(t,e){return new d("Generic",{vault:p(t),record:p(e)})}static counter(t,e){return new d("Counter",{vault:p(t),counter:e})}}class g{constructor(t){this.procedureArgs=t}async generateSLIP10Seed(t,e){return await l("plugin:stronghold|execute_procedure",{...this.procedureArgs,procedure:{type:"SLIP10Generate",payload:{output:t,sizeBytes:e}}}).then((t=>Uint8Array.from(t)))}async deriveSLIP10(t,e,r,n){return await l("plugin:stronghold|execute_procedure",{...this.procedureArgs,procedure:{type:"SLIP10Derive",payload:{chain:t,input:{type:e,payload:r},output:n}}}).then((t=>Uint8Array.from(t)))}async recoverBIP39(t,e,r){return await l("plugin:stronghold|execute_procedure",{...this.procedureArgs,procedure:{type:"BIP39Recover",payload:{mnemonic:t,passphrase:r,output:e}}}).then((t=>Uint8Array.from(t)))}async generateBIP39(t,e){return await l("plugin:stronghold|execute_procedure",{...this.procedureArgs,procedure:{type:"BIP39Generate",payload:{output:t,passphrase:e}}}).then((t=>Uint8Array.from(t)))}async getEd25519PublicKey(t){return await l("plugin:stronghold|execute_procedure",{...this.procedureArgs,procedure:{type:"PublicKey",payload:{type:"Ed25519",privateKey:t}}}).then((t=>Uint8Array.from(t)))}async signEd25519(t,e){return await l("plugin:stronghold|execute_procedure",{...this.procedureArgs,procedure:{type:"Ed25519Sign",payload:{privateKey:t,msg:e}}}).then((t=>Uint8Array.from(t)))}}class y{constructor(t,e){this.path=t,this.name=p(e)}getVault(t){return new w(this.path,this.name,p(t))}getStore(){return new _(this.path,this.name)}}class _{constructor(t,e){this.path=t,this.client=e}async get(t){return await l("plugin:stronghold|get_store_record",{snapshotPath:this.path,client:this.client,key:p(t)}).then((t=>Uint8Array.from(t)))}async insert(t,e,r){return await l("plugin:stronghold|save_store_record",{snapshotPath:this.path,client:this.client,key:p(t),value:e,lifetime:r})}async remove(t){return await l("plugin:stronghold|remove_store_record",{snapshotPath:this.path,client:this.client,key:p(t)}).then((t=>null!=t?Uint8Array.from(t):null))}}class w extends g{constructor(t,e,r){super({snapshotPath:t,client:e,vault:r}),this.path=t,this.client=p(e),this.name=p(r)}async insert(t,e){return await l("plugin:stronghold|save_secret",{snapshotPath:this.path,client:this.client,vault:this.name,recordPath:p(t),secret:e})}async remove(t){return await l("plugin:stronghold|remove_secret",{snapshotPath:this.path,client:this.client,vault:this.name,location:t})}}return t.Client=y,t.Location=d,t.Store=_,t.Stronghold=class{constructor(t,e){this.path=t,this.reload(e)}async reload(t){return await l("plugin:stronghold|initialize",{snapshotPath:this.path,password:t})}async unload(){return await l("plugin:stronghold|destroy",{snapshotPath:this.path})}async loadClient(t){return await l("plugin:stronghold|load_client",{snapshotPath:this.path,client:p(t)}).then((()=>new y(this.path,t)))}async createClient(t){return await l("plugin:stronghold|create_client",{snapshotPath:this.path,client:p(t)}).then((()=>new y(this.path,t)))}async save(){return await l("plugin:stronghold|save",{snapshotPath:this.path})}},t.Vault=w,t}({});Object.defineProperty(window.__TAURI__,"stronghold",{value:__TAURI_STRONGHOLD__})}
