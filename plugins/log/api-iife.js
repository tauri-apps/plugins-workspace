if("__TAURI__"in window){var __TAURI_PLUGIN_LOG__=function(t){"use strict";function e(t,e,n,r){if("a"===n&&!r)throw new TypeError("Private accessor was defined without a getter");if("function"==typeof e?t!==e||!r:!e.has(t))throw new TypeError("Cannot read private member from an object whose class did not declare it");return"m"===n?r:"a"===n?r.call(t):r?r.value:e.get(t)}var n,r;function o(t){return t instanceof Map}function i(t){return t instanceof Set}"function"==typeof SuppressedError&&SuppressedError,"function"==typeof SuppressedError&&SuppressedError;const l=0,s=1,a=2,c=16,f={indentationLvl:0,currentDepth:0,stylize:t=>t,showHidden:!1,depth:4,colors:!1,showProxy:!1,breakLength:80,escapeSequences:!0,compact:3,sorted:!1,getters:!1,trailingComma:!1,indentLevel:0};function u(){return{budget:{},seen:[],circular:new Map,quotes:[],...f}}const p=new RegExp("^[A-Z][a-zA-Z0-9]+$"),h=new Set(Object.getOwnPropertyNames(globalThis).filter((t=>p.test(t)))),d=t=>void 0===t&&void 0!==t,g=new RegExp("[\0-'\\-]","g"),y=new RegExp("^[a-zA-Z_][a-zA-Z_0-9]*$"),$=new RegExp("^(0|[1-9][0-9]*)$"),b=["\\x00","\\x01","\\x02","\\x03","\\x04","\\x05","\\x06","\\x07","\\b","\\t","\\n","\\x0B","\\f","\\r","\\x0E","\\x0F","\\x10","\\x11","\\x12","\\x13","\\x14","\\x15","\\x16","\\x17","\\x18","\\x19","\\x1A","\\x1B","\\x1C","\\x1D","\\x1E","\\x1F","","","","","","","","\\'","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","\\\\","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","\\x7F","\\x80","\\x81","\\x82","\\x83","\\x84","\\x85","\\x86","\\x87","\\x88","\\x89","\\x8A","\\x8B","\\x8C","\\x8D","\\x8E","\\x8F","\\x90","\\x91","\\x92","\\x93","\\x94","\\x95","\\x96","\\x97","\\x98","\\x99","\\x9A","\\x9B","\\x9C","\\x9D","\\x9E","\\x9F"],m=t=>b[t.charCodeAt(0)],v=new RegExp("[\\u001B\\u009B][[\\]()#;?]*(?:(?:(?:(?:;[-a-zA-Z\\d\\/#&.:=?%@~_]+)*|[a-zA-Z\\d]+(?:;[-a-zA-Z\\d\\/#&.:=?%@~_]*)*)?\\u0007)|(?:(?:\\d{1,4}(?:;\\d{0,4})*)?[\\dA-PR-TZcf-ntqry=><~]))","g");function w(t,e=!0){let n=0;e&&(t=function(t){return t.replace(v,"")}(t)),t=t.normalize("NFC");for(const e of t){const t=e.codePointAt(0);_(t)?n+=2:x(t)||n++}return n}const x=t=>t<=31||t>=127&&t<=159||t>=768&&t<=879||t>=8203&&t<=8207||t>=8400&&t<=8447||t>=65024&&t<=65039||t>=65056&&t<=65071||t>=917760&&t<=917999;function _(t){return t>=4352&&(t<=4447||9001===t||9002===t||t>=11904&&t<=12871&&12351!==t||t>=12880&&t<=19903||t>=19968&&t<=42182||t>=43360&&t<=43388||t>=44032&&t<=55203||t>=63744&&t<=64255||t>=65040&&t<=65049||t>=65072&&t<=65131||t>=65281&&t<=65376||t>=65504&&t<=65510||t>=110592&&t<=110593||t>=127488&&t<=127569||t>=127744&&t<=128591||t>=131072&&t<=262141)}function O(t,e={__proto__:null}){const n={...u(),...e},r=t[0];let o=0,i="";if("string"==typeof r&&t.length>1){o++;let e=0;for(let l=0;l<r.length-1;l++)if("%"==r[l]){const s=r[++l];if(o<t.length){let a=null;if("s"==s)a=String(t[o++]);else if(["d","i"].includes(s)){const e=t[o++];a="bigint"==typeof e?`${e}n`:"number"==typeof e?`${Number.parseInt(String(e))}`:"NaN"}else if("f"==s){const e=t[o++];a="number"==typeof e?`${e}`:"NaN"}else["O","o"].includes(s)?a=L(n,t[o++],0):"c"==s&&(a="");null!=a&&(i+=r.slice(e,l-1)+a,e=l+1)}"%"==s&&(i+=r.slice(e,l-1)+"%",e=l+1)}i+=r.slice(e)}for(;o<t.length;o++)o>0&&(i+=" "),"string"==typeof t[o]?i+=t[o]:i+=L(n,t[o],0);if(n.indentLevel>0){const t="  ".repeat(n.indentLevel);i=t+i.replaceAll("\n",`\n${t}`)}return i}function L(t,e,n,r){if("object"!=typeof e&&"function"!=typeof e&&!d(e))return A(t.stylize,e,t);if(null===e)return t.stylize("null","null");if(t.seen.includes(e)){let n=1;return void 0===t.circular?(t.circular=new Map,t.circular.set(e,n)):(n=t.circular.get(e),void 0===n&&(n=t.circular.size+1,t.circular.set(e,n))),t.stylize(`[Circular *${n}]`,"special")}return function(t,e,n,r){let s,c=[];t.showHidden&&(n<=t.depth||null===t.depth)&&(s=[]);const f=ot(e,t,n,s);void 0!==s&&0===s.length&&(s=void 0);let u=e[Symbol.toStringTag];"string"!=typeof u&&(u="");const p=u;let h,d="",g=()=>[],y=[],$=!0,b=0,m=l;if(Reflect.has(e,Symbol.iterator)||null===f)if($=!1,Array.isArray(e)){const t="Array"!==f||""!==p?Y(f,p,"Array",`(${e.length})`):"";if(c=Object.getOwnPropertyNames(e),y=[`${t}[`,"]"],0===e.length&&0===c.length&&void 0===s)return`${y[0]}]`;m=a,g=N}else if(i(e)){const n=e.size,r=Y(f,p,"Set",`(${n})`);if(c=K(e,t.showHidden),g=null!==f?D.bind(null,e):D.bind(null,new Set(e.values())),0===n&&0===c.length&&void 0===s)return`${r}{}`;y=[`${r}{`,"}"]}else if(o(e)){const n=e.size,r=Y(f,p,"Map",`(${n})`);if(c=K(e,t.showHidden),g=null!==f?I.bind(null,e):I.bind(null,new Map(e.entries())),0===n&&0===c.length&&void 0===s)return`${r}{}`;y=[`${r}{`,"}"]}else if(function(t){return t instanceof Int8Array||t instanceof Uint8Array||t instanceof Uint8ClampedArray||t instanceof Int16Array||t instanceof Uint16Array||t instanceof Int32Array||t instanceof Uint32Array||t instanceof Float32Array||t instanceof Float64Array||t instanceof BigInt64Array||t instanceof BigUint64Array}(e)){c=Object.getOwnPropertyNames(e);const n="",r=e.length;if(y=[`${Y(f,p,n,`(${r})`)}[`,"]"],0===e.length&&0===c.length&&!t.showHidden)return`${y[0]}]`;g=B.bind(null,e,r),m=a}else $=!0;if($)if(c=K(e,t.showHidden),y=["{","}"],"Object"===f){if(!function(t){return"object"==typeof t&&"[object Arguments]"===Object.prototype.toString.call(t)}(e)?""!==p&&(y[0]=`${Y(f,p,"Object")}{`):y[0]="[Arguments] {",0===c.length&&void 0===s)return`${y[0]}}`}else if("function"==typeof e){if(d=function(t,e,n){const r=t.toString();if(r.startsWith("class")&&r.endsWith("}")){const o=r.slice(5,-1),i=o.indexOf("{");if(-1!==i&&(!o.slice(0,i).includes("(")||null!==tt.exec(RegExp.prototype[Symbol.replace].call(X,o,""))))return function(t,e,n){function r(t){return Object.hasOwn(t,"name")}const o=r(t)&&t.name||"(anonymous)";let i=`class ${o}`;"Function"!==e&&null!==e&&(i+=` [${e}]`);""!==n&&e!==n&&(i+=` [${n}]`);if(null!==e){const e=Object.getPrototypeOf(t).name;e&&(i+=` extends ${e}`)}else i+=" extends [null prototype]";return`[${i}]`}(t,e,n)}let o="Function";(function(t){return"function"==typeof t&&"GeneratorFunction"===t[Symbol.toStringTag]})(t)&&(o=`Generator${o}`);(function(t){return"function"==typeof t&&"AsyncFunction"===t[Symbol.toStringTag]})(t)&&(o=`Async${o}`);let i=`[${o}`;null===e&&(i+=" (null prototype)");""===t.name?i+=" (anonymous)":i+=`: ${t.name}`;i+="]",e!==o&&null!==e&&(i+=` ${e}`);""!==n&&e!==n&&(i+=` [${n}]`);return i}(e,f,p),0===c.length&&void 0===s)return t.stylize(d,"special")}else if(function(t){return t instanceof RegExp}(e)){d=(null!==f?e:new RegExp(e)).toString();const r=Y(f,p,"RegExp");if("RegExp "!==r&&(d=`${r}${d}`),0===c.length&&void 0===s||n>t.depth&&null!==t.depth)return t.stylize(d,"regexp")}else if(function(t){return t instanceof Date}(e)){if(Number.isNaN(e.getTime()))return t.stylize("Invalid Date","date");if(d=e.toISOString(),0===c.length&&void 0===s)return t.stylize(d,"date")}else{if(void 0!==globalThis.Temporal&&(Object.prototype.isPrototypeOf.call(globalThis.Temporal.Instant.prototype,e)||Object.prototype.isPrototypeOf.call(globalThis.Temporal.ZonedDateTime.prototype,e)||Object.prototype.isPrototypeOf.call(globalThis.Temporal.PlainDate.prototype,e)||Object.prototype.isPrototypeOf.call(globalThis.Temporal.PlainTime.prototype,e)||Object.prototype.isPrototypeOf.call(globalThis.Temporal.PlainDateTime.prototype,e)||Object.prototype.isPrototypeOf.call(globalThis.Temporal.PlainYearMonth.prototype,e)||Object.prototype.isPrototypeOf.call(globalThis.Temporal.PlainMonthDay.prototype,e)||Object.prototype.isPrototypeOf.call(globalThis.Temporal.Duration.prototype,e)||Object.prototype.isPrototypeOf.call(globalThis.Temporal.TimeZone.prototype,e)||Object.prototype.isPrototypeOf.call(globalThis.Temporal.Calendar.prototype,e)))return t.stylize(e.toString(),"temporal");if(function(t){return t instanceof ArrayBuffer||t instanceof SharedArrayBuffer}(e)){const n=function(t){return t instanceof ArrayBuffer}(e)?"ArrayBuffer":"SharedArrayBuffer",o=Y(f,p,n);if(void 0===r)g=F;else if(0===c.length&&void 0===s)return o+`{ byteLength: ${S(t.stylize,e.byteLength)} }`;y[0]=`${o}{`,c.unshift("byteLength")}else if(function(t){return ArrayBuffer.isView(t)&&t instanceof DataView}(e))y[0]=`${Y(f,p,"DataView")}{`,c.unshift("byteLength","byteOffset","buffer");else if(function(t){return t instanceof Promise}(e))y[0]=`${Y(f,p,"Promise")}{`,g=W;else if(function(t){return t instanceof WeakSet}(e))y[0]=`${Y(f,p,"WeakSet")}{`,g=t.showHidden?H:Z;else if(function(t){return t instanceof WeakMap}(e))y[0]=`${Y(f,p,"WeakMap")}{`,g=t.showHidden?q:Z;else{if(0===c.length&&void 0===s)return`${J(e,f,p)}{}`;y[0]=`${J(e,f,p)}{`}}if(n>t.depth&&null!==t.depth){let n=J(e,f,p).slice(0,-1);return null!==f&&(n=`[${n}]`),t.stylize(n,"special")}n+=1,t.seen.push(e),t.currentDepth=n;try{for(h=g(t,e,n),b=0;b<c.length;b++)h.push(nt(t,e,n,c[b],m));void 0!==s&&h.push(...s)}catch(e){return t.stylize(`[Internal Formatting Error] ${e.stack}`,"internalError")}if(void 0!==t.circular){const n=t.circular.get(e);if(void 0!==n){const e=t.stylize(`<ref *${n}>`,"special");!0!==t.compact?d=""===d?e:`${e} ${d}`:y[0]=`${e} ${y[0]}`}}if(t.seen.pop(),t.sorted){const e=!0===t.sorted?void 0:t.sorted;if(m===l)h=h.sort(e);else if(c.length>1){const t=h.slice(h.length-c.length).sort(e);h.splice(h.length-c.length,c.length,...t)}}const v=function(t,e,n,r,o,i,l){if(!0!==t.compact){if("number"==typeof t.compact&&t.compact>=1){const s=e.length;if(o===a&&s>6&&(e=function(t,e,n){let r=0,o=0,i=0,l=e.length;100<e.length&&l--;const s=2,a=[];for(;i<l;i++){const n=w(e[i],t.colors);a[i]=n,r+=n+s,o<n&&(o=n)}const c=o+s;if(3*c+t.indentationLvl<t.breakLength&&(r/c>5||o<=6)){const o=2.5,i=Math.sqrt(c-r/e.length),f=Math.max(c-3-i,1),u=Math.min(Math.round(Math.sqrt(o*f*l)/f),Math.floor((t.breakLength-t.indentationLvl)/c),4*t.compact,15);if(u<=1)return e;const p=[],h=[];for(let t=0;t<u;t++){let n=0;for(let r=t;r<e.length;r+=u)a[r]>n&&(n=a[r]);n+=s,h[t]=n}let d=String.prototype.padStart;if(void 0!==n)for(let t=0;t<e.length;t++)if("number"!=typeof n[t]&&"bigint"!=typeof n[t]){d=String.prototype.padEnd;break}for(let t=0;t<l;t+=u){const n=Math.min(t+u,l);let r="",o=t;for(;o<n-1;o++){const n=h[o-t]+e[o].length-a[o];r+=d.call(`${e[o]}, `,n," ")}if(d===String.prototype.padStart){const n=h[o-t]+e[o].length-a[o]-s;r+=String.prototype.padStart.call(e[o],n," ")}else r+=e[o];p.push(r)}100<e.length&&p.push(e[l]),e=p}return e}(t,e,l)),t.currentDepth-i<t.compact&&s===e.length){if(Q(t,e,e.length+t.indentationLvl+r[0].length+n.length+10,n)){const t=e.join(", ");if(!t.includes("\n"))return`${n?`${n} `:""}${r[0]} ${t} ${r[1]}`}}}const s=`\n${" ".repeat(t.indentationLvl)}`;return`${n?`${n} `:""}${r[0]}${s}  ${e.join(`,${s}  `)}${t.trailingComma?",":""}${s}${r[1]}`}if(Q(t,e,0,n))return`${r[0]}${n?` ${n}`:""} ${e.join(", ")} `+r[1];const s=" ".repeat(t.indentationLvl),c=""===n&&1===r[0].length?" ":`${n?` ${n}`:""}\n${s}  `;return`${r[0]}${c}${e.join(`,\n${s}  `)} ${r[1]}`}(t,h,d,y,m,n,[e]),x=t.budget[t.indentationLvl]||0,_=x+v.length;t.budget[t.indentationLvl]=_,_>2**27&&(t.depth=-1);return v}(t,e,n,r)}const j=new RegExp("(?<=\n)");function A(t,e,n){return"string"==typeof e?e.length>c&&e.length>n.breakLength-n.indentationLvl-4?e.split(j).map((e=>t(P(e,n),"string"))).join(` +\n${" ".repeat(n.indentationLvl+2)}`):t(P(e,n),"string"):"number"==typeof e?S(t,e):"bigint"==typeof e?T(t,e):"boolean"==typeof e?t(`${e}`,"boolean"):void 0===e?t("undefined","undefined"):t(z(e,n),"symbol")}function S(t,e){return t(Object.is(e,-0)?"-0":`${e}`,"number")}function T(t,e){return t(`${e}n`,"bigint")}const E=new RegExp(/^[a-zA-Z_][a-zA-Z_.0-9]*$/);function z(t,e){const n=t.description;return void 0===n||E.test(n)?t.toString():`Symbol(${P(n,e)})`}function P(t,e){const n=e.quotes.find((e=>!t.includes(e)))??e.quotes[0],r=new RegExp(`(?=[${n}\\\\])`,"g");return t=t.replace(r,"\\"),e.escapeSequences&&(t=function(t){return t.replace(M,(t=>R[t])).replace(k,(t=>"\\x"+t.charCodeAt(0).toString(16).padStart(2,"0")))}(t)),`${n}${t}${n}`}const M=new RegExp(/([\b\f\n\r\t\v])/g),R=Object.freeze({"\b":"\\b","\f":"\\f","\n":"\\n","\r":"\\r","\t":"\\t","\v":"\\v"}),k=new RegExp("[\0--]","g");function D(t,e,n,r){e.indentationLvl+=2;const o=[...t],i=t.size,l=Math.min(100,i),s=i-l,a=[];for(let t=0;t<l;t++)a.push(L(e,o[t],r));return s>0&&a.push(`... ${s} more item${s>1?"s":""}`),e.indentationLvl-=2,a}function I(t,e,n,r){e.indentationLvl+=2;const o=[...t],i=t.size,l=Math.min(100,i),s=i-l,a=[];for(let t=0;t<l;t++)a.push(`${L(e,o[t][0],r)} => ${L(e,o[t][1],r)}`);return s>0&&a.push(`... ${s} more item${s>1?"s":""}`),e.indentationLvl-=2,a}function N(t,e,n){const r=e.length,o=Math.min(100,r),i=r-o,l=[];for(let r=0;r<o;r++){if(!Object.hasOwn(e,r))return C(t,e,n,o,l,r);l.push(nt(t,e,n,r,s))}return i>0&&l.push(`... ${i} more item${i>1?"s":""}`),l}function C(t,e,n,r,o,i){const l=Object.keys(e);let a=i;for(;i<l.length&&o.length<r;i++){const c=l[i],f=+c;if(f>2**32-2)break;if(`${a}`!==c){if(!$.test(c))break;const e=f-a,n=`<${e} empty item${e>1?"s":""}>`;if(o.push(t.stylize(n,"undefined")),a=f,o.length===r)break}o.push(nt(t,e,n,c,s)),a++}const c=e.length-a;if(o.length!==r){if(c>0){const e=`<${c} empty item${c>1?"s":""}>`;o.push(t.stylize(e,"undefined"))}}else c>0&&o.push(`... ${c} more item${c>1?"s":""}`);return o}function B(t,e,n,r,o){const i=Math.min(100,e),l=t.length-i,s=[],a=t.length>0&&"number"==typeof t[0]?S:T;for(let e=0;e<i;++e)s[e]=a(n.stylize,t[e]);if(l>0&&(s[i]=`... ${l} more item${l>1?"s":""}`),n.showHidden){n.indentationLvl+=2;for(const e of["BYTES_PER_ELEMENT","length","byteLength","byteOffset","buffer"]){const r=L(n,t[e],o,!0);s.push(`[${e}]: ${r}`)}n.indentationLvl-=2}return s}const U=new RegExp("(.{2})","g");function F(t,e,n){let r;try{r=e.byteLength}catch{r=function(t){return V??=Object.getOwnPropertyDescriptor(SharedArrayBuffer.prototype,"byteLength").get,V.call(t)}(e)}const o=Math.min(100,r);let i;try{i=new Uint8Array(e,0,o)}catch{return[t.stylize("(detached)","special")]}let l=function(t,e,n){const r=t.length;(!e||e<0)&&(e=0);(!n||n<0||n>r)&&(n=r);let o="";for(let r=e;r<n;++r)o+=G[t[r]];return o}(i).replace(U,"$1 ").trim();const s=r-o;return s>0&&(l+=` ... ${s} more byte${s>1?"s":""}`),[`${t.stylize("[Uint8Contents]","special")}: <${l}>`]}function W(t,e,n){return["Promise"]}function Z(t){return[t.stylize("<items unknown>","special")]}function H(t,e,n){return["WeakSet"]}function q(t,e,n){return["WeakMap"]}const G=function(){const t="0123456789abcdef",e=[];for(let n=0;n<16;++n){const r=16*n;for(let o=0;o<16;++o)e[r+o]=t[n]+t[o]}return e}();let V;function K(t,e){let n;const r=Object.getOwnPropertySymbols(t);if(e)n=Object.getOwnPropertyNames(t),0!==r.length&&n.push(...r);else{try{n=Object.keys(t)}catch(e){n=Object.getOwnPropertyNames(t)}if(0!==r.length){const e=e=>Object.prototype.propertyIsEnumerable.call(t,e);n.push(...r.filter(e))}}return n}function Y(t,e,n,r=""){return null===t?""!==e&&n!==e?`[${n}${r}: null prototype] [${e}] `:`[${n}${r}: null prototype] `:""!==e&&t!==e?`${t}${r} [${e}] `:`${t}${r} `}function J(t,e,n){let r="";return null===e&&r===n&&(r="Object"),Y(e,n,r)}function Q(t,e,n,r){let o=e.length+n;if(o+e.length>t.breakLength)return!1;for(let n=0;n<e.length;n++)if(o+=e[n].length,o>t.breakLength)return!1;return""===r||!r.includes("\n")}const X=new RegExp("(\\/\\/.*?\\n)|(\\/\\*(.|\\n)*?\\*\\/)","g"),tt=new RegExp("^(\\s+[^(]*?)\\s*{");function et(t,e,n,r,o){let i,s=0,a=[];do{if(0!==s||e===n){if(null===(n=Object.getPrototypeOf(n)))return;const t=Object.getOwnPropertyDescriptor(n,"constructor");if(void 0!==t&&"function"==typeof t.value&&h.has(t.value.name))return}0===s?i=new Set:a.forEach((t=>i.add(t))),a=Reflect.ownKeys(n),t.seen.push(e);for(const c of a){if("constructor"===c||Object.hasOwn(e,c)||0!==s&&i.has(c))continue;const a=Object.getOwnPropertyDescriptor(n,c);if("function"==typeof a.value)continue;const f=nt(t,n,r,c,l,a,e);o.push(f)}t.seen.pop()}while(3!=++s)}function nt(t,e,n,r,o,i,a=e){let c,f,u=" ";if(void 0!==(i=i||Object.getOwnPropertyDescriptor(e,r)||{value:e[r],enumerable:!0}).value){const e=!0!==t.compact||o!==l?2:3;t.indentationLvl+=e,f=L(t,i.value,n),3===e&&t.breakLength<w(f,t.colors)&&(u=`\n${" ".repeat(t.indentationLvl)}`),t.indentationLvl-=e}else if(void 0!==i.get){const e=void 0!==i.set?"Getter/Setter":"Getter",r=t.stylize,o="special";if(t.getters&&(!0===t.getters||"get"===t.getters&&void 0===i.set||"set"===t.getters&&void 0!==i.set))try{const l=i.get.call(a);if(t.indentationLvl+=2,null===l)f=`${r(`[${e}:`,o)} ${r("null","null")}${r("]",o)}`;else if("object"==typeof l)f=`${r(`[${e}]`,o)} ${L(t,l,n)}`;else{const n=A(r,l,t);f=`${r(`[${e}:`,o)} ${n}${r("]",o)}`}t.indentationLvl-=2}catch(t){const n=`<Inspection threw (${t.message})>`;f=`${r(`[${e}:`,o)} ${n}${r("]",o)}`}else f=t.stylize(`[${e}]`,o)}else f=void 0!==i.set?t.stylize("[Setter]","special"):t.stylize("undefined","undefined");if(o===s)return f;if("symbol"==typeof r)c=`[${t.stylize(z(r,t),"symbol")}]`;else if("__proto__"===r)c="['__proto__']";else if(!1===i.enumerable){c=`[${r.replace(g,m)}]`}else c=y.test(r)?t.stylize(r,"name"):t.stylize(P(r,t),"string");return`${c}:${u}${f}`}function rt(t,e){try{return Object.prototype.isPrototypeOf.call(t,e)}catch{return!1}}function ot(t,e,n,r){let o;const i=t;for(;t||d(t);){let l;try{l=Object.getOwnPropertyDescriptor(t,"constructor")}catch{}if(void 0!==l&&"function"==typeof l.value&&""!==l.value.name&&rt(l.value.prototype,i))return void 0===r||o===t&&h.has(l.value.name)||et(e,i,o||i,n,r),String(l.value.name);t=Object.getPrototypeOf(t),void 0===o&&(o=t)}if(null===o)return null;const l=i.prototype.name;if(n>e.depth&&null!==e.depth)return`${l} <Complex prototype>`;const s=ot(o,e,n+1,r);return null===s?`${l} <${function(t,e={__proto__:null}){const n={...u(),...e};return L(n,t,0)}(o,{...e,depth:-1,__proto__:null})}>`:`${l} <${s}>`}const it={middleMiddle:"─",rowMiddle:"┼",topRight:"┐",topLeft:"┌",leftMiddle:"├",topMiddle:"┬",bottomRight:"┘",bottomLeft:"└",bottomMiddle:"┴",rightMiddle:"┤",left:"│ ",right:" │",middle:" │ "};function lt(t,e,n){let r=it.left;for(let o=0;o<t.length;o++){const i=t[o],l=w(i),s=" ".repeat(e[o]-l);r+=n?.[o]?`${s}${i}`:`${i}${s}`,o!==t.length-1&&(r+=it.middle)}return r+=it.right,r}const st=new Map,at=new Map,ct=Symbol("isConsoleInstance");var ft;async function ut(t,e,n){const r=(new Error).stack?.split("\n").map((t=>t.split("@"))),o=r?.filter((([t,e])=>t.length>0&&"[native code]"!==e)),{file:i,line:l,keyValues:s}=n??{};let a=o?.[0]?.filter((t=>t.length>0)).join("@");"Error"===a&&(a="webview::unknown"),await async function(t,e={},n){return window.__TAURI_INTERNALS__.invoke(t,e,n)}("plugin:log|log",{level:t,message:e,location:a,file:i,line:l,keyValues:s})}return n=new WeakMap,r=ct,function(t){t[t.Trace=1]="Trace",t[t.Debug=2]="Debug",t[t.Info=3]="Info",t[t.Warn=4]="Warn",t[t.Error=5]="Error"}(ft||(ft={})),Object.defineProperty(globalThis,"console",{value:new class{constructor(t){n.set(this,void 0),this.indentLevel=0,this[r]=!1,this.log=(...t)=>{e(this,n,"f").call(this,O(t,{...u(),indentLevel:this.indentLevel,__proto__:null})+"\n",1)},this.debug=(...t)=>{e(this,n,"f").call(this,O(t,{...u(),indentLevel:this.indentLevel,__proto__:null})+"\n",0)},this.info=(...t)=>{e(this,n,"f").call(this,O(t,{...u(),indentLevel:this.indentLevel,__proto__:null})+"\n",1)},this.dir=(t=void 0,r={__proto__:null})=>{e(this,n,"f").call(this,O([t],{...u(),...r})+"\n",1)},this.dirxml=this.dir,this.warn=(...t)=>{e(this,n,"f").call(this,O(t,{...u(),indentLevel:this.indentLevel,__proto__:null})+"\n",2)},this.error=(...t)=>{e(this,n,"f").call(this,O(t,{...u(),indentLevel:this.indentLevel,__proto__:null})+"\n",3)},this.assert=(t=!1,...e)=>{if(t)return;if(0===e.length)return void this.error("Assertion failed");const[n,...r]=e;"string"!=typeof n?this.error("Assertion failed:",...e):this.error(`Assertion failed: ${n}`,...r)},this.count=(t="default")=>{if(t=String(t),st.has(t)){const e=st.get(t)||0;st.set(t,e+1)}else st.set(t,1);this.info(`${t}: ${st.get(t)}`)},this.countReset=(t="default")=>{t=String(t),st.has(t)?st.set(t,0):this.warn(`Count for '${t}' does not exist`)},this.table=(t=void 0,e)=>{if(void 0!==e&&!Array.isArray(e))throw new Error("The 'properties' argument must be of type Array. Received type "+typeof e);if(null===t||"object"!=typeof t)return this.log(t);const n=t=>function(t,e){const n=void 0===e.strAbbreviateSize?1e4:e.strAbbreviateSize;if("string"==typeof t){const r=t.length>n?t.slice(0,n)+"...":t;return e.stylize(P(r,e),"string")}return L(e,t,0)}(t,{...u(),depth:1,compact:!0});let r;const l=i(t),s=o(t),a=l||s?"(iter idx)":"(idx)";if(l)r=[...t];else if(s){let e=0;r={__proto__:null},t.forEach(((t,n)=>{r[e]={Key:n,Values:t},e++}))}else r=t;const c=Object.keys(r),f=c.length,p=e?Object.fromEntries(e.map((t=>[t,new Array(f).fill("")]))):{},h=[],d=[];let g=!1;c.forEach(((t,o)=>{const i=r[t],l=null===i||"function"!=typeof i&&"object"!=typeof i;if(void 0===e&&l)g=!0,d.push(n(i));else{const t=i||{},r=e||Object.keys(t);for(let e=0;e<r.length;++e){const i=r[e];!l&&Reflect.has(t,i)&&(Reflect.has(p,i)||(p[i]=new Array(f).fill("")),p[i][o]=n(t[i]))}d.push("")}h.push(t)}));const y=Object.keys(p),$=Object.values(p);((t,e)=>{this.log(function(t,e){const n=[],r=t.map((t=>w(t))),o=e.reduce(((t,e)=>Math.max(t,e.length)),0),i=new Array(r.length).fill(!0);for(let a=0;a<t.length;a++){const t=e[a];for(let e=0;e<o;e++){void 0===n[e]&&(n[e]=[]);const o=n[e][a]=(s=e,null!=(l=t)&&Object.hasOwn(l,s)?t[e]:""),c=r[a]||0,f=w(o);r[a]=Math.max(c,f),i[a]=i[a]&&Number.isInteger(+o)}}var l,s;const a=r.map((t=>it.middleMiddle.repeat(t+2)));let c=`\n${it.topLeft}${a.join(it.topMiddle)}${it.topRight}\n${lt(t,r)}\n${it.leftMiddle}${a.join(it.rowMiddle)}${it.rightMiddle}\n`;for(let t=0;t<n.length;++t)c+=`${lt(n[t],r,i)}\n`;return c+=`${it.bottomLeft}${a.join(it.bottomMiddle)}`+it.bottomRight,c}(t,e))})([a,...e||[...y,!s&&g&&"Values"]].filter(Boolean),[h,...$,d])},this.time=(t="default")=>{t=String(t),at.has(t)?this.warn(`Timer '${t}' already exists`):at.set(t,Date.now())},this.timeLog=(t="default",...e)=>{if(t=String(t),!at.has(t))return void this.warn(`Timer '${t}' does not exist`);const n=at.get(t),r=Date.now()-n;this.info(`${t}: ${r}ms`,...e)},this.timeEnd=(t="default")=>{if(t=String(t),!at.has(t))return void this.warn(`Timer '${t}' does not exist`);const e=at.get(t);at.delete(t);const n=Date.now()-e;this.info(`${t}: ${n}ms`)},this.group=(...t)=>{t.length>0&&this.log(...t),this.indentLevel+=2},this.groupCollapsed=this.group,this.groupEnd=()=>{this.indentLevel>0&&(this.indentLevel-=2)},this.clear=()=>{this.indentLevel=0,e(this,n,"f").call(this,"[1;1H",1),e(this,n,"f").call(this,"[0J",1)},this.trace=(...t)=>{const e={name:"Trace",message:O(t,{...u(),indentLevel:0,__proto__:null})};try{Error.prototype.captureStackTrace.call(e,this.trace)}catch(e){}this.error(e.stack)},function(t,e,n,r,o){if("m"===r)throw new TypeError("Private method is not writable");if("a"===r&&!o)throw new TypeError("Private accessor was defined without a setter");if("function"==typeof e?t!==e||!o:!e.has(t))throw new TypeError("Cannot write private member to an object whose class did not declare it");"a"===r?o.call(t,n):o?o.value=n:e.set(t,n)}(this,n,t,"f"),this[ct]=!0,this.indentLevel=0;const l=Object.create({},{[Symbol.toStringTag]:{enumerable:!1,writable:!1,configurable:!0,value:"console"}});return Object.assign(l,this),l}}(((t,e)=>{let n;switch(e){case 0:n=ft.Debug;break;case 1:default:n=ft.Info;break;case 2:n=ft.Warn;break;case 3:n=ft.Error}ut(n,t)})),enumerable:!1,configurable:!0,writable:!0}),t.debug=async function(t,e){await ut(ft.Debug,t,e)},t.error=async function(t,e){await ut(ft.Error,t,e)},t.info=async function(t,e){await ut(ft.Info,t,e)},t.trace=async function(t,e){await ut(ft.Trace,t,e)},t.warn=async function(t,e){await ut(ft.Warn,t,e)},t}({});Object.defineProperty(window.__TAURI__,"log",{value:__TAURI_PLUGIN_LOG__})}
