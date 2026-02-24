(function(){const t=document.createElement("link").relList;if(t&&t.supports&&t.supports("modulepreload"))return;for(const i of document.querySelectorAll('link[rel="modulepreload"]'))s(i);new MutationObserver(i=>{for(const a of i)if(a.type==="childList")for(const o of a.addedNodes)o.tagName==="LINK"&&o.rel==="modulepreload"&&s(o)}).observe(document,{childList:!0,subtree:!0});function n(i){const a={};return i.integrity&&(a.integrity=i.integrity),i.referrerPolicy&&(a.referrerPolicy=i.referrerPolicy),i.crossOrigin==="use-credentials"?a.credentials="include":i.crossOrigin==="anonymous"?a.credentials="omit":a.credentials="same-origin",a}function s(i){if(i.ep)return;i.ep=!0;const a=n(i);fetch(i.href,a)}})();const Un=globalThis,Li=Un.ShadowRoot&&(Un.ShadyCSS===void 0||Un.ShadyCSS.nativeShadow)&&"adoptedStyleSheets"in Document.prototype&&"replace"in CSSStyleSheet.prototype,Mi=Symbol(),Ka=new WeakMap;let Ar=class{constructor(t,n,s){if(this._$cssResult$=!0,s!==Mi)throw Error("CSSResult is not constructable. Use `unsafeCSS` or `css` instead.");this.cssText=t,this.t=n}get styleSheet(){let t=this.o;const n=this.t;if(Li&&t===void 0){const s=n!==void 0&&n.length===1;s&&(t=Ka.get(n)),t===void 0&&((this.o=t=new CSSStyleSheet).replaceSync(this.cssText),s&&Ka.set(n,t))}return t}toString(){return this.cssText}};const ep=e=>new Ar(typeof e=="string"?e:e+"",void 0,Mi),tp=(e,...t)=>{const n=e.length===1?e[0]:t.reduce((s,i,a)=>s+(o=>{if(o._$cssResult$===!0)return o.cssText;if(typeof o=="number")return o;throw Error("Value passed to 'css' function must be a 'css' function result: "+o+". Use 'unsafeCSS' to pass non-literal values, but take care to ensure page security.")})(i)+e[a+1],e[0]);return new Ar(n,e,Mi)},np=(e,t)=>{if(Li)e.adoptedStyleSheets=t.map(n=>n instanceof CSSStyleSheet?n:n.styleSheet);else for(const n of t){const s=document.createElement("style"),i=Un.litNonce;i!==void 0&&s.setAttribute("nonce",i),s.textContent=n.cssText,e.appendChild(s)}},ja=Li?e=>e:e=>e instanceof CSSStyleSheet?(t=>{let n="";for(const s of t.cssRules)n+=s.cssText;return ep(n)})(e):e;const{is:sp,defineProperty:ip,getOwnPropertyDescriptor:ap,getOwnPropertyNames:op,getOwnPropertySymbols:rp,getPrototypeOf:lp}=Object,is=globalThis,Wa=is.trustedTypes,cp=Wa?Wa.emptyScript:"",dp=is.reactiveElementPolyfillSupport,en=(e,t)=>e,qn={toAttribute(e,t){switch(t){case Boolean:e=e?cp:null;break;case Object:case Array:e=e==null?e:JSON.stringify(e)}return e},fromAttribute(e,t){let n=e;switch(t){case Boolean:n=e!==null;break;case Number:n=e===null?null:Number(e);break;case Object:case Array:try{n=JSON.parse(e)}catch{n=null}}return n}},Ii=(e,t)=>!sp(e,t),qa={attribute:!0,type:String,converter:qn,reflect:!1,useDefault:!1,hasChanged:Ii};Symbol.metadata??=Symbol("metadata"),is.litPropertyMetadata??=new WeakMap;let Pt=class extends HTMLElement{static addInitializer(t){this._$Ei(),(this.l??=[]).push(t)}static get observedAttributes(){return this.finalize(),this._$Eh&&[...this._$Eh.keys()]}static createProperty(t,n=qa){if(n.state&&(n.attribute=!1),this._$Ei(),this.prototype.hasOwnProperty(t)&&((n=Object.create(n)).wrapped=!0),this.elementProperties.set(t,n),!n.noAccessor){const s=Symbol(),i=this.getPropertyDescriptor(t,s,n);i!==void 0&&ip(this.prototype,t,i)}}static getPropertyDescriptor(t,n,s){const{get:i,set:a}=ap(this.prototype,t)??{get(){return this[n]},set(o){this[n]=o}};return{get:i,set(o){const l=i?.call(this);a?.call(this,o),this.requestUpdate(t,l,s)},configurable:!0,enumerable:!0}}static getPropertyOptions(t){return this.elementProperties.get(t)??qa}static _$Ei(){if(this.hasOwnProperty(en("elementProperties")))return;const t=lp(this);t.finalize(),t.l!==void 0&&(this.l=[...t.l]),this.elementProperties=new Map(t.elementProperties)}static finalize(){if(this.hasOwnProperty(en("finalized")))return;if(this.finalized=!0,this._$Ei(),this.hasOwnProperty(en("properties"))){const n=this.properties,s=[...op(n),...rp(n)];for(const i of s)this.createProperty(i,n[i])}const t=this[Symbol.metadata];if(t!==null){const n=litPropertyMetadata.get(t);if(n!==void 0)for(const[s,i]of n)this.elementProperties.set(s,i)}this._$Eh=new Map;for(const[n,s]of this.elementProperties){const i=this._$Eu(n,s);i!==void 0&&this._$Eh.set(i,n)}this.elementStyles=this.finalizeStyles(this.styles)}static finalizeStyles(t){const n=[];if(Array.isArray(t)){const s=new Set(t.flat(1/0).reverse());for(const i of s)n.unshift(ja(i))}else t!==void 0&&n.push(ja(t));return n}static _$Eu(t,n){const s=n.attribute;return s===!1?void 0:typeof s=="string"?s:typeof t=="string"?t.toLowerCase():void 0}constructor(){super(),this._$Ep=void 0,this.isUpdatePending=!1,this.hasUpdated=!1,this._$Em=null,this._$Ev()}_$Ev(){this._$ES=new Promise(t=>this.enableUpdating=t),this._$AL=new Map,this._$E_(),this.requestUpdate(),this.constructor.l?.forEach(t=>t(this))}addController(t){(this._$EO??=new Set).add(t),this.renderRoot!==void 0&&this.isConnected&&t.hostConnected?.()}removeController(t){this._$EO?.delete(t)}_$E_(){const t=new Map,n=this.constructor.elementProperties;for(const s of n.keys())this.hasOwnProperty(s)&&(t.set(s,this[s]),delete this[s]);t.size>0&&(this._$Ep=t)}createRenderRoot(){const t=this.shadowRoot??this.attachShadow(this.constructor.shadowRootOptions);return np(t,this.constructor.elementStyles),t}connectedCallback(){this.renderRoot??=this.createRenderRoot(),this.enableUpdating(!0),this._$EO?.forEach(t=>t.hostConnected?.())}enableUpdating(t){}disconnectedCallback(){this._$EO?.forEach(t=>t.hostDisconnected?.())}attributeChangedCallback(t,n,s){this._$AK(t,s)}_$ET(t,n){const s=this.constructor.elementProperties.get(t),i=this.constructor._$Eu(t,s);if(i!==void 0&&s.reflect===!0){const a=(s.converter?.toAttribute!==void 0?s.converter:qn).toAttribute(n,s.type);this._$Em=t,a==null?this.removeAttribute(i):this.setAttribute(i,a),this._$Em=null}}_$AK(t,n){const s=this.constructor,i=s._$Eh.get(t);if(i!==void 0&&this._$Em!==i){const a=s.getPropertyOptions(i),o=typeof a.converter=="function"?{fromAttribute:a.converter}:a.converter?.fromAttribute!==void 0?a.converter:qn;this._$Em=i;const l=o.fromAttribute(n,a.type);this[i]=l??this._$Ej?.get(i)??l,this._$Em=null}}requestUpdate(t,n,s,i=!1,a){if(t!==void 0){const o=this.constructor;if(i===!1&&(a=this[t]),s??=o.getPropertyOptions(t),!((s.hasChanged??Ii)(a,n)||s.useDefault&&s.reflect&&a===this._$Ej?.get(t)&&!this.hasAttribute(o._$Eu(t,s))))return;this.C(t,n,s)}this.isUpdatePending===!1&&(this._$ES=this._$EP())}C(t,n,{useDefault:s,reflect:i,wrapped:a},o){s&&!(this._$Ej??=new Map).has(t)&&(this._$Ej.set(t,o??n??this[t]),a!==!0||o!==void 0)||(this._$AL.has(t)||(this.hasUpdated||s||(n=void 0),this._$AL.set(t,n)),i===!0&&this._$Em!==t&&(this._$Eq??=new Set).add(t))}async _$EP(){this.isUpdatePending=!0;try{await this._$ES}catch(n){Promise.reject(n)}const t=this.scheduleUpdate();return t!=null&&await t,!this.isUpdatePending}scheduleUpdate(){return this.performUpdate()}performUpdate(){if(!this.isUpdatePending)return;if(!this.hasUpdated){if(this.renderRoot??=this.createRenderRoot(),this._$Ep){for(const[i,a]of this._$Ep)this[i]=a;this._$Ep=void 0}const s=this.constructor.elementProperties;if(s.size>0)for(const[i,a]of s){const{wrapped:o}=a,l=this[i];o!==!0||this._$AL.has(i)||l===void 0||this.C(i,void 0,a,l)}}let t=!1;const n=this._$AL;try{t=this.shouldUpdate(n),t?(this.willUpdate(n),this._$EO?.forEach(s=>s.hostUpdate?.()),this.update(n)):this._$EM()}catch(s){throw t=!1,this._$EM(),s}t&&this._$AE(n)}willUpdate(t){}_$AE(t){this._$EO?.forEach(n=>n.hostUpdated?.()),this.hasUpdated||(this.hasUpdated=!0,this.firstUpdated(t)),this.updated(t)}_$EM(){this._$AL=new Map,this.isUpdatePending=!1}get updateComplete(){return this.getUpdateComplete()}getUpdateComplete(){return this._$ES}shouldUpdate(t){return!0}update(t){this._$Eq&&=this._$Eq.forEach(n=>this._$ET(n,this[n])),this._$EM()}updated(t){}firstUpdated(t){}};Pt.elementStyles=[],Pt.shadowRootOptions={mode:"open"},Pt[en("elementProperties")]=new Map,Pt[en("finalized")]=new Map,dp?.({ReactiveElement:Pt}),(is.reactiveElementVersions??=[]).push("2.1.2");const Ri=globalThis,Ga=e=>e,Gn=Ri.trustedTypes,Va=Gn?Gn.createPolicy("lit-html",{createHTML:e=>e}):void 0,_r="$lit$",Ye=`lit$${Math.random().toFixed(9).slice(2)}$`,Cr="?"+Ye,up=`<${Cr}>`,yt=document,on=()=>yt.createComment(""),rn=e=>e===null||typeof e!="object"&&typeof e!="function",Pi=Array.isArray,gp=e=>Pi(e)||typeof e?.[Symbol.iterator]=="function",Ls=`[ 	
\f\r]`,zt=/<(?:(!--|\/[^a-zA-Z])|(\/?[a-zA-Z][^>\s]*)|(\/?$))/g,Qa=/-->/g,Ya=/>/g,rt=RegExp(`>|${Ls}(?:([^\\s"'>=/]+)(${Ls}*=${Ls}*(?:[^ 	
\f\r"'\`<>=]|("|')|))|$)`,"g"),Ja=/'/g,Za=/"/g,Tr=/^(?:script|style|textarea|title)$/i,Er=e=>(t,...n)=>({_$litType$:e,strings:t,values:n}),r=Er(1),Tn=Er(2),et=Symbol.for("lit-noChange"),m=Symbol.for("lit-nothing"),Xa=new WeakMap,mt=yt.createTreeWalker(yt,129);function Lr(e,t){if(!Pi(e)||!e.hasOwnProperty("raw"))throw Error("invalid template strings array");return Va!==void 0?Va.createHTML(t):t}const pp=(e,t)=>{const n=e.length-1,s=[];let i,a=t===2?"<svg>":t===3?"<math>":"",o=zt;for(let l=0;l<n;l++){const d=e[l];let g,f,p=-1,b=0;for(;b<d.length&&(o.lastIndex=b,f=o.exec(d),f!==null);)b=o.lastIndex,o===zt?f[1]==="!--"?o=Qa:f[1]!==void 0?o=Ya:f[2]!==void 0?(Tr.test(f[2])&&(i=RegExp("</"+f[2],"g")),o=rt):f[3]!==void 0&&(o=rt):o===rt?f[0]===">"?(o=i??zt,p=-1):f[1]===void 0?p=-2:(p=o.lastIndex-f[2].length,g=f[1],o=f[3]===void 0?rt:f[3]==='"'?Za:Ja):o===Za||o===Ja?o=rt:o===Qa||o===Ya?o=zt:(o=rt,i=void 0);const u=o===rt&&e[l+1].startsWith("/>")?" ":"";a+=o===zt?d+up:p>=0?(s.push(g),d.slice(0,p)+_r+d.slice(p)+Ye+u):d+Ye+(p===-2?l:u)}return[Lr(e,a+(e[n]||"<?>")+(t===2?"</svg>":t===3?"</math>":"")),s]};class ln{constructor({strings:t,_$litType$:n},s){let i;this.parts=[];let a=0,o=0;const l=t.length-1,d=this.parts,[g,f]=pp(t,n);if(this.el=ln.createElement(g,s),mt.currentNode=this.el.content,n===2||n===3){const p=this.el.content.firstChild;p.replaceWith(...p.childNodes)}for(;(i=mt.nextNode())!==null&&d.length<l;){if(i.nodeType===1){if(i.hasAttributes())for(const p of i.getAttributeNames())if(p.endsWith(_r)){const b=f[o++],u=i.getAttribute(p).split(Ye),v=/([.?@])?(.*)/.exec(b);d.push({type:1,index:a,name:v[2],strings:u,ctor:v[1]==="."?fp:v[1]==="?"?vp:v[1]==="@"?mp:os}),i.removeAttribute(p)}else p.startsWith(Ye)&&(d.push({type:6,index:a}),i.removeAttribute(p));if(Tr.test(i.tagName)){const p=i.textContent.split(Ye),b=p.length-1;if(b>0){i.textContent=Gn?Gn.emptyScript:"";for(let u=0;u<b;u++)i.append(p[u],on()),mt.nextNode(),d.push({type:2,index:++a});i.append(p[b],on())}}}else if(i.nodeType===8)if(i.data===Cr)d.push({type:2,index:a});else{let p=-1;for(;(p=i.data.indexOf(Ye,p+1))!==-1;)d.push({type:7,index:a}),p+=Ye.length-1}a++}}static createElement(t,n){const s=yt.createElement("template");return s.innerHTML=t,s}}function Nt(e,t,n=e,s){if(t===et)return t;let i=s!==void 0?n._$Co?.[s]:n._$Cl;const a=rn(t)?void 0:t._$litDirective$;return i?.constructor!==a&&(i?._$AO?.(!1),a===void 0?i=void 0:(i=new a(e),i._$AT(e,n,s)),s!==void 0?(n._$Co??=[])[s]=i:n._$Cl=i),i!==void 0&&(t=Nt(e,i._$AS(e,t.values),i,s)),t}class hp{constructor(t,n){this._$AV=[],this._$AN=void 0,this._$AD=t,this._$AM=n}get parentNode(){return this._$AM.parentNode}get _$AU(){return this._$AM._$AU}u(t){const{el:{content:n},parts:s}=this._$AD,i=(t?.creationScope??yt).importNode(n,!0);mt.currentNode=i;let a=mt.nextNode(),o=0,l=0,d=s[0];for(;d!==void 0;){if(o===d.index){let g;d.type===2?g=new as(a,a.nextSibling,this,t):d.type===1?g=new d.ctor(a,d.name,d.strings,this,t):d.type===6&&(g=new bp(a,this,t)),this._$AV.push(g),d=s[++l]}o!==d?.index&&(a=mt.nextNode(),o++)}return mt.currentNode=yt,i}p(t){let n=0;for(const s of this._$AV)s!==void 0&&(s.strings!==void 0?(s._$AI(t,s,n),n+=s.strings.length-2):s._$AI(t[n])),n++}}let as=class Mr{get _$AU(){return this._$AM?._$AU??this._$Cv}constructor(t,n,s,i){this.type=2,this._$AH=m,this._$AN=void 0,this._$AA=t,this._$AB=n,this._$AM=s,this.options=i,this._$Cv=i?.isConnected??!0}get parentNode(){let t=this._$AA.parentNode;const n=this._$AM;return n!==void 0&&t?.nodeType===11&&(t=n.parentNode),t}get startNode(){return this._$AA}get endNode(){return this._$AB}_$AI(t,n=this){t=Nt(this,t,n),rn(t)?t===m||t==null||t===""?(this._$AH!==m&&this._$AR(),this._$AH=m):t!==this._$AH&&t!==et&&this._(t):t._$litType$!==void 0?this.$(t):t.nodeType!==void 0?this.T(t):gp(t)?this.k(t):this._(t)}O(t){return this._$AA.parentNode.insertBefore(t,this._$AB)}T(t){this._$AH!==t&&(this._$AR(),this._$AH=this.O(t))}_(t){this._$AH!==m&&rn(this._$AH)?this._$AA.nextSibling.data=t:this.T(yt.createTextNode(t)),this._$AH=t}$(t){const{values:n,_$litType$:s}=t,i=typeof s=="number"?this._$AC(t):(s.el===void 0&&(s.el=ln.createElement(Lr(s.h,s.h[0]),this.options)),s);if(this._$AH?._$AD===i)this._$AH.p(n);else{const a=new hp(i,this),o=a.u(this.options);a.p(n),this.T(o),this._$AH=a}}_$AC(t){let n=Xa.get(t.strings);return n===void 0&&Xa.set(t.strings,n=new ln(t)),n}k(t){Pi(this._$AH)||(this._$AH=[],this._$AR());const n=this._$AH;let s,i=0;for(const a of t)i===n.length?n.push(s=new Mr(this.O(on()),this.O(on()),this,this.options)):s=n[i],s._$AI(a),i++;i<n.length&&(this._$AR(s&&s._$AB.nextSibling,i),n.length=i)}_$AR(t=this._$AA.nextSibling,n){for(this._$AP?.(!1,!0,n);t!==this._$AB;){const s=Ga(t).nextSibling;Ga(t).remove(),t=s}}setConnected(t){this._$AM===void 0&&(this._$Cv=t,this._$AP?.(t))}},os=class{get tagName(){return this.element.tagName}get _$AU(){return this._$AM._$AU}constructor(t,n,s,i,a){this.type=1,this._$AH=m,this._$AN=void 0,this.element=t,this.name=n,this._$AM=i,this.options=a,s.length>2||s[0]!==""||s[1]!==""?(this._$AH=Array(s.length-1).fill(new String),this.strings=s):this._$AH=m}_$AI(t,n=this,s,i){const a=this.strings;let o=!1;if(a===void 0)t=Nt(this,t,n,0),o=!rn(t)||t!==this._$AH&&t!==et,o&&(this._$AH=t);else{const l=t;let d,g;for(t=a[0],d=0;d<a.length-1;d++)g=Nt(this,l[s+d],n,d),g===et&&(g=this._$AH[d]),o||=!rn(g)||g!==this._$AH[d],g===m?t=m:t!==m&&(t+=(g??"")+a[d+1]),this._$AH[d]=g}o&&!i&&this.j(t)}j(t){t===m?this.element.removeAttribute(this.name):this.element.setAttribute(this.name,t??"")}},fp=class extends os{constructor(){super(...arguments),this.type=3}j(t){this.element[this.name]=t===m?void 0:t}},vp=class extends os{constructor(){super(...arguments),this.type=4}j(t){this.element.toggleAttribute(this.name,!!t&&t!==m)}},mp=class extends os{constructor(t,n,s,i,a){super(t,n,s,i,a),this.type=5}_$AI(t,n=this){if((t=Nt(this,t,n,0)??m)===et)return;const s=this._$AH,i=t===m&&s!==m||t.capture!==s.capture||t.once!==s.once||t.passive!==s.passive,a=t!==m&&(s===m||i);i&&this.element.removeEventListener(this.name,this,s),a&&this.element.addEventListener(this.name,this,t),this._$AH=t}handleEvent(t){typeof this._$AH=="function"?this._$AH.call(this.options?.host??this.element,t):this._$AH.handleEvent(t)}},bp=class{constructor(t,n,s){this.element=t,this.type=6,this._$AN=void 0,this._$AM=n,this.options=s}get _$AU(){return this._$AM._$AU}_$AI(t){Nt(this,t)}};const yp={I:as},xp=Ri.litHtmlPolyfillSupport;xp?.(ln,as),(Ri.litHtmlVersions??=[]).push("3.3.2");const $p=(e,t,n)=>{const s=n?.renderBefore??t;let i=s._$litPart$;if(i===void 0){const a=n?.renderBefore??null;s._$litPart$=i=new as(t.insertBefore(on(),a),a,void 0,n??{})}return i._$AI(e),i};const Di=globalThis;let Ft=class extends Pt{constructor(){super(...arguments),this.renderOptions={host:this},this._$Do=void 0}createRenderRoot(){const t=super.createRenderRoot();return this.renderOptions.renderBefore??=t.firstChild,t}update(t){const n=this.render();this.hasUpdated||(this.renderOptions.isConnected=this.isConnected),super.update(t),this._$Do=$p(n,this.renderRoot,this.renderOptions)}connectedCallback(){super.connectedCallback(),this._$Do?.setConnected(!0)}disconnectedCallback(){super.disconnectedCallback(),this._$Do?.setConnected(!1)}render(){return et}};Ft._$litElement$=!0,Ft.finalized=!0,Di.litElementHydrateSupport?.({LitElement:Ft});const wp=Di.litElementPolyfillSupport;wp?.({LitElement:Ft});(Di.litElementVersions??=[]).push("4.2.2");const Ir=e=>(t,n)=>{n!==void 0?n.addInitializer(()=>{customElements.define(e,t)}):customElements.define(e,t)};const kp={attribute:!0,type:String,converter:qn,reflect:!1,hasChanged:Ii},Sp=(e=kp,t,n)=>{const{kind:s,metadata:i}=n;let a=globalThis.litPropertyMetadata.get(i);if(a===void 0&&globalThis.litPropertyMetadata.set(i,a=new Map),s==="setter"&&((e=Object.create(e)).wrapped=!0),a.set(n.name,e),s==="accessor"){const{name:o}=n;return{set(l){const d=t.get.call(this);t.set.call(this,l),this.requestUpdate(o,d,e,!0,l)},init(l){return l!==void 0&&this.C(o,void 0,e,l),l}}}if(s==="setter"){const{name:o}=n;return function(l){const d=this[o];t.call(this,l),this.requestUpdate(o,d,e,!0,l)}}throw Error("Unsupported decorator location: "+s)};function zn(e){return(t,n)=>typeof n=="object"?Sp(e,t,n):((s,i,a)=>{const o=i.hasOwnProperty(a);return i.constructor.createProperty(a,s),o?Object.getOwnPropertyDescriptor(i,a):void 0})(e,t,n)}function A(e){return zn({...e,state:!0,attribute:!1})}async function xe(e,t){if(!(!e.client||!e.connected)&&!e.channelsLoading){e.channelsLoading=!0,e.channelsError=null;try{const n=await e.client.request("channels.status",{probe:t,timeoutMs:8e3});e.channelsSnapshot=n,e.channelsLastSuccess=Date.now()}catch(n){e.channelsError=String(n)}finally{e.channelsLoading=!1}}}async function Ap(e,t){if(!(!e.client||!e.connected||e.whatsappBusy)){e.whatsappBusy=!0;try{const n=await e.client.request("web.login.start",{force:t,timeoutMs:3e4});e.whatsappLoginMessage=n.message??null,e.whatsappLoginQrDataUrl=n.qrDataUrl??null,e.whatsappLoginConnected=null}catch(n){e.whatsappLoginMessage=String(n),e.whatsappLoginQrDataUrl=null,e.whatsappLoginConnected=null}finally{e.whatsappBusy=!1}}}async function _p(e){if(!(!e.client||!e.connected||e.whatsappBusy)){e.whatsappBusy=!0;try{const t=await e.client.request("web.login.wait",{timeoutMs:12e4});e.whatsappLoginMessage=t.message??null,e.whatsappLoginConnected=t.connected??null,t.connected&&(e.whatsappLoginQrDataUrl=null)}catch(t){e.whatsappLoginMessage=String(t),e.whatsappLoginConnected=null}finally{e.whatsappBusy=!1}}}async function Cp(e){if(!(!e.client||!e.connected||e.whatsappBusy)){e.whatsappBusy=!0;try{await e.client.request("channels.logout",{channel:"whatsapp"}),e.whatsappLoginMessage="Logged out.",e.whatsappLoginQrDataUrl=null,e.whatsappLoginConnected=null}catch(t){e.whatsappLoginMessage=String(t)}finally{e.whatsappBusy=!1}}}function ke(e){if(e)return Array.isArray(e.type)?e.type.filter(n=>n!=="null")[0]??e.type[0]:e.type}function Rr(e){if(!e)return"";if(e.default!==void 0)return e.default;switch(ke(e)){case"object":return{};case"array":return[];case"boolean":return!1;case"number":case"integer":return 0;case"string":return"";default:return""}}function Fi(e){return e.filter(t=>typeof t=="string").join(".")}function Ce(e,t){const n=Fi(e),s=t[n];if(s)return s;const i=n.split(".");for(const[a,o]of Object.entries(t)){if(!a.includes("*"))continue;const l=a.split(".");if(l.length!==i.length)continue;let d=!0;for(let g=0;g<i.length;g+=1)if(l[g]!=="*"&&l[g]!==i[g]){d=!1;break}if(d)return o}}function Ge(e){return e.replace(/_/g," ").replace(/([a-z0-9])([A-Z])/g,"$1 $2").replace(/\s+/g," ").replace(/^./,t=>t.toUpperCase())}function eo(e,t){const n=e.trim();if(n==="")return;const s=Number(n);return!Number.isFinite(s)||t&&!Number.isInteger(s)?e:s}function to(e){const t=e.trim();return t==="true"?!0:t==="false"?!1:e}function Qe(e,t){if(e==null)return e;if(t.allOf&&t.allOf.length>0){let s=e;for(const i of t.allOf)s=Qe(s,i);return s}const n=ke(t);if(t.anyOf||t.oneOf){const s=(t.anyOf??t.oneOf??[]).filter(i=>!(i.type==="null"||Array.isArray(i.type)&&i.type.includes("null")));if(s.length===1)return Qe(e,s[0]);if(typeof e=="string")for(const i of s){const a=ke(i);if(a==="number"||a==="integer"){const o=eo(e,a==="integer");if(o===void 0||typeof o=="number")return o}if(a==="boolean"){const o=to(e);if(typeof o=="boolean")return o}}for(const i of s){const a=ke(i);if(a==="object"&&typeof e=="object"&&!Array.isArray(e)||a==="array"&&Array.isArray(e))return Qe(e,i)}return e}if(n==="number"||n==="integer"){if(typeof e=="string"){const s=eo(e,n==="integer");if(s===void 0||typeof s=="number")return s}return e}if(n==="boolean"){if(typeof e=="string"){const s=to(e);if(typeof s=="boolean")return s}return e}if(n==="object"){if(typeof e!="object"||Array.isArray(e))return e;const s=e,i=t.properties??{},a=t.additionalProperties&&typeof t.additionalProperties=="object"?t.additionalProperties:null,o={};for(const[l,d]of Object.entries(s)){const g=i[l]??a,f=g?Qe(d,g):d;f!==void 0&&(o[l]=f)}return o}if(n==="array"){if(!Array.isArray(e))return e;if(Array.isArray(t.items)){const i=t.items;return e.map((a,o)=>{const l=o<i.length?i[o]:void 0;return l?Qe(a,l):a})}const s=t.items;return s?e.map(i=>Qe(i,s)).filter(i=>i!==void 0):e}return e}function xt(e){return typeof structuredClone=="function"?structuredClone(e):JSON.parse(JSON.stringify(e))}function cn(e){return`${JSON.stringify(e,null,2).trimEnd()}
`}function Pr(e,t,n){if(t.length===0)return;let s=e;for(let a=0;a<t.length-1;a+=1){const o=t[a],l=t[a+1];if(typeof o=="number"){if(!Array.isArray(s))return;s[o]==null&&(s[o]=typeof l=="number"?[]:{}),s=s[o]}else{if(typeof s!="object"||s==null)return;const d=s;d[o]==null&&(d[o]=typeof l=="number"?[]:{}),s=d[o]}}const i=t[t.length-1];if(typeof i=="number"){Array.isArray(s)&&(s[i]=n);return}typeof s=="object"&&s!=null&&(s[i]=n)}function Dr(e,t){if(t.length===0)return;let n=e;for(let i=0;i<t.length-1;i+=1){const a=t[i];if(typeof a=="number"){if(!Array.isArray(n))return;n=n[a]}else{if(typeof n!="object"||n==null)return;n=n[a]}if(n==null)return}const s=t[t.length-1];if(typeof s=="number"){Array.isArray(n)&&n.splice(s,1);return}typeof n=="object"&&n!=null&&delete n[s]}async function Ie(e){if(!(!e.client||!e.connected)){e.configLoading=!0,e.lastError=null;try{const t=await e.client.request("config.get",{});Ep(e,t)}catch(t){e.lastError=String(t)}finally{e.configLoading=!1}}}async function Fr(e){if(!(!e.client||!e.connected)&&!e.configSchemaLoading){e.configSchemaLoading=!0;try{const t=await e.client.request("config.schema",{});Tp(e,t)}catch(t){e.lastError=String(t)}finally{e.configSchemaLoading=!1}}}function Tp(e,t){e.configSchema=t.schema??null,e.configUiHints=t.uiHints??{},e.configSchemaVersion=t.version??null}function Ep(e,t){e.configSnapshot=t;const n=typeof t.raw=="string"?t.raw:t.config&&typeof t.config=="object"?cn(t.config):e.configRaw;!e.configFormDirty||e.configFormMode==="raw"?e.configRaw=n:e.configForm?e.configRaw=cn(e.configForm):e.configRaw=n,e.configValid=typeof t.valid=="boolean"?t.valid:null,e.configIssues=Array.isArray(t.issues)?t.issues:[],e.configFormDirty||(e.configForm=xt(t.config??{}),e.configFormOriginal=xt(t.config??{}),e.configRawOriginal=n)}function Lp(e){return!e||typeof e!="object"||Array.isArray(e)?null:e}function Nr(e){if(e.configFormMode!=="form"||!e.configForm)return e.configRaw;const t=Lp(e.configSchema),n=t?Qe(e.configForm,t):e.configForm;return cn(n)}async function Hn(e){if(!(!e.client||!e.connected)){e.configSaving=!0,e.lastError=null;try{const t=Nr(e),n=e.configSnapshot?.hash;if(!n){e.lastError="Config hash missing; reload and retry.";return}await e.client.request("config.set",{raw:t,baseHash:n}),e.configFormDirty=!1,await Ie(e)}catch(t){e.lastError=String(t)}finally{e.configSaving=!1}}}async function Mp(e){if(!(!e.client||!e.connected)){e.configApplying=!0,e.lastError=null;try{const t=Nr(e),n=e.configSnapshot?.hash;if(!n){e.lastError="Config hash missing; reload and retry.";return}await e.client.request("config.apply",{raw:t,baseHash:n,sessionKey:e.applySessionKey}),e.configFormDirty=!1,await Ie(e)}catch(t){e.lastError=String(t)}finally{e.configApplying=!1}}}async function Ip(e){if(!(!e.client||!e.connected)){e.updateRunning=!0,e.lastError=null;try{await e.client.request("update.run",{sessionKey:e.applySessionKey})}catch(t){e.lastError=String(t)}finally{e.updateRunning=!1}}}function we(e,t,n){const s=xt(e.configForm??e.configSnapshot?.config??{});Pr(s,t,n),e.configForm=s,e.configFormDirty=!0,e.configFormMode==="form"&&(e.configRaw=cn(s))}function je(e,t){const n=xt(e.configForm??e.configSnapshot?.config??{});Dr(n,t),e.configForm=n,e.configFormDirty=!0,e.configFormMode==="form"&&(e.configRaw=cn(n))}function Rp(e){const{values:t,original:n}=e;return t.name!==n.name||t.displayName!==n.displayName||t.about!==n.about||t.picture!==n.picture||t.banner!==n.banner||t.website!==n.website||t.nip05!==n.nip05||t.lud16!==n.lud16}function Pp(e){const{state:t,callbacks:n,accountId:s}=e,i=Rp(t),a=(l,d,g={})=>{const{type:f="text",placeholder:p,maxLength:b,help:u}=g,v=t.values[l]??"",y=t.fieldErrors[l],k=`nostr-profile-${l}`;return f==="textarea"?r`
        <div class="form-field" style="margin-bottom: 12px;">
          <label for="${k}" style="display: block; margin-bottom: 4px; font-weight: 500;">
            ${d}
          </label>
          <textarea
            id="${k}"
            .value=${v}
            placeholder=${p??""}
            maxlength=${b??2e3}
            rows="3"
            style="width: 100%; padding: 8px; border: 1px solid var(--border-color); border-radius: 4px; resize: vertical; font-family: inherit;"
            @input=${C=>{const $=C.target;n.onFieldChange(l,$.value)}}
            ?disabled=${t.saving}
          ></textarea>
          ${u?r`<div style="font-size: 12px; color: var(--text-muted); margin-top: 2px;">${u}</div>`:m}
          ${y?r`<div style="font-size: 12px; color: var(--danger-color); margin-top: 2px;">${y}</div>`:m}
        </div>
      `:r`
      <div class="form-field" style="margin-bottom: 12px;">
        <label for="${k}" style="display: block; margin-bottom: 4px; font-weight: 500;">
          ${d}
        </label>
        <input
          id="${k}"
          type=${f}
          .value=${v}
          placeholder=${p??""}
          maxlength=${b??256}
          style="width: 100%; padding: 8px; border: 1px solid var(--border-color); border-radius: 4px;"
          @input=${C=>{const $=C.target;n.onFieldChange(l,$.value)}}
          ?disabled=${t.saving}
        />
        ${u?r`<div style="font-size: 12px; color: var(--text-muted); margin-top: 2px;">${u}</div>`:m}
        ${y?r`<div style="font-size: 12px; color: var(--danger-color); margin-top: 2px;">${y}</div>`:m}
      </div>
    `},o=()=>{const l=t.values.picture;return l?r`
      <div style="margin-bottom: 12px;">
        <img
          src=${l}
          alt="Profile picture preview"
          style="max-width: 80px; max-height: 80px; border-radius: 50%; object-fit: cover; border: 2px solid var(--border-color);"
          @error=${d=>{const g=d.target;g.style.display="none"}}
          @load=${d=>{const g=d.target;g.style.display="block"}}
        />
      </div>
    `:m};return r`
    <div class="nostr-profile-form" style="padding: 16px; background: var(--bg-secondary); border-radius: 8px; margin-top: 12px;">
      <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 16px;">
        <div style="font-weight: 600; font-size: 16px;">Edit Profile</div>
        <div style="font-size: 12px; color: var(--text-muted);">Account: ${s}</div>
      </div>

      ${t.error?r`<div class="callout danger" style="margin-bottom: 12px;">${t.error}</div>`:m}

      ${t.success?r`<div class="callout success" style="margin-bottom: 12px;">${t.success}</div>`:m}

      ${o()}

      ${a("name","Username",{placeholder:"satoshi",maxLength:256,help:"Short username (e.g., satoshi)"})}

      ${a("displayName","Display Name",{placeholder:"Satoshi Nakamoto",maxLength:256,help:"Your full display name"})}

      ${a("about","Bio",{type:"textarea",placeholder:"Tell people about yourself...",maxLength:2e3,help:"A brief bio or description"})}

      ${a("picture","Avatar URL",{type:"url",placeholder:"https://example.com/avatar.jpg",help:"HTTPS URL to your profile picture"})}

      ${t.showAdvanced?r`
            <div style="border-top: 1px solid var(--border-color); padding-top: 12px; margin-top: 12px;">
              <div style="font-weight: 500; margin-bottom: 12px; color: var(--text-muted);">Advanced</div>

              ${a("banner","Banner URL",{type:"url",placeholder:"https://example.com/banner.jpg",help:"HTTPS URL to a banner image"})}

              ${a("website","Website",{type:"url",placeholder:"https://example.com",help:"Your personal website"})}

              ${a("nip05","NIP-05 Identifier",{placeholder:"you@example.com",help:"Verifiable identifier (e.g., you@domain.com)"})}

              ${a("lud16","Lightning Address",{placeholder:"you@getalby.com",help:"Lightning address for tips (LUD-16)"})}
            </div>
          `:m}

      <div style="display: flex; gap: 8px; margin-top: 16px; flex-wrap: wrap;">
        <button
          class="btn primary"
          @click=${n.onSave}
          ?disabled=${t.saving||!i}
        >
          ${t.saving?"Saving...":"Save & Publish"}
        </button>

        <button
          class="btn"
          @click=${n.onImport}
          ?disabled=${t.importing||t.saving}
        >
          ${t.importing?"Importing...":"Import from Relays"}
        </button>

        <button
          class="btn"
          @click=${n.onToggleAdvanced}
        >
          ${t.showAdvanced?"Hide Advanced":"Show Advanced"}
        </button>

        <button
          class="btn"
          @click=${n.onCancel}
          ?disabled=${t.saving}
        >
          Cancel
        </button>
      </div>

      ${i?r`
              <div style="font-size: 12px; color: var(--warning-color); margin-top: 8px">
                You have unsaved changes
              </div>
            `:m}
    </div>
  `}function Dp(e){const t={name:e?.name??"",displayName:e?.displayName??"",about:e?.about??"",picture:e?.picture??"",banner:e?.banner??"",website:e?.website??"",nip05:e?.nip05??"",lud16:e?.lud16??""};return{values:t,original:{...t},saving:!1,importing:!1,error:null,success:null,fieldErrors:{},showAdvanced:!!(e?.banner||e?.website||e?.nip05||e?.lud16)}}async function Fp(e,t){await Ap(e,t),await xe(e,!0)}async function Np(e){await _p(e),await xe(e,!0)}async function Op(e){await Cp(e),await xe(e,!0)}async function Bp(e){await Hn(e),await Ie(e),await xe(e,!0)}async function Up(e){await Ie(e),await xe(e,!0)}function zp(e){if(!Array.isArray(e))return{};const t={};for(const n of e){if(typeof n!="string")continue;const[s,...i]=n.split(":");if(!s||i.length===0)continue;const a=s.trim(),o=i.join(":").trim();a&&o&&(t[a]=o)}return t}function Or(e){return(e.channelsSnapshot?.channelAccounts?.nostr??[])[0]?.accountId??e.nostrProfileAccountId??"default"}function Br(e,t=""){return`/api/channels/nostr/${encodeURIComponent(e)}/profile${t}`}function Hp(e){const t=e.hello?.auth?.deviceToken?.trim();if(t)return`Bearer ${t}`;const n=e.settings.token.trim();if(n)return`Bearer ${n}`;const s=e.password.trim();return s?`Bearer ${s}`:null}function Ur(e){const t=Hp(e);return t?{Authorization:t}:{}}function Kp(e,t,n){e.nostrProfileAccountId=t,e.nostrProfileFormState=Dp(n??void 0)}function jp(e){e.nostrProfileFormState=null,e.nostrProfileAccountId=null}function Wp(e,t,n){const s=e.nostrProfileFormState;s&&(e.nostrProfileFormState={...s,values:{...s.values,[t]:n},fieldErrors:{...s.fieldErrors,[t]:""}})}function qp(e){const t=e.nostrProfileFormState;t&&(e.nostrProfileFormState={...t,showAdvanced:!t.showAdvanced})}async function Gp(e){const t=e.nostrProfileFormState;if(!t||t.saving)return;const n=Or(e);e.nostrProfileFormState={...t,saving:!0,error:null,success:null,fieldErrors:{}};try{const s=await fetch(Br(n),{method:"PUT",headers:{"Content-Type":"application/json",...Ur(e)},body:JSON.stringify(t.values)}),i=await s.json().catch(()=>null);if(!s.ok||i?.ok===!1||!i){const a=i?.error??`Profile update failed (${s.status})`;e.nostrProfileFormState={...t,saving:!1,error:a,success:null,fieldErrors:zp(i?.details)};return}if(!i.persisted){e.nostrProfileFormState={...t,saving:!1,error:"Profile publish failed on all relays.",success:null};return}e.nostrProfileFormState={...t,saving:!1,error:null,success:"Profile published to relays.",fieldErrors:{},original:{...t.values}},await xe(e,!0)}catch(s){e.nostrProfileFormState={...t,saving:!1,error:`Profile update failed: ${String(s)}`,success:null}}}async function Vp(e){const t=e.nostrProfileFormState;if(!t||t.importing)return;const n=Or(e);e.nostrProfileFormState={...t,importing:!0,error:null,success:null};try{const s=await fetch(Br(n,"/import"),{method:"POST",headers:{"Content-Type":"application/json",...Ur(e)},body:JSON.stringify({autoMerge:!0})}),i=await s.json().catch(()=>null);if(!s.ok||i?.ok===!1||!i){const d=i?.error??`Profile import failed (${s.status})`;e.nostrProfileFormState={...t,importing:!1,error:d,success:null};return}const a=i.merged??i.imported??null,o=a?{...t.values,...a}:t.values,l=!!(o.banner||o.website||o.nip05||o.lud16);e.nostrProfileFormState={...t,importing:!1,values:o,error:null,success:i.saved?"Profile imported from relays. Review and publish.":"Profile imported. Review and publish.",showAdvanced:l},i.saved&&await xe(e,!0)}catch(s){e.nostrProfileFormState={...t,importing:!1,error:`Profile import failed: ${String(s)}`,success:null}}}function zr(e){const t=(e??"").trim();if(!t)return null;const n=t.split(":").filter(Boolean);if(n.length<3||n[0]!=="agent")return null;const s=n[1]?.trim(),i=n.slice(2).join(":");return!s||!i?null:{agentId:s,rest:i}}const ti=450;function hn(e,t=!1,n=!1){e.chatScrollFrame&&cancelAnimationFrame(e.chatScrollFrame),e.chatScrollTimeout!=null&&(clearTimeout(e.chatScrollTimeout),e.chatScrollTimeout=null);const s=()=>{const i=e.querySelector(".chat-thread");if(i){const a=getComputedStyle(i).overflowY;if(a==="auto"||a==="scroll"||i.scrollHeight-i.clientHeight>1)return i}return document.scrollingElement??document.documentElement};e.updateComplete.then(()=>{e.chatScrollFrame=requestAnimationFrame(()=>{e.chatScrollFrame=null;const i=s();if(!i)return;const a=i.scrollHeight-i.scrollTop-i.clientHeight,o=t&&!e.chatHasAutoScrolled;if(!(o||e.chatUserNearBottom||a<ti)){e.chatNewMessagesBelow=!0;return}o&&(e.chatHasAutoScrolled=!0);const d=n&&(typeof window>"u"||typeof window.matchMedia!="function"||!window.matchMedia("(prefers-reduced-motion: reduce)").matches),g=i.scrollHeight;typeof i.scrollTo=="function"?i.scrollTo({top:g,behavior:d?"smooth":"auto"}):i.scrollTop=g,e.chatUserNearBottom=!0,e.chatNewMessagesBelow=!1;const f=o?150:120;e.chatScrollTimeout=window.setTimeout(()=>{e.chatScrollTimeout=null;const p=s();if(!p)return;const b=p.scrollHeight-p.scrollTop-p.clientHeight;(o||e.chatUserNearBottom||b<ti)&&(p.scrollTop=p.scrollHeight,e.chatUserNearBottom=!0)},f)})})}function Hr(e,t=!1){e.logsScrollFrame&&cancelAnimationFrame(e.logsScrollFrame),e.updateComplete.then(()=>{e.logsScrollFrame=requestAnimationFrame(()=>{e.logsScrollFrame=null;const n=e.querySelector(".log-stream");if(!n)return;const s=n.scrollHeight-n.scrollTop-n.clientHeight;(t||s<80)&&(n.scrollTop=n.scrollHeight)})})}function Qp(e,t){const n=t.currentTarget;if(!n)return;const s=n.scrollHeight-n.scrollTop-n.clientHeight;e.chatUserNearBottom=s<ti,e.chatUserNearBottom&&(e.chatNewMessagesBelow=!1)}function Yp(e,t){const n=t.currentTarget;if(!n)return;const s=n.scrollHeight-n.scrollTop-n.clientHeight;e.logsAtBottom=s<80}function no(e){e.chatHasAutoScrolled=!1,e.chatUserNearBottom=!0,e.chatNewMessagesBelow=!1}function Jp(e,t){if(e.length===0)return;const n=new Blob([`${e.join(`
`)}
`],{type:"text/plain"}),s=URL.createObjectURL(n),i=document.createElement("a"),a=new Date().toISOString().slice(0,19).replace(/[:T]/g,"-");i.href=s,i.download=`aisopod-logs-${t}-${a}.log`,i.click(),URL.revokeObjectURL(s)}function Zp(e){if(typeof ResizeObserver>"u")return;const t=e.querySelector(".topbar");if(!t)return;const n=()=>{const{height:s}=t.getBoundingClientRect();e.style.setProperty("--topbar-height",`${s}px`)};n(),e.topbarObserver=new ResizeObserver(()=>n()),e.topbarObserver.observe(t)}async function rs(e){if(!(!e.client||!e.connected)&&!e.debugLoading){e.debugLoading=!0;try{const[t,n,s,i]=await Promise.all([e.client.request("status",{}),e.client.request("health",{}),e.client.request("models.list",{}),e.client.request("last-heartbeat",{})]);e.debugStatus=t,e.debugHealth=n;const a=s;e.debugModels=Array.isArray(a?.models)?a?.models:[],e.debugHeartbeat=i}catch(t){e.debugCallError=String(t)}finally{e.debugLoading=!1}}}async function Xp(e){if(!(!e.client||!e.connected)){e.debugCallError=null,e.debugCallResult=null;try{const t=e.debugCallParams.trim()?JSON.parse(e.debugCallParams):{},n=await e.client.request(e.debugCallMethod.trim(),t);e.debugCallResult=JSON.stringify(n,null,2)}catch(t){e.debugCallError=String(t)}}}const eh=2e3,th=new Set(["trace","debug","info","warn","error","fatal"]);function nh(e){if(typeof e!="string")return null;const t=e.trim();if(!t.startsWith("{")||!t.endsWith("}"))return null;try{const n=JSON.parse(t);return!n||typeof n!="object"?null:n}catch{return null}}function sh(e){if(typeof e!="string")return null;const t=e.toLowerCase();return th.has(t)?t:null}function ih(e){if(!e.trim())return{raw:e,message:e};try{const t=JSON.parse(e),n=t&&typeof t._meta=="object"&&t._meta!==null?t._meta:null,s=typeof t.time=="string"?t.time:typeof n?.date=="string"?n?.date:null,i=sh(n?.logLevelName??n?.level),a=typeof t[0]=="string"?t[0]:typeof n?.name=="string"?n?.name:null,o=nh(a);let l=null;o&&(typeof o.subsystem=="string"?l=o.subsystem:typeof o.module=="string"&&(l=o.module)),!l&&a&&a.length<120&&(l=a);let d=null;return typeof t[1]=="string"?d=t[1]:!o&&typeof t[0]=="string"?d=t[0]:typeof t.message=="string"&&(d=t.message),{raw:e,time:s,level:i,subsystem:l,message:d??e,meta:n??void 0}}catch{return{raw:e,message:e}}}async function Ni(e,t){if(!(!e.client||!e.connected)&&!(e.logsLoading&&!t?.quiet)){t?.quiet||(e.logsLoading=!0),e.logsError=null;try{const s=await e.client.request("logs.tail",{cursor:t?.reset?void 0:e.logsCursor??void 0,limit:e.logsLimit,maxBytes:e.logsMaxBytes}),a=(Array.isArray(s.lines)?s.lines.filter(l=>typeof l=="string"):[]).map(ih),o=!!(t?.reset||s.reset||e.logsCursor==null);e.logsEntries=o?a:[...e.logsEntries,...a].slice(-eh),typeof s.cursor=="number"&&(e.logsCursor=s.cursor),typeof s.file=="string"&&(e.logsFile=s.file),e.logsTruncated=!!s.truncated,e.logsLastFetchAt=Date.now()}catch(n){e.logsError=String(n)}finally{t?.quiet||(e.logsLoading=!1)}}}async function ls(e,t){if(!(!e.client||!e.connected)&&!e.nodesLoading){e.nodesLoading=!0,t?.quiet||(e.lastError=null);try{const n=await e.client.request("node.list",{});e.nodes=Array.isArray(n.nodes)?n.nodes:[]}catch(n){t?.quiet||(e.lastError=String(n))}finally{e.nodesLoading=!1}}}function ah(e){e.nodesPollInterval==null&&(e.nodesPollInterval=window.setInterval(()=>{ls(e,{quiet:!0})},5e3))}function oh(e){e.nodesPollInterval!=null&&(clearInterval(e.nodesPollInterval),e.nodesPollInterval=null)}function Oi(e){e.logsPollInterval==null&&(e.logsPollInterval=window.setInterval(()=>{e.tab==="logs"&&Ni(e,{quiet:!0})},2e3))}function Bi(e){e.logsPollInterval!=null&&(clearInterval(e.logsPollInterval),e.logsPollInterval=null)}function Ui(e){e.debugPollInterval==null&&(e.debugPollInterval=window.setInterval(()=>{e.tab==="debug"&&rs(e)},3e3))}function zi(e){e.debugPollInterval!=null&&(clearInterval(e.debugPollInterval),e.debugPollInterval=null)}async function Kr(e,t){if(!(!e.client||!e.connected||e.agentIdentityLoading)&&!e.agentIdentityById[t]){e.agentIdentityLoading=!0,e.agentIdentityError=null;try{const n=await e.client.request("agent.identity.get",{agentId:t});n&&(e.agentIdentityById={...e.agentIdentityById,[t]:n})}catch(n){e.agentIdentityError=String(n)}finally{e.agentIdentityLoading=!1}}}async function jr(e,t){if(!e.client||!e.connected||e.agentIdentityLoading)return;const n=t.filter(s=>!e.agentIdentityById[s]);if(n.length!==0){e.agentIdentityLoading=!0,e.agentIdentityError=null;try{for(const s of n){const i=await e.client.request("agent.identity.get",{agentId:s});i&&(e.agentIdentityById={...e.agentIdentityById,[s]:i})}}catch(s){e.agentIdentityError=String(s)}finally{e.agentIdentityLoading=!1}}}async function Kn(e,t){if(!(!e.client||!e.connected)&&!e.agentSkillsLoading){e.agentSkillsLoading=!0,e.agentSkillsError=null;try{const n=await e.client.request("skills.status",{agentId:t});n&&(e.agentSkillsReport=n,e.agentSkillsAgentId=t)}catch(n){e.agentSkillsError=String(n)}finally{e.agentSkillsLoading=!1}}}async function Hi(e){if(!(!e.client||!e.connected)&&!e.agentsLoading){e.agentsLoading=!0,e.agentsError=null;try{const t=await e.client.request("agents.list",{});if(t){e.agentsList=t;const n=e.agentsSelectedId,s=t.agents.some(i=>i.id===n);(!n||!s)&&(e.agentsSelectedId=t.defaultId??t.agents[0]?.id??null)}}catch(t){e.agentsError=String(t)}finally{e.agentsLoading=!1}}}function Ki(e,t){if(e==null||!Number.isFinite(e)||e<=0)return;if(e<1e3)return`${Math.round(e)}ms`;const n=t?.spaced?" ":"",s=Math.round(e/1e3),i=Math.floor(s/3600),a=Math.floor(s%3600/60),o=s%60;if(i>=24){const l=Math.floor(i/24),d=i%24;return d>0?`${l}d${n}${d}h`:`${l}d`}return i>0?a>0?`${i}h${n}${a}m`:`${i}h`:a>0?o>0?`${a}m${n}${o}s`:`${a}m`:`${o}s`}function ji(e,t="n/a"){if(e==null||!Number.isFinite(e)||e<0)return t;if(e<1e3)return`${Math.round(e)}ms`;const n=Math.round(e/1e3);if(n<60)return`${n}s`;const s=Math.round(n/60);if(s<60)return`${s}m`;const i=Math.round(s/60);return i<24?`${i}h`:`${Math.round(i/24)}d`}function Y(e,t){const n=t?.fallback??"n/a";if(e==null||!Number.isFinite(e))return n;const s=Date.now()-e,i=Math.abs(s),a=s>=0,o=Math.round(i/1e3);if(o<60)return a?"just now":"in <1m";const l=Math.round(o/60);if(l<60)return a?`${l}m ago`:`in ${l}m`;const d=Math.round(l/60);if(d<48)return a?`${d}h ago`:`in ${d}h`;const g=Math.round(d/24);return a?`${g}d ago`:`in ${g}d`}const rh=/<\s*\/?\s*(?:think(?:ing)?|thought|antthinking|final)\b/i,En=/<\s*\/?\s*final\b[^<>]*>/gi,so=/<\s*(\/?)\s*(?:think(?:ing)?|thought|antthinking)\b[^<>]*>/gi;function io(e){const t=[],n=/(^|\n)(```|~~~)[^\n]*\n[\s\S]*?(?:\n\2(?:\n|$)|$)/g;for(const i of e.matchAll(n)){const a=(i.index??0)+i[1].length;t.push({start:a,end:a+i[0].length-i[1].length})}const s=/`+[^`]+`+/g;for(const i of e.matchAll(s)){const a=i.index??0,o=a+i[0].length;t.some(d=>a>=d.start&&o<=d.end)||t.push({start:a,end:o})}return t.sort((i,a)=>i.start-a.start),t}function ao(e,t){return t.some(n=>e>=n.start&&e<n.end)}function lh(e,t){return e.trimStart()}function ch(e,t){if(!e||!rh.test(e))return e;let n=e;if(En.test(n)){En.lastIndex=0;const l=[],d=io(n);for(const g of n.matchAll(En)){const f=g.index??0;l.push({start:f,length:g[0].length,inCode:ao(f,d)})}for(let g=l.length-1;g>=0;g--){const f=l[g];f.inCode||(n=n.slice(0,f.start)+n.slice(f.start+f.length))}}else En.lastIndex=0;const s=io(n);so.lastIndex=0;let i="",a=0,o=!1;for(const l of n.matchAll(so)){const d=l.index??0,g=l[1]==="/";ao(d,s)||(o?g&&(o=!1):(i+=n.slice(a,d),g||(o=!0)),a=d+l[0].length)}return i+=n.slice(a),lh(i)}function $t(e){return!e&&e!==0?"n/a":new Date(e).toLocaleString()}function ni(e){return!e||e.length===0?"none":e.filter(t=>!!(t&&t.trim())).join(", ")}function si(e,t=120){return e.length<=t?e:`${e.slice(0,Math.max(0,t-1))}…`}function Wr(e,t){return e.length<=t?{text:e,truncated:!1,total:e.length}:{text:e.slice(0,Math.max(0,t)),truncated:!0,total:e.length}}function Vn(e,t){const n=Number(e);return Number.isFinite(n)?n:t}function Ms(e){return ch(e)}async function fn(e){if(!(!e.client||!e.connected))try{const t=await e.client.request("cron.status",{});e.cronStatus=t}catch(t){e.cronError=String(t)}}async function cs(e){if(!(!e.client||!e.connected)&&!e.cronLoading){e.cronLoading=!0,e.cronError=null;try{const t=await e.client.request("cron.list",{includeDisabled:!0});e.cronJobs=Array.isArray(t.jobs)?t.jobs:[]}catch(t){e.cronError=String(t)}finally{e.cronLoading=!1}}}function dh(e){if(e.scheduleKind==="at"){const n=Date.parse(e.scheduleAt);if(!Number.isFinite(n))throw new Error("Invalid run time.");return{kind:"at",at:new Date(n).toISOString()}}if(e.scheduleKind==="every"){const n=Vn(e.everyAmount,0);if(n<=0)throw new Error("Invalid interval amount.");const s=e.everyUnit;return{kind:"every",everyMs:n*(s==="minutes"?6e4:s==="hours"?36e5:864e5)}}const t=e.cronExpr.trim();if(!t)throw new Error("Cron expression required.");return{kind:"cron",expr:t,tz:e.cronTz.trim()||void 0}}function uh(e){if(e.payloadKind==="systemEvent"){const i=e.payloadText.trim();if(!i)throw new Error("System event text required.");return{kind:"systemEvent",text:i}}const t=e.payloadText.trim();if(!t)throw new Error("Agent message required.");const n={kind:"agentTurn",message:t},s=Vn(e.timeoutSeconds,0);return s>0&&(n.timeoutSeconds=s),n}async function gh(e){if(!(!e.client||!e.connected||e.cronBusy)){e.cronBusy=!0,e.cronError=null;try{const t=dh(e.cronForm),n=uh(e.cronForm),s=e.cronForm.sessionTarget==="isolated"&&e.cronForm.payloadKind==="agentTurn"&&e.cronForm.deliveryMode?{mode:e.cronForm.deliveryMode==="announce"?"announce":"none",channel:e.cronForm.deliveryChannel.trim()||"last",to:e.cronForm.deliveryTo.trim()||void 0}:void 0,i=e.cronForm.agentId.trim(),a={name:e.cronForm.name.trim(),description:e.cronForm.description.trim()||void 0,agentId:i||void 0,enabled:e.cronForm.enabled,schedule:t,sessionTarget:e.cronForm.sessionTarget,wakeMode:e.cronForm.wakeMode,payload:n,delivery:s};if(!a.name)throw new Error("Name required.");await e.client.request("cron.add",a),e.cronForm={...e.cronForm,name:"",description:"",payloadText:""},await cs(e),await fn(e)}catch(t){e.cronError=String(t)}finally{e.cronBusy=!1}}}async function ph(e,t,n){if(!(!e.client||!e.connected||e.cronBusy)){e.cronBusy=!0,e.cronError=null;try{await e.client.request("cron.update",{id:t.id,patch:{enabled:n}}),await cs(e),await fn(e)}catch(s){e.cronError=String(s)}finally{e.cronBusy=!1}}}async function hh(e,t){if(!(!e.client||!e.connected||e.cronBusy)){e.cronBusy=!0,e.cronError=null;try{await e.client.request("cron.run",{id:t.id,mode:"force"}),await qr(e,t.id)}catch(n){e.cronError=String(n)}finally{e.cronBusy=!1}}}async function fh(e,t){if(!(!e.client||!e.connected||e.cronBusy)){e.cronBusy=!0,e.cronError=null;try{await e.client.request("cron.remove",{id:t.id}),e.cronRunsJobId===t.id&&(e.cronRunsJobId=null,e.cronRuns=[]),await cs(e),await fn(e)}catch(n){e.cronError=String(n)}finally{e.cronBusy=!1}}}async function qr(e,t){if(!(!e.client||!e.connected))try{const n=await e.client.request("cron.runs",{id:t,limit:50});e.cronRunsJobId=t,e.cronRuns=Array.isArray(n.entries)?n.entries:[]}catch(n){e.cronError=String(n)}}function Wi(e){return e.trim()}function vh(e){if(!Array.isArray(e))return[];const t=new Set;for(const n of e){const s=n.trim();s&&t.add(s)}return[...t].toSorted()}const Gr="aisopod.device.auth.v1";function qi(){try{const e=window.localStorage.getItem(Gr);if(!e)return null;const t=JSON.parse(e);return!t||t.version!==1||!t.deviceId||typeof t.deviceId!="string"||!t.tokens||typeof t.tokens!="object"?null:t}catch{return null}}function Vr(e){try{window.localStorage.setItem(Gr,JSON.stringify(e))}catch{}}function mh(e){const t=qi();if(!t||t.deviceId!==e.deviceId)return null;const n=Wi(e.role),s=t.tokens[n];return!s||typeof s.token!="string"?null:s}function Qr(e){const t=Wi(e.role),n={version:1,deviceId:e.deviceId,tokens:{}},s=qi();s&&s.deviceId===e.deviceId&&(n.tokens={...s.tokens});const i={token:e.token,role:t,scopes:vh(e.scopes),updatedAtMs:Date.now()};return n.tokens[t]=i,Vr(n),i}function Yr(e){const t=qi();if(!t||t.deviceId!==e.deviceId)return;const n=Wi(e.role);if(!t.tokens[n])return;const s={...t,tokens:{...t.tokens}};delete s.tokens[n],Vr(s)}const Jr={p:0x7fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffedn,n:0x1000000000000000000000000000000014def9dea2f79cd65812631a5cf5d3edn,h:8n,a:0x7fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffecn,d:0x52036cee2b6ffe738cc740797779e89800700a4d4141d8ab75eb4dca135978a3n,Gx:0x216936d3cd6e53fec0a4e231fdd6dc5c692cc7609525a7b2c9562d608f25d51an,Gy:0x6666666666666666666666666666666666666666666666666666666666666658n},{p:pe,n:jn,Gx:oo,Gy:ro,a:Is,d:Rs,h:bh}=Jr,wt=32,Gi=64,yh=(...e)=>{"captureStackTrace"in Error&&typeof Error.captureStackTrace=="function"&&Error.captureStackTrace(...e)},le=(e="")=>{const t=new Error(e);throw yh(t,le),t},xh=e=>typeof e=="bigint",$h=e=>typeof e=="string",wh=e=>e instanceof Uint8Array||ArrayBuffer.isView(e)&&e.constructor.name==="Uint8Array",nt=(e,t,n="")=>{const s=wh(e),i=e?.length,a=t!==void 0;if(!s||a&&i!==t){const o=n&&`"${n}" `,l=a?` of length ${t}`:"",d=s?`length=${i}`:`type=${typeof e}`;le(o+"expected Uint8Array"+l+", got "+d)}return e},ds=e=>new Uint8Array(e),Zr=e=>Uint8Array.from(e),Xr=(e,t)=>e.toString(16).padStart(t,"0"),el=e=>Array.from(nt(e)).map(t=>Xr(t,2)).join(""),We={_0:48,_9:57,A:65,F:70,a:97,f:102},lo=e=>{if(e>=We._0&&e<=We._9)return e-We._0;if(e>=We.A&&e<=We.F)return e-(We.A-10);if(e>=We.a&&e<=We.f)return e-(We.a-10)},tl=e=>{const t="hex invalid";if(!$h(e))return le(t);const n=e.length,s=n/2;if(n%2)return le(t);const i=ds(s);for(let a=0,o=0;a<s;a++,o+=2){const l=lo(e.charCodeAt(o)),d=lo(e.charCodeAt(o+1));if(l===void 0||d===void 0)return le(t);i[a]=l*16+d}return i},nl=()=>globalThis?.crypto,kh=()=>nl()?.subtle??le("crypto.subtle must be defined, consider polyfill"),dn=(...e)=>{const t=ds(e.reduce((s,i)=>s+nt(i).length,0));let n=0;return e.forEach(s=>{t.set(s,n),n+=s.length}),t},Sh=(e=wt)=>nl().getRandomValues(ds(e)),Qn=BigInt,ut=(e,t,n,s="bad number: out of range")=>xh(e)&&t<=e&&e<n?e:le(s),N=(e,t=pe)=>{const n=e%t;return n>=0n?n:t+n},sl=e=>N(e,jn),Ah=(e,t)=>{(e===0n||t<=0n)&&le("no inverse n="+e+" mod="+t);let n=N(e,t),s=t,i=0n,a=1n;for(;n!==0n;){const o=s/n,l=s%n,d=i-a*o;s=n,n=l,i=a,a=d}return s===1n?N(i,t):le("no inverse")},_h=e=>{const t=rl[e];return typeof t!="function"&&le("hashes."+e+" not set"),t},Ps=e=>e instanceof Ae?e:le("Point expected"),ii=2n**256n;class Ae{static BASE;static ZERO;X;Y;Z;T;constructor(t,n,s,i){const a=ii;this.X=ut(t,0n,a),this.Y=ut(n,0n,a),this.Z=ut(s,1n,a),this.T=ut(i,0n,a),Object.freeze(this)}static CURVE(){return Jr}static fromAffine(t){return new Ae(t.x,t.y,1n,N(t.x*t.y))}static fromBytes(t,n=!1){const s=Rs,i=Zr(nt(t,wt)),a=t[31];i[31]=a&-129;const o=al(i);ut(o,0n,n?ii:pe);const d=N(o*o),g=N(d-1n),f=N(s*d+1n);let{isValid:p,value:b}=Th(g,f);p||le("bad point: y not sqrt");const u=(b&1n)===1n,v=(a&128)!==0;return!n&&b===0n&&v&&le("bad point: x==0, isLastByteOdd"),v!==u&&(b=N(-b)),new Ae(b,o,1n,N(b*o))}static fromHex(t,n){return Ae.fromBytes(tl(t),n)}get x(){return this.toAffine().x}get y(){return this.toAffine().y}assertValidity(){const t=Is,n=Rs,s=this;if(s.is0())return le("bad point: ZERO");const{X:i,Y:a,Z:o,T:l}=s,d=N(i*i),g=N(a*a),f=N(o*o),p=N(f*f),b=N(d*t),u=N(f*N(b+g)),v=N(p+N(n*N(d*g)));if(u!==v)return le("bad point: equation left != right (1)");const y=N(i*a),k=N(o*l);return y!==k?le("bad point: equation left != right (2)"):this}equals(t){const{X:n,Y:s,Z:i}=this,{X:a,Y:o,Z:l}=Ps(t),d=N(n*l),g=N(a*i),f=N(s*l),p=N(o*i);return d===g&&f===p}is0(){return this.equals(Dt)}negate(){return new Ae(N(-this.X),this.Y,this.Z,N(-this.T))}double(){const{X:t,Y:n,Z:s}=this,i=Is,a=N(t*t),o=N(n*n),l=N(2n*N(s*s)),d=N(i*a),g=t+n,f=N(N(g*g)-a-o),p=d+o,b=p-l,u=d-o,v=N(f*b),y=N(p*u),k=N(f*u),C=N(b*p);return new Ae(v,y,C,k)}add(t){const{X:n,Y:s,Z:i,T:a}=this,{X:o,Y:l,Z:d,T:g}=Ps(t),f=Is,p=Rs,b=N(n*o),u=N(s*l),v=N(a*p*g),y=N(i*d),k=N((n+s)*(o+l)-b-u),C=N(y-v),$=N(y+v),T=N(u-f*b),_=N(k*C),L=N($*T),E=N(k*T),P=N(C*$);return new Ae(_,L,P,E)}subtract(t){return this.add(Ps(t).negate())}multiply(t,n=!0){if(!n&&(t===0n||this.is0()))return Dt;if(ut(t,1n,jn),t===1n)return this;if(this.equals(kt))return Bh(t).p;let s=Dt,i=kt;for(let a=this;t>0n;a=a.double(),t>>=1n)t&1n?s=s.add(a):n&&(i=i.add(a));return s}multiplyUnsafe(t){return this.multiply(t,!1)}toAffine(){const{X:t,Y:n,Z:s}=this;if(this.equals(Dt))return{x:0n,y:1n};const i=Ah(s,pe);N(s*i)!==1n&&le("invalid inverse");const a=N(t*i),o=N(n*i);return{x:a,y:o}}toBytes(){const{x:t,y:n}=this.assertValidity().toAffine(),s=il(n);return s[31]|=t&1n?128:0,s}toHex(){return el(this.toBytes())}clearCofactor(){return this.multiply(Qn(bh),!1)}isSmallOrder(){return this.clearCofactor().is0()}isTorsionFree(){let t=this.multiply(jn/2n,!1).double();return jn%2n&&(t=t.add(this)),t.is0()}}const kt=new Ae(oo,ro,1n,N(oo*ro)),Dt=new Ae(0n,1n,1n,0n);Ae.BASE=kt;Ae.ZERO=Dt;const il=e=>tl(Xr(ut(e,0n,ii),Gi)).reverse(),al=e=>Qn("0x"+el(Zr(nt(e)).reverse())),De=(e,t)=>{let n=e;for(;t-- >0n;)n*=n,n%=pe;return n},Ch=e=>{const n=e*e%pe*e%pe,s=De(n,2n)*n%pe,i=De(s,1n)*e%pe,a=De(i,5n)*i%pe,o=De(a,10n)*a%pe,l=De(o,20n)*o%pe,d=De(l,40n)*l%pe,g=De(d,80n)*d%pe,f=De(g,80n)*d%pe,p=De(f,10n)*a%pe;return{pow_p_5_8:De(p,2n)*e%pe,b2:n}},co=0x2b8324804fc1df0b2b4d00993dfbd7a72f431806ad2fe478c4ee1b274a0ea0b0n,Th=(e,t)=>{const n=N(t*t*t),s=N(n*n*t),i=Ch(e*s).pow_p_5_8;let a=N(e*n*i);const o=N(t*a*a),l=a,d=N(a*co),g=o===e,f=o===N(-e),p=o===N(-e*co);return g&&(a=l),(f||p)&&(a=d),(N(a)&1n)===1n&&(a=N(-a)),{isValid:g||f,value:a}},ai=e=>sl(al(e)),Vi=(...e)=>rl.sha512Async(dn(...e)),Eh=(...e)=>_h("sha512")(dn(...e)),ol=e=>{const t=e.slice(0,wt);t[0]&=248,t[31]&=127,t[31]|=64;const n=e.slice(wt,Gi),s=ai(t),i=kt.multiply(s),a=i.toBytes();return{head:t,prefix:n,scalar:s,point:i,pointBytes:a}},Qi=e=>Vi(nt(e,wt)).then(ol),Lh=e=>ol(Eh(nt(e,wt))),Mh=e=>Qi(e).then(t=>t.pointBytes),Ih=e=>Vi(e.hashable).then(e.finish),Rh=(e,t,n)=>{const{pointBytes:s,scalar:i}=e,a=ai(t),o=kt.multiply(a).toBytes();return{hashable:dn(o,s,n),finish:g=>{const f=sl(a+ai(g)*i);return nt(dn(o,il(f)),Gi)}}},Ph=async(e,t)=>{const n=nt(e),s=await Qi(t),i=await Vi(s.prefix,n);return Ih(Rh(s,i,n))},rl={sha512Async:async e=>{const t=kh(),n=dn(e);return ds(await t.digest("SHA-512",n.buffer))},sha512:void 0},Dh=(e=Sh(wt))=>e,Fh={getExtendedPublicKeyAsync:Qi,getExtendedPublicKey:Lh,randomSecretKey:Dh},Yn=8,Nh=256,ll=Math.ceil(Nh/Yn)+1,oi=2**(Yn-1),Oh=()=>{const e=[];let t=kt,n=t;for(let s=0;s<ll;s++){n=t,e.push(n);for(let i=1;i<oi;i++)n=n.add(t),e.push(n);t=n.double()}return e};let uo;const go=(e,t)=>{const n=t.negate();return e?n:t},Bh=e=>{const t=uo||(uo=Oh());let n=Dt,s=kt;const i=2**Yn,a=i,o=Qn(i-1),l=Qn(Yn);for(let d=0;d<ll;d++){let g=Number(e&o);e>>=l,g>oi&&(g-=a,e+=1n);const f=d*oi,p=f,b=f+Math.abs(g)-1,u=d%2!==0,v=g<0;g===0?s=s.add(go(u,t[p])):n=n.add(go(v,t[b]))}return e!==0n&&le("invalid wnaf"),{p:n,f:s}},Ds="aisopod-device-identity-v1";function ri(e){let t="";for(const n of e)t+=String.fromCharCode(n);return btoa(t).replaceAll("+","-").replaceAll("/","_").replace(/=+$/g,"")}function cl(e){const t=e.replaceAll("-","+").replaceAll("_","/"),n=t+"=".repeat((4-t.length%4)%4),s=atob(n),i=new Uint8Array(s.length);for(let a=0;a<s.length;a+=1)i[a]=s.charCodeAt(a);return i}function Uh(e){return Array.from(e).map(t=>t.toString(16).padStart(2,"0")).join("")}async function dl(e){const t=await crypto.subtle.digest("SHA-256",e.slice().buffer);return Uh(new Uint8Array(t))}async function zh(){const e=Fh.randomSecretKey(),t=await Mh(e);return{deviceId:await dl(t),publicKey:ri(t),privateKey:ri(e)}}async function Yi(){try{const n=localStorage.getItem(Ds);if(n){const s=JSON.parse(n);if(s?.version===1&&typeof s.deviceId=="string"&&typeof s.publicKey=="string"&&typeof s.privateKey=="string"){const i=await dl(cl(s.publicKey));if(i!==s.deviceId){const a={...s,deviceId:i};return localStorage.setItem(Ds,JSON.stringify(a)),{deviceId:i,publicKey:s.publicKey,privateKey:s.privateKey}}return{deviceId:s.deviceId,publicKey:s.publicKey,privateKey:s.privateKey}}}}catch{}const e=await zh(),t={version:1,deviceId:e.deviceId,publicKey:e.publicKey,privateKey:e.privateKey,createdAtMs:Date.now()};return localStorage.setItem(Ds,JSON.stringify(t)),e}async function Hh(e,t){const n=cl(e),s=new TextEncoder().encode(t),i=await Ph(s,n);return ri(i)}async function st(e,t){if(!(!e.client||!e.connected)&&!e.devicesLoading){e.devicesLoading=!0,t?.quiet||(e.devicesError=null);try{const n=await e.client.request("device.pair.list",{});e.devicesList={pending:Array.isArray(n?.pending)?n.pending:[],paired:Array.isArray(n?.paired)?n.paired:[]}}catch(n){t?.quiet||(e.devicesError=String(n))}finally{e.devicesLoading=!1}}}async function Kh(e,t){if(!(!e.client||!e.connected))try{await e.client.request("device.pair.approve",{requestId:t}),await st(e)}catch(n){e.devicesError=String(n)}}async function jh(e,t){if(!(!e.client||!e.connected||!window.confirm("Reject this device pairing request?")))try{await e.client.request("device.pair.reject",{requestId:t}),await st(e)}catch(s){e.devicesError=String(s)}}async function Wh(e,t){if(!(!e.client||!e.connected))try{const n=await e.client.request("device.token.rotate",t);if(n?.token){const s=await Yi(),i=n.role??t.role;(n.deviceId===s.deviceId||t.deviceId===s.deviceId)&&Qr({deviceId:s.deviceId,role:i,token:n.token,scopes:n.scopes??t.scopes??[]}),window.prompt("New device token (copy and store securely):",n.token)}await st(e)}catch(n){e.devicesError=String(n)}}async function qh(e,t){if(!(!e.client||!e.connected||!window.confirm(`Revoke token for ${t.deviceId} (${t.role})?`)))try{await e.client.request("device.token.revoke",t);const s=await Yi();t.deviceId===s.deviceId&&Yr({deviceId:s.deviceId,role:t.role}),await st(e)}catch(s){e.devicesError=String(s)}}function Gh(e){if(!e||e.kind==="gateway")return{method:"exec.approvals.get",params:{}};const t=e.nodeId.trim();return t?{method:"exec.approvals.node.get",params:{nodeId:t}}:null}function Vh(e,t){if(!e||e.kind==="gateway")return{method:"exec.approvals.set",params:t};const n=e.nodeId.trim();return n?{method:"exec.approvals.node.set",params:{...t,nodeId:n}}:null}async function Ji(e,t){if(!(!e.client||!e.connected)&&!e.execApprovalsLoading){e.execApprovalsLoading=!0,e.lastError=null;try{const n=Gh(t);if(!n){e.lastError="Select a node before loading exec approvals.";return}const s=await e.client.request(n.method,n.params);Qh(e,s)}catch(n){e.lastError=String(n)}finally{e.execApprovalsLoading=!1}}}function Qh(e,t){e.execApprovalsSnapshot=t,e.execApprovalsDirty||(e.execApprovalsForm=xt(t.file??{}))}async function Yh(e,t){if(!(!e.client||!e.connected)){e.execApprovalsSaving=!0,e.lastError=null;try{const n=e.execApprovalsSnapshot?.hash;if(!n){e.lastError="Exec approvals hash missing; reload and retry.";return}const s=e.execApprovalsForm??e.execApprovalsSnapshot?.file??{},i=Vh(t,{file:s,baseHash:n});if(!i){e.lastError="Select a node before saving exec approvals.";return}await e.client.request(i.method,i.params),e.execApprovalsDirty=!1,await Ji(e,t)}catch(n){e.lastError=String(n)}finally{e.execApprovalsSaving=!1}}}function Jh(e,t,n){const s=xt(e.execApprovalsForm??e.execApprovalsSnapshot?.file??{});Pr(s,t,n),e.execApprovalsForm=s,e.execApprovalsDirty=!0}function Zh(e,t){const n=xt(e.execApprovalsForm??e.execApprovalsSnapshot?.file??{});Dr(n,t),e.execApprovalsForm=n,e.execApprovalsDirty=!0}async function Zi(e){if(!(!e.client||!e.connected)&&!e.presenceLoading){e.presenceLoading=!0,e.presenceError=null,e.presenceStatus=null;try{const t=await e.client.request("system-presence",{});Array.isArray(t)?(e.presenceEntries=t,e.presenceStatus=t.length===0?"No instances yet.":null):(e.presenceEntries=[],e.presenceStatus="No presence payload.")}catch(t){e.presenceError=String(t)}finally{e.presenceLoading=!1}}}async function _t(e,t){if(!(!e.client||!e.connected)&&!e.sessionsLoading){e.sessionsLoading=!0,e.sessionsError=null;try{const n=t?.includeGlobal??e.sessionsIncludeGlobal,s=t?.includeUnknown??e.sessionsIncludeUnknown,i=t?.activeMinutes??Vn(e.sessionsFilterActive,0),a=t?.limit??Vn(e.sessionsFilterLimit,0),o={includeGlobal:n,includeUnknown:s};i>0&&(o.activeMinutes=i),a>0&&(o.limit=a);const l=await e.client.request("sessions.list",o);l&&(e.sessionsResult=l)}catch(n){e.sessionsError=String(n)}finally{e.sessionsLoading=!1}}}async function Xh(e,t,n){if(!e.client||!e.connected)return;const s={key:t};"label"in n&&(s.label=n.label),"thinkingLevel"in n&&(s.thinkingLevel=n.thinkingLevel),"verboseLevel"in n&&(s.verboseLevel=n.verboseLevel),"reasoningLevel"in n&&(s.reasoningLevel=n.reasoningLevel);try{await e.client.request("sessions.patch",s),await _t(e)}catch(i){e.sessionsError=String(i)}}async function ef(e,t){if(!(!e.client||!e.connected||e.sessionsLoading||!window.confirm(`Delete session "${t}"?

Deletes the session entry and archives its transcript.`))){e.sessionsLoading=!0,e.sessionsError=null;try{await e.client.request("sessions.delete",{key:t,deleteTranscript:!0}),await _t(e)}catch(s){e.sessionsError=String(s)}finally{e.sessionsLoading=!1}}}function Ot(e,t,n){if(!t.trim())return;const s={...e.skillMessages};n?s[t]=n:delete s[t],e.skillMessages=s}function us(e){return e instanceof Error?e.message:String(e)}async function vn(e,t){if(t?.clearMessages&&Object.keys(e.skillMessages).length>0&&(e.skillMessages={}),!(!e.client||!e.connected)&&!e.skillsLoading){e.skillsLoading=!0,e.skillsError=null;try{const n=await e.client.request("skills.status",{});n&&(e.skillsReport=n)}catch(n){e.skillsError=us(n)}finally{e.skillsLoading=!1}}}function tf(e,t,n){e.skillEdits={...e.skillEdits,[t]:n}}async function nf(e,t,n){if(!(!e.client||!e.connected)){e.skillsBusyKey=t,e.skillsError=null;try{await e.client.request("skills.update",{skillKey:t,enabled:n}),await vn(e),Ot(e,t,{kind:"success",message:n?"Skill enabled":"Skill disabled"})}catch(s){const i=us(s);e.skillsError=i,Ot(e,t,{kind:"error",message:i})}finally{e.skillsBusyKey=null}}}async function sf(e,t){if(!(!e.client||!e.connected)){e.skillsBusyKey=t,e.skillsError=null;try{const n=e.skillEdits[t]??"";await e.client.request("skills.update",{skillKey:t,apiKey:n}),await vn(e),Ot(e,t,{kind:"success",message:"API key saved"})}catch(n){const s=us(n);e.skillsError=s,Ot(e,t,{kind:"error",message:s})}finally{e.skillsBusyKey=null}}}async function af(e,t,n,s){if(!(!e.client||!e.connected)){e.skillsBusyKey=t,e.skillsError=null;try{const i=await e.client.request("skills.install",{name:n,installId:s,timeoutMs:12e4});await vn(e),Ot(e,t,{kind:"success",message:i?.message??"Installed"})}catch(i){const a=us(i);e.skillsError=a,Ot(e,t,{kind:"error",message:a})}finally{e.skillsBusyKey=null}}}const of=[{label:"Chat",tabs:["chat"]},{label:"Control",tabs:["overview","channels","instances","sessions","usage","cron"]},{label:"Agent",tabs:["agents","skills","nodes"]},{label:"Settings",tabs:["config","debug","logs"]}],ul={agents:"/agents",overview:"/overview",channels:"/channels",instances:"/instances",sessions:"/sessions",usage:"/usage",cron:"/cron",skills:"/skills",nodes:"/nodes",chat:"/chat",config:"/config",debug:"/debug",logs:"/logs"},gl=new Map(Object.entries(ul).map(([e,t])=>[t,e]));function mn(e){if(!e)return"";let t=e.trim();return t.startsWith("/")||(t=`/${t}`),t==="/"?"":(t.endsWith("/")&&(t=t.slice(0,-1)),t)}function un(e){if(!e)return"/";let t=e.trim();return t.startsWith("/")||(t=`/${t}`),t.length>1&&t.endsWith("/")&&(t=t.slice(0,-1)),t}function gs(e,t=""){const n=mn(t),s=ul[e];return n?`${n}${s}`:s}function pl(e,t=""){const n=mn(t);let s=e||"/";n&&(s===n?s="/":s.startsWith(`${n}/`)&&(s=s.slice(n.length)));let i=un(s).toLowerCase();return i.endsWith("/index.html")&&(i="/"),i==="/"?"chat":gl.get(i)??null}function rf(e){let t=un(e);if(t.endsWith("/index.html")&&(t=un(t.slice(0,-11))),t==="/")return"";const n=t.split("/").filter(Boolean);if(n.length===0)return"";for(let s=0;s<n.length;s++){const i=`/${n.slice(s).join("/")}`.toLowerCase();if(gl.has(i)){const a=n.slice(0,s);return a.length?`/${a.join("/")}`:""}}return`/${n.join("/")}`}function lf(e){switch(e){case"agents":return"folder";case"chat":return"messageSquare";case"overview":return"barChart";case"channels":return"link";case"instances":return"radio";case"sessions":return"fileText";case"usage":return"barChart";case"cron":return"loader";case"skills":return"zap";case"nodes":return"monitor";case"config":return"settings";case"debug":return"bug";case"logs":return"scrollText";default:return"folder"}}function li(e){switch(e){case"agents":return"Agents";case"overview":return"Overview";case"channels":return"Channels";case"instances":return"Instances";case"sessions":return"Sessions";case"usage":return"Usage";case"cron":return"Cron Jobs";case"skills":return"Skills";case"nodes":return"Nodes";case"chat":return"Chat";case"config":return"Config";case"debug":return"Debug";case"logs":return"Logs";default:return"Control"}}function cf(e){switch(e){case"agents":return"Manage agent workspaces, tools, and identities.";case"overview":return"Gateway status, entry points, and a fast health read.";case"channels":return"Manage channels and settings.";case"instances":return"Presence beacons from connected clients and nodes.";case"sessions":return"Inspect active sessions and adjust per-session defaults.";case"usage":return"";case"cron":return"Schedule wakeups and recurring agent runs.";case"skills":return"Manage skill availability and API key injection.";case"nodes":return"Paired devices, capabilities, and command exposure.";case"chat":return"Direct gateway chat session for quick interventions.";case"config":return"Edit ~/.aisopod/aisopod.json safely.";case"debug":return"Gateway snapshots, events, and manual RPC calls.";case"logs":return"Live tail of the gateway file logs.";default:return""}}const hl="aisopod.control.settings.v1";function df(){const t={gatewayUrl:`${location.protocol==="https:"?"wss":"ws"}://${location.host}`,token:"",sessionKey:"main",lastActiveSessionKey:"main",theme:"system",chatFocusMode:!1,chatShowThinking:!0,splitRatio:.6,navCollapsed:!1,navGroupsCollapsed:{}};try{const n=localStorage.getItem(hl);if(!n)return t;const s=JSON.parse(n);return{gatewayUrl:typeof s.gatewayUrl=="string"&&s.gatewayUrl.trim()?s.gatewayUrl.trim():t.gatewayUrl,token:typeof s.token=="string"?s.token:t.token,sessionKey:typeof s.sessionKey=="string"&&s.sessionKey.trim()?s.sessionKey.trim():t.sessionKey,lastActiveSessionKey:typeof s.lastActiveSessionKey=="string"&&s.lastActiveSessionKey.trim()?s.lastActiveSessionKey.trim():typeof s.sessionKey=="string"&&s.sessionKey.trim()||t.lastActiveSessionKey,theme:s.theme==="light"||s.theme==="dark"||s.theme==="system"?s.theme:t.theme,chatFocusMode:typeof s.chatFocusMode=="boolean"?s.chatFocusMode:t.chatFocusMode,chatShowThinking:typeof s.chatShowThinking=="boolean"?s.chatShowThinking:t.chatShowThinking,splitRatio:typeof s.splitRatio=="number"&&s.splitRatio>=.4&&s.splitRatio<=.7?s.splitRatio:t.splitRatio,navCollapsed:typeof s.navCollapsed=="boolean"?s.navCollapsed:t.navCollapsed,navGroupsCollapsed:typeof s.navGroupsCollapsed=="object"&&s.navGroupsCollapsed!==null?s.navGroupsCollapsed:t.navGroupsCollapsed}}catch{return t}}function uf(e){localStorage.setItem(hl,JSON.stringify(e))}const Ln=e=>Number.isNaN(e)?.5:e<=0?0:e>=1?1:e,gf=()=>typeof window>"u"||typeof window.matchMedia!="function"?!1:window.matchMedia("(prefers-reduced-motion: reduce)").matches??!1,Mn=e=>{e.classList.remove("theme-transition"),e.style.removeProperty("--theme-switch-x"),e.style.removeProperty("--theme-switch-y")},pf=({nextTheme:e,applyTheme:t,context:n,currentTheme:s})=>{if(s===e)return;const i=globalThis.document??null;if(!i){t();return}const a=i.documentElement,o=i,l=gf();if(!!o.startViewTransition&&!l){let g=.5,f=.5;if(n?.pointerClientX!==void 0&&n?.pointerClientY!==void 0&&typeof window<"u")g=Ln(n.pointerClientX/window.innerWidth),f=Ln(n.pointerClientY/window.innerHeight);else if(n?.element){const p=n.element.getBoundingClientRect();p.width>0&&p.height>0&&typeof window<"u"&&(g=Ln((p.left+p.width/2)/window.innerWidth),f=Ln((p.top+p.height/2)/window.innerHeight))}a.style.setProperty("--theme-switch-x",`${g*100}%`),a.style.setProperty("--theme-switch-y",`${f*100}%`),a.classList.add("theme-transition");try{const p=o.startViewTransition?.(()=>{t()});p?.finished?p.finished.finally(()=>Mn(a)):Mn(a)}catch{Mn(a),t()}return}t(),Mn(a)};function hf(){return typeof window>"u"||typeof window.matchMedia!="function"||window.matchMedia("(prefers-color-scheme: dark)").matches?"dark":"light"}function Xi(e){return e==="system"?hf():e}function tt(e,t){const n={...t,lastActiveSessionKey:t.lastActiveSessionKey?.trim()||t.sessionKey.trim()||"main"};e.settings=n,uf(n),t.theme!==e.theme&&(e.theme=t.theme,ps(e,Xi(t.theme))),e.applySessionKey=e.settings.lastActiveSessionKey}function fl(e,t){const n=t.trim();n&&e.settings.lastActiveSessionKey!==n&&tt(e,{...e.settings,lastActiveSessionKey:n})}function ff(e){if(!window.location.search&&!window.location.hash)return;const t=new URL(window.location.href),n=new URLSearchParams(t.search),s=new URLSearchParams(t.hash.startsWith("#")?t.hash.slice(1):t.hash),i=n.get("token")??s.get("token"),a=n.get("password")??s.get("password"),o=n.get("session")??s.get("session"),l=n.get("gatewayUrl")??s.get("gatewayUrl");let d=!1;if(i!=null){const f=i.trim();f&&f!==e.settings.token&&tt(e,{...e.settings,token:f}),n.delete("token"),s.delete("token"),d=!0}if(a!=null&&(n.delete("password"),s.delete("password"),d=!0),o!=null){const f=o.trim();f&&(e.sessionKey=f,tt(e,{...e.settings,sessionKey:f,lastActiveSessionKey:f}))}if(l!=null){const f=l.trim();f&&f!==e.settings.gatewayUrl&&(e.pendingGatewayUrl=f),n.delete("gatewayUrl"),s.delete("gatewayUrl"),d=!0}if(!d)return;t.search=n.toString();const g=s.toString();t.hash=g?`#${g}`:"",window.history.replaceState({},"",t.toString())}function vf(e,t){e.tab!==t&&(e.tab=t),t==="chat"&&(e.chatHasAutoScrolled=!1),t==="logs"?Oi(e):Bi(e),t==="debug"?Ui(e):zi(e),ea(e),ml(e,t,!1)}function mf(e,t,n){pf({nextTheme:t,applyTheme:()=>{e.theme=t,tt(e,{...e.settings,theme:t}),ps(e,Xi(t))},context:n,currentTheme:e.theme})}async function ea(e){if(e.tab==="overview"&&await bl(e),e.tab==="channels"&&await Af(e),e.tab==="instances"&&await Zi(e),e.tab==="sessions"&&await _t(e),e.tab==="cron"&&await Jn(e),e.tab==="skills"&&await vn(e),e.tab==="agents"){await Hi(e),await Ie(e);const t=e.agentsList?.agents?.map(s=>s.id)??[];t.length>0&&jr(e,t);const n=e.agentsSelectedId??e.agentsList?.defaultId??e.agentsList?.agents?.[0]?.id;n&&(Kr(e,n),e.agentsPanel==="skills"&&Kn(e,n),e.agentsPanel==="channels"&&xe(e,!1),e.agentsPanel==="cron"&&Jn(e))}e.tab==="nodes"&&(await ls(e),await st(e),await Ie(e),await Ji(e)),e.tab==="chat"&&(await Al(e),hn(e,!e.chatHasAutoScrolled)),e.tab==="config"&&(await Fr(e),await Ie(e)),e.tab==="debug"&&(await rs(e),e.eventLog=e.eventLogBuffer),e.tab==="logs"&&(e.logsAtBottom=!0,await Ni(e,{reset:!0}),Hr(e,!0))}function bf(){if(typeof window>"u")return"";const e=window.__AISOPOD_CONTROL_UI_BASE_PATH__;return typeof e=="string"&&e.trim()?mn(e):rf(window.location.pathname)}function yf(e){e.theme=e.settings.theme??"system",ps(e,Xi(e.theme))}function ps(e,t){if(e.themeResolved=t,typeof document>"u")return;const n=document.documentElement;n.dataset.theme=t,n.style.colorScheme=t}function xf(e){if(typeof window>"u"||typeof window.matchMedia!="function")return;if(e.themeMedia=window.matchMedia("(prefers-color-scheme: dark)"),e.themeMediaHandler=n=>{e.theme==="system"&&ps(e,n.matches?"dark":"light")},typeof e.themeMedia.addEventListener=="function"){e.themeMedia.addEventListener("change",e.themeMediaHandler);return}e.themeMedia.addListener(e.themeMediaHandler)}function $f(e){if(!e.themeMedia||!e.themeMediaHandler)return;if(typeof e.themeMedia.removeEventListener=="function"){e.themeMedia.removeEventListener("change",e.themeMediaHandler);return}e.themeMedia.removeListener(e.themeMediaHandler),e.themeMedia=null,e.themeMediaHandler=null}function wf(e,t){if(typeof window>"u")return;const n=pl(window.location.pathname,e.basePath)??"chat";vl(e,n),ml(e,n,t)}function kf(e){if(typeof window>"u")return;const t=pl(window.location.pathname,e.basePath);if(!t)return;const s=new URL(window.location.href).searchParams.get("session")?.trim();s&&(e.sessionKey=s,tt(e,{...e.settings,sessionKey:s,lastActiveSessionKey:s})),vl(e,t)}function vl(e,t){e.tab!==t&&(e.tab=t),t==="chat"&&(e.chatHasAutoScrolled=!1),t==="logs"?Oi(e):Bi(e),t==="debug"?Ui(e):zi(e),e.connected&&ea(e)}function ml(e,t,n){if(typeof window>"u")return;const s=un(gs(t,e.basePath)),i=un(window.location.pathname),a=new URL(window.location.href);t==="chat"&&e.sessionKey?a.searchParams.set("session",e.sessionKey):a.searchParams.delete("session"),i!==s&&(a.pathname=s),n?window.history.replaceState({},"",a.toString()):window.history.pushState({},"",a.toString())}function Sf(e,t,n){if(typeof window>"u")return;const s=new URL(window.location.href);s.searchParams.set("session",t),window.history.replaceState({},"",s.toString())}async function bl(e){await Promise.all([xe(e,!1),Zi(e),_t(e),fn(e),rs(e)])}async function Af(e){await Promise.all([xe(e,!0),Fr(e),Ie(e)])}async function Jn(e){await Promise.all([xe(e,!1),fn(e),cs(e)])}const po=50,_f=80,Cf=12e4;function Tf(e){if(!e||typeof e!="object")return null;const t=e;if(typeof t.text=="string")return t.text;const n=t.content;if(!Array.isArray(n))return null;const s=n.map(i=>{if(!i||typeof i!="object")return null;const a=i;return a.type==="text"&&typeof a.text=="string"?a.text:null}).filter(i=>!!i);return s.length===0?null:s.join(`
`)}function ho(e){if(e==null)return null;if(typeof e=="number"||typeof e=="boolean")return String(e);const t=Tf(e);let n;if(typeof e=="string")n=e;else if(t)n=t;else try{n=JSON.stringify(e,null,2)}catch{n=String(e)}const s=Wr(n,Cf);return s.truncated?`${s.text}

… truncated (${s.total} chars, showing first ${s.text.length}).`:s.text}function Ef(e){const t=[];return t.push({type:"toolcall",name:e.name,arguments:e.args??{}}),e.output&&t.push({type:"toolresult",name:e.name,text:e.output}),{role:"assistant",toolCallId:e.toolCallId,runId:e.runId,content:t,timestamp:e.startedAt}}function Lf(e){if(e.toolStreamOrder.length<=po)return;const t=e.toolStreamOrder.length-po,n=e.toolStreamOrder.splice(0,t);for(const s of n)e.toolStreamById.delete(s)}function Mf(e){e.chatToolMessages=e.toolStreamOrder.map(t=>e.toolStreamById.get(t)?.message).filter(t=>!!t)}function ci(e){e.toolStreamSyncTimer!=null&&(clearTimeout(e.toolStreamSyncTimer),e.toolStreamSyncTimer=null),Mf(e)}function If(e,t=!1){if(t){ci(e);return}e.toolStreamSyncTimer==null&&(e.toolStreamSyncTimer=window.setTimeout(()=>ci(e),_f))}function hs(e){e.toolStreamById.clear(),e.toolStreamOrder=[],e.chatToolMessages=[],ci(e)}const Rf=5e3;function Pf(e,t){const n=t.data??{},s=typeof n.phase=="string"?n.phase:"";e.compactionClearTimer!=null&&(window.clearTimeout(e.compactionClearTimer),e.compactionClearTimer=null),s==="start"?e.compactionStatus={active:!0,startedAt:Date.now(),completedAt:null}:s==="end"&&(e.compactionStatus={active:!1,startedAt:e.compactionStatus?.startedAt??null,completedAt:Date.now()},e.compactionClearTimer=window.setTimeout(()=>{e.compactionStatus=null,e.compactionClearTimer=null},Rf))}function Df(e,t){if(!t)return;if(t.stream==="compaction"){Pf(e,t);return}if(t.stream!=="tool")return;const n=typeof t.sessionKey=="string"?t.sessionKey:void 0;if(n&&n!==e.sessionKey||!n&&e.chatRunId&&t.runId!==e.chatRunId||e.chatRunId&&t.runId!==e.chatRunId||!e.chatRunId)return;const s=t.data??{},i=typeof s.toolCallId=="string"?s.toolCallId:"";if(!i)return;const a=typeof s.name=="string"?s.name:"tool",o=typeof s.phase=="string"?s.phase:"",l=o==="start"?s.args:void 0,d=o==="update"?ho(s.partialResult):o==="result"?ho(s.result):void 0,g=Date.now();let f=e.toolStreamById.get(i);f?(f.name=a,l!==void 0&&(f.args=l),d!==void 0&&(f.output=d||void 0),f.updatedAt=g):(f={toolCallId:i,runId:t.runId,sessionKey:n,name:a,args:l,output:d||void 0,startedAt:typeof t.ts=="number"?t.ts:g,updatedAt:g,message:{}},e.toolStreamById.set(i,f),e.toolStreamOrder.push(i)),f.message=Ef(f),Lf(e),If(e,o==="result")}const Ff=/^\[([^\]]+)\]\s*/,Nf=["WebChat","WhatsApp","Telegram","Signal","Slack","Discord","Google Chat","iMessage","Teams","Matrix","Zalo","Zalo Personal","BlueBubbles"];function Of(e){return/\d{4}-\d{2}-\d{2}T\d{2}:\d{2}Z\b/.test(e)||/\d{4}-\d{2}-\d{2} \d{2}:\d{2}\b/.test(e)?!0:Nf.some(t=>e.startsWith(`${t} `))}function Fs(e){const t=e.match(Ff);if(!t)return e;const n=t[1]??"";return Of(n)?e.slice(t[0].length):e}const Ns=new WeakMap,Os=new WeakMap;function di(e){const t=e,n=typeof t.role=="string"?t.role:"",s=t.content;if(typeof s=="string")return n==="assistant"?Ms(s):Fs(s);if(Array.isArray(s)){const i=s.map(a=>{const o=a;return o.type==="text"&&typeof o.text=="string"?o.text:null}).filter(a=>typeof a=="string");if(i.length>0){const a=i.join(`
`);return n==="assistant"?Ms(a):Fs(a)}}return typeof t.text=="string"?n==="assistant"?Ms(t.text):Fs(t.text):null}function yl(e){if(!e||typeof e!="object")return di(e);const t=e;if(Ns.has(t))return Ns.get(t)??null;const n=di(e);return Ns.set(t,n),n}function fo(e){const n=e.content,s=[];if(Array.isArray(n))for(const l of n){const d=l;if(d.type==="thinking"&&typeof d.thinking=="string"){const g=d.thinking.trim();g&&s.push(g)}}if(s.length>0)return s.join(`
`);const i=Uf(e);if(!i)return null;const o=[...i.matchAll(/<\s*think(?:ing)?\s*>([\s\S]*?)<\s*\/\s*think(?:ing)?\s*>/gi)].map(l=>(l[1]??"").trim()).filter(Boolean);return o.length>0?o.join(`
`):null}function Bf(e){if(!e||typeof e!="object")return fo(e);const t=e;if(Os.has(t))return Os.get(t)??null;const n=fo(e);return Os.set(t,n),n}function Uf(e){const t=e,n=t.content;if(typeof n=="string")return n;if(Array.isArray(n)){const s=n.map(i=>{const a=i;return a.type==="text"&&typeof a.text=="string"?a.text:null}).filter(i=>typeof i=="string");if(s.length>0)return s.join(`
`)}return typeof t.text=="string"?t.text:null}function zf(e){const t=e.trim();if(!t)return"";const n=t.split(/\r?\n/).map(s=>s.trim()).filter(Boolean).map(s=>`_${s}_`);return n.length?["_Reasoning:_",...n].join(`
`):""}let vo=!1;function mo(e){e[6]=e[6]&15|64,e[8]=e[8]&63|128;let t="";for(let n=0;n<e.length;n++)t+=e[n].toString(16).padStart(2,"0");return`${t.slice(0,8)}-${t.slice(8,12)}-${t.slice(12,16)}-${t.slice(16,20)}-${t.slice(20)}`}function Hf(){const e=new Uint8Array(16),t=Date.now();for(let n=0;n<e.length;n++)e[n]=Math.floor(Math.random()*256);return e[0]^=t&255,e[1]^=t>>>8&255,e[2]^=t>>>16&255,e[3]^=t>>>24&255,e}function Kf(){vo||(vo=!0,console.warn("[uuid] crypto API missing; falling back to weak randomness"))}function ta(e=globalThis.crypto){if(e&&typeof e.randomUUID=="function")return e.randomUUID();if(e&&typeof e.getRandomValues=="function"){const t=new Uint8Array(16);return e.getRandomValues(t),mo(t)}return Kf(),mo(Hf())}async function gn(e){if(!(!e.client||!e.connected)){e.chatLoading=!0,e.lastError=null;try{const t=await e.client.request("chat.history",{sessionKey:e.sessionKey,limit:200});e.chatMessages=Array.isArray(t.messages)?t.messages:[],e.chatThinkingLevel=t.thinkingLevel??null}catch(t){e.lastError=String(t)}finally{e.chatLoading=!1}}}function jf(e){const t=/^data:([^;]+);base64,(.+)$/.exec(e);return t?{mimeType:t[1],content:t[2]}:null}async function Wf(e,t,n){if(!e.client||!e.connected)return null;const s=t.trim(),i=n&&n.length>0;if(!s&&!i)return null;const a=Date.now(),o=[];if(s&&o.push({type:"text",text:s}),i)for(const g of n)o.push({type:"image",source:{type:"base64",media_type:g.mimeType,data:g.dataUrl}});e.chatMessages=[...e.chatMessages,{role:"user",content:o,timestamp:a}],e.chatSending=!0,e.lastError=null;const l=ta();e.chatRunId=l,e.chatStream="",e.chatStreamStartedAt=a;const d=i?n.map(g=>{const f=jf(g.dataUrl);return f?{type:"image",mimeType:f.mimeType,content:f.content}:null}).filter(g=>g!==null):void 0;try{return await e.client.request("chat.send",{sessionKey:e.sessionKey,message:s,deliver:!1,idempotencyKey:l,attachments:d}),l}catch(g){const f=String(g);return e.chatRunId=null,e.chatStream=null,e.chatStreamStartedAt=null,e.lastError=f,e.chatMessages=[...e.chatMessages,{role:"assistant",content:[{type:"text",text:"Error: "+f}],timestamp:Date.now()}],null}finally{e.chatSending=!1}}async function qf(e){if(!e.client||!e.connected)return!1;const t=e.chatRunId;try{return await e.client.request("chat.abort",t?{sessionKey:e.sessionKey,runId:t}:{sessionKey:e.sessionKey}),!0}catch(n){return e.lastError=String(n),!1}}function Gf(e,t){if(!t||t.sessionKey!==e.sessionKey)return null;if(t.runId&&e.chatRunId&&t.runId!==e.chatRunId)return t.state==="final"?"final":null;if(t.state==="delta"){const n=di(t.message);if(typeof n=="string"){const s=e.chatStream??"";(!s||n.length>=s.length)&&(e.chatStream=n)}}else t.state==="final"||t.state==="aborted"?(e.chatStream=null,e.chatRunId=null,e.chatStreamStartedAt=null):t.state==="error"&&(e.chatStream=null,e.chatRunId=null,e.chatStreamStartedAt=null,e.lastError=t.errorMessage??"chat error");return t.state}const xl=120;function $l(e){return e.chatSending||!!e.chatRunId}function Vf(e){const t=e.trim();if(!t)return!1;const n=t.toLowerCase();return n==="/stop"?!0:n==="stop"||n==="esc"||n==="abort"||n==="wait"||n==="exit"}function Qf(e){const t=e.trim();if(!t)return!1;const n=t.toLowerCase();return n==="/new"||n==="/reset"?!0:n.startsWith("/new ")||n.startsWith("/reset ")}async function wl(e){e.connected&&(e.chatMessage="",await qf(e))}function Yf(e,t,n,s){const i=t.trim(),a=!!(n&&n.length>0);!i&&!a||(e.chatQueue=[...e.chatQueue,{id:ta(),text:i,createdAt:Date.now(),attachments:a?n?.map(o=>({...o})):void 0,refreshSessions:s}])}async function kl(e,t,n){hs(e);const s=await Wf(e,t,n?.attachments),i=!!s;return!i&&n?.previousDraft!=null&&(e.chatMessage=n.previousDraft),!i&&n?.previousAttachments&&(e.chatAttachments=n.previousAttachments),i&&fl(e,e.sessionKey),i&&n?.restoreDraft&&n.previousDraft?.trim()&&(e.chatMessage=n.previousDraft),i&&n?.restoreAttachments&&n.previousAttachments?.length&&(e.chatAttachments=n.previousAttachments),hn(e),i&&!e.chatRunId&&Sl(e),i&&n?.refreshSessions&&s&&e.refreshSessionsAfterChat.add(s),i}async function Sl(e){if(!e.connected||$l(e))return;const[t,...n]=e.chatQueue;if(!t)return;e.chatQueue=n,await kl(e,t.text,{attachments:t.attachments,refreshSessions:t.refreshSessions})||(e.chatQueue=[t,...e.chatQueue])}function Jf(e,t){e.chatQueue=e.chatQueue.filter(n=>n.id!==t)}async function Zf(e,t,n){if(!e.connected)return;const s=e.chatMessage,i=(t??e.chatMessage).trim(),a=e.chatAttachments??[],o=t==null?a:[],l=o.length>0;if(!i&&!l)return;if(Vf(i)){await wl(e);return}const d=Qf(i);if(t==null&&(e.chatMessage="",e.chatAttachments=[]),$l(e)){Yf(e,i,o,d);return}await kl(e,i,{previousDraft:t==null?s:void 0,restoreDraft:!!(t&&n?.restoreDraft),attachments:l?o:void 0,previousAttachments:t==null?a:void 0,restoreAttachments:!!(t&&n?.restoreDraft),refreshSessions:d})}async function Al(e,t){await Promise.all([gn(e),_t(e,{activeMinutes:xl}),ui(e)]),t?.scheduleScroll!==!1&&hn(e)}const Xf=Sl;function ev(e){const t=zr(e.sessionKey);return t?.agentId?t.agentId:e.hello?.snapshot?.sessionDefaults?.defaultAgentId?.trim()||"main"}function tv(e,t){const n=mn(e),s=encodeURIComponent(t);return n?`${n}/avatar/${s}?meta=1`:`/avatar/${s}?meta=1`}async function ui(e){if(!e.connected){e.chatAvatarUrl=null;return}const t=ev(e);if(!t){e.chatAvatarUrl=null;return}e.chatAvatarUrl=null;const n=tv(e.basePath,t);try{const s=await fetch(n,{method:"GET"});if(!s.ok){e.chatAvatarUrl=null;return}const i=await s.json(),a=typeof i.avatarUrl=="string"?i.avatarUrl.trim():"";e.chatAvatarUrl=a||null}catch{e.chatAvatarUrl=null}}const nv={trace:!0,debug:!0,info:!0,warn:!0,error:!0,fatal:!0},sv={name:"",description:"",agentId:"",enabled:!0,scheduleKind:"every",scheduleAt:"",everyAmount:"30",everyUnit:"minutes",cronExpr:"0 7 * * *",cronTz:"",sessionTarget:"isolated",wakeMode:"now",payloadKind:"agentTurn",payloadText:"",deliveryMode:"announce",deliveryChannel:"last",deliveryTo:"",timeoutSeconds:""},iv=50,av=200,ov="Assistant";function bo(e,t){if(typeof e!="string")return;const n=e.trim();if(n)return n.length<=t?n:n.slice(0,t)}function gi(e){const t=bo(e?.name,iv)??ov,n=bo(e?.avatar??void 0,av)??null;return{agentId:typeof e?.agentId=="string"&&e.agentId.trim()?e.agentId.trim():null,name:t,avatar:n}}function rv(){return gi(typeof window>"u"?{}:{name:window.__AISOPOD_ASSISTANT_NAME__,avatar:window.__AISOPOD_ASSISTANT_AVATAR__})}async function _l(e,t){if(!e.client||!e.connected)return;const n=e.sessionKey.trim(),s=n?{sessionKey:n}:{};try{const i=await e.client.request("agent.identity.get",s);if(!i)return;const a=gi(i);e.assistantName=a.name,e.assistantAvatar=a.avatar,e.assistantAgentId=a.agentId??null}catch{}}function pi(e){return typeof e=="object"&&e!==null}function lv(e){if(!pi(e))return null;const t=typeof e.id=="string"?e.id.trim():"",n=e.request;if(!t||!pi(n))return null;const s=typeof n.command=="string"?n.command.trim():"";if(!s)return null;const i=typeof e.createdAtMs=="number"?e.createdAtMs:0,a=typeof e.expiresAtMs=="number"?e.expiresAtMs:0;return!i||!a?null:{id:t,request:{command:s,cwd:typeof n.cwd=="string"?n.cwd:null,host:typeof n.host=="string"?n.host:null,security:typeof n.security=="string"?n.security:null,ask:typeof n.ask=="string"?n.ask:null,agentId:typeof n.agentId=="string"?n.agentId:null,resolvedPath:typeof n.resolvedPath=="string"?n.resolvedPath:null,sessionKey:typeof n.sessionKey=="string"?n.sessionKey:null},createdAtMs:i,expiresAtMs:a}}function cv(e){if(!pi(e))return null;const t=typeof e.id=="string"?e.id.trim():"";return t?{id:t,decision:typeof e.decision=="string"?e.decision:null,resolvedBy:typeof e.resolvedBy=="string"?e.resolvedBy:null,ts:typeof e.ts=="number"?e.ts:null}:null}function Cl(e){const t=Date.now();return e.filter(n=>n.expiresAtMs>t)}function dv(e,t){const n=Cl(e).filter(s=>s.id!==t.id);return n.push(t),n}function yo(e,t){return Cl(e).filter(n=>n.id!==t)}function uv(e){const t=e.version??(e.nonce?"v2":"v1"),n=e.scopes.join(","),s=e.token??"",i=[t,e.deviceId,e.clientId,e.clientMode,e.role,n,String(e.signedAtMs),s];return t==="v2"&&i.push(e.nonce??""),i.join("|")}const Tl={WEBCHAT_UI:"webchat-ui",CONTROL_UI:"openclaw-control-ui",WEBCHAT:"webchat",CLI:"cli",GATEWAY_CLIENT:"gateway-client",MACOS_APP:"openclaw-macos",IOS_APP:"openclaw-ios",ANDROID_APP:"openclaw-android",NODE_HOST:"node-host",TEST:"test",FINGERPRINT:"fingerprint",PROBE:"openclaw-probe"},xo=Tl,hi={WEBCHAT:"webchat",CLI:"cli",UI:"ui",BACKEND:"backend",NODE:"node",PROBE:"probe",TEST:"test"};new Set(Object.values(Tl));new Set(Object.values(hi));const gv=4008;class pv{constructor(t){this.opts=t,this.ws=null,this.pending=new Map,this.closed=!1,this.lastSeq=null,this.connectNonce=null,this.connectSent=!1,this.connectTimer=null,this.backoffMs=800}start(){this.closed=!1,this.connect()}stop(){this.closed=!0,this.ws?.close(),this.ws=null,this.flushPending(new Error("gateway client stopped"))}get connected(){return this.ws?.readyState===WebSocket.OPEN}connect(){this.closed||(this.ws=new WebSocket(this.opts.url),this.ws.addEventListener("open",()=>this.queueConnect()),this.ws.addEventListener("message",t=>this.handleMessage(String(t.data??""))),this.ws.addEventListener("close",t=>{const n=String(t.reason??"");this.ws=null,this.flushPending(new Error(`gateway closed (${t.code}): ${n}`)),this.opts.onClose?.({code:t.code,reason:n}),this.scheduleReconnect()}),this.ws.addEventListener("error",()=>{}))}scheduleReconnect(){if(this.closed)return;const t=this.backoffMs;this.backoffMs=Math.min(this.backoffMs*1.7,15e3),window.setTimeout(()=>this.connect(),t)}flushPending(t){for(const[,n]of this.pending)n.reject(t);this.pending.clear()}async sendConnect(){if(this.connectSent)return;this.connectSent=!0,this.connectTimer!==null&&(window.clearTimeout(this.connectTimer),this.connectTimer=null);const t=typeof crypto<"u"&&!!crypto.subtle,n=["operator.admin","operator.approvals","operator.pairing"],s="operator";let i=null,a=!1,o=this.opts.token;if(t){i=await Yi();const f=mh({deviceId:i.deviceId,role:s})?.token;o=f??this.opts.token,a=!!(f&&this.opts.token)}const l=o||this.opts.password?{token:o,password:this.opts.password}:void 0;let d;if(t&&i){const f=Date.now(),p=this.connectNonce??void 0,b=uv({deviceId:i.deviceId,clientId:this.opts.clientName??xo.CONTROL_UI,clientMode:this.opts.mode??hi.WEBCHAT,role:s,scopes:n,signedAtMs:f,token:o??null,nonce:p}),u=await Hh(i.privateKey,b);d={id:i.deviceId,publicKey:i.publicKey,signature:u,signedAt:f,nonce:p}}const g={minProtocol:3,maxProtocol:3,client:{id:this.opts.clientName??xo.CONTROL_UI,version:this.opts.clientVersion??"dev",platform:this.opts.platform??navigator.platform??"web",mode:this.opts.mode??hi.WEBCHAT,instanceId:this.opts.instanceId},role:s,scopes:n,device:d,caps:[],auth:l,userAgent:navigator.userAgent,locale:navigator.language};this.request("connect",g).then(f=>{f?.auth?.deviceToken&&i&&Qr({deviceId:i.deviceId,role:f.auth.role??s,token:f.auth.deviceToken,scopes:f.auth.scopes??[]}),this.backoffMs=800,this.opts.onHello?.(f)}).catch(()=>{a&&i&&Yr({deviceId:i.deviceId,role:s}),this.ws?.close(gv,"connect failed")})}handleMessage(t){let n;try{n=JSON.parse(t)}catch{return}const s=n;if(s.type==="event"){const i=n;if(i.event==="connect.challenge"){const o=i.payload,l=o&&typeof o.nonce=="string"?o.nonce:null;l&&(this.connectNonce=l,this.sendConnect());return}const a=typeof i.seq=="number"?i.seq:null;a!==null&&(this.lastSeq!==null&&a>this.lastSeq+1&&this.opts.onGap?.({expected:this.lastSeq+1,received:a}),this.lastSeq=a);try{this.opts.onEvent?.(i)}catch(o){console.error("[gateway] event handler error:",o)}return}if(s.type==="res"){const i=n,a=this.pending.get(i.id);if(!a)return;this.pending.delete(i.id),i.ok?a.resolve(i.payload):a.reject(new Error(i.error?.message??"request failed"));return}}request(t,n){if(!this.ws||this.ws.readyState!==WebSocket.OPEN)return Promise.reject(new Error("gateway not connected"));const s=ta(),i={type:"req",id:s,method:t,params:n},a=new Promise((o,l)=>{this.pending.set(s,{resolve:d=>o(d),reject:l})});return this.ws.send(JSON.stringify(i)),a}queueConnect(){this.connectNonce=null,this.connectSent=!1,this.connectTimer!==null&&window.clearTimeout(this.connectTimer),this.connectTimer=window.setTimeout(()=>{this.sendConnect()},750)}}function Bs(e,t){const n=(e??"").trim(),s=t.mainSessionKey?.trim();if(!s)return n;if(!n)return s;const i=t.mainKey?.trim()||"main",a=t.defaultAgentId?.trim();return n==="main"||n===i||a&&(n===`agent:${a}:main`||n===`agent:${a}:${i}`)?s:n}function hv(e,t){if(!t?.mainSessionKey)return;const n=Bs(e.sessionKey,t),s=Bs(e.settings.sessionKey,t),i=Bs(e.settings.lastActiveSessionKey,t),a=n||s||e.sessionKey,o={...e.settings,sessionKey:s||a,lastActiveSessionKey:i||a},l=o.sessionKey!==e.settings.sessionKey||o.lastActiveSessionKey!==e.settings.lastActiveSessionKey;a!==e.sessionKey&&(e.sessionKey=a),l&&tt(e,o)}function El(e){e.lastError=null,e.hello=null,e.connected=!1,e.execApprovalQueue=[],e.execApprovalError=null;const t=e.client,n=new pv({url:e.settings.gatewayUrl,token:e.settings.token.trim()?e.settings.token:void 0,password:e.password.trim()?e.password:void 0,clientName:"aisopod-control-ui",mode:"webchat",onHello:s=>{e.client===n&&(e.connected=!0,e.lastError=null,e.hello=s,mv(e,s),e.chatRunId=null,e.chatStream=null,e.chatStreamStartedAt=null,hs(e),_l(e),Hi(e),ls(e,{quiet:!0}),st(e,{quiet:!0}),ea(e))},onClose:({code:s,reason:i})=>{e.client===n&&(e.connected=!1,s!==1012&&(e.lastError=`disconnected (${s}): ${i||"no reason"}`))},onEvent:s=>{e.client===n&&fv(e,s)},onGap:({expected:s,received:i})=>{e.client===n&&(e.lastError=`event gap detected (expected seq ${s}, got ${i}); refresh recommended`)}});e.client=n,t?.stop(),n.start()}function fv(e,t){try{vv(e,t)}catch(n){console.error("[gateway] handleGatewayEvent error:",t.event,n)}}function vv(e,t){if(e.eventLogBuffer=[{ts:Date.now(),event:t.event,payload:t.payload},...e.eventLogBuffer].slice(0,250),e.tab==="debug"&&(e.eventLog=e.eventLogBuffer),t.event==="agent"){if(e.onboarding)return;Df(e,t.payload);return}if(t.event==="chat"){const n=t.payload;n?.sessionKey&&fl(e,n.sessionKey);const s=Gf(e,n);if(s==="final"||s==="error"||s==="aborted"){hs(e),Xf(e);const i=n?.runId;i&&e.refreshSessionsAfterChat.has(i)&&(e.refreshSessionsAfterChat.delete(i),s==="final"&&_t(e,{activeMinutes:xl}))}s==="final"&&gn(e);return}if(t.event==="presence"){const n=t.payload;n?.presence&&Array.isArray(n.presence)&&(e.presenceEntries=n.presence,e.presenceError=null,e.presenceStatus=null);return}if(t.event==="cron"&&e.tab==="cron"&&Jn(e),(t.event==="device.pair.requested"||t.event==="device.pair.resolved")&&st(e,{quiet:!0}),t.event==="exec.approval.requested"){const n=lv(t.payload);if(n){e.execApprovalQueue=dv(e.execApprovalQueue,n),e.execApprovalError=null;const s=Math.max(0,n.expiresAtMs-Date.now()+500);window.setTimeout(()=>{e.execApprovalQueue=yo(e.execApprovalQueue,n.id)},s)}return}if(t.event==="exec.approval.resolved"){const n=cv(t.payload);n&&(e.execApprovalQueue=yo(e.execApprovalQueue,n.id))}}function mv(e,t){const n=t.snapshot;n?.presence&&Array.isArray(n.presence)&&(e.presenceEntries=n.presence),n?.health&&(e.debugHealth=n.health),n?.sessionDefaults&&hv(e,n.sessionDefaults)}function bv(e){e.basePath=bf(),ff(e),wf(e,!0),yf(e),xf(e),window.addEventListener("popstate",e.popStateHandler),El(e),ah(e),e.tab==="logs"&&Oi(e),e.tab==="debug"&&Ui(e)}function yv(e){Zp(e)}function xv(e){window.removeEventListener("popstate",e.popStateHandler),oh(e),Bi(e),zi(e),$f(e),e.topbarObserver?.disconnect(),e.topbarObserver=null}function $v(e,t){if(!(e.tab==="chat"&&e.chatManualRefreshInFlight)){if(e.tab==="chat"&&(t.has("chatMessages")||t.has("chatToolMessages")||t.has("chatStream")||t.has("chatLoading")||t.has("tab"))){const n=t.has("tab"),s=t.has("chatLoading")&&t.get("chatLoading")===!0&&!e.chatLoading;hn(e,n||s||!e.chatHasAutoScrolled)}e.tab==="logs"&&(t.has("logsEntries")||t.has("logsAutoFollow")||t.has("tab"))&&e.logsAutoFollow&&e.logsAtBottom&&Hr(e,t.has("tab")||t.has("logsAutoFollow"))}}async function Ll(e,t){if(!(!e.client||!e.connected)&&!e.usageLoading){e.usageLoading=!0,e.usageError=null;try{const n=t?.startDate??e.usageStartDate,s=t?.endDate??e.usageEndDate,[i,a]=await Promise.all([e.client.request("sessions.usage",{startDate:n,endDate:s,limit:1e3,includeContextWeight:!0}),e.client.request("usage.cost",{startDate:n,endDate:s})]);i&&(e.usageResult=i),a&&(e.usageCostSummary=a)}catch(n){e.usageError=String(n)}finally{e.usageLoading=!1}}}async function wv(e,t){if(!(!e.client||!e.connected)&&!e.usageTimeSeriesLoading){e.usageTimeSeriesLoading=!0,e.usageTimeSeries=null;try{const n=await e.client.request("sessions.usage.timeseries",{key:t});n&&(e.usageTimeSeries=n)}catch{e.usageTimeSeries=null}finally{e.usageTimeSeriesLoading=!1}}}async function kv(e,t){if(!(!e.client||!e.connected)&&!e.usageSessionLogsLoading){e.usageSessionLogsLoading=!0,e.usageSessionLogs=null;try{const n=await e.client.request("sessions.usage.logs",{key:t,limit:500});n&&Array.isArray(n.logs)&&(e.usageSessionLogs=n.logs)}catch{e.usageSessionLogs=null}finally{e.usageSessionLogsLoading=!1}}}const Sv=new Set(["agent","channel","chat","provider","model","tool","label","key","session","id","has","mintokens","maxtokens","mincost","maxcost","minmessages","maxmessages"]),Zn=e=>e.trim().toLowerCase(),Av=e=>{const t=e.replace(/[.+^${}()|[\]\\]/g,"\\$&").replace(/\*/g,".*").replace(/\?/g,".");return new RegExp(`^${t}$`,"i")},gt=e=>{let t=e.trim().toLowerCase();if(!t)return null;t.startsWith("$")&&(t=t.slice(1));let n=1;t.endsWith("k")?(n=1e3,t=t.slice(0,-1)):t.endsWith("m")&&(n=1e6,t=t.slice(0,-1));const s=Number(t);return Number.isFinite(s)?s*n:null},na=e=>(e.match(/"[^"]+"|\S+/g)??[]).map(n=>{const s=n.replace(/^"|"$/g,""),i=s.indexOf(":");if(i>0){const a=s.slice(0,i),o=s.slice(i+1);return{key:a,value:o,raw:s}}return{value:s,raw:s}}),_v=e=>[e.label,e.key,e.sessionId].filter(n=>!!n).map(n=>n.toLowerCase()),$o=e=>{const t=new Set;e.modelProvider&&t.add(e.modelProvider.toLowerCase()),e.providerOverride&&t.add(e.providerOverride.toLowerCase()),e.origin?.provider&&t.add(e.origin.provider.toLowerCase());for(const n of e.usage?.modelUsage??[])n.provider&&t.add(n.provider.toLowerCase());return Array.from(t)},wo=e=>{const t=new Set;e.model&&t.add(e.model.toLowerCase());for(const n of e.usage?.modelUsage??[])n.model&&t.add(n.model.toLowerCase());return Array.from(t)},Cv=e=>(e.usage?.toolUsage?.tools??[]).map(t=>t.name.toLowerCase()),Tv=(e,t)=>{const n=Zn(t.value??"");if(!n)return!0;if(!t.key)return _v(e).some(i=>i.includes(n));switch(Zn(t.key)){case"agent":return e.agentId?.toLowerCase().includes(n)??!1;case"channel":return e.channel?.toLowerCase().includes(n)??!1;case"chat":return e.chatType?.toLowerCase().includes(n)??!1;case"provider":return $o(e).some(i=>i.includes(n));case"model":return wo(e).some(i=>i.includes(n));case"tool":return Cv(e).some(i=>i.includes(n));case"label":return e.label?.toLowerCase().includes(n)??!1;case"key":case"session":case"id":if(n.includes("*")||n.includes("?")){const i=Av(n);return i.test(e.key)||(e.sessionId?i.test(e.sessionId):!1)}return e.key.toLowerCase().includes(n)||(e.sessionId?.toLowerCase().includes(n)??!1);case"has":switch(n){case"tools":return(e.usage?.toolUsage?.totalCalls??0)>0;case"errors":return(e.usage?.messageCounts?.errors??0)>0;case"context":return!!e.contextWeight;case"usage":return!!e.usage;case"model":return wo(e).length>0;case"provider":return $o(e).length>0;default:return!0}case"mintokens":{const i=gt(n);return i===null?!0:(e.usage?.totalTokens??0)>=i}case"maxtokens":{const i=gt(n);return i===null?!0:(e.usage?.totalTokens??0)<=i}case"mincost":{const i=gt(n);return i===null?!0:(e.usage?.totalCost??0)>=i}case"maxcost":{const i=gt(n);return i===null?!0:(e.usage?.totalCost??0)<=i}case"minmessages":{const i=gt(n);return i===null?!0:(e.usage?.messageCounts?.total??0)>=i}case"maxmessages":{const i=gt(n);return i===null?!0:(e.usage?.messageCounts?.total??0)<=i}default:return!0}},Ev=(e,t)=>{const n=na(t);if(n.length===0)return{sessions:e,warnings:[]};const s=[];for(const a of n){if(!a.key)continue;const o=Zn(a.key);if(!Sv.has(o)){s.push(`Unknown filter: ${a.key}`);continue}if(a.value===""&&s.push(`Missing value for ${a.key}`),o==="has"){const l=new Set(["tools","errors","context","usage","model","provider"]);a.value&&!l.has(Zn(a.value))&&s.push(`Unknown has:${a.value}`)}["mintokens","maxtokens","mincost","maxcost","minmessages","maxmessages"].includes(o)&&a.value&&gt(a.value)===null&&s.push(`Invalid number for ${a.key}`)}return{sessions:e.filter(a=>n.every(o=>Tv(a,o))),warnings:s}};function Lv(e){const t=e.split(`
`),n=new Map,s=[];for(const l of t){const d=/^\[Tool:\s*([^\]]+)\]/.exec(l.trim());if(d){const g=d[1];n.set(g,(n.get(g)??0)+1);continue}l.trim().startsWith("[Tool Result]")||s.push(l)}const i=Array.from(n.entries()).toSorted((l,d)=>d[1]-l[1]),a=i.reduce((l,[,d])=>l+d,0),o=i.length>0?`Tools: ${i.map(([l,d])=>`${l}×${d}`).join(", ")} (${a} calls)`:"";return{tools:i,summary:o,cleanContent:s.join(`
`).trim()}}function Mv(e){return{byChannel:Array.from(e.byChannelMap.entries()).map(([t,n])=>({channel:t,totals:n})).toSorted((t,n)=>n.totals.totalCost-t.totals.totalCost),latency:e.latencyTotals.count>0?{count:e.latencyTotals.count,avgMs:e.latencyTotals.sum/e.latencyTotals.count,minMs:e.latencyTotals.min===Number.POSITIVE_INFINITY?0:e.latencyTotals.min,maxMs:e.latencyTotals.max,p95Ms:e.latencyTotals.p95Max}:void 0,dailyLatency:Array.from(e.dailyLatencyMap.values()).map(t=>({date:t.date,count:t.count,avgMs:t.count?t.sum/t.count:0,minMs:t.min===Number.POSITIVE_INFINITY?0:t.min,maxMs:t.max,p95Ms:t.p95Max})).toSorted((t,n)=>t.date.localeCompare(n.date)),modelDaily:Array.from(e.modelDailyMap.values()).toSorted((t,n)=>t.date.localeCompare(n.date)||n.cost-t.cost),daily:Array.from(e.dailyMap.values()).toSorted((t,n)=>t.date.localeCompare(n.date))}}const Iv=4;function lt(e){return Math.round(e/Iv)}function U(e){return e>=1e6?`${(e/1e6).toFixed(1)}M`:e>=1e3?`${(e/1e3).toFixed(1)}K`:String(e)}function Rv(e){const t=new Date;return t.setHours(e,0,0,0),t.toLocaleTimeString(void 0,{hour:"numeric"})}function Pv(e,t){const n=Array.from({length:24},()=>0),s=Array.from({length:24},()=>0);for(const i of e){const a=i.usage;if(!a?.messageCounts||a.messageCounts.total===0)continue;const o=a.firstActivity??i.updatedAt,l=a.lastActivity??i.updatedAt;if(!o||!l)continue;const d=Math.min(o,l),g=Math.max(o,l),p=Math.max(g-d,1)/6e4;let b=d;for(;b<g;){const u=new Date(b),v=sa(u,t),y=ia(u,t),k=Math.min(y.getTime(),g),$=Math.max((k-b)/6e4,0)/p;n[v]+=a.messageCounts.errors*$,s[v]+=a.messageCounts.total*$,b=k+1}}return s.map((i,a)=>{const o=n[a],l=i>0?o/i:0;return{hour:a,rate:l,errors:o,msgs:i}}).filter(i=>i.msgs>0&&i.errors>0).toSorted((i,a)=>a.rate-i.rate).slice(0,5).map(i=>({label:Rv(i.hour),value:`${(i.rate*100).toFixed(2)}%`,sub:`${Math.round(i.errors)} errors · ${Math.round(i.msgs)} msgs`}))}const Dv=["Sun","Mon","Tue","Wed","Thu","Fri","Sat"];function sa(e,t){return t==="utc"?e.getUTCHours():e.getHours()}function Fv(e,t){return t==="utc"?e.getUTCDay():e.getDay()}function ia(e,t){const n=new Date(e);return t==="utc"?n.setUTCMinutes(59,59,999):n.setMinutes(59,59,999),n}function Nv(e,t){const n=Array.from({length:24},()=>0),s=Array.from({length:7},()=>0);let i=0,a=!1;for(const l of e){const d=l.usage;if(!d||!d.totalTokens||d.totalTokens<=0)continue;i+=d.totalTokens;const g=d.firstActivity??l.updatedAt,f=d.lastActivity??l.updatedAt;if(!g||!f)continue;a=!0;const p=Math.min(g,f),b=Math.max(g,f),v=Math.max(b-p,1)/6e4;let y=p;for(;y<b;){const k=new Date(y),C=sa(k,t),$=Fv(k,t),T=ia(k,t),_=Math.min(T.getTime(),b),E=Math.max((_-y)/6e4,0)/v;n[C]+=d.totalTokens*E,s[$]+=d.totalTokens*E,y=_+1}}const o=Dv.map((l,d)=>({label:l,tokens:s[d]}));return{hasData:a,totalTokens:i,hourTotals:n,weekdayTotals:o}}function Ov(e,t,n,s){const i=Nv(e,t);if(!i.hasData)return r`
      <div class="card usage-mosaic">
        <div class="usage-mosaic-header">
          <div>
            <div class="usage-mosaic-title">Activity by Time</div>
            <div class="usage-mosaic-sub">Estimates require session timestamps.</div>
          </div>
          <div class="usage-mosaic-total">${U(0)} tokens</div>
        </div>
        <div class="muted" style="padding: 12px; text-align: center;">No timeline data yet.</div>
      </div>
    `;const a=Math.max(...i.hourTotals,1),o=Math.max(...i.weekdayTotals.map(l=>l.tokens),1);return r`
    <div class="card usage-mosaic">
      <div class="usage-mosaic-header">
        <div>
          <div class="usage-mosaic-title">Activity by Time</div>
          <div class="usage-mosaic-sub">
            Estimated from session spans (first/last activity). Time zone: ${t==="utc"?"UTC":"Local"}.
          </div>
        </div>
        <div class="usage-mosaic-total">${U(i.totalTokens)} tokens</div>
      </div>
      <div class="usage-mosaic-grid">
        <div class="usage-mosaic-section">
          <div class="usage-mosaic-section-title">Day of Week</div>
          <div class="usage-daypart-grid">
            ${i.weekdayTotals.map(l=>{const d=Math.min(l.tokens/o,1),g=l.tokens>0?`rgba(255, 77, 77, ${.12+d*.6})`:"transparent";return r`
                <div class="usage-daypart-cell" style="background: ${g};">
                  <div class="usage-daypart-label">${l.label}</div>
                  <div class="usage-daypart-value">${U(l.tokens)}</div>
                </div>
              `})}
          </div>
        </div>
        <div class="usage-mosaic-section">
          <div class="usage-mosaic-section-title">
            <span>Hours</span>
            <span class="usage-mosaic-sub">0 → 23</span>
          </div>
          <div class="usage-hour-grid">
            ${i.hourTotals.map((l,d)=>{const g=Math.min(l/a,1),f=l>0?`rgba(255, 77, 77, ${.08+g*.7})`:"transparent",p=`${d}:00 · ${U(l)} tokens`,b=g>.7?"rgba(255, 77, 77, 0.6)":"rgba(255, 77, 77, 0.2)",u=n.includes(d);return r`
                <div
                  class="usage-hour-cell ${u?"selected":""}"
                  style="background: ${f}; border-color: ${b};"
                  title="${p}"
                  @click=${v=>s(d,v.shiftKey)}
                ></div>
              `})}
          </div>
          <div class="usage-hour-labels">
            <span>Midnight</span>
            <span>4am</span>
            <span>8am</span>
            <span>Noon</span>
            <span>4pm</span>
            <span>8pm</span>
          </div>
          <div class="usage-hour-legend">
            <span></span>
            Low → High token density
          </div>
        </div>
      </div>
    </div>
  `}function Q(e,t=2){return`$${e.toFixed(t)}`}function Us(e){return`${e.getFullYear()}-${String(e.getMonth()+1).padStart(2,"0")}-${String(e.getDate()).padStart(2,"0")}`}function Ml(e){const t=/^(\d{4})-(\d{2})-(\d{2})$/.exec(e);if(!t)return null;const[,n,s,i]=t,a=new Date(Date.UTC(Number(n),Number(s)-1,Number(i)));return Number.isNaN(a.valueOf())?null:a}function Il(e){const t=Ml(e);return t?t.toLocaleDateString(void 0,{month:"short",day:"numeric"}):e}function Bv(e){const t=Ml(e);return t?t.toLocaleDateString(void 0,{month:"long",day:"numeric",year:"numeric"}):e}const In=()=>({input:0,output:0,cacheRead:0,cacheWrite:0,totalTokens:0,totalCost:0,inputCost:0,outputCost:0,cacheReadCost:0,cacheWriteCost:0,missingCostEntries:0}),Rn=(e,t)=>{e.input+=t.input??0,e.output+=t.output??0,e.cacheRead+=t.cacheRead??0,e.cacheWrite+=t.cacheWrite??0,e.totalTokens+=t.totalTokens??0,e.totalCost+=t.totalCost??0,e.inputCost+=t.inputCost??0,e.outputCost+=t.outputCost??0,e.cacheReadCost+=t.cacheReadCost??0,e.cacheWriteCost+=t.cacheWriteCost??0,e.missingCostEntries+=t.missingCostEntries??0},Uv=(e,t)=>{if(e.length===0)return t??{messages:{total:0,user:0,assistant:0,toolCalls:0,toolResults:0,errors:0},tools:{totalCalls:0,uniqueTools:0,tools:[]},byModel:[],byProvider:[],byAgent:[],byChannel:[],daily:[]};const n={total:0,user:0,assistant:0,toolCalls:0,toolResults:0,errors:0},s=new Map,i=new Map,a=new Map,o=new Map,l=new Map,d=new Map,g=new Map,f=new Map,p={count:0,sum:0,min:Number.POSITIVE_INFINITY,max:0,p95Max:0};for(const u of e){const v=u.usage;if(v){if(v.messageCounts&&(n.total+=v.messageCounts.total,n.user+=v.messageCounts.user,n.assistant+=v.messageCounts.assistant,n.toolCalls+=v.messageCounts.toolCalls,n.toolResults+=v.messageCounts.toolResults,n.errors+=v.messageCounts.errors),v.toolUsage)for(const y of v.toolUsage.tools)s.set(y.name,(s.get(y.name)??0)+y.count);if(v.modelUsage)for(const y of v.modelUsage){const k=`${y.provider??"unknown"}::${y.model??"unknown"}`,C=i.get(k)??{provider:y.provider,model:y.model,count:0,totals:In()};C.count+=y.count,Rn(C.totals,y.totals),i.set(k,C);const $=y.provider??"unknown",T=a.get($)??{provider:y.provider,model:void 0,count:0,totals:In()};T.count+=y.count,Rn(T.totals,y.totals),a.set($,T)}if(v.latency){const{count:y,avgMs:k,minMs:C,maxMs:$,p95Ms:T}=v.latency;y>0&&(p.count+=y,p.sum+=k*y,p.min=Math.min(p.min,C),p.max=Math.max(p.max,$),p.p95Max=Math.max(p.p95Max,T))}if(u.agentId){const y=o.get(u.agentId)??In();Rn(y,v),o.set(u.agentId,y)}if(u.channel){const y=l.get(u.channel)??In();Rn(y,v),l.set(u.channel,y)}for(const y of v.dailyBreakdown??[]){const k=d.get(y.date)??{date:y.date,tokens:0,cost:0,messages:0,toolCalls:0,errors:0};k.tokens+=y.tokens,k.cost+=y.cost,d.set(y.date,k)}for(const y of v.dailyMessageCounts??[]){const k=d.get(y.date)??{date:y.date,tokens:0,cost:0,messages:0,toolCalls:0,errors:0};k.messages+=y.total,k.toolCalls+=y.toolCalls,k.errors+=y.errors,d.set(y.date,k)}for(const y of v.dailyLatency??[]){const k=g.get(y.date)??{date:y.date,count:0,sum:0,min:Number.POSITIVE_INFINITY,max:0,p95Max:0};k.count+=y.count,k.sum+=y.avgMs*y.count,k.min=Math.min(k.min,y.minMs),k.max=Math.max(k.max,y.maxMs),k.p95Max=Math.max(k.p95Max,y.p95Ms),g.set(y.date,k)}for(const y of v.dailyModelUsage??[]){const k=`${y.date}::${y.provider??"unknown"}::${y.model??"unknown"}`,C=f.get(k)??{date:y.date,provider:y.provider,model:y.model,tokens:0,cost:0,count:0};C.tokens+=y.tokens,C.cost+=y.cost,C.count+=y.count,f.set(k,C)}}}const b=Mv({byChannelMap:l,latencyTotals:p,dailyLatencyMap:g,modelDailyMap:f,dailyMap:d});return{messages:n,tools:{totalCalls:Array.from(s.values()).reduce((u,v)=>u+v,0),uniqueTools:s.size,tools:Array.from(s.entries()).map(([u,v])=>({name:u,count:v})).toSorted((u,v)=>v.count-u.count)},byModel:Array.from(i.values()).toSorted((u,v)=>v.totals.totalCost-u.totals.totalCost),byProvider:Array.from(a.values()).toSorted((u,v)=>v.totals.totalCost-u.totals.totalCost),byAgent:Array.from(o.entries()).map(([u,v])=>({agentId:u,totals:v})).toSorted((u,v)=>v.totals.totalCost-u.totals.totalCost),...b}},zv=(e,t,n)=>{let s=0,i=0;for(const f of e){const p=f.usage?.durationMs??0;p>0&&(s+=p,i+=1)}const a=i?s/i:0,o=t&&s>0?t.totalTokens/(s/6e4):void 0,l=t&&s>0?t.totalCost/(s/6e4):void 0,d=n.messages.total?n.messages.errors/n.messages.total:0,g=n.daily.filter(f=>f.messages>0&&f.errors>0).map(f=>({date:f.date,errors:f.errors,messages:f.messages,rate:f.errors/f.messages})).toSorted((f,p)=>p.rate-f.rate||p.errors-f.errors)[0];return{durationSumMs:s,durationCount:i,avgDurationMs:a,throughputTokensPerMin:o,throughputCostPerMin:l,errorRate:d,peakErrorDay:g}};function zs(e,t,n="text/plain"){const s=new Blob([t],{type:`${n};charset=utf-8`}),i=URL.createObjectURL(s),a=document.createElement("a");a.href=i,a.download=e,a.click(),URL.revokeObjectURL(i)}function Hv(e){return/[",\n]/.test(e)?`"${e.replaceAll('"','""')}"`:e}function Xn(e){return e.map(t=>t==null?"":Hv(String(t))).join(",")}const Kv=e=>{const t=[Xn(["key","label","agentId","channel","provider","model","updatedAt","durationMs","messages","errors","toolCalls","inputTokens","outputTokens","cacheReadTokens","cacheWriteTokens","totalTokens","totalCost"])];for(const n of e){const s=n.usage;t.push(Xn([n.key,n.label??"",n.agentId??"",n.channel??"",n.modelProvider??n.providerOverride??"",n.model??n.modelOverride??"",n.updatedAt?new Date(n.updatedAt).toISOString():"",s?.durationMs??"",s?.messageCounts?.total??"",s?.messageCounts?.errors??"",s?.messageCounts?.toolCalls??"",s?.input??"",s?.output??"",s?.cacheRead??"",s?.cacheWrite??"",s?.totalTokens??"",s?.totalCost??""]))}return t.join(`
`)},jv=e=>{const t=[Xn(["date","inputTokens","outputTokens","cacheReadTokens","cacheWriteTokens","totalTokens","inputCost","outputCost","cacheReadCost","cacheWriteCost","totalCost"])];for(const n of e)t.push(Xn([n.date,n.input,n.output,n.cacheRead,n.cacheWrite,n.totalTokens,n.inputCost??"",n.outputCost??"",n.cacheReadCost??"",n.cacheWriteCost??"",n.totalCost]));return t.join(`
`)},Wv=(e,t,n)=>{const s=e.trim();if(!s)return[];const i=s.length?s.split(/\s+/):[],a=i.length?i[i.length-1]:"",[o,l]=a.includes(":")?[a.slice(0,a.indexOf(":")),a.slice(a.indexOf(":")+1)]:["",""],d=o.toLowerCase(),g=l.toLowerCase(),f=$=>{const T=new Set;for(const _ of $)_&&T.add(_);return Array.from(T)},p=f(t.map($=>$.agentId)).slice(0,6),b=f(t.map($=>$.channel)).slice(0,6),u=f([...t.map($=>$.modelProvider),...t.map($=>$.providerOverride),...n?.byProvider.map($=>$.provider)??[]]).slice(0,6),v=f([...t.map($=>$.model),...n?.byModel.map($=>$.model)??[]]).slice(0,6),y=f(n?.tools.tools.map($=>$.name)??[]).slice(0,6);if(!d)return[{label:"agent:",value:"agent:"},{label:"channel:",value:"channel:"},{label:"provider:",value:"provider:"},{label:"model:",value:"model:"},{label:"tool:",value:"tool:"},{label:"has:errors",value:"has:errors"},{label:"has:tools",value:"has:tools"},{label:"minTokens:",value:"minTokens:"},{label:"maxCost:",value:"maxCost:"}];const k=[],C=($,T)=>{for(const _ of T)(!g||_.toLowerCase().includes(g))&&k.push({label:`${$}:${_}`,value:`${$}:${_}`})};switch(d){case"agent":C("agent",p);break;case"channel":C("channel",b);break;case"provider":C("provider",u);break;case"model":C("model",v);break;case"tool":C("tool",y);break;case"has":["errors","tools","context","usage","model","provider"].forEach($=>{(!g||$.includes(g))&&k.push({label:`has:${$}`,value:`has:${$}`})});break}return k},qv=(e,t)=>{const n=e.trim();if(!n)return`${t} `;const s=n.split(/\s+/);return s[s.length-1]=t,`${s.join(" ")} `},ht=e=>e.trim().toLowerCase(),Gv=(e,t)=>{const n=e.trim();if(!n)return`${t} `;const s=n.split(/\s+/),i=s[s.length-1]??"",a=t.includes(":")?t.split(":")[0]:null,o=i.includes(":")?i.split(":")[0]:null;return i.endsWith(":")&&a&&o===a?(s[s.length-1]=t,`${s.join(" ")} `):s.includes(t)?`${s.join(" ")} `:`${s.join(" ")} ${t} `},ko=(e,t)=>{const s=e.trim().split(/\s+/).filter(Boolean).filter(i=>i!==t);return s.length?`${s.join(" ")} `:""},So=(e,t,n)=>{const s=ht(t),a=[...na(e).filter(o=>ht(o.key??"")!==s).map(o=>o.raw),...n.map(o=>`${t}:${o}`)];return a.length?`${a.join(" ")} `:""};function Je(e,t){return t===0?0:e/t*100}function Vv(e){const t=e.totalCost||0;return{input:{tokens:e.input,cost:e.inputCost||0,pct:Je(e.inputCost||0,t)},output:{tokens:e.output,cost:e.outputCost||0,pct:Je(e.outputCost||0,t)},cacheRead:{tokens:e.cacheRead,cost:e.cacheReadCost||0,pct:Je(e.cacheReadCost||0,t)},cacheWrite:{tokens:e.cacheWrite,cost:e.cacheWriteCost||0,pct:Je(e.cacheWriteCost||0,t)},totalCost:t}}function Qv(e,t,n,s,i,a,o,l){if(!(e.length>0||t.length>0||n.length>0))return m;const g=n.length===1?s.find(v=>v.key===n[0]):null,f=g?(g.label||g.key).slice(0,20)+((g.label||g.key).length>20?"…":""):n.length===1?n[0].slice(0,8)+"…":`${n.length} sessions`,p=g?g.label||g.key:n.length===1?n[0]:n.join(", "),b=e.length===1?e[0]:`${e.length} days`,u=t.length===1?`${t[0]}:00`:`${t.length} hours`;return r`
    <div class="active-filters">
      ${e.length>0?r`
            <div class="filter-chip">
              <span class="filter-chip-label">Days: ${b}</span>
              <button class="filter-chip-remove" @click=${i} title="Remove filter">×</button>
            </div>
          `:m}
      ${t.length>0?r`
            <div class="filter-chip">
              <span class="filter-chip-label">Hours: ${u}</span>
              <button class="filter-chip-remove" @click=${a} title="Remove filter">×</button>
            </div>
          `:m}
      ${n.length>0?r`
            <div class="filter-chip" title="${p}">
              <span class="filter-chip-label">Session: ${f}</span>
              <button class="filter-chip-remove" @click=${o} title="Remove filter">×</button>
            </div>
          `:m}
      ${(e.length>0||t.length>0)&&n.length>0?r`
            <button class="btn btn-sm filter-clear-btn" @click=${l}>
              Clear All
            </button>
          `:m}
    </div>
  `}function Yv(e,t,n,s,i,a){if(!e.length)return r`
      <div class="daily-chart-compact">
        <div class="sessions-panel-title">Daily Usage</div>
        <div class="muted" style="padding: 20px; text-align: center">No data</div>
      </div>
    `;const o=n==="tokens",l=e.map(p=>o?p.totalTokens:p.totalCost),d=Math.max(...l,o?1:1e-4),g=e.length>30?12:e.length>20?18:e.length>14?24:32,f=e.length<=14;return r`
    <div class="daily-chart-compact">
      <div class="daily-chart-header">
        <div class="chart-toggle small sessions-toggle">
          <button
            class="toggle-btn ${s==="total"?"active":""}"
            @click=${()=>i("total")}
          >
            Total
          </button>
          <button
            class="toggle-btn ${s==="by-type"?"active":""}"
            @click=${()=>i("by-type")}
          >
            By Type
          </button>
        </div>
        <div class="card-title">Daily ${o?"Token":"Cost"} Usage</div>
      </div>
      <div class="daily-chart">
        <div class="daily-chart-bars" style="--bar-max-width: ${g}px">
          ${e.map((p,b)=>{const v=l[b]/d*100,y=t.includes(p.date),k=Il(p.date),C=e.length>20?String(parseInt(p.date.slice(8),10)):k,$=e.length>20?"font-size: 8px":"",T=s==="by-type"?o?[{value:p.output,class:"output"},{value:p.input,class:"input"},{value:p.cacheWrite,class:"cache-write"},{value:p.cacheRead,class:"cache-read"}]:[{value:p.outputCost??0,class:"output"},{value:p.inputCost??0,class:"input"},{value:p.cacheWriteCost??0,class:"cache-write"},{value:p.cacheReadCost??0,class:"cache-read"}]:[],_=s==="by-type"?o?[`Output ${U(p.output)}`,`Input ${U(p.input)}`,`Cache write ${U(p.cacheWrite)}`,`Cache read ${U(p.cacheRead)}`]:[`Output ${Q(p.outputCost??0)}`,`Input ${Q(p.inputCost??0)}`,`Cache write ${Q(p.cacheWriteCost??0)}`,`Cache read ${Q(p.cacheReadCost??0)}`]:[],L=o?U(p.totalTokens):Q(p.totalCost);return r`
              <div
                class="daily-bar-wrapper ${y?"selected":""}"
                @click=${E=>a(p.date,E.shiftKey)}
              >
                ${s==="by-type"?r`
                        <div
                          class="daily-bar"
                          style="height: ${v.toFixed(1)}%; display: flex; flex-direction: column;"
                        >
                          ${(()=>{const E=T.reduce((P,j)=>P+j.value,0)||1;return T.map(P=>r`
                                <div
                                  class="cost-segment ${P.class}"
                                  style="height: ${P.value/E*100}%"
                                ></div>
                              `)})()}
                        </div>
                      `:r`
                        <div class="daily-bar" style="height: ${v.toFixed(1)}%"></div>
                      `}
                ${f?r`<div class="daily-bar-total">${L}</div>`:m}
                <div class="daily-bar-label" style="${$}">${C}</div>
                <div class="daily-bar-tooltip">
                  <strong>${Bv(p.date)}</strong><br />
                  ${U(p.totalTokens)} tokens<br />
                  ${Q(p.totalCost)}
                  ${_.length?r`${_.map(E=>r`<div>${E}</div>`)}`:m}
                </div>
              </div>
            `})}
        </div>
      </div>
    </div>
  `}function Jv(e,t){const n=Vv(e),s=t==="tokens",i=e.totalTokens||1,a={output:Je(e.output,i),input:Je(e.input,i),cacheWrite:Je(e.cacheWrite,i),cacheRead:Je(e.cacheRead,i)};return r`
    <div class="cost-breakdown cost-breakdown-compact">
      <div class="cost-breakdown-header">${s?"Tokens":"Cost"} by Type</div>
      <div class="cost-breakdown-bar">
        <div class="cost-segment output" style="width: ${(s?a.output:n.output.pct).toFixed(1)}%"
          title="Output: ${s?U(e.output):Q(n.output.cost)}"></div>
        <div class="cost-segment input" style="width: ${(s?a.input:n.input.pct).toFixed(1)}%"
          title="Input: ${s?U(e.input):Q(n.input.cost)}"></div>
        <div class="cost-segment cache-write" style="width: ${(s?a.cacheWrite:n.cacheWrite.pct).toFixed(1)}%"
          title="Cache Write: ${s?U(e.cacheWrite):Q(n.cacheWrite.cost)}"></div>
        <div class="cost-segment cache-read" style="width: ${(s?a.cacheRead:n.cacheRead.pct).toFixed(1)}%"
          title="Cache Read: ${s?U(e.cacheRead):Q(n.cacheRead.cost)}"></div>
      </div>
      <div class="cost-breakdown-legend">
        <span class="legend-item"><span class="legend-dot output"></span>Output ${s?U(e.output):Q(n.output.cost)}</span>
        <span class="legend-item"><span class="legend-dot input"></span>Input ${s?U(e.input):Q(n.input.cost)}</span>
        <span class="legend-item"><span class="legend-dot cache-write"></span>Cache Write ${s?U(e.cacheWrite):Q(n.cacheWrite.cost)}</span>
        <span class="legend-item"><span class="legend-dot cache-read"></span>Cache Read ${s?U(e.cacheRead):Q(n.cacheRead.cost)}</span>
      </div>
      <div class="cost-breakdown-total">
        Total: ${s?U(e.totalTokens):Q(e.totalCost)}
      </div>
    </div>
  `}function ft(e,t,n){return r`
    <div class="usage-insight-card">
      <div class="usage-insight-title">${e}</div>
      ${t.length===0?r`<div class="muted">${n}</div>`:r`
              <div class="usage-list">
                ${t.map(s=>r`
                    <div class="usage-list-item">
                      <span>${s.label}</span>
                      <span class="usage-list-value">
                        <span>${s.value}</span>
                        ${s.sub?r`<span class="usage-list-sub">${s.sub}</span>`:m}
                      </span>
                    </div>
                  `)}
              </div>
            `}
    </div>
  `}function Ao(e,t,n){return r`
    <div class="usage-insight-card">
      <div class="usage-insight-title">${e}</div>
      ${t.length===0?r`<div class="muted">${n}</div>`:r`
              <div class="usage-error-list">
                ${t.map(s=>r`
                    <div class="usage-error-row">
                      <div class="usage-error-date">${s.label}</div>
                      <div class="usage-error-rate">${s.value}</div>
                      ${s.sub?r`<div class="usage-error-sub">${s.sub}</div>`:m}
                    </div>
                  `)}
              </div>
            `}
    </div>
  `}function Zv(e,t,n,s,i,a,o){if(!e)return m;const l=t.messages.total?Math.round(e.totalTokens/t.messages.total):0,d=t.messages.total?e.totalCost/t.messages.total:0,g=e.input+e.cacheRead,f=g>0?e.cacheRead/g:0,p=g>0?`${(f*100).toFixed(1)}%`:"—",b=n.errorRate*100,u=n.throughputTokensPerMin!==void 0?`${U(Math.round(n.throughputTokensPerMin))} tok/min`:"—",v=n.throughputCostPerMin!==void 0?`${Q(n.throughputCostPerMin,4)} / min`:"—",y=n.durationCount>0?Ki(n.avgDurationMs,{spaced:!0})??"—":"—",k="Cache hit rate = cache read / (input + cache read). Higher is better.",C="Error rate = errors / total messages. Lower is better.",$="Throughput shows tokens per minute over active time. Higher is better.",T="Average tokens per message in this range.",_=s?"Average cost per message when providers report costs. Cost data is missing for some or all sessions in this range.":"Average cost per message when providers report costs.",L=t.daily.filter(O=>O.messages>0&&O.errors>0).map(O=>{const K=O.errors/O.messages;return{label:Il(O.date),value:`${(K*100).toFixed(2)}%`,sub:`${O.errors} errors · ${O.messages} msgs · ${U(O.tokens)}`,rate:K}}).toSorted((O,K)=>K.rate-O.rate).slice(0,5).map(({rate:O,...K})=>K),E=t.byModel.slice(0,5).map(O=>({label:O.model??"unknown",value:Q(O.totals.totalCost),sub:`${U(O.totals.totalTokens)} · ${O.count} msgs`})),P=t.byProvider.slice(0,5).map(O=>({label:O.provider??"unknown",value:Q(O.totals.totalCost),sub:`${U(O.totals.totalTokens)} · ${O.count} msgs`})),j=t.tools.tools.slice(0,6).map(O=>({label:O.name,value:`${O.count}`,sub:"calls"})),Z=t.byAgent.slice(0,5).map(O=>({label:O.agentId,value:Q(O.totals.totalCost),sub:U(O.totals.totalTokens)})),ae=t.byChannel.slice(0,5).map(O=>({label:O.channel,value:Q(O.totals.totalCost),sub:U(O.totals.totalTokens)}));return r`
    <section class="card" style="margin-top: 16px;">
      <div class="card-title">Usage Overview</div>
      <div class="usage-summary-grid">
        <div class="usage-summary-card">
          <div class="usage-summary-title">
            Messages
            <span class="usage-summary-hint" title="Total user + assistant messages in range.">?</span>
          </div>
          <div class="usage-summary-value">${t.messages.total}</div>
          <div class="usage-summary-sub">
            ${t.messages.user} user · ${t.messages.assistant} assistant
          </div>
        </div>
        <div class="usage-summary-card">
          <div class="usage-summary-title">
            Tool Calls
            <span class="usage-summary-hint" title="Total tool call count across sessions.">?</span>
          </div>
          <div class="usage-summary-value">${t.tools.totalCalls}</div>
          <div class="usage-summary-sub">${t.tools.uniqueTools} tools used</div>
        </div>
        <div class="usage-summary-card">
          <div class="usage-summary-title">
            Errors
            <span class="usage-summary-hint" title="Total message/tool errors in range.">?</span>
          </div>
          <div class="usage-summary-value">${t.messages.errors}</div>
          <div class="usage-summary-sub">${t.messages.toolResults} tool results</div>
        </div>
        <div class="usage-summary-card">
          <div class="usage-summary-title">
            Avg Tokens / Msg
            <span class="usage-summary-hint" title=${T}>?</span>
          </div>
          <div class="usage-summary-value">${U(l)}</div>
          <div class="usage-summary-sub">Across ${t.messages.total||0} messages</div>
        </div>
        <div class="usage-summary-card">
          <div class="usage-summary-title">
            Avg Cost / Msg
            <span class="usage-summary-hint" title=${_}>?</span>
          </div>
          <div class="usage-summary-value">${Q(d,4)}</div>
          <div class="usage-summary-sub">${Q(e.totalCost)} total</div>
        </div>
        <div class="usage-summary-card">
          <div class="usage-summary-title">
            Sessions
            <span class="usage-summary-hint" title="Distinct sessions in the range.">?</span>
          </div>
          <div class="usage-summary-value">${a}</div>
          <div class="usage-summary-sub">of ${o} in range</div>
        </div>
        <div class="usage-summary-card">
          <div class="usage-summary-title">
            Throughput
            <span class="usage-summary-hint" title=${$}>?</span>
          </div>
          <div class="usage-summary-value">${u}</div>
          <div class="usage-summary-sub">${v}</div>
        </div>
        <div class="usage-summary-card">
          <div class="usage-summary-title">
            Error Rate
            <span class="usage-summary-hint" title=${C}>?</span>
          </div>
          <div class="usage-summary-value ${b>5?"bad":b>1?"warn":"good"}">${b.toFixed(2)}%</div>
          <div class="usage-summary-sub">
            ${t.messages.errors} errors · ${y} avg session
          </div>
        </div>
        <div class="usage-summary-card">
          <div class="usage-summary-title">
            Cache Hit Rate
            <span class="usage-summary-hint" title=${k}>?</span>
          </div>
          <div class="usage-summary-value ${f>.6?"good":f>.3?"warn":"bad"}">${p}</div>
          <div class="usage-summary-sub">
            ${U(e.cacheRead)} cached · ${U(g)} prompt
          </div>
        </div>
      </div>
      <div class="usage-insights-grid">
        ${ft("Top Models",E,"No model data")}
        ${ft("Top Providers",P,"No provider data")}
        ${ft("Top Tools",j,"No tool calls")}
        ${ft("Top Agents",Z,"No agent data")}
        ${ft("Top Channels",ae,"No channel data")}
        ${Ao("Peak Error Days",L,"No error data")}
        ${Ao("Peak Error Hours",i,"No error data")}
      </div>
    </section>
  `}function Xv(e,t,n,s,i,a,o,l,d,g,f,p,b,u,v){const y=M=>b.includes(M),k=M=>{const z=M.label||M.key;return z.startsWith("agent:")&&z.includes("?token=")?z.slice(0,z.indexOf("?token=")):z},C=async M=>{const z=k(M);try{await navigator.clipboard.writeText(z)}catch{}},$=M=>{const z=[];return y("channel")&&M.channel&&z.push(`channel:${M.channel}`),y("agent")&&M.agentId&&z.push(`agent:${M.agentId}`),y("provider")&&(M.modelProvider||M.providerOverride)&&z.push(`provider:${M.modelProvider??M.providerOverride}`),y("model")&&M.model&&z.push(`model:${M.model}`),y("messages")&&M.usage?.messageCounts&&z.push(`msgs:${M.usage.messageCounts.total}`),y("tools")&&M.usage?.toolUsage&&z.push(`tools:${M.usage.toolUsage.totalCalls}`),y("errors")&&M.usage?.messageCounts&&z.push(`errors:${M.usage.messageCounts.errors}`),y("duration")&&M.usage?.durationMs&&z.push(`dur:${Ki(M.usage.durationMs,{spaced:!0})??"—"}`),z},T=M=>{const z=M.usage;if(!z)return 0;if(n.length>0&&z.dailyBreakdown&&z.dailyBreakdown.length>0){const oe=z.dailyBreakdown.filter(re=>n.includes(re.date));return s?oe.reduce((re,ee)=>re+ee.tokens,0):oe.reduce((re,ee)=>re+ee.cost,0)}return s?z.totalTokens??0:z.totalCost??0},_=[...e].toSorted((M,z)=>{switch(i){case"recent":return(z.updatedAt??0)-(M.updatedAt??0);case"messages":return(z.usage?.messageCounts?.total??0)-(M.usage?.messageCounts?.total??0);case"errors":return(z.usage?.messageCounts?.errors??0)-(M.usage?.messageCounts?.errors??0);case"cost":return T(z)-T(M);default:return T(z)-T(M)}}),L=a==="asc"?_.toReversed():_,E=L.reduce((M,z)=>M+T(z),0),P=L.length?E/L.length:0,j=L.reduce((M,z)=>M+(z.usage?.messageCounts?.errors??0),0),Z=new Set(t),ae=L.filter(M=>Z.has(M.key)),O=ae.length,K=new Map(L.map(M=>[M.key,M])),ue=o.map(M=>K.get(M)).filter(M=>!!M);return r`
    <div class="card sessions-card">
      <div class="sessions-card-header">
        <div class="card-title">Sessions</div>
        <div class="sessions-card-count">
          ${e.length} shown${u!==e.length?` · ${u} total`:""}
        </div>
      </div>
      <div class="sessions-card-meta">
        <div class="sessions-card-stats">
          <span>${s?U(P):Q(P)} avg</span>
          <span>${j} errors</span>
        </div>
        <div class="chart-toggle small">
          <button
            class="toggle-btn ${l==="all"?"active":""}"
            @click=${()=>p("all")}
          >
            All
          </button>
          <button
            class="toggle-btn ${l==="recent"?"active":""}"
            @click=${()=>p("recent")}
          >
            Recently viewed
          </button>
        </div>
        <label class="sessions-sort">
          <span>Sort</span>
          <select
            @change=${M=>g(M.target.value)}
          >
            <option value="cost" ?selected=${i==="cost"}>Cost</option>
            <option value="errors" ?selected=${i==="errors"}>Errors</option>
            <option value="messages" ?selected=${i==="messages"}>Messages</option>
            <option value="recent" ?selected=${i==="recent"}>Recent</option>
            <option value="tokens" ?selected=${i==="tokens"}>Tokens</option>
          </select>
        </label>
        <button
          class="btn btn-sm sessions-action-btn icon"
          @click=${()=>f(a==="desc"?"asc":"desc")}
          title=${a==="desc"?"Descending":"Ascending"}
        >
          ${a==="desc"?"↓":"↑"}
        </button>
        ${O>0?r`
                <button class="btn btn-sm sessions-action-btn sessions-clear-btn" @click=${v}>
                  Clear Selection
                </button>
              `:m}
      </div>
      ${l==="recent"?ue.length===0?r`
                <div class="muted" style="padding: 20px; text-align: center">No recent sessions</div>
              `:r`
                <div class="session-bars" style="max-height: 220px; margin-top: 6px;">
                  ${ue.map(M=>{const z=T(M),oe=Z.has(M.key),re=k(M),ee=$(M);return r`
                      <div
                        class="session-bar-row ${oe?"selected":""}"
                        @click=${se=>d(M.key,se.shiftKey)}
                        title="${M.key}"
                      >
                        <div class="session-bar-label">
                          <div class="session-bar-title">${re}</div>
                          ${ee.length>0?r`<div class="session-bar-meta">${ee.join(" · ")}</div>`:m}
                        </div>
                        <div class="session-bar-track" style="display: none;"></div>
                        <div class="session-bar-actions">
                          <button
                            class="session-copy-btn"
                            title="Copy session name"
                            @click=${se=>{se.stopPropagation(),C(M)}}
                          >
                            Copy
                          </button>
                          <div class="session-bar-value">${s?U(z):Q(z)}</div>
                        </div>
                      </div>
                    `})}
                </div>
              `:e.length===0?r`
                <div class="muted" style="padding: 20px; text-align: center">No sessions in range</div>
              `:r`
                <div class="session-bars">
                  ${L.slice(0,50).map(M=>{const z=T(M),oe=t.includes(M.key),re=k(M),ee=$(M);return r`
                      <div
                        class="session-bar-row ${oe?"selected":""}"
                        @click=${se=>d(M.key,se.shiftKey)}
                        title="${M.key}"
                      >
                        <div class="session-bar-label">
                          <div class="session-bar-title">${re}</div>
                          ${ee.length>0?r`<div class="session-bar-meta">${ee.join(" · ")}</div>`:m}
                        </div>
                        <div class="session-bar-track" style="display: none;"></div>
                        <div class="session-bar-actions">
                          <button
                            class="session-copy-btn"
                            title="Copy session name"
                            @click=${se=>{se.stopPropagation(),C(M)}}
                          >
                            Copy
                          </button>
                          <div class="session-bar-value">${s?U(z):Q(z)}</div>
                        </div>
                      </div>
                    `})}
                  ${e.length>50?r`<div class="muted" style="padding: 8px; text-align: center; font-size: 11px;">+${e.length-50} more</div>`:m}
                </div>
              `}
      ${O>1?r`
              <div style="margin-top: 10px;">
                <div class="sessions-card-count">Selected (${O})</div>
                <div class="session-bars" style="max-height: 160px; margin-top: 6px;">
                  ${ae.map(M=>{const z=T(M),oe=k(M),re=$(M);return r`
                      <div
                        class="session-bar-row selected"
                        @click=${ee=>d(M.key,ee.shiftKey)}
                        title="${M.key}"
                      >
                        <div class="session-bar-label">
                          <div class="session-bar-title">${oe}</div>
                          ${re.length>0?r`<div class="session-bar-meta">${re.join(" · ")}</div>`:m}
                        </div>
                  <div class="session-bar-track" style="display: none;"></div>
                        <div class="session-bar-actions">
                          <button
                            class="session-copy-btn"
                            title="Copy session name"
                            @click=${ee=>{ee.stopPropagation(),C(M)}}
                          >
                            Copy
                          </button>
                          <div class="session-bar-value">${s?U(z):Q(z)}</div>
                        </div>
                      </div>
                    `})}
                </div>
              </div>
            `:m}
    </div>
  `}function Ze(e,t){return!t||t<=0?0:e/t*100}function em(){return m}function tm(e){const t=e.usage;if(!t)return r`
      <div class="muted">No usage data for this session.</div>
    `;const n=o=>o?new Date(o).toLocaleString():"—",s=[];e.channel&&s.push(`channel:${e.channel}`),e.agentId&&s.push(`agent:${e.agentId}`),(e.modelProvider||e.providerOverride)&&s.push(`provider:${e.modelProvider??e.providerOverride}`),e.model&&s.push(`model:${e.model}`);const i=t.toolUsage?.tools.slice(0,6).map(o=>({label:o.name,value:`${o.count}`,sub:"calls"}))??[],a=t.modelUsage?.slice(0,6).map(o=>({label:o.model??"unknown",value:Q(o.totals.totalCost),sub:U(o.totals.totalTokens)}))??[];return r`
    ${s.length>0?r`<div class="usage-badges">${s.map(o=>r`<span class="usage-badge">${o}</span>`)}</div>`:m}
    <div class="session-summary-grid">
      <div class="session-summary-card">
        <div class="session-summary-title">Messages</div>
        <div class="session-summary-value">${t.messageCounts?.total??0}</div>
        <div class="session-summary-meta">${t.messageCounts?.user??0} user · ${t.messageCounts?.assistant??0} assistant</div>
      </div>
      <div class="session-summary-card">
        <div class="session-summary-title">Tool Calls</div>
        <div class="session-summary-value">${t.toolUsage?.totalCalls??0}</div>
        <div class="session-summary-meta">${t.toolUsage?.uniqueTools??0} tools</div>
      </div>
      <div class="session-summary-card">
        <div class="session-summary-title">Errors</div>
        <div class="session-summary-value">${t.messageCounts?.errors??0}</div>
        <div class="session-summary-meta">${t.messageCounts?.toolResults??0} tool results</div>
      </div>
      <div class="session-summary-card">
        <div class="session-summary-title">Duration</div>
        <div class="session-summary-value">${Ki(t.durationMs,{spaced:!0})??"—"}</div>
        <div class="session-summary-meta">${n(t.firstActivity)} → ${n(t.lastActivity)}</div>
      </div>
    </div>
    <div class="usage-insights-grid" style="margin-top: 12px;">
      ${ft("Top Tools",i,"No tool calls")}
      ${ft("Model Mix",a,"No model data")}
    </div>
  `}function nm(e,t,n,s,i,a,o,l,d,g,f,p,b,u,v,y,k,C,$,T,_,L,E){const P=e.label||e.key,j=P.length>50?P.slice(0,50)+"…":P,Z=e.usage;return r`
    <div class="card session-detail-panel">
      <div class="session-detail-header">
        <div class="session-detail-header-left">
          <div class="session-detail-title">${j}</div>
        </div>
        <div class="session-detail-stats">
          ${Z?r`
            <span><strong>${U(Z.totalTokens)}</strong> tokens</span>
            <span><strong>${Q(Z.totalCost)}</strong></span>
          `:m}
        </div>
        <button class="session-close-btn" @click=${E} title="Close session details">×</button>
      </div>
      <div class="session-detail-content">
        ${tm(e)}
        <div class="session-detail-row">
          ${sm(t,n,s,i,a,o,l,d,g)}
        </div>
        <div class="session-detail-bottom">
          ${am(f,p,b,u,v,y,k,C,$,T)}
          ${im(e.contextWeight,Z,_,L)}
        </div>
      </div>
    </div>
  `}function sm(e,t,n,s,i,a,o,l,d){if(t)return r`
      <div class="session-timeseries-compact">
        <div class="muted" style="padding: 20px; text-align: center">Loading...</div>
      </div>
    `;if(!e||e.points.length<2)return r`
      <div class="session-timeseries-compact">
        <div class="muted" style="padding: 20px; text-align: center">No timeline data</div>
      </div>
    `;let g=e.points;if(o||l||d&&d.length>0){const K=o?new Date(o+"T00:00:00").getTime():0,ue=l?new Date(l+"T23:59:59").getTime():1/0;g=e.points.filter(M=>{if(M.timestamp<K||M.timestamp>ue)return!1;if(d&&d.length>0){const z=new Date(M.timestamp),oe=`${z.getFullYear()}-${String(z.getMonth()+1).padStart(2,"0")}-${String(z.getDate()).padStart(2,"0")}`;return d.includes(oe)}return!0})}if(g.length<2)return r`
      <div class="session-timeseries-compact">
        <div class="muted" style="padding: 20px; text-align: center">No data in range</div>
      </div>
    `;let f=0,p=0,b=0,u=0,v=0,y=0;g=g.map(K=>(f+=K.totalTokens,p+=K.cost,b+=K.output,u+=K.input,v+=K.cacheRead,y+=K.cacheWrite,{...K,cumulativeTokens:f,cumulativeCost:p}));const k=400,C=80,$={top:16,right:10,bottom:20,left:40},T=k-$.left-$.right,_=C-$.top-$.bottom,L=n==="cumulative",E=n==="per-turn"&&i==="by-type",P=b+u+v+y,j=g.map(K=>L?K.cumulativeTokens:E?K.input+K.output+K.cacheRead+K.cacheWrite:K.totalTokens),Z=Math.max(...j,1),ae=Math.max(2,Math.min(8,T/g.length*.7)),O=Math.max(1,(T-ae*g.length)/(g.length-1||1));return r`
    <div class="session-timeseries-compact">
      <div class="timeseries-header-row">
        <div class="card-title" style="font-size: 13px;">Usage Over Time</div>
        <div class="timeseries-controls">
          <div class="chart-toggle small">
            <button
              class="toggle-btn ${L?"":"active"}"
              @click=${()=>s("per-turn")}
            >
              Per Turn
            </button>
            <button
              class="toggle-btn ${L?"active":""}"
              @click=${()=>s("cumulative")}
            >
              Cumulative
            </button>
          </div>
          ${L?m:r`
                  <div class="chart-toggle small">
                    <button
                      class="toggle-btn ${i==="total"?"active":""}"
                      @click=${()=>a("total")}
                    >
                      Total
                    </button>
                    <button
                      class="toggle-btn ${i==="by-type"?"active":""}"
                      @click=${()=>a("by-type")}
                    >
                      By Type
                    </button>
                  </div>
                `}
        </div>
      </div>
      <svg viewBox="0 0 ${k} ${C+15}" class="timeseries-svg" style="width: 100%; height: auto;">
        <!-- Y axis -->
        <line x1="${$.left}" y1="${$.top}" x2="${$.left}" y2="${$.top+_}" stroke="var(--border)" />
        <!-- X axis -->
        <line x1="${$.left}" y1="${$.top+_}" x2="${k-$.right}" y2="${$.top+_}" stroke="var(--border)" />
        <!-- Y axis labels -->
        <text x="${$.left-4}" y="${$.top+4}" text-anchor="end" class="axis-label" style="font-size: 9px; fill: var(--text-muted)">${U(Z)}</text>
        <text x="${$.left-4}" y="${$.top+_}" text-anchor="end" class="axis-label" style="font-size: 9px; fill: var(--text-muted)">0</text>
        <!-- X axis labels (first and last) -->
        ${g.length>0?Tn`
          <text x="${$.left}" y="${$.top+_+12}" text-anchor="start" style="font-size: 8px; fill: var(--text-muted)">${new Date(g[0].timestamp).toLocaleDateString(void 0,{month:"short",day:"numeric"})}</text>
          <text x="${k-$.right}" y="${$.top+_+12}" text-anchor="end" style="font-size: 8px; fill: var(--text-muted)">${new Date(g[g.length-1].timestamp).toLocaleDateString(void 0,{month:"short",day:"numeric"})}</text>
        `:m}
        <!-- Bars -->
        ${g.map((K,ue)=>{const M=j[ue],z=$.left+ue*(ae+O),oe=M/Z*_,re=$.top+_-oe,se=[new Date(K.timestamp).toLocaleDateString(void 0,{month:"short",day:"numeric",hour:"2-digit",minute:"2-digit"}),`${U(M)} tokens`];E&&(se.push(`Output ${U(K.output)}`),se.push(`Input ${U(K.input)}`),se.push(`Cache write ${U(K.cacheWrite)}`),se.push(`Cache read ${U(K.cacheRead)}`));const R=se.join(" · ");if(!E)return Tn`<rect x="${z}" y="${re}" width="${ae}" height="${oe}" class="ts-bar" rx="1" style="cursor: pointer;"><title>${R}</title></rect>`;const D=[{value:K.output,class:"output"},{value:K.input,class:"input"},{value:K.cacheWrite,class:"cache-write"},{value:K.cacheRead,class:"cache-read"}];let F=$.top+_;return Tn`
            ${D.map(W=>{if(W.value<=0||M<=0)return m;const $e=oe*(W.value/M);return F-=$e,Tn`<rect x="${z}" y="${F}" width="${ae}" height="${$e}" class="ts-bar ${W.class}" rx="1"><title>${R}</title></rect>`})}
          `})}
      </svg>
      <div class="timeseries-summary">${g.length} msgs · ${U(f)} · ${Q(p)}</div>
      ${E?r`
              <div style="margin-top: 8px;">
                <div class="card-title" style="font-size: 12px; margin-bottom: 6px;">Tokens by Type</div>
                <div class="cost-breakdown-bar" style="height: 18px;">
                  <div class="cost-segment output" style="width: ${Ze(b,P).toFixed(1)}%"></div>
                  <div class="cost-segment input" style="width: ${Ze(u,P).toFixed(1)}%"></div>
                  <div class="cost-segment cache-write" style="width: ${Ze(y,P).toFixed(1)}%"></div>
                  <div class="cost-segment cache-read" style="width: ${Ze(v,P).toFixed(1)}%"></div>
                </div>
                <div class="cost-breakdown-legend">
                  <div class="legend-item" title="Assistant output tokens">
                    <span class="legend-dot output"></span>Output ${U(b)}
                  </div>
                  <div class="legend-item" title="User + tool input tokens">
                    <span class="legend-dot input"></span>Input ${U(u)}
                  </div>
                  <div class="legend-item" title="Tokens written to cache">
                    <span class="legend-dot cache-write"></span>Cache Write ${U(y)}
                  </div>
                  <div class="legend-item" title="Tokens read from cache">
                    <span class="legend-dot cache-read"></span>Cache Read ${U(v)}
                  </div>
                </div>
                <div class="cost-breakdown-total">Total: ${U(P)}</div>
              </div>
            `:m}
    </div>
  `}function im(e,t,n,s){if(!e)return r`
      <div class="context-details-panel">
        <div class="muted" style="padding: 20px; text-align: center">No context data</div>
      </div>
    `;const i=lt(e.systemPrompt.chars),a=lt(e.skills.promptChars),o=lt(e.tools.listChars+e.tools.schemaChars),l=lt(e.injectedWorkspaceFiles.reduce((T,_)=>T+_.injectedChars,0)),d=i+a+o+l;let g="";if(t&&t.totalTokens>0){const T=t.input+t.cacheRead;T>0&&(g=`~${Math.min(d/T*100,100).toFixed(0)}% of input`)}const f=e.skills.entries.toSorted((T,_)=>_.blockChars-T.blockChars),p=e.tools.entries.toSorted((T,_)=>_.summaryChars+_.schemaChars-(T.summaryChars+T.schemaChars)),b=e.injectedWorkspaceFiles.toSorted((T,_)=>_.injectedChars-T.injectedChars),u=4,v=n,y=v?f:f.slice(0,u),k=v?p:p.slice(0,u),C=v?b:b.slice(0,u),$=f.length>u||p.length>u||b.length>u;return r`
    <div class="context-details-panel">
      <div class="context-breakdown-header">
        <div class="card-title" style="font-size: 13px;">System Prompt Breakdown</div>
        ${$?r`<button class="context-expand-btn" @click=${s}>
                ${v?"Collapse":"Expand all"}
              </button>`:m}
      </div>
      <p class="context-weight-desc">${g||"Base context per message"}</p>
      <div class="context-stacked-bar">
        <div class="context-segment system" style="width: ${Ze(i,d).toFixed(1)}%" title="System: ~${U(i)}"></div>
        <div class="context-segment skills" style="width: ${Ze(a,d).toFixed(1)}%" title="Skills: ~${U(a)}"></div>
        <div class="context-segment tools" style="width: ${Ze(o,d).toFixed(1)}%" title="Tools: ~${U(o)}"></div>
        <div class="context-segment files" style="width: ${Ze(l,d).toFixed(1)}%" title="Files: ~${U(l)}"></div>
      </div>
      <div class="context-legend">
        <span class="legend-item"><span class="legend-dot system"></span>Sys ~${U(i)}</span>
        <span class="legend-item"><span class="legend-dot skills"></span>Skills ~${U(a)}</span>
        <span class="legend-item"><span class="legend-dot tools"></span>Tools ~${U(o)}</span>
        <span class="legend-item"><span class="legend-dot files"></span>Files ~${U(l)}</span>
      </div>
      <div class="context-total">Total: ~${U(d)}</div>
      <div class="context-breakdown-grid">
        ${f.length>0?(()=>{const T=f.length-y.length;return r`
                  <div class="context-breakdown-card">
                    <div class="context-breakdown-title">Skills (${f.length})</div>
                    <div class="context-breakdown-list">
                      ${y.map(_=>r`
                          <div class="context-breakdown-item">
                            <span class="mono">${_.name}</span>
                            <span class="muted">~${U(lt(_.blockChars))}</span>
                          </div>
                        `)}
                    </div>
                    ${T>0?r`<div class="context-breakdown-more">+${T} more</div>`:m}
                  </div>
                `})():m}
        ${p.length>0?(()=>{const T=p.length-k.length;return r`
                  <div class="context-breakdown-card">
                    <div class="context-breakdown-title">Tools (${p.length})</div>
                    <div class="context-breakdown-list">
                      ${k.map(_=>r`
                          <div class="context-breakdown-item">
                            <span class="mono">${_.name}</span>
                            <span class="muted">~${U(lt(_.summaryChars+_.schemaChars))}</span>
                          </div>
                        `)}
                    </div>
                    ${T>0?r`<div class="context-breakdown-more">+${T} more</div>`:m}
                  </div>
                `})():m}
        ${b.length>0?(()=>{const T=b.length-C.length;return r`
                  <div class="context-breakdown-card">
                    <div class="context-breakdown-title">Files (${b.length})</div>
                    <div class="context-breakdown-list">
                      ${C.map(_=>r`
                          <div class="context-breakdown-item">
                            <span class="mono">${_.name}</span>
                            <span class="muted">~${U(lt(_.injectedChars))}</span>
                          </div>
                        `)}
                    </div>
                    ${T>0?r`<div class="context-breakdown-more">+${T} more</div>`:m}
                  </div>
                `})():m}
      </div>
    </div>
  `}function am(e,t,n,s,i,a,o,l,d,g){if(t)return r`
      <div class="session-logs-compact">
        <div class="session-logs-header">Conversation</div>
        <div class="muted" style="padding: 20px; text-align: center">Loading...</div>
      </div>
    `;if(!e||e.length===0)return r`
      <div class="session-logs-compact">
        <div class="session-logs-header">Conversation</div>
        <div class="muted" style="padding: 20px; text-align: center">No messages</div>
      </div>
    `;const f=i.query.trim().toLowerCase(),p=e.map(C=>{const $=Lv(C.content),T=$.cleanContent||C.content;return{log:C,toolInfo:$,cleanContent:T}}),b=Array.from(new Set(p.flatMap(C=>C.toolInfo.tools.map(([$])=>$)))).toSorted((C,$)=>C.localeCompare($)),u=p.filter(C=>!(i.roles.length>0&&!i.roles.includes(C.log.role)||i.hasTools&&C.toolInfo.tools.length===0||i.tools.length>0&&!C.toolInfo.tools.some(([T])=>i.tools.includes(T))||f&&!C.cleanContent.toLowerCase().includes(f))),v=i.roles.length>0||i.tools.length>0||i.hasTools||f?`${u.length} of ${e.length}`:`${e.length}`,y=new Set(i.roles),k=new Set(i.tools);return r`
    <div class="session-logs-compact">
      <div class="session-logs-header">
        <span>Conversation <span style="font-weight: normal; color: var(--text-muted);">(${v} messages)</span></span>
        <button class="btn btn-sm usage-action-btn usage-secondary-btn" @click=${s}>
          ${n?"Collapse All":"Expand All"}
        </button>
      </div>
      <div class="usage-filters-inline" style="margin: 10px 12px;">
        <select
          multiple
          size="4"
          @change=${C=>a(Array.from(C.target.selectedOptions).map($=>$.value))}
        >
          <option value="user" ?selected=${y.has("user")}>User</option>
          <option value="assistant" ?selected=${y.has("assistant")}>Assistant</option>
          <option value="tool" ?selected=${y.has("tool")}>Tool</option>
          <option value="toolResult" ?selected=${y.has("toolResult")}>Tool result</option>
        </select>
        <select
          multiple
          size="4"
          @change=${C=>o(Array.from(C.target.selectedOptions).map($=>$.value))}
        >
          ${b.map(C=>r`<option value=${C} ?selected=${k.has(C)}>${C}</option>`)}
        </select>
        <label class="usage-filters-inline" style="gap: 6px;">
          <input
            type="checkbox"
            .checked=${i.hasTools}
            @change=${C=>l(C.target.checked)}
          />
          Has tools
        </label>
        <input
          type="text"
          placeholder="Search conversation"
          .value=${i.query}
          @input=${C=>d(C.target.value)}
        />
        <button class="btn btn-sm usage-action-btn usage-secondary-btn" @click=${g}>
          Clear
        </button>
      </div>
      <div class="session-logs-list">
        ${u.map(C=>{const{log:$,toolInfo:T,cleanContent:_}=C,L=$.role==="user"?"user":"assistant",E=$.role==="user"?"You":$.role==="assistant"?"Assistant":"Tool";return r`
          <div class="session-log-entry ${L}">
            <div class="session-log-meta">
              <span class="session-log-role">${E}</span>
              <span>${new Date($.timestamp).toLocaleString()}</span>
              ${$.tokens?r`<span>${U($.tokens)}</span>`:m}
            </div>
            <div class="session-log-content">${_}</div>
            ${T.tools.length>0?r`
                    <details class="session-log-tools" ?open=${n}>
                      <summary>${T.summary}</summary>
                      <div class="session-log-tools-list">
                        ${T.tools.map(([P,j])=>r`
                            <span class="session-log-tools-pill">${P} × ${j}</span>
                          `)}
                      </div>
                    </details>
                  `:m}
          </div>
        `})}
        ${u.length===0?r`
                <div class="muted" style="padding: 12px">No messages match the filters.</div>
              `:m}
      </div>
    </div>
  `}const om=`
  .usage-page-header {
    margin: 4px 0 12px;
  }
  .usage-page-title {
    font-size: 28px;
    font-weight: 700;
    letter-spacing: -0.02em;
    margin-bottom: 4px;
  }
  .usage-page-subtitle {
    font-size: 13px;
    color: var(--text-muted);
    margin: 0 0 12px;
  }
  /* ===== FILTERS & HEADER ===== */
  .usage-filters-inline {
    display: flex;
    gap: 8px;
    align-items: center;
    flex-wrap: wrap;
  }
  .usage-filters-inline select {
    padding: 6px 10px;
    border: 1px solid var(--border);
    border-radius: 6px;
    background: var(--bg);
    color: var(--text);
    font-size: 13px;
  }
  .usage-filters-inline input[type="date"] {
    padding: 6px 10px;
    border: 1px solid var(--border);
    border-radius: 6px;
    background: var(--bg);
    color: var(--text);
    font-size: 13px;
  }
  .usage-filters-inline input[type="text"] {
    padding: 6px 10px;
    border: 1px solid var(--border);
    border-radius: 6px;
    background: var(--bg);
    color: var(--text);
    font-size: 13px;
    min-width: 180px;
  }
  .usage-filters-inline .btn-sm {
    padding: 6px 12px;
    font-size: 14px;
  }
  .usage-refresh-indicator {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    padding: 4px 10px;
    background: rgba(255, 77, 77, 0.1);
    border-radius: 4px;
    font-size: 12px;
    color: #ff4d4d;
  }
  .usage-refresh-indicator::before {
    content: "";
    width: 10px;
    height: 10px;
    border: 2px solid #ff4d4d;
    border-top-color: transparent;
    border-radius: 50%;
    animation: usage-spin 0.6s linear infinite;
  }
  @keyframes usage-spin {
    to { transform: rotate(360deg); }
  }
  .active-filters {
    display: flex;
    align-items: center;
    gap: 8px;
    flex-wrap: wrap;
  }
  .filter-chip {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 4px 8px 4px 12px;
    background: var(--accent-subtle);
    border: 1px solid var(--accent);
    border-radius: 16px;
    font-size: 12px;
  }
  .filter-chip-label {
    color: var(--accent);
    font-weight: 500;
  }
  .filter-chip-remove {
    background: none;
    border: none;
    color: var(--accent);
    cursor: pointer;
    padding: 2px 4px;
    font-size: 14px;
    line-height: 1;
    opacity: 0.7;
    transition: opacity 0.15s;
  }
  .filter-chip-remove:hover {
    opacity: 1;
  }
  .filter-clear-btn {
    padding: 4px 10px !important;
    font-size: 12px !important;
    line-height: 1 !important;
    margin-left: 8px;
  }
  .usage-query-bar {
    display: grid;
    grid-template-columns: minmax(220px, 1fr) auto;
    gap: 10px;
    align-items: center;
    /* Keep the dropdown filter row from visually touching the query row. */
    margin-bottom: 10px;
  }
  .usage-query-actions {
    display: flex;
    align-items: center;
    gap: 6px;
    flex-wrap: nowrap;
    justify-self: end;
  }
  .usage-query-actions .btn {
    height: 34px;
    padding: 0 14px;
    border-radius: 999px;
    font-weight: 600;
    font-size: 13px;
    line-height: 1;
    border: 1px solid var(--border);
    background: var(--bg-secondary);
    color: var(--text);
    box-shadow: none;
    transition: background 0.15s, border-color 0.15s, color 0.15s;
  }
  .usage-query-actions .btn:hover {
    background: var(--bg);
    border-color: var(--border-strong);
  }
  .usage-action-btn {
    height: 34px;
    padding: 0 14px;
    border-radius: 999px;
    font-weight: 600;
    font-size: 13px;
    line-height: 1;
    border: 1px solid var(--border);
    background: var(--bg-secondary);
    color: var(--text);
    box-shadow: none;
    transition: background 0.15s, border-color 0.15s, color 0.15s;
  }
  .usage-action-btn:hover {
    background: var(--bg);
    border-color: var(--border-strong);
  }
  .usage-primary-btn {
    background: #ff4d4d;
    color: #fff;
    border-color: #ff4d4d;
    box-shadow: inset 0 -1px 0 rgba(0, 0, 0, 0.12);
  }
  .btn.usage-primary-btn {
    background: #ff4d4d !important;
    border-color: #ff4d4d !important;
    color: #fff !important;
  }
  .usage-primary-btn:hover {
    background: #e64545;
    border-color: #e64545;
  }
  .btn.usage-primary-btn:hover {
    background: #e64545 !important;
    border-color: #e64545 !important;
  }
  .usage-primary-btn:disabled {
    background: rgba(255, 77, 77, 0.18);
    border-color: rgba(255, 77, 77, 0.3);
    color: #ff4d4d;
    box-shadow: none;
    cursor: default;
    opacity: 1;
  }
  .usage-primary-btn[disabled] {
    background: rgba(255, 77, 77, 0.18) !important;
    border-color: rgba(255, 77, 77, 0.3) !important;
    color: #ff4d4d !important;
    opacity: 1 !important;
  }
  .usage-secondary-btn {
    background: var(--bg-secondary);
    color: var(--text);
    border-color: var(--border);
  }
  .usage-query-input {
    width: 100%;
    min-width: 220px;
    padding: 6px 10px;
    border: 1px solid var(--border);
    border-radius: 6px;
    background: var(--bg);
    color: var(--text);
    font-size: 13px;
  }
  .usage-query-suggestions {
    display: flex;
    flex-wrap: wrap;
    gap: 6px;
    margin-top: 6px;
  }
  .usage-query-suggestion {
    padding: 4px 8px;
    border-radius: 999px;
    border: 1px solid var(--border);
    background: var(--bg-secondary);
    font-size: 11px;
    color: var(--text);
    cursor: pointer;
    transition: background 0.15s;
  }
  .usage-query-suggestion:hover {
    background: var(--bg-hover);
  }
  .usage-filter-row {
    display: flex;
    flex-wrap: wrap;
    gap: 8px;
    align-items: center;
    margin-top: 14px;
  }
  details.usage-filter-select {
    position: relative;
    border: 1px solid var(--border);
    border-radius: 10px;
    padding: 6px 10px;
    background: var(--bg);
    font-size: 12px;
    min-width: 140px;
  }
  details.usage-filter-select summary {
    cursor: pointer;
    list-style: none;
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 6px;
    font-weight: 500;
  }
  details.usage-filter-select summary::-webkit-details-marker {
    display: none;
  }
  .usage-filter-badge {
    font-size: 11px;
    color: var(--text-muted);
  }
  .usage-filter-popover {
    position: absolute;
    left: 0;
    top: calc(100% + 6px);
    background: var(--bg);
    border: 1px solid var(--border);
    border-radius: 10px;
    padding: 10px;
    box-shadow: 0 10px 30px rgba(0,0,0,0.08);
    min-width: 220px;
    z-index: 20;
  }
  .usage-filter-actions {
    display: flex;
    gap: 6px;
    margin-bottom: 8px;
  }
  .usage-filter-actions button {
    border-radius: 999px;
    padding: 4px 10px;
    font-size: 11px;
  }
  .usage-filter-options {
    display: flex;
    flex-direction: column;
    gap: 6px;
    max-height: 200px;
    overflow: auto;
  }
  .usage-filter-option {
    display: flex;
    align-items: center;
    gap: 6px;
    font-size: 12px;
  }
  .usage-query-hint {
    font-size: 11px;
    color: var(--text-muted);
  }
  .usage-query-chips {
    display: flex;
    flex-wrap: wrap;
    gap: 6px;
    margin-top: 6px;
  }
  .usage-query-chip {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    padding: 4px 8px;
    border-radius: 999px;
    border: 1px solid var(--border);
    background: var(--bg-secondary);
    font-size: 11px;
  }
  .usage-query-chip button {
    background: none;
    border: none;
    color: var(--text-muted);
    cursor: pointer;
    padding: 0;
    line-height: 1;
  }
  .usage-header {
    display: flex;
    flex-direction: column;
    gap: 10px;
    background: var(--bg);
  }
  .usage-header.pinned {
    position: sticky;
    top: 12px;
    z-index: 6;
    box-shadow: 0 6px 18px rgba(0, 0, 0, 0.06);
  }
  .usage-pin-btn {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    padding: 4px 8px;
    border-radius: 999px;
    border: 1px solid var(--border);
    background: var(--bg-secondary);
    font-size: 11px;
    color: var(--text);
    cursor: pointer;
  }
  .usage-pin-btn.active {
    background: var(--accent-subtle);
    border-color: var(--accent);
    color: var(--accent);
  }
  .usage-header-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
    flex-wrap: wrap;
  }
  .usage-header-title {
    display: flex;
    align-items: center;
    gap: 10px;
  }
  .usage-header-metrics {
    display: flex;
    align-items: center;
    gap: 12px;
    flex-wrap: wrap;
  }
  .usage-metric-badge {
    display: inline-flex;
    align-items: baseline;
    gap: 6px;
    padding: 2px 8px;
    border-radius: 999px;
    border: 1px solid var(--border);
    background: transparent;
    font-size: 11px;
    color: var(--text-muted);
  }
  .usage-metric-badge strong {
    font-size: 12px;
    color: var(--text);
  }
  .usage-controls {
    display: flex;
    align-items: center;
    gap: 10px;
    flex-wrap: wrap;
  }
  .usage-controls .active-filters {
    flex: 1 1 100%;
  }
  .usage-controls input[type="date"] {
    min-width: 140px;
  }
  .usage-presets {
    display: inline-flex;
    gap: 6px;
    flex-wrap: wrap;
  }
  .usage-presets .btn {
    padding: 4px 8px;
    font-size: 11px;
  }
  .usage-quick-filters {
    display: flex;
    gap: 8px;
    align-items: center;
    flex-wrap: wrap;
  }
  .usage-select {
    min-width: 120px;
    padding: 6px 10px;
    border: 1px solid var(--border);
    border-radius: 6px;
    background: var(--bg);
    color: var(--text);
    font-size: 12px;
  }
  .usage-export-menu summary {
    cursor: pointer;
    font-weight: 500;
    color: var(--text);
    list-style: none;
    display: inline-flex;
    align-items: center;
    gap: 6px;
  }
  .usage-export-menu summary::-webkit-details-marker {
    display: none;
  }
  .usage-export-menu {
    position: relative;
  }
  .usage-export-button {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    padding: 6px 10px;
    border-radius: 8px;
    border: 1px solid var(--border);
    background: var(--bg);
    font-size: 12px;
  }
  .usage-export-popover {
    position: absolute;
    right: 0;
    top: calc(100% + 6px);
    background: var(--bg);
    border: 1px solid var(--border);
    border-radius: 10px;
    padding: 8px;
    box-shadow: 0 10px 30px rgba(0,0,0,0.08);
    min-width: 160px;
    z-index: 10;
  }
  .usage-export-list {
    display: flex;
    flex-direction: column;
    gap: 6px;
  }
  .usage-export-item {
    text-align: left;
    padding: 6px 10px;
    border-radius: 8px;
    border: 1px solid var(--border);
    background: var(--bg-secondary);
    font-size: 12px;
  }
  .usage-summary-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(150px, 1fr));
    gap: 12px;
    margin-top: 12px;
  }
  .usage-summary-card {
    padding: 12px;
    border-radius: 8px;
    background: var(--bg-secondary);
    border: 1px solid var(--border);
  }
  .usage-mosaic {
    margin-top: 16px;
    padding: 16px;
  }
  .usage-mosaic-header {
    display: flex;
    align-items: baseline;
    justify-content: space-between;
    gap: 12px;
    margin-bottom: 12px;
  }
  .usage-mosaic-title {
    font-weight: 600;
  }
  .usage-mosaic-sub {
    font-size: 12px;
    color: var(--text-muted);
  }
  .usage-mosaic-grid {
    display: grid;
    grid-template-columns: minmax(200px, 1fr) minmax(260px, 2fr);
    gap: 16px;
    align-items: start;
  }
  .usage-mosaic-section {
    background: var(--bg-subtle);
    border: 1px solid var(--border);
    border-radius: 10px;
    padding: 12px;
  }
  .usage-mosaic-section-title {
    font-size: 12px;
    font-weight: 600;
    margin-bottom: 10px;
    display: flex;
    align-items: center;
    justify-content: space-between;
  }
  .usage-mosaic-total {
    font-size: 20px;
    font-weight: 700;
  }
  .usage-daypart-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(90px, 1fr));
    gap: 8px;
  }
  .usage-daypart-cell {
    border-radius: 8px;
    padding: 10px;
    color: var(--text);
    background: rgba(255, 77, 77, 0.08);
    border: 1px solid rgba(255, 77, 77, 0.2);
    display: flex;
    flex-direction: column;
    gap: 4px;
  }
  .usage-daypart-label {
    font-size: 12px;
    font-weight: 600;
  }
  .usage-daypart-value {
    font-size: 14px;
  }
  .usage-hour-grid {
    display: grid;
    grid-template-columns: repeat(24, minmax(6px, 1fr));
    gap: 4px;
  }
  .usage-hour-cell {
    height: 28px;
    border-radius: 6px;
    background: rgba(255, 77, 77, 0.1);
    border: 1px solid rgba(255, 77, 77, 0.2);
    cursor: pointer;
    transition: border-color 0.15s, box-shadow 0.15s;
  }
  .usage-hour-cell.selected {
    border-color: rgba(255, 77, 77, 0.8);
    box-shadow: 0 0 0 2px rgba(255, 77, 77, 0.2);
  }
  .usage-hour-labels {
    display: grid;
    grid-template-columns: repeat(6, minmax(0, 1fr));
    gap: 6px;
    margin-top: 8px;
    font-size: 11px;
    color: var(--text-muted);
  }
  .usage-hour-legend {
    display: flex;
    gap: 8px;
    align-items: center;
    margin-top: 10px;
    font-size: 11px;
    color: var(--text-muted);
  }
  .usage-hour-legend span {
    display: inline-block;
    width: 14px;
    height: 10px;
    border-radius: 4px;
    background: rgba(255, 77, 77, 0.15);
    border: 1px solid rgba(255, 77, 77, 0.2);
  }
  .usage-calendar-labels {
    display: grid;
    grid-template-columns: repeat(7, minmax(10px, 1fr));
    gap: 6px;
    font-size: 10px;
    color: var(--text-muted);
    margin-bottom: 6px;
  }
  .usage-calendar {
    display: grid;
    grid-template-columns: repeat(7, minmax(10px, 1fr));
    gap: 6px;
  }
  .usage-calendar-cell {
    height: 18px;
    border-radius: 4px;
    border: 1px solid rgba(255, 77, 77, 0.2);
    background: rgba(255, 77, 77, 0.08);
  }
  .usage-calendar-cell.empty {
    background: transparent;
    border-color: transparent;
  }
  .usage-summary-title {
    font-size: 11px;
    color: var(--text-muted);
    margin-bottom: 6px;
    display: inline-flex;
    align-items: center;
    gap: 6px;
  }
  .usage-info {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 16px;
    height: 16px;
    margin-left: 6px;
    border-radius: 999px;
    border: 1px solid var(--border);
    background: var(--bg);
    font-size: 10px;
    color: var(--text-muted);
    cursor: help;
  }
  .usage-summary-value {
    font-size: 16px;
    font-weight: 600;
    color: var(--text-strong);
  }
  .usage-summary-value.good {
    color: #1f8f4e;
  }
  .usage-summary-value.warn {
    color: #c57a00;
  }
  .usage-summary-value.bad {
    color: #c9372c;
  }
  .usage-summary-hint {
    font-size: 10px;
    color: var(--text-muted);
    cursor: help;
    border: 1px solid var(--border);
    border-radius: 999px;
    padding: 0 6px;
    line-height: 16px;
    height: 16px;
    display: inline-flex;
    align-items: center;
    justify-content: center;
  }
  .usage-summary-sub {
    font-size: 11px;
    color: var(--text-muted);
    margin-top: 4px;
  }
  .usage-list {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }
  .usage-list-item {
    display: flex;
    justify-content: space-between;
    gap: 12px;
    font-size: 12px;
    color: var(--text);
    align-items: flex-start;
  }
  .usage-list-value {
    display: flex;
    flex-direction: column;
    align-items: flex-end;
    gap: 2px;
    text-align: right;
  }
  .usage-list-sub {
    font-size: 11px;
    color: var(--text-muted);
  }
  .usage-list-item.button {
    border: none;
    background: transparent;
    padding: 0;
    text-align: left;
    cursor: pointer;
  }
  .usage-list-item.button:hover {
    color: var(--text-strong);
  }
`,rm=`
  .usage-list-item .muted {
    font-size: 11px;
  }
  .usage-error-list {
    display: flex;
    flex-direction: column;
    gap: 10px;
  }
  .usage-error-row {
    display: grid;
    grid-template-columns: 1fr auto;
    gap: 8px;
    align-items: center;
    font-size: 12px;
  }
  .usage-error-date {
    font-weight: 600;
  }
  .usage-error-rate {
    font-variant-numeric: tabular-nums;
  }
  .usage-error-sub {
    grid-column: 1 / -1;
    font-size: 11px;
    color: var(--text-muted);
  }
  .usage-badges {
    display: flex;
    flex-wrap: wrap;
    gap: 6px;
    margin-bottom: 8px;
  }
  .usage-badge {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    padding: 2px 8px;
    border: 1px solid var(--border);
    border-radius: 999px;
    font-size: 11px;
    background: var(--bg);
    color: var(--text);
  }
  .usage-meta-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(160px, 1fr));
    gap: 12px;
  }
  .usage-meta-item {
    display: flex;
    flex-direction: column;
    gap: 4px;
    font-size: 12px;
  }
  .usage-meta-item span {
    color: var(--text-muted);
    font-size: 11px;
  }
  .usage-insights-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(220px, 1fr));
    gap: 16px;
    margin-top: 12px;
  }
  .usage-insight-card {
    padding: 14px;
    border-radius: 10px;
    border: 1px solid var(--border);
    background: var(--bg-secondary);
  }
  .usage-insight-title {
    font-size: 12px;
    font-weight: 600;
    margin-bottom: 10px;
  }
  .usage-insight-subtitle {
    font-size: 11px;
    color: var(--text-muted);
    margin-top: 6px;
  }
  /* ===== CHART TOGGLE ===== */
  .chart-toggle {
    display: flex;
    background: var(--bg);
    border-radius: 6px;
    overflow: hidden;
    border: 1px solid var(--border);
  }
  .chart-toggle .toggle-btn {
    padding: 6px 14px;
    font-size: 13px;
    background: transparent;
    border: none;
    color: var(--text-muted);
    cursor: pointer;
    transition: all 0.15s;
  }
  .chart-toggle .toggle-btn:hover {
    color: var(--text);
  }
  .chart-toggle .toggle-btn.active {
    background: #ff4d4d;
    color: white;
  }
  .chart-toggle.small .toggle-btn {
    padding: 4px 8px;
    font-size: 11px;
  }
  .sessions-toggle {
    border-radius: 4px;
  }
  .sessions-toggle .toggle-btn {
    border-radius: 4px;
  }
  .daily-chart-header {
    display: flex;
    align-items: center;
    justify-content: flex-start;
    gap: 8px;
    margin-bottom: 6px;
  }

  /* ===== DAILY BAR CHART ===== */
  .daily-chart {
    margin-top: 12px;
  }
  .daily-chart-bars {
    display: flex;
    align-items: flex-end;
    height: 200px;
    gap: 4px;
    padding: 8px 4px 36px;
  }
  .daily-bar-wrapper {
    flex: 1;
    display: flex;
    flex-direction: column;
    align-items: center;
    height: 100%;
    justify-content: flex-end;
    cursor: pointer;
    position: relative;
    border-radius: 4px 4px 0 0;
    transition: background 0.15s;
    min-width: 0;
  }
  .daily-bar-wrapper:hover {
    background: var(--bg-hover);
  }
  .daily-bar-wrapper.selected {
    background: var(--accent-subtle);
  }
  .daily-bar-wrapper.selected .daily-bar {
    background: var(--accent);
  }
  .daily-bar {
    width: 100%;
    max-width: var(--bar-max-width, 32px);
    background: #ff4d4d;
    border-radius: 3px 3px 0 0;
    min-height: 2px;
    transition: all 0.15s;
    overflow: hidden;
  }
  .daily-bar-wrapper:hover .daily-bar {
    background: #cc3d3d;
  }
  .daily-bar-label {
    position: absolute;
    bottom: -28px;
    font-size: 10px;
    color: var(--text-muted);
    white-space: nowrap;
    text-align: center;
    transform: rotate(-35deg);
    transform-origin: top center;
  }
  .daily-bar-total {
    position: absolute;
    top: -16px;
    left: 50%;
    transform: translateX(-50%);
    font-size: 10px;
    color: var(--text-muted);
    white-space: nowrap;
  }
  .daily-bar-tooltip {
    position: absolute;
    bottom: calc(100% + 8px);
    left: 50%;
    transform: translateX(-50%);
    background: var(--bg);
    border: 1px solid var(--border);
    border-radius: 6px;
    padding: 8px 12px;
    font-size: 12px;
    white-space: nowrap;
    z-index: 100;
    box-shadow: 0 4px 12px rgba(0,0,0,0.15);
    pointer-events: none;
    opacity: 0;
    transition: opacity 0.15s;
  }
  .daily-bar-wrapper:hover .daily-bar-tooltip {
    opacity: 1;
  }

  /* ===== COST/TOKEN BREAKDOWN BAR ===== */
  .cost-breakdown {
    margin-top: 18px;
    padding: 16px;
    background: var(--bg-secondary);
    border-radius: 8px;
  }
  .cost-breakdown-header {
    font-weight: 600;
    font-size: 15px;
    letter-spacing: -0.02em;
    margin-bottom: 12px;
    color: var(--text-strong);
  }
  .cost-breakdown-bar {
    height: 28px;
    background: var(--bg);
    border-radius: 6px;
    overflow: hidden;
    display: flex;
  }
  .cost-segment {
    height: 100%;
    transition: width 0.3s ease;
    position: relative;
  }
  .cost-segment.output {
    background: #ef4444;
  }
  .cost-segment.input {
    background: #f59e0b;
  }
  .cost-segment.cache-write {
    background: #10b981;
  }
  .cost-segment.cache-read {
    background: #06b6d4;
  }
  .cost-breakdown-legend {
    display: flex;
    flex-wrap: wrap;
    gap: 16px;
    margin-top: 12px;
  }
  .cost-breakdown-total {
    margin-top: 10px;
    font-size: 12px;
    color: var(--text-muted);
  }
  .legend-item {
    display: flex;
    align-items: center;
    gap: 6px;
    font-size: 12px;
    color: var(--text);
    cursor: help;
  }
  .legend-dot {
    width: 10px;
    height: 10px;
    border-radius: 2px;
    flex-shrink: 0;
  }
  .legend-dot.output {
    background: #ef4444;
  }
  .legend-dot.input {
    background: #f59e0b;
  }
  .legend-dot.cache-write {
    background: #10b981;
  }
  .legend-dot.cache-read {
    background: #06b6d4;
  }
  .legend-dot.system {
    background: #ff4d4d;
  }
  .legend-dot.skills {
    background: #8b5cf6;
  }
  .legend-dot.tools {
    background: #ec4899;
  }
  .legend-dot.files {
    background: #f59e0b;
  }
  .cost-breakdown-note {
    margin-top: 10px;
    font-size: 11px;
    color: var(--text-muted);
    line-height: 1.4;
  }

  /* ===== SESSION BARS (scrollable list) ===== */
  .session-bars {
    margin-top: 16px;
    max-height: 400px;
    overflow-y: auto;
    border: 1px solid var(--border);
    border-radius: 8px;
    background: var(--bg);
  }
  .session-bar-row {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 10px 14px;
    border-bottom: 1px solid var(--border);
    cursor: pointer;
    transition: background 0.15s;
  }
  .session-bar-row:last-child {
    border-bottom: none;
  }
  .session-bar-row:hover {
    background: var(--bg-hover);
  }
  .session-bar-row.selected {
    background: var(--accent-subtle);
  }
  .session-bar-label {
    flex: 1 1 auto;
    min-width: 0;
    font-size: 13px;
    color: var(--text);
    display: flex;
    flex-direction: column;
    gap: 2px;
  }
  .session-bar-title {
    /* Prefer showing the full name; wrap instead of truncating. */
    white-space: normal;
    overflow-wrap: anywhere;
    word-break: break-word;
  }
  .session-bar-meta {
    font-size: 10px;
    color: var(--text-muted);
    font-weight: 400;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .session-bar-track {
    flex: 0 0 90px;
    height: 6px;
    background: var(--bg-secondary);
    border-radius: 4px;
    overflow: hidden;
    opacity: 0.6;
  }
  .session-bar-fill {
    height: 100%;
    background: rgba(255, 77, 77, 0.7);
    border-radius: 4px;
    transition: width 0.3s ease;
  }
  .session-bar-value {
    flex: 0 0 70px;
    text-align: right;
    font-size: 12px;
    font-family: var(--font-mono);
    color: var(--text-muted);
  }
  .session-bar-actions {
    display: inline-flex;
    align-items: center;
    gap: 8px;
    flex: 0 0 auto;
  }
  .session-copy-btn {
    height: 26px;
    padding: 0 10px;
    border-radius: 999px;
    border: 1px solid var(--border);
    background: var(--bg-secondary);
    font-size: 11px;
    font-weight: 600;
    color: var(--text-muted);
    cursor: pointer;
    transition: background 0.15s, border-color 0.15s, color 0.15s;
  }
  .session-copy-btn:hover {
    background: var(--bg);
    border-color: var(--border-strong);
    color: var(--text);
  }

  /* ===== TIME SERIES CHART ===== */
  .session-timeseries {
    margin-top: 24px;
    padding: 16px;
    background: var(--bg-secondary);
    border-radius: 8px;
  }
  .timeseries-header-row {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 12px;
  }
  .timeseries-controls {
    display: flex;
    gap: 6px;
    align-items: center;
  }
  .timeseries-header {
    font-weight: 600;
    color: var(--text);
  }
  .timeseries-chart {
    width: 100%;
    overflow: hidden;
  }
  .timeseries-svg {
    width: 100%;
    height: auto;
    display: block;
  }
  .timeseries-svg .axis-label {
    font-size: 10px;
    fill: var(--text-muted);
  }
  .timeseries-svg .ts-area {
    fill: #ff4d4d;
    fill-opacity: 0.1;
  }
  .timeseries-svg .ts-line {
    fill: none;
    stroke: #ff4d4d;
    stroke-width: 2;
  }
  .timeseries-svg .ts-dot {
    fill: #ff4d4d;
    transition: r 0.15s, fill 0.15s;
  }
  .timeseries-svg .ts-dot:hover {
    r: 5;
  }
  .timeseries-svg .ts-bar {
    fill: #ff4d4d;
    transition: fill 0.15s;
  }
  .timeseries-svg .ts-bar:hover {
    fill: #cc3d3d;
  }
  .timeseries-svg .ts-bar.output { fill: #ef4444; }
  .timeseries-svg .ts-bar.input { fill: #f59e0b; }
  .timeseries-svg .ts-bar.cache-write { fill: #10b981; }
  .timeseries-svg .ts-bar.cache-read { fill: #06b6d4; }
  .timeseries-summary {
    margin-top: 12px;
    font-size: 13px;
    color: var(--text-muted);
    display: flex;
    flex-wrap: wrap;
    gap: 8px;
  }
  .timeseries-loading {
    padding: 24px;
    text-align: center;
    color: var(--text-muted);
  }

  /* ===== SESSION LOGS ===== */
  .session-logs {
    margin-top: 24px;
    background: var(--bg-secondary);
    border-radius: 8px;
    overflow: hidden;
  }
  .session-logs-header {
    padding: 10px 14px;
    font-weight: 600;
    border-bottom: 1px solid var(--border);
    display: flex;
    justify-content: space-between;
    align-items: center;
    font-size: 13px;
    background: var(--bg-secondary);
  }
  .session-logs-loading {
    padding: 24px;
    text-align: center;
    color: var(--text-muted);
  }
  .session-logs-list {
    max-height: 400px;
    overflow-y: auto;
  }
  .session-log-entry {
    padding: 10px 14px;
    border-bottom: 1px solid var(--border);
    display: flex;
    flex-direction: column;
    gap: 6px;
    background: var(--bg);
  }
  .session-log-entry:last-child {
    border-bottom: none;
  }
  .session-log-entry.user {
    border-left: 3px solid var(--accent);
  }
  .session-log-entry.assistant {
    border-left: 3px solid var(--border-strong);
  }
  .session-log-meta {
    display: flex;
    gap: 8px;
    align-items: center;
    font-size: 11px;
    color: var(--text-muted);
    flex-wrap: wrap;
  }
  .session-log-role {
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.04em;
    font-size: 10px;
    padding: 2px 6px;
    border-radius: 999px;
    background: var(--bg-secondary);
    border: 1px solid var(--border);
  }
  .session-log-entry.user .session-log-role {
    color: var(--accent);
  }
  .session-log-entry.assistant .session-log-role {
    color: var(--text-muted);
  }
  .session-log-content {
    font-size: 13px;
    line-height: 1.5;
    color: var(--text);
    white-space: pre-wrap;
    word-break: break-word;
    background: var(--bg-secondary);
    border-radius: 8px;
    padding: 8px 10px;
    border: 1px solid var(--border);
    max-height: 220px;
    overflow-y: auto;
  }

  /* ===== CONTEXT WEIGHT BREAKDOWN ===== */
  .context-weight-breakdown {
    margin-top: 24px;
    padding: 16px;
    background: var(--bg-secondary);
    border-radius: 8px;
  }
  .context-weight-breakdown .context-weight-header {
    font-weight: 600;
    font-size: 13px;
    margin-bottom: 4px;
    color: var(--text);
  }
  .context-weight-desc {
    font-size: 12px;
    color: var(--text-muted);
    margin: 0 0 12px 0;
  }
  .context-stacked-bar {
    height: 24px;
    background: var(--bg);
    border-radius: 6px;
    overflow: hidden;
    display: flex;
  }
  .context-segment {
    height: 100%;
    transition: width 0.3s ease;
  }
  .context-segment.system {
    background: #ff4d4d;
  }
  .context-segment.skills {
    background: #8b5cf6;
  }
  .context-segment.tools {
    background: #ec4899;
  }
  .context-segment.files {
    background: #f59e0b;
  }
  .context-legend {
    display: flex;
    flex-wrap: wrap;
    gap: 16px;
    margin-top: 12px;
  }
  .context-total {
    margin-top: 10px;
    font-size: 12px;
    font-weight: 600;
    color: var(--text-muted);
  }
  .context-details {
    margin-top: 12px;
    border: 1px solid var(--border);
    border-radius: 6px;
    overflow: hidden;
  }
  .context-details summary {
    padding: 10px 14px;
    font-size: 13px;
    font-weight: 500;
    cursor: pointer;
    background: var(--bg);
    border-bottom: 1px solid var(--border);
  }
  .context-details[open] summary {
    border-bottom: 1px solid var(--border);
  }
  .context-list {
    max-height: 200px;
    overflow-y: auto;
  }
  .context-list-header {
    display: flex;
    justify-content: space-between;
    padding: 8px 14px;
    font-size: 11px;
    text-transform: uppercase;
    color: var(--text-muted);
    background: var(--bg-secondary);
    border-bottom: 1px solid var(--border);
  }
  .context-list-item {
    display: flex;
    justify-content: space-between;
    padding: 8px 14px;
    font-size: 12px;
    border-bottom: 1px solid var(--border);
  }
  .context-list-item:last-child {
    border-bottom: none;
  }
  .context-list-item .mono {
    font-family: var(--font-mono);
    color: var(--text);
  }
  .context-list-item .muted {
    color: var(--text-muted);
    font-family: var(--font-mono);
  }

  /* ===== NO CONTEXT NOTE ===== */
  .no-context-note {
    margin-top: 24px;
    padding: 16px;
    background: var(--bg-secondary);
    border-radius: 8px;
    font-size: 13px;
    color: var(--text-muted);
    line-height: 1.5;
  }

  /* ===== TWO COLUMN LAYOUT ===== */
  .usage-grid {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 18px;
    margin-top: 18px;
    align-items: stretch;
  }
  .usage-grid-left {
    display: flex;
    flex-direction: column;
  }
  .usage-grid-right {
    display: flex;
    flex-direction: column;
  }
  
  /* ===== LEFT CARD (Daily + Breakdown) ===== */
  .usage-left-card {
    /* inherits background, border, shadow from .card */
    flex: 1;
    display: flex;
    flex-direction: column;
  }
  .usage-left-card .daily-chart-bars {
    flex: 1;
    min-height: 200px;
  }
  .usage-left-card .sessions-panel-title {
    font-weight: 600;
    font-size: 14px;
    margin-bottom: 12px;
  }
`,lm=`
  
  /* ===== COMPACT DAILY CHART ===== */
  .daily-chart-compact {
    margin-bottom: 16px;
  }
  .daily-chart-compact .sessions-panel-title {
    margin-bottom: 8px;
  }
  .daily-chart-compact .daily-chart-bars {
    height: 100px;
    padding-bottom: 20px;
  }
  
  /* ===== COMPACT COST BREAKDOWN ===== */
  .cost-breakdown-compact {
    padding: 0;
    margin: 0;
    background: transparent;
    border-top: 1px solid var(--border);
    padding-top: 12px;
  }
  .cost-breakdown-compact .cost-breakdown-header {
    margin-bottom: 8px;
  }
  .cost-breakdown-compact .cost-breakdown-legend {
    gap: 12px;
  }
  .cost-breakdown-compact .cost-breakdown-note {
    display: none;
  }
  
  /* ===== SESSIONS CARD ===== */
  .sessions-card {
    /* inherits background, border, shadow from .card */
    flex: 1;
    display: flex;
    flex-direction: column;
  }
  .sessions-card-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 8px;
  }
  .sessions-card-title {
    font-weight: 600;
    font-size: 14px;
  }
  .sessions-card-count {
    font-size: 12px;
    color: var(--text-muted);
  }
  .sessions-card-meta {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
    margin: 8px 0 10px;
    font-size: 12px;
    color: var(--text-muted);
  }
  .sessions-card-stats {
    display: inline-flex;
    gap: 12px;
  }
  .sessions-sort {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    font-size: 12px;
    color: var(--text-muted);
  }
  .sessions-sort select {
    padding: 4px 8px;
    border-radius: 6px;
    border: 1px solid var(--border);
    background: var(--bg);
    color: var(--text);
    font-size: 12px;
  }
  .sessions-action-btn {
    height: 28px;
    padding: 0 10px;
    border-radius: 8px;
    font-size: 12px;
    line-height: 1;
  }
  .sessions-action-btn.icon {
    width: 32px;
    padding: 0;
    display: inline-flex;
    align-items: center;
    justify-content: center;
  }
  .sessions-card-hint {
    font-size: 11px;
    color: var(--text-muted);
    margin-bottom: 8px;
  }
  .sessions-card .session-bars {
    max-height: 280px;
    background: var(--bg);
    border-radius: 6px;
    border: 1px solid var(--border);
    margin: 0;
    overflow-y: auto;
    padding: 8px;
  }
  .sessions-card .session-bar-row {
    padding: 6px 8px;
    border-radius: 6px;
    margin-bottom: 3px;
    border: 1px solid transparent;
    transition: all 0.15s;
  }
  .sessions-card .session-bar-row:hover {
    border-color: var(--border);
    background: var(--bg-hover);
  }
  .sessions-card .session-bar-row.selected {
    border-color: var(--accent);
    background: var(--accent-subtle);
    box-shadow: inset 0 0 0 1px rgba(255, 77, 77, 0.15);
  }
  .sessions-card .session-bar-label {
    flex: 1 1 auto;
    min-width: 140px;
    font-size: 12px;
  }
  .sessions-card .session-bar-value {
    flex: 0 0 60px;
    font-size: 11px;
    font-weight: 600;
  }
  .sessions-card .session-bar-track {
    flex: 0 0 70px;
    height: 5px;
    opacity: 0.5;
  }
  .sessions-card .session-bar-fill {
    background: rgba(255, 77, 77, 0.55);
  }
  .sessions-clear-btn {
    margin-left: auto;
  }
  
  /* ===== EMPTY DETAIL STATE ===== */
  .session-detail-empty {
    margin-top: 18px;
    background: var(--bg-secondary);
    border-radius: 8px;
    border: 2px dashed var(--border);
    padding: 32px;
    text-align: center;
  }
  .session-detail-empty-title {
    font-size: 15px;
    font-weight: 600;
    color: var(--text);
    margin-bottom: 8px;
  }
  .session-detail-empty-desc {
    font-size: 13px;
    color: var(--text-muted);
    margin-bottom: 16px;
    line-height: 1.5;
  }
  .session-detail-empty-features {
    display: flex;
    justify-content: center;
    gap: 24px;
    flex-wrap: wrap;
  }
  .session-detail-empty-feature {
    display: flex;
    align-items: center;
    gap: 6px;
    font-size: 12px;
    color: var(--text-muted);
  }
  .session-detail-empty-feature .icon {
    font-size: 16px;
  }
  
  /* ===== SESSION DETAIL PANEL ===== */
  .session-detail-panel {
    margin-top: 12px;
    /* inherits background, border-radius, shadow from .card */
    border: 2px solid var(--accent) !important;
  }
  .session-detail-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 8px 12px;
    border-bottom: 1px solid var(--border);
    cursor: pointer;
  }
  .session-detail-header:hover {
    background: var(--bg-hover);
  }
  .session-detail-title {
    font-weight: 600;
    font-size: 14px;
    display: flex;
    align-items: center;
    gap: 8px;
  }
  .session-detail-header-left {
    display: flex;
    align-items: center;
    gap: 8px;
  }
  .session-close-btn {
    background: var(--bg);
    border: 1px solid var(--border);
    color: var(--text);
    cursor: pointer;
    padding: 2px 8px;
    font-size: 16px;
    line-height: 1;
    border-radius: 4px;
    transition: background 0.15s, color 0.15s;
  }
  .session-close-btn:hover {
    background: var(--bg-hover);
    color: var(--text);
    border-color: var(--accent);
  }
  .session-detail-stats {
    display: flex;
    gap: 10px;
    font-size: 12px;
    color: var(--text-muted);
  }
  .session-detail-stats strong {
    color: var(--text);
    font-family: var(--font-mono);
  }
  .session-detail-content {
    padding: 12px;
  }
  .session-summary-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(140px, 1fr));
    gap: 8px;
    margin-bottom: 12px;
  }
  .session-summary-card {
    border: 1px solid var(--border);
    border-radius: 8px;
    padding: 8px;
    background: var(--bg-secondary);
  }
  .session-summary-title {
    font-size: 11px;
    color: var(--text-muted);
    margin-bottom: 4px;
  }
  .session-summary-value {
    font-size: 14px;
    font-weight: 600;
  }
  .session-summary-meta {
    font-size: 11px;
    color: var(--text-muted);
    margin-top: 4px;
  }
  .session-detail-row {
    display: grid;
    grid-template-columns: 1fr;
    gap: 10px;
    /* Separate "Usage Over Time" from the summary + Top Tools/Model Mix cards above. */
    margin-top: 12px;
    margin-bottom: 10px;
  }
  .session-detail-bottom {
    display: grid;
    grid-template-columns: minmax(0, 1.8fr) minmax(0, 1fr);
    gap: 10px;
    align-items: stretch;
  }
  .session-detail-bottom .session-logs-compact {
    margin: 0;
    display: flex;
    flex-direction: column;
  }
  .session-detail-bottom .session-logs-compact .session-logs-list {
    flex: 1 1 auto;
    max-height: none;
  }
  .context-details-panel {
    display: flex;
    flex-direction: column;
    gap: 8px;
    background: var(--bg);
    border-radius: 6px;
    border: 1px solid var(--border);
    padding: 12px;
  }
  .context-breakdown-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(160px, 1fr));
    gap: 10px;
    margin-top: 8px;
  }
  .context-breakdown-card {
    border: 1px solid var(--border);
    border-radius: 8px;
    padding: 8px;
    background: var(--bg-secondary);
  }
  .context-breakdown-title {
    font-size: 11px;
    font-weight: 600;
    margin-bottom: 6px;
  }
  .context-breakdown-list {
    display: flex;
    flex-direction: column;
    gap: 6px;
    font-size: 11px;
  }
  .context-breakdown-item {
    display: flex;
    justify-content: space-between;
    gap: 8px;
  }
  .context-breakdown-more {
    font-size: 10px;
    color: var(--text-muted);
    margin-top: 4px;
  }
  .context-breakdown-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
  }
  .context-expand-btn {
    border: 1px solid var(--border);
    background: var(--bg-secondary);
    color: var(--text-muted);
    font-size: 11px;
    padding: 4px 8px;
    border-radius: 999px;
    cursor: pointer;
    transition: all 0.15s;
  }
  .context-expand-btn:hover {
    color: var(--text);
    border-color: var(--border-strong);
    background: var(--bg);
  }
  
  /* ===== COMPACT TIMESERIES ===== */
  .session-timeseries-compact {
    background: var(--bg);
    border-radius: 6px;
    border: 1px solid var(--border);
    padding: 12px;
    margin: 0;
  }
  .session-timeseries-compact .timeseries-header-row {
    margin-bottom: 8px;
  }
  .session-timeseries-compact .timeseries-header {
    font-size: 12px;
  }
  .session-timeseries-compact .timeseries-summary {
    font-size: 11px;
    margin-top: 8px;
  }
  
  /* ===== COMPACT CONTEXT ===== */
  .context-weight-compact {
    background: var(--bg);
    border-radius: 6px;
    border: 1px solid var(--border);
    padding: 12px;
    margin: 0;
  }
  .context-weight-compact .context-weight-header {
    font-size: 12px;
    margin-bottom: 4px;
  }
  .context-weight-compact .context-weight-desc {
    font-size: 11px;
    margin-bottom: 8px;
  }
  .context-weight-compact .context-stacked-bar {
    height: 16px;
  }
  .context-weight-compact .context-legend {
    font-size: 11px;
    gap: 10px;
    margin-top: 8px;
  }
  .context-weight-compact .context-total {
    font-size: 11px;
    margin-top: 6px;
  }
  .context-weight-compact .context-details {
    margin-top: 8px;
  }
  .context-weight-compact .context-details summary {
    font-size: 12px;
    padding: 6px 10px;
  }
  
  /* ===== COMPACT LOGS ===== */
  .session-logs-compact {
    background: var(--bg);
    border-radius: 10px;
    border: 1px solid var(--border);
    overflow: hidden;
    margin: 0;
    display: flex;
    flex-direction: column;
  }
  .session-logs-compact .session-logs-header {
    padding: 10px 12px;
    font-size: 12px;
  }
  .session-logs-compact .session-logs-list {
    max-height: none;
    flex: 1 1 auto;
    overflow: auto;
  }
  .session-logs-compact .session-log-entry {
    padding: 8px 12px;
  }
  .session-logs-compact .session-log-content {
    font-size: 12px;
    max-height: 160px;
  }
  .session-log-tools {
    margin-top: 6px;
    border: 1px solid var(--border);
    border-radius: 8px;
    background: var(--bg-secondary);
    padding: 6px 8px;
    font-size: 11px;
    color: var(--text);
  }
  .session-log-tools summary {
    cursor: pointer;
    list-style: none;
    display: flex;
    align-items: center;
    gap: 6px;
    font-weight: 600;
  }
  .session-log-tools summary::-webkit-details-marker {
    display: none;
  }
  .session-log-tools-list {
    margin-top: 6px;
    display: flex;
    flex-wrap: wrap;
    gap: 6px;
  }
  .session-log-tools-pill {
    border: 1px solid var(--border);
    border-radius: 999px;
    padding: 2px 8px;
    font-size: 10px;
    background: var(--bg);
    color: var(--text);
  }

  /* ===== RESPONSIVE ===== */
  @media (max-width: 900px) {
    .usage-grid {
      grid-template-columns: 1fr;
    }
    .session-detail-row {
      grid-template-columns: 1fr;
    }
  }
  @media (max-width: 600px) {
    .session-bar-label {
      flex: 0 0 100px;
    }
    .cost-breakdown-legend {
      gap: 10px;
    }
    .legend-item {
      font-size: 11px;
    }
    .daily-chart-bars {
      height: 170px;
      gap: 6px;
      padding-bottom: 40px;
    }
    .daily-bar-label {
      font-size: 8px;
      bottom: -30px;
      transform: rotate(-45deg);
    }
    .usage-mosaic-grid {
      grid-template-columns: 1fr;
    }
    .usage-hour-grid {
      grid-template-columns: repeat(12, minmax(10px, 1fr));
    }
    .usage-hour-cell {
      height: 22px;
    }
  }
`,cm=[om,rm,lm].join(`
`);function dm(e){if(e.loading&&!e.totals)return r`
      <style>
        @keyframes initial-spin {
          to { transform: rotate(360deg); }
        }
        @keyframes initial-pulse {
          0%, 100% { opacity: 1; }
          50% { opacity: 0.7; }
        }
      </style>
      <section class="card">
        <div class="row" style="justify-content: space-between; align-items: flex-start; flex-wrap: wrap; gap: 12px;">
          <div style="flex: 1; min-width: 250px;">
            <div style="display: flex; align-items: center; gap: 10px; margin-bottom: 2px;">
              <div class="card-title" style="margin: 0;">Token Usage</div>
              <span style="
                display: inline-flex;
                align-items: center;
                gap: 6px;
                padding: 4px 10px;
                background: rgba(255, 77, 77, 0.1);
                border-radius: 4px;
                font-size: 12px;
                color: #ff4d4d;
              ">
                <span style="
                  width: 10px;
                  height: 10px;
                  border: 2px solid #ff4d4d;
                  border-top-color: transparent;
                  border-radius: 50%;
                  animation: initial-spin 0.6s linear infinite;
                "></span>
                Loading
              </span>
            </div>
          </div>
          <div style="display: flex; flex-direction: column; align-items: flex-end; gap: 8px;">
            <div style="display: flex; gap: 8px; align-items: center;">
              <input type="date" .value=${e.startDate} disabled style="padding: 6px 10px; border: 1px solid var(--border); border-radius: 6px; background: var(--bg); color: var(--text); font-size: 13px; opacity: 0.6;" />
              <span style="color: var(--text-muted);">to</span>
              <input type="date" .value=${e.endDate} disabled style="padding: 6px 10px; border: 1px solid var(--border); border-radius: 6px; background: var(--bg); color: var(--text); font-size: 13px; opacity: 0.6;" />
            </div>
          </div>
        </div>
      </section>
    `;const t=e.chartMode==="tokens",n=e.query.trim().length>0,s=e.queryDraft.trim().length>0,i=[...e.sessions].toSorted((R,D)=>{const F=t?R.usage?.totalTokens??0:R.usage?.totalCost??0;return(t?D.usage?.totalTokens??0:D.usage?.totalCost??0)-F}),a=e.selectedDays.length>0?i.filter(R=>{if(R.usage?.activityDates?.length)return R.usage.activityDates.some(W=>e.selectedDays.includes(W));if(!R.updatedAt)return!1;const D=new Date(R.updatedAt),F=`${D.getFullYear()}-${String(D.getMonth()+1).padStart(2,"0")}-${String(D.getDate()).padStart(2,"0")}`;return e.selectedDays.includes(F)}):i,o=(R,D)=>{if(D.length===0)return!0;const F=R.usage,W=F?.firstActivity??R.updatedAt,$e=F?.lastActivity??R.updatedAt;if(!W||!$e)return!1;const J=Math.min(W,$e),Se=Math.max(W,$e);let te=J;for(;te<=Se;){const he=new Date(te),Be=sa(he,e.timeZone);if(D.includes(Be))return!0;const Ue=ia(he,e.timeZone);te=Math.min(Ue.getTime(),Se)+1}return!1},l=e.selectedHours.length>0?a.filter(R=>o(R,e.selectedHours)):a,d=Ev(l,e.query),g=d.sessions,f=d.warnings,p=Wv(e.queryDraft,i,e.aggregates),b=na(e.query),u=R=>{const D=ht(R);return b.filter(F=>ht(F.key??"")===D).map(F=>F.value).filter(Boolean)},v=R=>{const D=new Set;for(const F of R)F&&D.add(F);return Array.from(D)},y=v(i.map(R=>R.agentId)).slice(0,12),k=v(i.map(R=>R.channel)).slice(0,12),C=v([...i.map(R=>R.modelProvider),...i.map(R=>R.providerOverride),...e.aggregates?.byProvider.map(R=>R.provider)??[]]).slice(0,12),$=v([...i.map(R=>R.model),...e.aggregates?.byModel.map(R=>R.model)??[]]).slice(0,12),T=v(e.aggregates?.tools.tools.map(R=>R.name)??[]).slice(0,12),_=e.selectedSessions.length===1?e.sessions.find(R=>R.key===e.selectedSessions[0])??g.find(R=>R.key===e.selectedSessions[0]):null,L=R=>R.reduce((D,F)=>(F.usage&&(D.input+=F.usage.input,D.output+=F.usage.output,D.cacheRead+=F.usage.cacheRead,D.cacheWrite+=F.usage.cacheWrite,D.totalTokens+=F.usage.totalTokens,D.totalCost+=F.usage.totalCost,D.inputCost+=F.usage.inputCost??0,D.outputCost+=F.usage.outputCost??0,D.cacheReadCost+=F.usage.cacheReadCost??0,D.cacheWriteCost+=F.usage.cacheWriteCost??0,D.missingCostEntries+=F.usage.missingCostEntries??0),D),{input:0,output:0,cacheRead:0,cacheWrite:0,totalTokens:0,totalCost:0,inputCost:0,outputCost:0,cacheReadCost:0,cacheWriteCost:0,missingCostEntries:0}),E=R=>e.costDaily.filter(F=>R.includes(F.date)).reduce((F,W)=>(F.input+=W.input,F.output+=W.output,F.cacheRead+=W.cacheRead,F.cacheWrite+=W.cacheWrite,F.totalTokens+=W.totalTokens,F.totalCost+=W.totalCost,F.inputCost+=W.inputCost??0,F.outputCost+=W.outputCost??0,F.cacheReadCost+=W.cacheReadCost??0,F.cacheWriteCost+=W.cacheWriteCost??0,F),{input:0,output:0,cacheRead:0,cacheWrite:0,totalTokens:0,totalCost:0,inputCost:0,outputCost:0,cacheReadCost:0,cacheWriteCost:0,missingCostEntries:0});let P,j;const Z=i.length;if(e.selectedSessions.length>0){const R=g.filter(D=>e.selectedSessions.includes(D.key));P=L(R),j=R.length}else e.selectedDays.length>0&&e.selectedHours.length===0?(P=E(e.selectedDays),j=g.length):e.selectedHours.length>0||n?(P=L(g),j=g.length):(P=e.totals,j=Z);const ae=e.selectedSessions.length>0?g.filter(R=>e.selectedSessions.includes(R.key)):n||e.selectedHours.length>0?g:e.selectedDays.length>0?a:i,O=Uv(ae,e.aggregates),K=e.selectedSessions.length>0?(()=>{const R=g.filter(F=>e.selectedSessions.includes(F.key)),D=new Set;for(const F of R)for(const W of F.usage?.activityDates??[])D.add(W);return D.size>0?e.costDaily.filter(F=>D.has(F.date)):e.costDaily})():e.costDaily,ue=zv(ae,P,O),M=!e.loading&&!e.totals&&e.sessions.length===0,z=(P?.missingCostEntries??0)>0||(P?P.totalTokens>0&&P.totalCost===0&&P.input+P.output+P.cacheRead+P.cacheWrite>0:!1),oe=[{label:"Today",days:1},{label:"7d",days:7},{label:"30d",days:30}],re=R=>{const D=new Date,F=new Date;F.setDate(F.getDate()-(R-1)),e.onStartDateChange(Us(F)),e.onEndDateChange(Us(D))},ee=(R,D,F)=>{if(F.length===0)return m;const W=u(R),$e=new Set(W.map(te=>ht(te))),J=F.length>0&&F.every(te=>$e.has(ht(te))),Se=W.length;return r`
      <details
        class="usage-filter-select"
        @toggle=${te=>{const he=te.currentTarget;if(!he.open)return;const Be=Ue=>{Ue.composedPath().includes(he)||(he.open=!1,window.removeEventListener("click",Be,!0))};window.addEventListener("click",Be,!0)}}
      >
        <summary>
          <span>${D}</span>
          ${Se>0?r`<span class="usage-filter-badge">${Se}</span>`:r`
                  <span class="usage-filter-badge">All</span>
                `}
        </summary>
        <div class="usage-filter-popover">
          <div class="usage-filter-actions">
            <button
              class="btn btn-sm"
              @click=${te=>{te.preventDefault(),te.stopPropagation(),e.onQueryDraftChange(So(e.queryDraft,R,F))}}
              ?disabled=${J}
            >
              Select All
            </button>
            <button
              class="btn btn-sm"
              @click=${te=>{te.preventDefault(),te.stopPropagation(),e.onQueryDraftChange(So(e.queryDraft,R,[]))}}
              ?disabled=${Se===0}
            >
              Clear
            </button>
          </div>
          <div class="usage-filter-options">
            ${F.map(te=>{const he=$e.has(ht(te));return r`
                <label class="usage-filter-option">
                  <input
                    type="checkbox"
                    .checked=${he}
                    @change=${Be=>{const Ue=Be.target,it=`${R}:${te}`;e.onQueryDraftChange(Ue.checked?Gv(e.queryDraft,it):ko(e.queryDraft,it))}}
                  />
                  <span>${te}</span>
                </label>
              `})}
          </div>
        </div>
      </details>
    `},se=Us(new Date);return r`
    <style>${cm}</style>

    <section class="usage-page-header">
      <div class="usage-page-title">Usage</div>
      <div class="usage-page-subtitle">See where tokens go, when sessions spike, and what drives cost.</div>
    </section>

    <section class="card usage-header ${e.headerPinned?"pinned":""}">
      <div class="usage-header-row">
        <div class="usage-header-title">
          <div class="card-title" style="margin: 0;">Filters</div>
          ${e.loading?r`
                  <span class="usage-refresh-indicator">Loading</span>
                `:m}
          ${M?r`
                  <span class="usage-query-hint">Select a date range and click Refresh to load usage.</span>
                `:m}
        </div>
        <div class="usage-header-metrics">
          ${P?r`
                <span class="usage-metric-badge">
                  <strong>${U(P.totalTokens)}</strong> tokens
                </span>
                <span class="usage-metric-badge">
                  <strong>${Q(P.totalCost)}</strong> cost
                </span>
                <span class="usage-metric-badge">
                  <strong>${j}</strong>
                  session${j!==1?"s":""}
                </span>
              `:m}
          <button
            class="usage-pin-btn ${e.headerPinned?"active":""}"
            title=${e.headerPinned?"Unpin filters":"Pin filters"}
            @click=${e.onToggleHeaderPinned}
          >
            ${e.headerPinned?"Pinned":"Pin"}
          </button>
          <details
            class="usage-export-menu"
            @toggle=${R=>{const D=R.currentTarget;if(!D.open)return;const F=W=>{W.composedPath().includes(D)||(D.open=!1,window.removeEventListener("click",F,!0))};window.addEventListener("click",F,!0)}}
          >
            <summary class="usage-export-button">Export ▾</summary>
            <div class="usage-export-popover">
              <div class="usage-export-list">
                <button
                  class="usage-export-item"
                  @click=${()=>zs(`aisopod-usage-sessions-${se}.csv`,Kv(g),"text/csv")}
                  ?disabled=${g.length===0}
                >
                  Sessions CSV
                </button>
                <button
                  class="usage-export-item"
                  @click=${()=>zs(`aisopod-usage-daily-${se}.csv`,jv(K),"text/csv")}
                  ?disabled=${K.length===0}
                >
                  Daily CSV
                </button>
                <button
                  class="usage-export-item"
                  @click=${()=>zs(`aisopod-usage-${se}.json`,JSON.stringify({totals:P,sessions:g,daily:K,aggregates:O},null,2),"application/json")}
                  ?disabled=${g.length===0&&K.length===0}
                >
                  JSON
                </button>
              </div>
            </div>
          </details>
        </div>
      </div>
      <div class="usage-header-row">
        <div class="usage-controls">
          ${Qv(e.selectedDays,e.selectedHours,e.selectedSessions,e.sessions,e.onClearDays,e.onClearHours,e.onClearSessions,e.onClearFilters)}
          <div class="usage-presets">
            ${oe.map(R=>r`
                <button class="btn btn-sm" @click=${()=>re(R.days)}>
                  ${R.label}
                </button>
              `)}
          </div>
          <input
            type="date"
            .value=${e.startDate}
            title="Start Date"
            @change=${R=>e.onStartDateChange(R.target.value)}
          />
          <span style="color: var(--text-muted);">to</span>
          <input
            type="date"
            .value=${e.endDate}
            title="End Date"
            @change=${R=>e.onEndDateChange(R.target.value)}
          />
          <select
            title="Time zone"
            .value=${e.timeZone}
            @change=${R=>e.onTimeZoneChange(R.target.value)}
          >
            <option value="local">Local</option>
            <option value="utc">UTC</option>
          </select>
          <div class="chart-toggle">
            <button
              class="toggle-btn ${t?"active":""}"
              @click=${()=>e.onChartModeChange("tokens")}
            >
              Tokens
            </button>
            <button
              class="toggle-btn ${t?"":"active"}"
              @click=${()=>e.onChartModeChange("cost")}
            >
              Cost
            </button>
          </div>
          <button
            class="btn btn-sm usage-action-btn usage-primary-btn"
            @click=${e.onRefresh}
            ?disabled=${e.loading}
          >
            Refresh
          </button>
        </div>
        
      </div>

      <div style="margin-top: 12px;">
          <div class="usage-query-bar">
          <input
            class="usage-query-input"
            type="text"
            .value=${e.queryDraft}
            placeholder="Filter sessions (e.g. key:agent:main:cron* model:gpt-4o has:errors minTokens:2000)"
            @input=${R=>e.onQueryDraftChange(R.target.value)}
            @keydown=${R=>{R.key==="Enter"&&(R.preventDefault(),e.onApplyQuery())}}
          />
          <div class="usage-query-actions">
            <button
              class="btn btn-sm usage-action-btn usage-secondary-btn"
              @click=${e.onApplyQuery}
              ?disabled=${e.loading||!s&&!n}
            >
              Filter (client-side)
            </button>
            ${s||n?r`<button class="btn btn-sm usage-action-btn usage-secondary-btn" @click=${e.onClearQuery}>Clear</button>`:m}
            <span class="usage-query-hint">
              ${n?`${g.length} of ${Z} sessions match`:`${Z} sessions in range`}
            </span>
          </div>
        </div>
        <div class="usage-filter-row">
          ${ee("agent","Agent",y)}
          ${ee("channel","Channel",k)}
          ${ee("provider","Provider",C)}
          ${ee("model","Model",$)}
          ${ee("tool","Tool",T)}
          <span class="usage-query-hint">
            Tip: use filters or click bars to filter days.
          </span>
        </div>
        ${b.length>0?r`
                <div class="usage-query-chips">
                  ${b.map(R=>{const D=R.raw;return r`
                      <span class="usage-query-chip">
                        ${D}
                        <button
                          title="Remove filter"
                          @click=${()=>e.onQueryDraftChange(ko(e.queryDraft,D))}
                        >
                          ×
                        </button>
                      </span>
                    `})}
                </div>
              `:m}
        ${p.length>0?r`
                <div class="usage-query-suggestions">
                  ${p.map(R=>r`
                      <button
                        class="usage-query-suggestion"
                        @click=${()=>e.onQueryDraftChange(qv(e.queryDraft,R.value))}
                      >
                        ${R.label}
                      </button>
                    `)}
                </div>
              `:m}
        ${f.length>0?r`
                <div class="callout warning" style="margin-top: 8px;">
                  ${f.join(" · ")}
                </div>
              `:m}
      </div>

      ${e.error?r`<div class="callout danger" style="margin-top: 12px;">${e.error}</div>`:m}

      ${e.sessionsLimitReached?r`
              <div class="callout warning" style="margin-top: 12px">
                Showing first 1,000 sessions. Narrow date range for complete results.
              </div>
            `:m}
    </section>

    ${Zv(P,O,ue,z,Pv(ae,e.timeZone),j,Z)}

    ${Ov(ae,e.timeZone,e.selectedHours,e.onSelectHour)}

    <!-- Two-column layout: Daily+Breakdown on left, Sessions on right -->
    <div class="usage-grid">
      <div class="usage-grid-left">
        <div class="card usage-left-card">
          ${Yv(K,e.selectedDays,e.chartMode,e.dailyChartMode,e.onDailyChartModeChange,e.onSelectDay)}
          ${P?Jv(P,e.chartMode):m}
        </div>
      </div>
      <div class="usage-grid-right">
        ${Xv(g,e.selectedSessions,e.selectedDays,t,e.sessionSort,e.sessionSortDir,e.recentSessions,e.sessionsTab,e.onSelectSession,e.onSessionSortChange,e.onSessionSortDirChange,e.onSessionsTabChange,e.visibleColumns,Z,e.onClearSessions)}
      </div>
    </div>

    <!-- Session Detail Panel (when selected) or Empty State -->
    ${_?nm(_,e.timeSeries,e.timeSeriesLoading,e.timeSeriesMode,e.onTimeSeriesModeChange,e.timeSeriesBreakdownMode,e.onTimeSeriesBreakdownChange,e.startDate,e.endDate,e.selectedDays,e.sessionLogs,e.sessionLogsLoading,e.sessionLogsExpanded,e.onToggleSessionLogsExpanded,{roles:e.logFilterRoles,tools:e.logFilterTools,hasTools:e.logFilterHasTools,query:e.logFilterQuery},e.onLogFilterRolesChange,e.onLogFilterToolsChange,e.onLogFilterHasToolsChange,e.onLogFilterQueryChange,e.onLogFilterClear,e.contextExpanded,e.onToggleContextExpanded,e.onClearSessions):em()}
  `}let Hs=null;const _o=e=>{Hs&&clearTimeout(Hs),Hs=window.setTimeout(()=>{Ll(e)},400)};function um(e){return e.tab!=="usage"?m:dm({loading:e.usageLoading,error:e.usageError,startDate:e.usageStartDate,endDate:e.usageEndDate,sessions:e.usageResult?.sessions??[],sessionsLimitReached:(e.usageResult?.sessions?.length??0)>=1e3,totals:e.usageResult?.totals??null,aggregates:e.usageResult?.aggregates??null,costDaily:e.usageCostSummary?.daily??[],selectedSessions:e.usageSelectedSessions,selectedDays:e.usageSelectedDays,selectedHours:e.usageSelectedHours,chartMode:e.usageChartMode,dailyChartMode:e.usageDailyChartMode,timeSeriesMode:e.usageTimeSeriesMode,timeSeriesBreakdownMode:e.usageTimeSeriesBreakdownMode,timeSeries:e.usageTimeSeries,timeSeriesLoading:e.usageTimeSeriesLoading,sessionLogs:e.usageSessionLogs,sessionLogsLoading:e.usageSessionLogsLoading,sessionLogsExpanded:e.usageSessionLogsExpanded,logFilterRoles:e.usageLogFilterRoles,logFilterTools:e.usageLogFilterTools,logFilterHasTools:e.usageLogFilterHasTools,logFilterQuery:e.usageLogFilterQuery,query:e.usageQuery,queryDraft:e.usageQueryDraft,sessionSort:e.usageSessionSort,sessionSortDir:e.usageSessionSortDir,recentSessions:e.usageRecentSessions,sessionsTab:e.usageSessionsTab,visibleColumns:e.usageVisibleColumns,timeZone:e.usageTimeZone,contextExpanded:e.usageContextExpanded,headerPinned:e.usageHeaderPinned,onStartDateChange:t=>{e.usageStartDate=t,e.usageSelectedDays=[],e.usageSelectedHours=[],e.usageSelectedSessions=[],_o(e)},onEndDateChange:t=>{e.usageEndDate=t,e.usageSelectedDays=[],e.usageSelectedHours=[],e.usageSelectedSessions=[],_o(e)},onRefresh:()=>Ll(e),onTimeZoneChange:t=>{e.usageTimeZone=t},onToggleContextExpanded:()=>{e.usageContextExpanded=!e.usageContextExpanded},onToggleSessionLogsExpanded:()=>{e.usageSessionLogsExpanded=!e.usageSessionLogsExpanded},onLogFilterRolesChange:t=>{e.usageLogFilterRoles=t},onLogFilterToolsChange:t=>{e.usageLogFilterTools=t},onLogFilterHasToolsChange:t=>{e.usageLogFilterHasTools=t},onLogFilterQueryChange:t=>{e.usageLogFilterQuery=t},onLogFilterClear:()=>{e.usageLogFilterRoles=[],e.usageLogFilterTools=[],e.usageLogFilterHasTools=!1,e.usageLogFilterQuery=""},onToggleHeaderPinned:()=>{e.usageHeaderPinned=!e.usageHeaderPinned},onSelectHour:(t,n)=>{if(n&&e.usageSelectedHours.length>0){const s=Array.from({length:24},(l,d)=>d),i=e.usageSelectedHours[e.usageSelectedHours.length-1],a=s.indexOf(i),o=s.indexOf(t);if(a!==-1&&o!==-1){const[l,d]=a<o?[a,o]:[o,a],g=s.slice(l,d+1);e.usageSelectedHours=[...new Set([...e.usageSelectedHours,...g])]}}else e.usageSelectedHours.includes(t)?e.usageSelectedHours=e.usageSelectedHours.filter(s=>s!==t):e.usageSelectedHours=[...e.usageSelectedHours,t]},onQueryDraftChange:t=>{e.usageQueryDraft=t,e.usageQueryDebounceTimer&&window.clearTimeout(e.usageQueryDebounceTimer),e.usageQueryDebounceTimer=window.setTimeout(()=>{e.usageQuery=e.usageQueryDraft,e.usageQueryDebounceTimer=null},250)},onApplyQuery:()=>{e.usageQueryDebounceTimer&&(window.clearTimeout(e.usageQueryDebounceTimer),e.usageQueryDebounceTimer=null),e.usageQuery=e.usageQueryDraft},onClearQuery:()=>{e.usageQueryDebounceTimer&&(window.clearTimeout(e.usageQueryDebounceTimer),e.usageQueryDebounceTimer=null),e.usageQueryDraft="",e.usageQuery=""},onSessionSortChange:t=>{e.usageSessionSort=t},onSessionSortDirChange:t=>{e.usageSessionSortDir=t},onSessionsTabChange:t=>{e.usageSessionsTab=t},onToggleColumn:t=>{e.usageVisibleColumns.includes(t)?e.usageVisibleColumns=e.usageVisibleColumns.filter(n=>n!==t):e.usageVisibleColumns=[...e.usageVisibleColumns,t]},onSelectSession:(t,n)=>{if(e.usageTimeSeries=null,e.usageSessionLogs=null,e.usageRecentSessions=[t,...e.usageRecentSessions.filter(s=>s!==t)].slice(0,8),n&&e.usageSelectedSessions.length>0){const s=e.usageChartMode==="tokens",a=[...e.usageResult?.sessions??[]].toSorted((g,f)=>{const p=s?g.usage?.totalTokens??0:g.usage?.totalCost??0;return(s?f.usage?.totalTokens??0:f.usage?.totalCost??0)-p}).map(g=>g.key),o=e.usageSelectedSessions[e.usageSelectedSessions.length-1],l=a.indexOf(o),d=a.indexOf(t);if(l!==-1&&d!==-1){const[g,f]=l<d?[l,d]:[d,l],p=a.slice(g,f+1),b=[...new Set([...e.usageSelectedSessions,...p])];e.usageSelectedSessions=b}}else e.usageSelectedSessions.length===1&&e.usageSelectedSessions[0]===t?e.usageSelectedSessions=[]:e.usageSelectedSessions=[t];e.usageSelectedSessions.length===1&&(wv(e,e.usageSelectedSessions[0]),kv(e,e.usageSelectedSessions[0]))},onSelectDay:(t,n)=>{if(n&&e.usageSelectedDays.length>0){const s=(e.usageCostSummary?.daily??[]).map(l=>l.date),i=e.usageSelectedDays[e.usageSelectedDays.length-1],a=s.indexOf(i),o=s.indexOf(t);if(a!==-1&&o!==-1){const[l,d]=a<o?[a,o]:[o,a],g=s.slice(l,d+1),f=[...new Set([...e.usageSelectedDays,...g])];e.usageSelectedDays=f}}else e.usageSelectedDays.includes(t)?e.usageSelectedDays=e.usageSelectedDays.filter(s=>s!==t):e.usageSelectedDays=[t]},onChartModeChange:t=>{e.usageChartMode=t},onDailyChartModeChange:t=>{e.usageDailyChartMode=t},onTimeSeriesModeChange:t=>{e.usageTimeSeriesMode=t},onTimeSeriesBreakdownChange:t=>{e.usageTimeSeriesBreakdownMode=t},onClearDays:()=>{e.usageSelectedDays=[]},onClearHours:()=>{e.usageSelectedHours=[]},onClearSessions:()=>{e.usageSelectedSessions=[],e.usageTimeSeries=null,e.usageSessionLogs=null},onClearFilters:()=>{e.usageSelectedDays=[],e.usageSelectedHours=[],e.usageSelectedSessions=[],e.usageTimeSeries=null,e.usageSessionLogs=null}})}const aa={CHILD:2},oa=e=>(...t)=>({_$litDirective$:e,values:t});let ra=class{constructor(t){}get _$AU(){return this._$AM._$AU}_$AT(t,n,s){this._$Ct=t,this._$AM=n,this._$Ci=s}_$AS(t,n){return this.update(t,n)}update(t,n){return this.render(...n)}};const{I:gm}=yp,Co=e=>e,pm=e=>e.strings===void 0,To=()=>document.createComment(""),Ht=(e,t,n)=>{const s=e._$AA.parentNode,i=t===void 0?e._$AB:t._$AA;if(n===void 0){const a=s.insertBefore(To(),i),o=s.insertBefore(To(),i);n=new gm(a,o,e,e.options)}else{const a=n._$AB.nextSibling,o=n._$AM,l=o!==e;if(l){let d;n._$AQ?.(e),n._$AM=e,n._$AP!==void 0&&(d=e._$AU)!==o._$AU&&n._$AP(d)}if(a!==i||l){let d=n._$AA;for(;d!==a;){const g=Co(d).nextSibling;Co(s).insertBefore(d,i),d=g}}}return n},ct=(e,t,n=e)=>(e._$AI(t,n),e),hm={},fm=(e,t=hm)=>e._$AH=t,vm=e=>e._$AH,Ks=e=>{e._$AR(),e._$AA.remove()};const Eo=(e,t,n)=>{const s=new Map;for(let i=t;i<=n;i++)s.set(e[i],i);return s},Rl=oa(class extends ra{constructor(e){if(super(e),e.type!==aa.CHILD)throw Error("repeat() can only be used in text expressions")}dt(e,t,n){let s;n===void 0?n=t:t!==void 0&&(s=t);const i=[],a=[];let o=0;for(const l of e)i[o]=s?s(l,o):o,a[o]=n(l,o),o++;return{values:a,keys:i}}render(e,t,n){return this.dt(e,t,n).values}update(e,[t,n,s]){const i=vm(e),{values:a,keys:o}=this.dt(t,n,s);if(!Array.isArray(i))return this.ut=o,a;const l=this.ut??=[],d=[];let g,f,p=0,b=i.length-1,u=0,v=a.length-1;for(;p<=b&&u<=v;)if(i[p]===null)p++;else if(i[b]===null)b--;else if(l[p]===o[u])d[u]=ct(i[p],a[u]),p++,u++;else if(l[b]===o[v])d[v]=ct(i[b],a[v]),b--,v--;else if(l[p]===o[v])d[v]=ct(i[p],a[v]),Ht(e,d[v+1],i[p]),p++,v--;else if(l[b]===o[u])d[u]=ct(i[b],a[u]),Ht(e,i[p],i[b]),b--,u++;else if(g===void 0&&(g=Eo(o,u,v),f=Eo(l,p,b)),g.has(l[p]))if(g.has(l[b])){const y=f.get(o[u]),k=y!==void 0?i[y]:null;if(k===null){const C=Ht(e,i[p]);ct(C,a[u]),d[u]=C}else d[u]=ct(k,a[u]),Ht(e,i[p],k),i[y]=null;u++}else Ks(i[b]),b--;else Ks(i[p]),p++;for(;u<=v;){const y=Ht(e,d[v+1]);ct(y,a[u]),d[u++]=y}for(;p<=b;){const y=i[p++];y!==null&&Ks(y)}return this.ut=o,fm(e,d),et}}),de={messageSquare:r`
    <svg viewBox="0 0 24 24">
      <path d="M21 15a2 2 0 0 1-2 2H7l-4 4V5a2 2 0 0 1 2-2h14a2 2 0 0 1 2 2z" />
    </svg>
  `,barChart:r`
    <svg viewBox="0 0 24 24">
      <line x1="12" x2="12" y1="20" y2="10" />
      <line x1="18" x2="18" y1="20" y2="4" />
      <line x1="6" x2="6" y1="20" y2="16" />
    </svg>
  `,link:r`
    <svg viewBox="0 0 24 24">
      <path d="M10 13a5 5 0 0 0 7.54.54l3-3a5 5 0 0 0-7.07-7.07l-1.72 1.71" />
      <path d="M14 11a5 5 0 0 0-7.54-.54l-3 3a5 5 0 0 0 7.07 7.07l1.71-1.71" />
    </svg>
  `,radio:r`
    <svg viewBox="0 0 24 24">
      <circle cx="12" cy="12" r="2" />
      <path
        d="M16.24 7.76a6 6 0 0 1 0 8.49m-8.48-.01a6 6 0 0 1 0-8.49m11.31-2.82a10 10 0 0 1 0 14.14m-14.14 0a10 10 0 0 1 0-14.14"
      />
    </svg>
  `,fileText:r`
    <svg viewBox="0 0 24 24">
      <path d="M14.5 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V7.5L14.5 2z" />
      <polyline points="14 2 14 8 20 8" />
      <line x1="16" x2="8" y1="13" y2="13" />
      <line x1="16" x2="8" y1="17" y2="17" />
      <line x1="10" x2="8" y1="9" y2="9" />
    </svg>
  `,zap:r`
    <svg viewBox="0 0 24 24"><polygon points="13 2 3 14 12 14 11 22 21 10 12 10 13 2" /></svg>
  `,monitor:r`
    <svg viewBox="0 0 24 24">
      <rect width="20" height="14" x="2" y="3" rx="2" />
      <line x1="8" x2="16" y1="21" y2="21" />
      <line x1="12" x2="12" y1="17" y2="21" />
    </svg>
  `,settings:r`
    <svg viewBox="0 0 24 24">
      <path
        d="M12.22 2h-.44a2 2 0 0 0-2 2v.18a2 2 0 0 1-1 1.73l-.43.25a2 2 0 0 1-2 0l-.15-.08a2 2 0 0 0-2.73.73l-.22.38a2 2 0 0 0 .73 2.73l.15.1a2 2 0 0 1 1 1.72v.51a2 2 0 0 1-1 1.74l-.15.09a2 2 0 0 0-.73 2.73l.22.38a2 2 0 0 0 2.73.73l.15-.08a2 2 0 0 1 2 0l.43.25a2 2 0 0 1 1 1.73V20a2 2 0 0 0 2 2h.44a2 2 0 0 0 2-2v-.18a2 2 0 0 1 1-1.73l.43-.25a2 2 0 0 1 2 0l.15.08a2 2 0 0 0 2.73-.73l.22-.39a2 2 0 0 0-.73-2.73l-.15-.08a2 2 0 0 1-1-1.74v-.5a2 2 0 0 1 1-1.74l.15-.09a2 2 0 0 0 .73-2.73l-.22-.38a2 2 0 0 0-2.73-.73l-.15.08a2 2 0 0 1-2 0l-.43-.25a2 2 0 0 1-1-1.73V4a2 2 0 0 0-2-2z"
      />
      <circle cx="12" cy="12" r="3" />
    </svg>
  `,bug:r`
    <svg viewBox="0 0 24 24">
      <path d="m8 2 1.88 1.88" />
      <path d="M14.12 3.88 16 2" />
      <path d="M9 7.13v-1a3.003 3.003 0 1 1 6 0v1" />
      <path d="M12 20c-3.3 0-6-2.7-6-6v-3a4 4 0 0 1 4-4h4a4 4 0 0 1 4 4v3c0 3.3-2.7 6-6 6" />
      <path d="M12 20v-9" />
      <path d="M6.53 9C4.6 8.8 3 7.1 3 5" />
      <path d="M6 13H2" />
      <path d="M3 21c0-2.1 1.7-3.9 3.8-4" />
      <path d="M20.97 5c0 2.1-1.6 3.8-3.5 4" />
      <path d="M22 13h-4" />
      <path d="M17.2 17c2.1.1 3.8 1.9 3.8 4" />
    </svg>
  `,scrollText:r`
    <svg viewBox="0 0 24 24">
      <path d="M8 21h12a2 2 0 0 0 2-2v-2H10v2a2 2 0 1 1-4 0V5a2 2 0 1 0-4 0v3h4" />
      <path d="M19 17V5a2 2 0 0 0-2-2H4" />
      <path d="M15 8h-5" />
      <path d="M15 12h-5" />
    </svg>
  `,folder:r`
    <svg viewBox="0 0 24 24">
      <path
        d="M20 20a2 2 0 0 0 2-2V8a2 2 0 0 0-2-2h-7.9a2 2 0 0 1-1.69-.9L9.6 3.9A2 2 0 0 0 7.93 3H4a2 2 0 0 0-2 2v13a2 2 0 0 0 2 2Z"
      />
    </svg>
  `,menu:r`
    <svg viewBox="0 0 24 24">
      <line x1="4" x2="20" y1="12" y2="12" />
      <line x1="4" x2="20" y1="6" y2="6" />
      <line x1="4" x2="20" y1="18" y2="18" />
    </svg>
  `,x:r`
    <svg viewBox="0 0 24 24">
      <path d="M18 6 6 18" />
      <path d="m6 6 12 12" />
    </svg>
  `,check:r`
    <svg viewBox="0 0 24 24"><path d="M20 6 9 17l-5-5" /></svg>
  `,arrowDown:r`
    <svg viewBox="0 0 24 24">
      <path d="M12 5v14" />
      <path d="m19 12-7 7-7-7" />
    </svg>
  `,copy:r`
    <svg viewBox="0 0 24 24">
      <rect width="14" height="14" x="8" y="8" rx="2" ry="2" />
      <path d="M4 16c-1.1 0-2-.9-2-2V4c0-1.1.9-2 2-2h10c1.1 0 2 .9 2 2" />
    </svg>
  `,search:r`
    <svg viewBox="0 0 24 24">
      <circle cx="11" cy="11" r="8" />
      <path d="m21 21-4.3-4.3" />
    </svg>
  `,brain:r`
    <svg viewBox="0 0 24 24">
      <path d="M12 5a3 3 0 1 0-5.997.125 4 4 0 0 0-2.526 5.77 4 4 0 0 0 .556 6.588A4 4 0 1 0 12 18Z" />
      <path d="M12 5a3 3 0 1 1 5.997.125 4 4 0 0 1 2.526 5.77 4 4 0 0 1-.556 6.588A4 4 0 1 1 12 18Z" />
      <path d="M15 13a4.5 4.5 0 0 1-3-4 4.5 4.5 0 0 1-3 4" />
      <path d="M17.599 6.5a3 3 0 0 0 .399-1.375" />
      <path d="M6.003 5.125A3 3 0 0 0 6.401 6.5" />
      <path d="M3.477 10.896a4 4 0 0 1 .585-.396" />
      <path d="M19.938 10.5a4 4 0 0 1 .585.396" />
      <path d="M6 18a4 4 0 0 1-1.967-.516" />
      <path d="M19.967 17.484A4 4 0 0 1 18 18" />
    </svg>
  `,book:r`
    <svg viewBox="0 0 24 24">
      <path d="M4 19.5v-15A2.5 2.5 0 0 1 6.5 2H20v20H6.5a2.5 2.5 0 0 1 0-5H20" />
    </svg>
  `,loader:r`
    <svg viewBox="0 0 24 24">
      <path d="M12 2v4" />
      <path d="m16.2 7.8 2.9-2.9" />
      <path d="M18 12h4" />
      <path d="m16.2 16.2 2.9 2.9" />
      <path d="M12 18v4" />
      <path d="m4.9 19.1 2.9-2.9" />
      <path d="M2 12h4" />
      <path d="m4.9 4.9 2.9 2.9" />
    </svg>
  `,wrench:r`
    <svg viewBox="0 0 24 24">
      <path
        d="M14.7 6.3a1 1 0 0 0 0 1.4l1.6 1.6a1 1 0 0 0 1.4 0l3.77-3.77a6 6 0 0 1-7.94 7.94l-6.91 6.91a2.12 2.12 0 0 1-3-3l6.91-6.91a6 6 0 0 1 7.94-7.94l-3.76 3.76z"
      />
    </svg>
  `,fileCode:r`
    <svg viewBox="0 0 24 24">
      <path d="M14.5 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V7.5L14.5 2z" />
      <polyline points="14 2 14 8 20 8" />
      <path d="m10 13-2 2 2 2" />
      <path d="m14 17 2-2-2-2" />
    </svg>
  `,edit:r`
    <svg viewBox="0 0 24 24">
      <path d="M11 4H4a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2v-7" />
      <path d="M18.5 2.5a2.121 2.121 0 0 1 3 3L12 15l-4 1 1-4 9.5-9.5z" />
    </svg>
  `,penLine:r`
    <svg viewBox="0 0 24 24">
      <path d="M12 20h9" />
      <path d="M16.5 3.5a2.121 2.121 0 0 1 3 3L7 19l-4 1 1-4L16.5 3.5z" />
    </svg>
  `,paperclip:r`
    <svg viewBox="0 0 24 24">
      <path
        d="m21.44 11.05-9.19 9.19a6 6 0 0 1-8.49-8.49l8.57-8.57A4 4 0 1 1 18 8.84l-8.59 8.57a2 2 0 0 1-2.83-2.83l8.49-8.48"
      />
    </svg>
  `,globe:r`
    <svg viewBox="0 0 24 24">
      <circle cx="12" cy="12" r="10" />
      <path d="M12 2a14.5 14.5 0 0 0 0 20 14.5 14.5 0 0 0 0-20" />
      <path d="M2 12h20" />
    </svg>
  `,image:r`
    <svg viewBox="0 0 24 24">
      <rect width="18" height="18" x="3" y="3" rx="2" ry="2" />
      <circle cx="9" cy="9" r="2" />
      <path d="m21 15-3.086-3.086a2 2 0 0 0-2.828 0L6 21" />
    </svg>
  `,smartphone:r`
    <svg viewBox="0 0 24 24">
      <rect width="14" height="20" x="5" y="2" rx="2" ry="2" />
      <path d="M12 18h.01" />
    </svg>
  `,plug:r`
    <svg viewBox="0 0 24 24">
      <path d="M12 22v-5" />
      <path d="M9 8V2" />
      <path d="M15 8V2" />
      <path d="M18 8v5a4 4 0 0 1-4 4h-4a4 4 0 0 1-4-4V8Z" />
    </svg>
  `,circle:r`
    <svg viewBox="0 0 24 24"><circle cx="12" cy="12" r="10" /></svg>
  `,puzzle:r`
    <svg viewBox="0 0 24 24">
      <path
        d="M19.439 7.85c-.049.322.059.648.289.878l1.568 1.568c.47.47.706 1.087.706 1.704s-.235 1.233-.706 1.704l-1.611 1.611a.98.98 0 0 1-.837.276c-.47-.07-.802-.48-.968-.925a2.501 2.501 0 1 0-3.214 3.214c.446.166.855.497.925.968a.979.979 0 0 1-.276.837l-1.61 1.61a2.404 2.404 0 0 1-1.705.707 2.402 2.402 0 0 1-1.704-.706l-1.568-1.568a1.026 1.026 0 0 0-.877-.29c-.493.074-.84.504-1.02.968a2.5 2.5 0 1 1-3.237-3.237c.464-.18.894-.527.967-1.02a1.026 1.026 0 0 0-.289-.877l-1.568-1.568A2.402 2.402 0 0 1 1.998 12c0-.617.236-1.234.706-1.704L4.23 8.77c.24-.24.581-.353.917-.303.515.076.874.54 1.02 1.02a2.5 2.5 0 1 0 3.237-3.237c-.48-.146-.944-.505-1.02-1.02a.98.98 0 0 1 .303-.917l1.526-1.526A2.402 2.402 0 0 1 11.998 2c.617 0 1.234.236 1.704.706l1.568 1.568c.23.23.556.338.877.29.493-.074.84-.504 1.02-.968a2.5 2.5 0 1 1 3.236 3.236c-.464.18-.894.527-.967 1.02Z"
      />
    </svg>
  `};function mm(e){const t=e.hello?.snapshot,n=t?.sessionDefaults?.mainSessionKey?.trim();if(n)return n;const s=t?.sessionDefaults?.mainKey?.trim();return s||"main"}function bm(e,t){e.sessionKey=t,e.chatMessage="",e.chatStream=null,e.chatStreamStartedAt=null,e.chatRunId=null,e.resetToolStream(),e.resetChatScroll(),e.applySettings({...e.settings,sessionKey:t,lastActiveSessionKey:t})}function ym(e,t){const n=gs(t,e.basePath);return r`
    <a
      href=${n}
      class="nav-item ${e.tab===t?"active":""}"
      @click=${s=>{if(!(s.defaultPrevented||s.button!==0||s.metaKey||s.ctrlKey||s.shiftKey||s.altKey)){if(s.preventDefault(),t==="chat"){const i=mm(e);e.sessionKey!==i&&(bm(e,i),e.loadAssistantIdentity())}e.setTab(t)}}}
      title=${li(t)}
    >
      <span class="nav-item__icon" aria-hidden="true">${de[lf(t)]}</span>
      <span class="nav-item__text">${li(t)}</span>
    </a>
  `}function xm(e){const t=$m(e.hello,e.sessionsResult),n=wm(e.sessionKey,e.sessionsResult,t),s=e.onboarding,i=e.onboarding,a=e.onboarding?!1:e.settings.chatShowThinking,o=e.onboarding?!0:e.settings.chatFocusMode,l=r`
    <svg
      width="18"
      height="18"
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      stroke-width="2"
      stroke-linecap="round"
      stroke-linejoin="round"
    >
      <path d="M21 12a9 9 0 1 1-9-9c2.52 0 4.93 1 6.74 2.74L21 8"></path>
      <path d="M21 3v5h-5"></path>
    </svg>
  `,d=r`
    <svg
      width="18"
      height="18"
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      stroke-width="2"
      stroke-linecap="round"
      stroke-linejoin="round"
    >
      <path d="M4 7V4h3"></path>
      <path d="M20 7V4h-3"></path>
      <path d="M4 17v3h3"></path>
      <path d="M20 17v3h-3"></path>
      <circle cx="12" cy="12" r="3"></circle>
    </svg>
  `;return r`
    <div class="chat-controls">
      <label class="field chat-controls__session">
        <select
          .value=${e.sessionKey}
          ?disabled=${!e.connected}
          @change=${g=>{const f=g.target.value;e.sessionKey=f,e.chatMessage="",e.chatStream=null,e.chatStreamStartedAt=null,e.chatRunId=null,e.resetToolStream(),e.resetChatScroll(),e.applySettings({...e.settings,sessionKey:f,lastActiveSessionKey:f}),e.loadAssistantIdentity(),Sf(e,f),gn(e)}}
        >
          ${Rl(n,g=>g.key,g=>r`<option value=${g.key}>
                ${g.displayName??g.key}
              </option>`)}
        </select>
      </label>
      <button
        class="btn btn--sm btn--icon"
        ?disabled=${e.chatLoading||!e.connected}
        @click=${async()=>{const g=e;g.chatManualRefreshInFlight=!0,g.chatNewMessagesBelow=!1,await g.updateComplete,g.resetToolStream();try{await Al(e,{scheduleScroll:!1}),g.scrollToBottom({smooth:!0})}finally{requestAnimationFrame(()=>{g.chatManualRefreshInFlight=!1,g.chatNewMessagesBelow=!1})}}}
        title="Refresh chat data"
      >
        ${l}
      </button>
      <span class="chat-controls__separator">|</span>
      <button
        class="btn btn--sm btn--icon ${a?"active":""}"
        ?disabled=${s}
        @click=${()=>{s||e.applySettings({...e.settings,chatShowThinking:!e.settings.chatShowThinking})}}
        aria-pressed=${a}
        title=${s?"Disabled during onboarding":"Toggle assistant thinking/working output"}
      >
        ${de.brain}
      </button>
      <button
        class="btn btn--sm btn--icon ${o?"active":""}"
        ?disabled=${i}
        @click=${()=>{i||e.applySettings({...e.settings,chatFocusMode:!e.settings.chatFocusMode})}}
        aria-pressed=${o}
        title=${i?"Disabled during onboarding":"Toggle focus mode (hide sidebar + page header)"}
      >
        ${d}
      </button>
    </div>
  `}function $m(e,t){const n=e?.snapshot,s=n?.sessionDefaults?.mainSessionKey?.trim();if(s)return s;const i=n?.sessionDefaults?.mainKey?.trim();return i||(t?.sessions?.some(a=>a.key==="main")?"main":null)}function js(e,t){const n=t?.displayName?.trim()||"",s=t?.label?.trim()||"";return n&&n!==e?`${n} (${e})`:s&&s!==e?`${s} (${e})`:e}function wm(e,t,n){const s=new Set,i=[],a=n&&t?.sessions?.find(l=>l.key===n),o=t?.sessions?.find(l=>l.key===e);if(n&&(s.add(n),i.push({key:n,displayName:js(n,a||void 0)})),s.has(e)||(s.add(e),i.push({key:e,displayName:js(e,o)})),t?.sessions)for(const l of t.sessions)s.has(l.key)||(s.add(l.key),i.push({key:l.key,displayName:js(l.key,l)}));return i}const km=["system","light","dark"];function Sm(e){const t=Math.max(0,km.indexOf(e.theme)),n=s=>i=>{const o={element:i.currentTarget};(i.clientX||i.clientY)&&(o.pointerClientX=i.clientX,o.pointerClientY=i.clientY),e.setTheme(s,o)};return r`
    <div class="theme-toggle" style="--theme-index: ${t};">
      <div class="theme-toggle__track" role="group" aria-label="Theme">
        <span class="theme-toggle__indicator"></span>
        <button
          class="theme-toggle__button ${e.theme==="system"?"active":""}"
          @click=${n("system")}
          aria-pressed=${e.theme==="system"}
          aria-label="System theme"
          title="System"
        >
          ${Cm()}
        </button>
        <button
          class="theme-toggle__button ${e.theme==="light"?"active":""}"
          @click=${n("light")}
          aria-pressed=${e.theme==="light"}
          aria-label="Light theme"
          title="Light"
        >
          ${Am()}
        </button>
        <button
          class="theme-toggle__button ${e.theme==="dark"?"active":""}"
          @click=${n("dark")}
          aria-pressed=${e.theme==="dark"}
          aria-label="Dark theme"
          title="Dark"
        >
          ${_m()}
        </button>
      </div>
    </div>
  `}function Am(){return r`
    <svg class="theme-icon" viewBox="0 0 24 24" aria-hidden="true">
      <circle cx="12" cy="12" r="4"></circle>
      <path d="M12 2v2"></path>
      <path d="M12 20v2"></path>
      <path d="m4.93 4.93 1.41 1.41"></path>
      <path d="m17.66 17.66 1.41 1.41"></path>
      <path d="M2 12h2"></path>
      <path d="M20 12h2"></path>
      <path d="m6.34 17.66-1.41 1.41"></path>
      <path d="m19.07 4.93-1.41 1.41"></path>
    </svg>
  `}function _m(){return r`
    <svg class="theme-icon" viewBox="0 0 24 24" aria-hidden="true">
      <path
        d="M20.985 12.486a9 9 0 1 1-9.473-9.472c.405-.022.617.46.402.803a6 6 0 0 0 8.268 8.268c.344-.215.825-.004.803.401"
      ></path>
    </svg>
  `}function Cm(){return r`
    <svg class="theme-icon" viewBox="0 0 24 24" aria-hidden="true">
      <rect width="20" height="14" x="2" y="3" rx="2"></rect>
      <line x1="8" x2="16" y1="21" y2="21"></line>
      <line x1="12" x2="12" y1="17" y2="21"></line>
    </svg>
  `}function Pl(e,t){if(!e)return e;const s=e.files.some(i=>i.name===t.name)?e.files.map(i=>i.name===t.name?t:i):[...e.files,t];return{...e,files:s}}async function Ws(e,t){if(!(!e.client||!e.connected||e.agentFilesLoading)){e.agentFilesLoading=!0,e.agentFilesError=null;try{const n=await e.client.request("agents.files.list",{agentId:t});n&&(e.agentFilesList=n,e.agentFileActive&&!n.files.some(s=>s.name===e.agentFileActive)&&(e.agentFileActive=null))}catch(n){e.agentFilesError=String(n)}finally{e.agentFilesLoading=!1}}}async function Tm(e,t,n,s){if(!(!e.client||!e.connected||e.agentFilesLoading)&&!Object.hasOwn(e.agentFileContents,n)){e.agentFilesLoading=!0,e.agentFilesError=null;try{const i=await e.client.request("agents.files.get",{agentId:t,name:n});if(i?.file){const a=i.file.content??"",o=e.agentFileContents[n]??"",l=e.agentFileDrafts[n],d=s?.preserveDraft??!0;e.agentFilesList=Pl(e.agentFilesList,i.file),e.agentFileContents={...e.agentFileContents,[n]:a},(!d||!Object.hasOwn(e.agentFileDrafts,n)||l===o)&&(e.agentFileDrafts={...e.agentFileDrafts,[n]:a})}}catch(i){e.agentFilesError=String(i)}finally{e.agentFilesLoading=!1}}}async function Em(e,t,n,s){if(!(!e.client||!e.connected||e.agentFileSaving)){e.agentFileSaving=!0,e.agentFilesError=null;try{const i=await e.client.request("agents.files.set",{agentId:t,name:n,content:s});i?.file&&(e.agentFilesList=Pl(e.agentFilesList,i.file),e.agentFileContents={...e.agentFileContents,[n]:s},e.agentFileDrafts={...e.agentFileDrafts,[n]:s})}catch(i){e.agentFilesError=String(i)}finally{e.agentFileSaving=!1}}}function Lm(e){const t=e.host??"unknown",n=e.ip?`(${e.ip})`:"",s=e.mode??"",i=e.version??"";return`${t} ${n} ${s} ${i}`.trim()}function Mm(e){const t=e.ts??null;return t?Y(t):"n/a"}function la(e){return e?`${$t(e)} (${Y(e)})`:"n/a"}function Im(e){if(e.totalTokens==null)return"n/a";const t=e.totalTokens??0,n=e.contextTokens??0;return n?`${t} / ${n}`:String(t)}function Rm(e){if(e==null)return"";try{return JSON.stringify(e,null,2)}catch{return String(e)}}function Pm(e){const t=e.state??{},n=t.nextRunAtMs?$t(t.nextRunAtMs):"n/a",s=t.lastRunAtMs?$t(t.lastRunAtMs):"n/a";return`${t.lastStatus??"n/a"} · next ${n} · last ${s}`}function Dl(e){const t=e.schedule;if(t.kind==="at"){const n=Date.parse(t.at);return Number.isFinite(n)?`At ${$t(n)}`:`At ${t.at}`}return t.kind==="every"?`Every ${ji(t.everyMs)}`:`Cron ${t.expr}${t.tz?` (${t.tz})`:""}`}function Dm(e){const t=e.payload;if(t.kind==="systemEvent")return`System: ${t.text}`;const n=`Agent: ${t.message}`,s=e.delivery;if(s&&s.mode!=="none"){const i=s.channel||s.to?` (${s.channel??"last"}${s.to?` -> ${s.to}`:""})`:"";return`${n} · ${s.mode}${i}`}return n}const Fm={bash:"exec","apply-patch":"apply_patch"},Nm={"group:memory":["memory_search","memory_get"],"group:web":["web_search","web_fetch"],"group:fs":["read","write","edit","apply_patch"],"group:runtime":["exec","process"],"group:sessions":["sessions_list","sessions_history","sessions_send","sessions_spawn","subagents","session_status"],"group:ui":["browser","canvas"],"group:automation":["cron","gateway"],"group:messaging":["message"],"group:nodes":["nodes"],"group:openclaw":["browser","canvas","nodes","cron","message","gateway","agents_list","sessions_list","sessions_history","sessions_send","sessions_spawn","subagents","session_status","memory_search","memory_get","web_search","web_fetch","image"]},Om={minimal:{allow:["session_status"]},coding:{allow:["group:fs","group:runtime","group:sessions","group:memory","image"]},messaging:{allow:["group:messaging","sessions_list","sessions_history","sessions_send","session_status"]},full:{}};function Oe(e){const t=e.trim().toLowerCase();return Fm[t]??t}function Bm(e){return e?e.map(Oe).filter(Boolean):[]}function Um(e){const t=Bm(e),n=[];for(const s of t){const i=Nm[s];if(i){n.push(...i);continue}n.push(s)}return Array.from(new Set(n))}function zm(e){if(!e)return;const t=Om[e];if(t&&!(!t.allow&&!t.deny))return{allow:t.allow?[...t.allow]:void 0,deny:t.deny?[...t.deny]:void 0}}const Lo=[{id:"fs",label:"Files",tools:[{id:"read",label:"read",description:"Read file contents"},{id:"write",label:"write",description:"Create or overwrite files"},{id:"edit",label:"edit",description:"Make precise edits"},{id:"apply_patch",label:"apply_patch",description:"Patch files (OpenAI)"}]},{id:"runtime",label:"Runtime",tools:[{id:"exec",label:"exec",description:"Run shell commands"},{id:"process",label:"process",description:"Manage background processes"}]},{id:"web",label:"Web",tools:[{id:"web_search",label:"web_search",description:"Search the web"},{id:"web_fetch",label:"web_fetch",description:"Fetch web content"}]},{id:"memory",label:"Memory",tools:[{id:"memory_search",label:"memory_search",description:"Semantic search"},{id:"memory_get",label:"memory_get",description:"Read memory files"}]},{id:"sessions",label:"Sessions",tools:[{id:"sessions_list",label:"sessions_list",description:"List sessions"},{id:"sessions_history",label:"sessions_history",description:"Session history"},{id:"sessions_send",label:"sessions_send",description:"Send to session"},{id:"sessions_spawn",label:"sessions_spawn",description:"Spawn sub-agent"},{id:"session_status",label:"session_status",description:"Session status"}]},{id:"ui",label:"UI",tools:[{id:"browser",label:"browser",description:"Control web browser"},{id:"canvas",label:"canvas",description:"Control canvases"}]},{id:"messaging",label:"Messaging",tools:[{id:"message",label:"message",description:"Send messages"}]},{id:"automation",label:"Automation",tools:[{id:"cron",label:"cron",description:"Schedule tasks"},{id:"gateway",label:"gateway",description:"Gateway control"}]},{id:"nodes",label:"Nodes",tools:[{id:"nodes",label:"nodes",description:"Nodes + devices"}]},{id:"agents",label:"Agents",tools:[{id:"agents_list",label:"agents_list",description:"List agents"}]},{id:"media",label:"Media",tools:[{id:"image",label:"image",description:"Image understanding"}]}],Hm=[{id:"minimal",label:"Minimal"},{id:"coding",label:"Coding"},{id:"messaging",label:"Messaging"},{id:"full",label:"Full"}];function fi(e){return e.name?.trim()||e.identity?.name?.trim()||e.id}function Pn(e){const t=e.trim();if(!t||t.length>16)return!1;let n=!1;for(let s=0;s<t.length;s+=1)if(t.charCodeAt(s)>127){n=!0;break}return!(!n||t.includes("://")||t.includes("/")||t.includes("."))}function fs(e,t){const n=t?.emoji?.trim();if(n&&Pn(n))return n;const s=e.identity?.emoji?.trim();if(s&&Pn(s))return s;const i=t?.avatar?.trim();if(i&&Pn(i))return i;const a=e.identity?.avatar?.trim();return a&&Pn(a)?a:""}function Fl(e,t){return t&&e===t?"default":null}function Km(e){if(e==null||!Number.isFinite(e))return"-";if(e<1024)return`${e} B`;const t=["KB","MB","GB","TB"];let n=e/1024,s=0;for(;n>=1024&&s<t.length-1;)n/=1024,s+=1;return`${n.toFixed(n<10?1:0)} ${t[s]}`}function vs(e,t){const n=e;return{entry:(n?.agents?.list??[]).find(a=>a?.id===t),defaults:n?.agents?.defaults,globalTools:n?.tools}}function Mo(e,t,n,s,i){const a=vs(t,e.id),l=(n&&n.agentId===e.id?n.workspace:null)||a.entry?.workspace||a.defaults?.workspace||"default",d=a.entry?.model?tn(a.entry?.model):tn(a.defaults?.model),g=i?.name?.trim()||e.identity?.name?.trim()||e.name?.trim()||a.entry?.name||e.id,f=fs(e,i)||"-",p=Array.isArray(a.entry?.skills)?a.entry?.skills:null,b=p?.length??null;return{workspace:l,model:d,identityName:g,identityEmoji:f,skillsLabel:p?`${b} selected`:"all skills",isDefault:!!(s&&e.id===s)}}function tn(e){if(!e)return"-";if(typeof e=="string")return e.trim()||"-";if(typeof e=="object"&&e){const t=e,n=t.primary?.trim();if(n){const s=Array.isArray(t.fallbacks)?t.fallbacks.length:0;return s>0?`${n} (+${s} fallback)`:n}}return"-"}function Io(e){const t=e.match(/^(.+) \(\+\d+ fallback\)$/);return t?t[1]:e}function Ro(e){if(!e)return null;if(typeof e=="string")return e.trim()||null;if(typeof e=="object"&&e){const t=e;return(typeof t.primary=="string"?t.primary:typeof t.model=="string"?t.model:typeof t.id=="string"?t.id:typeof t.value=="string"?t.value:null)?.trim()||null}return null}function jm(e){if(!e||typeof e=="string")return null;if(typeof e=="object"&&e){const t=e,n=Array.isArray(t.fallbacks)?t.fallbacks:Array.isArray(t.fallback)?t.fallback:null;return n?n.filter(s=>typeof s=="string"):null}return null}function Wm(e){return e.split(",").map(t=>t.trim()).filter(Boolean)}function qm(e){const n=e?.agents?.defaults?.models;if(!n||typeof n!="object")return[];const s=[];for(const[i,a]of Object.entries(n)){const o=i.trim();if(!o)continue;const l=a&&typeof a=="object"&&"alias"in a&&typeof a.alias=="string"?a.alias?.trim():void 0,d=l&&l!==o?`${l} (${o})`:o;s.push({value:o,label:d})}return s}function Gm(e,t){const n=qm(e),s=t?n.some(i=>i.value===t):!1;return t&&!s&&n.unshift({value:t,label:`Current (${t})`}),n.length===0?r`
      <option value="" disabled>No configured models</option>
    `:n.map(i=>r`<option value=${i.value}>${i.label}</option>`)}function Vm(e){const t=Oe(e);if(!t)return{kind:"exact",value:""};if(t==="*")return{kind:"all"};if(!t.includes("*"))return{kind:"exact",value:t};const n=t.replace(/[.*+?^${}()|[\\]\\]/g,"\\$&");return{kind:"regex",value:new RegExp(`^${n.replaceAll("\\*",".*")}$`)}}function vi(e){return Array.isArray(e)?Um(e).map(Vm).filter(t=>t.kind!=="exact"||t.value.length>0):[]}function nn(e,t){for(const n of t)if(n.kind==="all"||n.kind==="exact"&&e===n.value||n.kind==="regex"&&n.value.test(e))return!0;return!1}function Qm(e,t){if(!t)return!0;const n=Oe(e),s=vi(t.deny);if(nn(n,s))return!1;const i=vi(t.allow);return!!(i.length===0||nn(n,i)||n==="apply_patch"&&nn("exec",i))}function Po(e,t){if(!Array.isArray(t)||t.length===0)return!1;const n=Oe(e),s=vi(t);return!!(nn(n,s)||n==="apply_patch"&&nn("exec",s))}function Ym(e){return zm(e)??void 0}function Nl(e,t){return r`
    <section class="card">
      <div class="card-title">Agent Context</div>
      <div class="card-sub">${t}</div>
      <div class="agents-overview-grid" style="margin-top: 16px;">
        <div class="agent-kv">
          <div class="label">Workspace</div>
          <div class="mono">${e.workspace}</div>
        </div>
        <div class="agent-kv">
          <div class="label">Primary Model</div>
          <div class="mono">${e.model}</div>
        </div>
        <div class="agent-kv">
          <div class="label">Identity Name</div>
          <div>${e.identityName}</div>
        </div>
        <div class="agent-kv">
          <div class="label">Identity Emoji</div>
          <div>${e.identityEmoji}</div>
        </div>
        <div class="agent-kv">
          <div class="label">Skills Filter</div>
          <div>${e.skillsLabel}</div>
        </div>
        <div class="agent-kv">
          <div class="label">Default</div>
          <div>${e.isDefault?"yes":"no"}</div>
        </div>
      </div>
    </section>
  `}function Jm(e,t){const n=e.channelMeta?.find(s=>s.id===t);return n?.label?n.label:e.channelLabels?.[t]??t}function Zm(e){if(!e)return[];const t=new Set;for(const i of e.channelOrder??[])t.add(i);for(const i of e.channelMeta??[])t.add(i.id);for(const i of Object.keys(e.channelAccounts??{}))t.add(i);const n=[],s=e.channelOrder?.length?e.channelOrder:Array.from(t);for(const i of s)t.has(i)&&(n.push(i),t.delete(i));for(const i of t)n.push(i);return n.map(i=>({id:i,label:Jm(e,i),accounts:e.channelAccounts?.[i]??[]}))}const Xm=["groupPolicy","streamMode","dmPolicy"];function eb(e,t){if(!e)return null;const s=(e.channels??{})[t];if(s&&typeof s=="object")return s;const i=e[t];return i&&typeof i=="object"?i:null}function tb(e){if(e==null)return"n/a";if(typeof e=="string"||typeof e=="number"||typeof e=="boolean")return String(e);try{return JSON.stringify(e)}catch{return"n/a"}}function nb(e,t){const n=eb(e,t);return n?Xm.flatMap(s=>s in n?[{label:s,value:tb(n[s])}]:[]):[]}function sb(e){let t=0,n=0,s=0;for(const i of e){const a=i.probe&&typeof i.probe=="object"&&"ok"in i.probe?!!i.probe.ok:!1;(i.connected===!0||i.running===!0||a)&&(t+=1),i.configured&&(n+=1),i.enabled&&(s+=1)}return{total:e.length,connected:t,configured:n,enabled:s}}function ib(e){const t=Zm(e.snapshot),n=e.lastSuccess?Y(e.lastSuccess):"never";return r`
    <section class="grid grid-cols-2">
      ${Nl(e.context,"Workspace, identity, and model configuration.")}
      <section class="card">
        <div class="row" style="justify-content: space-between;">
          <div>
            <div class="card-title">Channels</div>
            <div class="card-sub">Gateway-wide channel status snapshot.</div>
          </div>
          <button class="btn btn--sm" ?disabled=${e.loading} @click=${e.onRefresh}>
            ${e.loading?"Refreshing…":"Refresh"}
          </button>
        </div>
        <div class="muted" style="margin-top: 8px;">
          Last refresh: ${n}
        </div>
        ${e.error?r`<div class="callout danger" style="margin-top: 12px;">${e.error}</div>`:m}
        ${e.snapshot?m:r`
                <div class="callout info" style="margin-top: 12px">Load channels to see live status.</div>
              `}
        ${t.length===0?r`
                <div class="muted" style="margin-top: 16px">No channels found.</div>
              `:r`
                <div class="list" style="margin-top: 16px;">
                  ${t.map(s=>{const i=sb(s.accounts),a=i.total?`${i.connected}/${i.total} connected`:"no accounts",o=i.configured?`${i.configured} configured`:"not configured",l=i.total?`${i.enabled} enabled`:"disabled",d=nb(e.configForm,s.id);return r`
                      <div class="list-item">
                        <div class="list-main">
                          <div class="list-title">${s.label}</div>
                          <div class="list-sub mono">${s.id}</div>
                        </div>
                        <div class="list-meta">
                          <div>${a}</div>
                          <div>${o}</div>
                          <div>${l}</div>
                          ${d.length>0?d.map(g=>r`<div>${g.label}: ${g.value}</div>`):m}
                        </div>
                      </div>
                    `})}
                </div>
              `}
      </section>
    </section>
  `}function ab(e){const t=e.jobs.filter(n=>n.agentId===e.agentId);return r`
    <section class="grid grid-cols-2">
      ${Nl(e.context,"Workspace and scheduling targets.")}
      <section class="card">
        <div class="row" style="justify-content: space-between;">
          <div>
            <div class="card-title">Scheduler</div>
            <div class="card-sub">Gateway cron status.</div>
          </div>
          <button class="btn btn--sm" ?disabled=${e.loading} @click=${e.onRefresh}>
            ${e.loading?"Refreshing…":"Refresh"}
          </button>
        </div>
        <div class="stat-grid" style="margin-top: 16px;">
          <div class="stat">
            <div class="stat-label">Enabled</div>
            <div class="stat-value">
              ${e.status?e.status.enabled?"Yes":"No":"n/a"}
            </div>
          </div>
          <div class="stat">
            <div class="stat-label">Jobs</div>
            <div class="stat-value">${e.status?.jobs??"n/a"}</div>
          </div>
          <div class="stat">
            <div class="stat-label">Next wake</div>
            <div class="stat-value">${la(e.status?.nextWakeAtMs??null)}</div>
          </div>
        </div>
        ${e.error?r`<div class="callout danger" style="margin-top: 12px;">${e.error}</div>`:m}
      </section>
    </section>
    <section class="card">
      <div class="card-title">Agent Cron Jobs</div>
      <div class="card-sub">Scheduled jobs targeting this agent.</div>
      ${t.length===0?r`
              <div class="muted" style="margin-top: 16px">No jobs assigned.</div>
            `:r`
              <div class="list" style="margin-top: 16px;">
                ${t.map(n=>r`
                    <div class="list-item">
                      <div class="list-main">
                        <div class="list-title">${n.name}</div>
                        ${n.description?r`<div class="list-sub">${n.description}</div>`:m}
                        <div class="chip-row" style="margin-top: 6px;">
                          <span class="chip">${Dl(n)}</span>
                          <span class="chip ${n.enabled?"chip-ok":"chip-warn"}">
                            ${n.enabled?"enabled":"disabled"}
                          </span>
                          <span class="chip">${n.sessionTarget}</span>
                        </div>
                      </div>
                      <div class="list-meta">
                        <div class="mono">${Pm(n)}</div>
                        <div class="muted">${Dm(n)}</div>
                      </div>
                    </div>
                  `)}
              </div>
            `}
    </section>
  `}function ob(e){const t=e.agentFilesList?.agentId===e.agentId?e.agentFilesList:null,n=t?.files??[],s=e.agentFileActive??null,i=s?n.find(d=>d.name===s)??null:null,a=s?e.agentFileContents[s]??"":"",o=s?e.agentFileDrafts[s]??a:"",l=s?o!==a:!1;return r`
    <section class="card">
      <div class="row" style="justify-content: space-between;">
        <div>
          <div class="card-title">Core Files</div>
          <div class="card-sub">Bootstrap persona, identity, and tool guidance.</div>
        </div>
        <button
          class="btn btn--sm"
          ?disabled=${e.agentFilesLoading}
          @click=${()=>e.onLoadFiles(e.agentId)}
        >
          ${e.agentFilesLoading?"Loading…":"Refresh"}
        </button>
      </div>
      ${t?r`<div class="muted mono" style="margin-top: 8px;">Workspace: ${t.workspace}</div>`:m}
      ${e.agentFilesError?r`<div class="callout danger" style="margin-top: 12px;">${e.agentFilesError}</div>`:m}
      ${t?r`
              <div class="agent-files-grid" style="margin-top: 16px;">
                <div class="agent-files-list">
                  ${n.length===0?r`
                          <div class="muted">No files found.</div>
                        `:n.map(d=>rb(d,s,()=>e.onSelectFile(d.name)))}
                </div>
                <div class="agent-files-editor">
                  ${i?r`
                          <div class="agent-file-header">
                            <div>
                              <div class="agent-file-title mono">${i.name}</div>
                              <div class="agent-file-sub mono">${i.path}</div>
                            </div>
                            <div class="agent-file-actions">
                              <button
                                class="btn btn--sm"
                                ?disabled=${!l}
                                @click=${()=>e.onFileReset(i.name)}
                              >
                                Reset
                              </button>
                              <button
                                class="btn btn--sm primary"
                                ?disabled=${e.agentFileSaving||!l}
                                @click=${()=>e.onFileSave(i.name)}
                              >
                                ${e.agentFileSaving?"Saving…":"Save"}
                              </button>
                            </div>
                          </div>
                          ${i.missing?r`
                                  <div class="callout info" style="margin-top: 10px">
                                    This file is missing. Saving will create it in the agent workspace.
                                  </div>
                                `:m}
                          <label class="field" style="margin-top: 12px;">
                            <span>Content</span>
                            <textarea
                              .value=${o}
                              @input=${d=>e.onFileDraftChange(i.name,d.target.value)}
                            ></textarea>
                          </label>
                        `:r`
                          <div class="muted">Select a file to edit.</div>
                        `}
                </div>
              </div>
            `:r`
              <div class="callout info" style="margin-top: 12px">
                Load the agent workspace files to edit core instructions.
              </div>
            `}
    </section>
  `}function rb(e,t,n){const s=e.missing?"Missing":`${Km(e.size)} · ${Y(e.updatedAtMs??null)}`;return r`
    <button
      type="button"
      class="agent-file-row ${t===e.name?"active":""}"
      @click=${n}
    >
      <div>
        <div class="agent-file-name mono">${e.name}</div>
        <div class="agent-file-meta">${s}</div>
      </div>
      ${e.missing?r`
              <span class="agent-pill warn">missing</span>
            `:m}
    </button>
  `}const Dn=[{id:"workspace",label:"Workspace Skills",sources:["aisopod-workspace"]},{id:"built-in",label:"Built-in Skills",sources:["aisopod-bundled"]},{id:"installed",label:"Installed Skills",sources:["aisopod-managed"]},{id:"extra",label:"Extra Skills",sources:["aisopod-extra"]}];function Ol(e){const t=new Map;for(const a of Dn)t.set(a.id,{id:a.id,label:a.label,skills:[]});const n=Dn.find(a=>a.id==="built-in"),s={id:"other",label:"Other Skills",skills:[]};for(const a of e){const o=a.bundled?n:Dn.find(l=>l.sources.includes(a.source));o?t.get(o.id)?.skills.push(a):s.skills.push(a)}const i=Dn.map(a=>t.get(a.id)).filter(a=>!!(a&&a.skills.length>0));return s.skills.length>0&&i.push(s),i}function Bl(e){return[...e.missing.bins.map(t=>`bin:${t}`),...e.missing.env.map(t=>`env:${t}`),...e.missing.config.map(t=>`config:${t}`),...e.missing.os.map(t=>`os:${t}`)]}function Ul(e){const t=[];return e.disabled&&t.push("disabled"),e.blockedByAllowlist&&t.push("blocked by allowlist"),t}function zl(e){const t=e.skill,n=!!e.showBundledBadge;return r`
    <div class="chip-row" style="margin-top: 6px;">
      <span class="chip">${t.source}</span>
      ${n?r`
              <span class="chip">bundled</span>
            `:m}
      <span class="chip ${t.eligible?"chip-ok":"chip-warn"}">
        ${t.eligible?"eligible":"blocked"}
      </span>
      ${t.disabled?r`
              <span class="chip chip-warn">disabled</span>
            `:m}
    </div>
  `}function lb(e){const t=vs(e.configForm,e.agentId),n=t.entry?.tools??{},s=t.globalTools??{},i=n.profile??s.profile??"full",a=n.profile?"agent override":s.profile?"global default":"default",o=Array.isArray(n.allow)&&n.allow.length>0,l=Array.isArray(s.allow)&&s.allow.length>0,d=!!e.configForm&&!e.configLoading&&!e.configSaving&&!o,g=o?[]:Array.isArray(n.alsoAllow)?n.alsoAllow:[],f=o?[]:Array.isArray(n.deny)?n.deny:[],p=o?{allow:n.allow??[],deny:n.deny??[]}:Ym(i)??void 0,b=Lo.flatMap(C=>C.tools.map($=>$.id)),u=C=>{const $=Qm(C,p),T=Po(C,g),_=Po(C,f);return{allowed:($||T)&&!_,baseAllowed:$,denied:_}},v=b.filter(C=>u(C).allowed).length,y=(C,$)=>{const T=new Set(g.map(P=>Oe(P)).filter(P=>P.length>0)),_=new Set(f.map(P=>Oe(P)).filter(P=>P.length>0)),L=u(C).baseAllowed,E=Oe(C);$?(_.delete(E),L||T.add(E)):(T.delete(E),_.add(E)),e.onOverridesChange(e.agentId,[...T],[..._])},k=C=>{const $=new Set(g.map(_=>Oe(_)).filter(_=>_.length>0)),T=new Set(f.map(_=>Oe(_)).filter(_=>_.length>0));for(const _ of b){const L=u(_).baseAllowed,E=Oe(_);C?(T.delete(E),L||$.add(E)):($.delete(E),T.add(E))}e.onOverridesChange(e.agentId,[...$],[...T])};return r`
    <section class="card">
      <div class="row" style="justify-content: space-between;">
        <div>
          <div class="card-title">Tool Access</div>
          <div class="card-sub">
            Profile + per-tool overrides for this agent.
            <span class="mono">${v}/${b.length}</span> enabled.
          </div>
        </div>
        <div class="row" style="gap: 8px;">
          <button class="btn btn--sm" ?disabled=${!d} @click=${()=>k(!0)}>
            Enable All
          </button>
          <button class="btn btn--sm" ?disabled=${!d} @click=${()=>k(!1)}>
            Disable All
          </button>
          <button class="btn btn--sm" ?disabled=${e.configLoading} @click=${e.onConfigReload}>
            Reload Config
          </button>
          <button
            class="btn btn--sm primary"
            ?disabled=${e.configSaving||!e.configDirty}
            @click=${e.onConfigSave}
          >
            ${e.configSaving?"Saving…":"Save"}
          </button>
        </div>
      </div>

      ${e.configForm?m:r`
              <div class="callout info" style="margin-top: 12px">
                Load the gateway config to adjust tool profiles.
              </div>
            `}
      ${o?r`
              <div class="callout info" style="margin-top: 12px">
                This agent is using an explicit allowlist in config. Tool overrides are managed in the Config tab.
              </div>
            `:m}
      ${l?r`
              <div class="callout info" style="margin-top: 12px">
                Global tools.allow is set. Agent overrides cannot enable tools that are globally blocked.
              </div>
            `:m}

      <div class="agent-tools-meta" style="margin-top: 16px;">
        <div class="agent-kv">
          <div class="label">Profile</div>
          <div class="mono">${i}</div>
        </div>
        <div class="agent-kv">
          <div class="label">Source</div>
          <div>${a}</div>
        </div>
        ${e.configDirty?r`
                <div class="agent-kv">
                  <div class="label">Status</div>
                  <div class="mono">unsaved</div>
                </div>
              `:m}
      </div>

      <div class="agent-tools-presets" style="margin-top: 16px;">
        <div class="label">Quick Presets</div>
        <div class="agent-tools-buttons">
          ${Hm.map(C=>r`
              <button
                class="btn btn--sm ${i===C.id?"active":""}"
                ?disabled=${!d}
                @click=${()=>e.onProfileChange(e.agentId,C.id,!0)}
              >
                ${C.label}
              </button>
            `)}
          <button
            class="btn btn--sm"
            ?disabled=${!d}
            @click=${()=>e.onProfileChange(e.agentId,null,!1)}
          >
            Inherit
          </button>
        </div>
      </div>

      <div class="agent-tools-grid" style="margin-top: 20px;">
        ${Lo.map(C=>r`
              <div class="agent-tools-section">
                <div class="agent-tools-header">${C.label}</div>
                <div class="agent-tools-list">
                  ${C.tools.map($=>{const{allowed:T}=u($.id);return r`
                      <div class="agent-tool-row">
                        <div>
                          <div class="agent-tool-title mono">${$.label}</div>
                          <div class="agent-tool-sub">${$.description}</div>
                        </div>
                        <label class="cfg-toggle">
                          <input
                            type="checkbox"
                            .checked=${T}
                            ?disabled=${!d}
                            @change=${_=>y($.id,_.target.checked)}
                          />
                          <span class="cfg-toggle__track"></span>
                        </label>
                      </div>
                    `})}
                </div>
              </div>
            `)}
      </div>
    </section>
  `}function cb(e){const t=!!e.configForm&&!e.configLoading&&!e.configSaving,n=vs(e.configForm,e.agentId),s=Array.isArray(n.entry?.skills)?n.entry?.skills:void 0,i=new Set((s??[]).map(u=>u.trim()).filter(Boolean)),a=s!==void 0,o=!!(e.report&&e.activeAgentId===e.agentId),l=o?e.report?.skills??[]:[],d=e.filter.trim().toLowerCase(),g=d?l.filter(u=>[u.name,u.description,u.source].join(" ").toLowerCase().includes(d)):l,f=Ol(g),p=a?l.filter(u=>i.has(u.name)).length:l.length,b=l.length;return r`
    <section class="card">
      <div class="row" style="justify-content: space-between;">
        <div>
          <div class="card-title">Skills</div>
          <div class="card-sub">
            Per-agent skill allowlist and workspace skills.
            ${b>0?r`<span class="mono">${p}/${b}</span>`:m}
          </div>
        </div>
        <div class="row" style="gap: 8px;">
          <button class="btn btn--sm" ?disabled=${!t} @click=${()=>e.onClear(e.agentId)}>
            Use All
          </button>
          <button
            class="btn btn--sm"
            ?disabled=${!t}
            @click=${()=>e.onDisableAll(e.agentId)}
          >
            Disable All
          </button>
          <button class="btn btn--sm" ?disabled=${e.configLoading} @click=${e.onConfigReload}>
            Reload Config
          </button>
          <button class="btn btn--sm" ?disabled=${e.loading} @click=${e.onRefresh}>
            ${e.loading?"Loading…":"Refresh"}
          </button>
          <button
            class="btn btn--sm primary"
            ?disabled=${e.configSaving||!e.configDirty}
            @click=${e.onConfigSave}
          >
            ${e.configSaving?"Saving…":"Save"}
          </button>
        </div>
      </div>

      ${e.configForm?m:r`
              <div class="callout info" style="margin-top: 12px">
                Load the gateway config to set per-agent skills.
              </div>
            `}
      ${a?r`
              <div class="callout info" style="margin-top: 12px">This agent uses a custom skill allowlist.</div>
            `:r`
              <div class="callout info" style="margin-top: 12px">
                All skills are enabled. Disabling any skill will create a per-agent allowlist.
              </div>
            `}
      ${!o&&!e.loading?r`
              <div class="callout info" style="margin-top: 12px">
                Load skills for this agent to view workspace-specific entries.
              </div>
            `:m}
      ${e.error?r`<div class="callout danger" style="margin-top: 12px;">${e.error}</div>`:m}

      <div class="filters" style="margin-top: 14px;">
        <label class="field" style="flex: 1;">
          <span>Filter</span>
          <input
            .value=${e.filter}
            @input=${u=>e.onFilterChange(u.target.value)}
            placeholder="Search skills"
          />
        </label>
        <div class="muted">${g.length} shown</div>
      </div>

      ${g.length===0?r`
              <div class="muted" style="margin-top: 16px">No skills found.</div>
            `:r`
              <div class="agent-skills-groups" style="margin-top: 16px;">
                ${f.map(u=>db(u,{agentId:e.agentId,allowSet:i,usingAllowlist:a,editable:t,onToggle:e.onToggle}))}
              </div>
            `}
    </section>
  `}function db(e,t){const n=e.id==="workspace"||e.id==="built-in";return r`
    <details class="agent-skills-group" ?open=${!n}>
      <summary class="agent-skills-header">
        <span>${e.label}</span>
        <span class="muted">${e.skills.length}</span>
      </summary>
      <div class="list skills-grid">
        ${e.skills.map(s=>ub(s,{agentId:t.agentId,allowSet:t.allowSet,usingAllowlist:t.usingAllowlist,editable:t.editable,onToggle:t.onToggle}))}
      </div>
    </details>
  `}function ub(e,t){const n=t.usingAllowlist?t.allowSet.has(e.name):!0,s=Bl(e),i=Ul(e);return r`
    <div class="list-item agent-skill-row">
      <div class="list-main">
        <div class="list-title">${e.emoji?`${e.emoji} `:""}${e.name}</div>
        <div class="list-sub">${e.description}</div>
        ${zl({skill:e})}
        ${s.length>0?r`<div class="muted" style="margin-top: 6px;">Missing: ${s.join(", ")}</div>`:m}
        ${i.length>0?r`<div class="muted" style="margin-top: 6px;">Reason: ${i.join(", ")}</div>`:m}
      </div>
      <div class="list-meta">
        <label class="cfg-toggle">
          <input
            type="checkbox"
            .checked=${n}
            ?disabled=${!t.editable}
            @change=${a=>t.onToggle(t.agentId,e.name,a.target.checked)}
          />
          <span class="cfg-toggle__track"></span>
        </label>
      </div>
    </div>
  `}function gb(e){const t=e.agentsList?.agents??[],n=e.agentsList?.defaultId??null,s=e.selectedAgentId??n??t[0]?.id??null,i=s?t.find(a=>a.id===s)??null:null;return r`
    <div class="agents-layout">
      <section class="card agents-sidebar">
        <div class="row" style="justify-content: space-between;">
          <div>
            <div class="card-title">Agents</div>
            <div class="card-sub">${t.length} configured.</div>
          </div>
          <button class="btn btn--sm" ?disabled=${e.loading} @click=${e.onRefresh}>
            ${e.loading?"Loading…":"Refresh"}
          </button>
        </div>
        ${e.error?r`<div class="callout danger" style="margin-top: 12px;">${e.error}</div>`:m}
        <div class="agent-list" style="margin-top: 12px;">
          ${t.length===0?r`
                  <div class="muted">No agents found.</div>
                `:t.map(a=>{const o=Fl(a.id,n),l=fs(a,e.agentIdentityById[a.id]??null);return r`
                    <button
                      type="button"
                      class="agent-row ${s===a.id?"active":""}"
                      @click=${()=>e.onSelectAgent(a.id)}
                    >
                      <div class="agent-avatar">${l||fi(a).slice(0,1)}</div>
                      <div class="agent-info">
                        <div class="agent-title">${fi(a)}</div>
                        <div class="agent-sub mono">${a.id}</div>
                      </div>
                      ${o?r`<span class="agent-pill">${o}</span>`:m}
                    </button>
                  `})}
        </div>
      </section>
      <section class="agents-main">
        ${i?r`
                ${pb(i,n,e.agentIdentityById[i.id]??null)}
                ${hb(e.activePanel,a=>e.onSelectPanel(a))}
                ${e.activePanel==="overview"?fb({agent:i,defaultId:n,configForm:e.configForm,agentFilesList:e.agentFilesList,agentIdentity:e.agentIdentityById[i.id]??null,agentIdentityError:e.agentIdentityError,agentIdentityLoading:e.agentIdentityLoading,configLoading:e.configLoading,configSaving:e.configSaving,configDirty:e.configDirty,onConfigReload:e.onConfigReload,onConfigSave:e.onConfigSave,onModelChange:e.onModelChange,onModelFallbacksChange:e.onModelFallbacksChange}):m}
                ${e.activePanel==="files"?ob({agentId:i.id,agentFilesList:e.agentFilesList,agentFilesLoading:e.agentFilesLoading,agentFilesError:e.agentFilesError,agentFileActive:e.agentFileActive,agentFileContents:e.agentFileContents,agentFileDrafts:e.agentFileDrafts,agentFileSaving:e.agentFileSaving,onLoadFiles:e.onLoadFiles,onSelectFile:e.onSelectFile,onFileDraftChange:e.onFileDraftChange,onFileReset:e.onFileReset,onFileSave:e.onFileSave}):m}
                ${e.activePanel==="tools"?lb({agentId:i.id,configForm:e.configForm,configLoading:e.configLoading,configSaving:e.configSaving,configDirty:e.configDirty,onProfileChange:e.onToolsProfileChange,onOverridesChange:e.onToolsOverridesChange,onConfigReload:e.onConfigReload,onConfigSave:e.onConfigSave}):m}
                ${e.activePanel==="skills"?cb({agentId:i.id,report:e.agentSkillsReport,loading:e.agentSkillsLoading,error:e.agentSkillsError,activeAgentId:e.agentSkillsAgentId,configForm:e.configForm,configLoading:e.configLoading,configSaving:e.configSaving,configDirty:e.configDirty,filter:e.skillsFilter,onFilterChange:e.onSkillsFilterChange,onRefresh:e.onSkillsRefresh,onToggle:e.onAgentSkillToggle,onClear:e.onAgentSkillsClear,onDisableAll:e.onAgentSkillsDisableAll,onConfigReload:e.onConfigReload,onConfigSave:e.onConfigSave}):m}
                ${e.activePanel==="channels"?ib({context:Mo(i,e.configForm,e.agentFilesList,n,e.agentIdentityById[i.id]??null),configForm:e.configForm,snapshot:e.channelsSnapshot,loading:e.channelsLoading,error:e.channelsError,lastSuccess:e.channelsLastSuccess,onRefresh:e.onChannelsRefresh}):m}
                ${e.activePanel==="cron"?ab({context:Mo(i,e.configForm,e.agentFilesList,n,e.agentIdentityById[i.id]??null),agentId:i.id,jobs:e.cronJobs,status:e.cronStatus,loading:e.cronLoading,error:e.cronError,onRefresh:e.onCronRefresh}):m}
              `:r`
                <div class="card">
                  <div class="card-title">Select an agent</div>
                  <div class="card-sub">Pick an agent to inspect its workspace and tools.</div>
                </div>
              `}
      </section>
    </div>
  `}function pb(e,t,n){const s=Fl(e.id,t),i=fi(e),a=e.identity?.theme?.trim()||"Agent workspace and routing.",o=fs(e,n);return r`
    <section class="card agent-header">
      <div class="agent-header-main">
        <div class="agent-avatar agent-avatar--lg">${o||i.slice(0,1)}</div>
        <div>
          <div class="card-title">${i}</div>
          <div class="card-sub">${a}</div>
        </div>
      </div>
      <div class="agent-header-meta">
        <div class="mono">${e.id}</div>
        ${s?r`<span class="agent-pill">${s}</span>`:m}
      </div>
    </section>
  `}function hb(e,t){return r`
    <div class="agent-tabs">
      ${[{id:"overview",label:"Overview"},{id:"files",label:"Files"},{id:"tools",label:"Tools"},{id:"skills",label:"Skills"},{id:"channels",label:"Channels"},{id:"cron",label:"Cron Jobs"}].map(s=>r`
          <button
            class="agent-tab ${e===s.id?"active":""}"
            type="button"
            @click=${()=>t(s.id)}
          >
            ${s.label}
          </button>
        `)}
    </div>
  `}function fb(e){const{agent:t,configForm:n,agentFilesList:s,agentIdentity:i,agentIdentityLoading:a,agentIdentityError:o,configLoading:l,configSaving:d,configDirty:g,onConfigReload:f,onConfigSave:p,onModelChange:b,onModelFallbacksChange:u}=e,v=vs(n,t.id),k=(s&&s.agentId===t.id?s.workspace:null)||v.entry?.workspace||v.defaults?.workspace||"default",C=v.entry?.model?tn(v.entry?.model):tn(v.defaults?.model),$=tn(v.defaults?.model),T=Ro(v.entry?.model)||(C!=="-"?Io(C):null),_=Ro(v.defaults?.model)||($!=="-"?Io($):null),L=T??_??null,E=jm(v.entry?.model),P=E?E.join(", "):"",j=i?.name?.trim()||t.identity?.name?.trim()||t.name?.trim()||v.entry?.name||"-",ae=fs(t,i)||"-",O=Array.isArray(v.entry?.skills)?v.entry?.skills:null,K=O?.length??null,ue=a?"Loading…":o?"Unavailable":"",M=!!(e.defaultId&&t.id===e.defaultId);return r`
    <section class="card">
      <div class="card-title">Overview</div>
      <div class="card-sub">Workspace paths and identity metadata.</div>
      <div class="agents-overview-grid" style="margin-top: 16px;">
        <div class="agent-kv">
          <div class="label">Workspace</div>
          <div class="mono">${k}</div>
        </div>
        <div class="agent-kv">
          <div class="label">Primary Model</div>
          <div class="mono">${C}</div>
        </div>
        <div class="agent-kv">
          <div class="label">Identity Name</div>
          <div>${j}</div>
          ${ue?r`<div class="agent-kv-sub muted">${ue}</div>`:m}
        </div>
        <div class="agent-kv">
          <div class="label">Default</div>
          <div>${M?"yes":"no"}</div>
        </div>
        <div class="agent-kv">
          <div class="label">Identity Emoji</div>
          <div>${ae}</div>
        </div>
        <div class="agent-kv">
          <div class="label">Skills Filter</div>
          <div>${O?`${K} selected`:"all skills"}</div>
        </div>
      </div>

      <div class="agent-model-select" style="margin-top: 20px;">
        <div class="label">Model Selection</div>
        <div class="row" style="gap: 12px; flex-wrap: wrap;">
          <label class="field" style="min-width: 260px; flex: 1;">
            <span>Primary model${M?" (default)":""}</span>
            <select
              .value=${L??""}
              ?disabled=${!n||l||d}
              @change=${z=>b(t.id,z.target.value||null)}
            >
              ${M?m:r`
                      <option value="">
                        ${_?`Inherit default (${_})`:"Inherit default"}
                      </option>
                    `}
              ${Gm(n,L??void 0)}
            </select>
          </label>
          <label class="field" style="min-width: 260px; flex: 1;">
            <span>Fallbacks (comma-separated)</span>
            <input
              .value=${P}
              ?disabled=${!n||l||d}
              placeholder="provider/model, provider/model"
              @input=${z=>u(t.id,Wm(z.target.value))}
            />
          </label>
        </div>
        <div class="row" style="justify-content: flex-end; gap: 8px;">
          <button class="btn btn--sm" ?disabled=${l} @click=${f}>
            Reload Config
          </button>
          <button
            class="btn btn--sm primary"
            ?disabled=${d||!g}
            @click=${p}
          >
            ${d?"Saving…":"Save"}
          </button>
        </div>
      </div>
    </section>
  `}const vb=new Set(["title","description","default","nullable"]);function mb(e){return Object.keys(e??{}).filter(n=>!vb.has(n)).length===0}function bb(e){if(e===void 0)return"";try{return JSON.stringify(e,null,2)??""}catch{return""}}const pn={chevronDown:r`
    <svg
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      stroke-width="2"
      stroke-linecap="round"
      stroke-linejoin="round"
    >
      <polyline points="6 9 12 15 18 9"></polyline>
    </svg>
  `,plus:r`
    <svg
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      stroke-width="2"
      stroke-linecap="round"
      stroke-linejoin="round"
    >
      <line x1="12" y1="5" x2="12" y2="19"></line>
      <line x1="5" y1="12" x2="19" y2="12"></line>
    </svg>
  `,minus:r`
    <svg
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      stroke-width="2"
      stroke-linecap="round"
      stroke-linejoin="round"
    >
      <line x1="5" y1="12" x2="19" y2="12"></line>
    </svg>
  `,trash:r`
    <svg
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      stroke-width="2"
      stroke-linecap="round"
      stroke-linejoin="round"
    >
      <polyline points="3 6 5 6 21 6"></polyline>
      <path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"></path>
    </svg>
  `,edit:r`
    <svg
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      stroke-width="2"
      stroke-linecap="round"
      stroke-linejoin="round"
    >
      <path d="M11 4H4a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2v-7"></path>
      <path d="M18.5 2.5a2.121 2.121 0 0 1 3 3L12 15l-4 1 1-4 9.5-9.5z"></path>
    </svg>
  `};function qe(e){const{schema:t,value:n,path:s,hints:i,unsupported:a,disabled:o,onPatch:l}=e,d=e.showLabel??!0,g=ke(t),f=Ce(s,i),p=f?.label??t.title??Ge(String(s.at(-1))),b=f?.help??t.description,u=Fi(s);if(a.has(u))return r`<div class="cfg-field cfg-field--error">
      <div class="cfg-field__label">${p}</div>
      <div class="cfg-field__error">Unsupported schema node. Use Raw mode.</div>
    </div>`;if(t.anyOf||t.oneOf){const y=(t.anyOf??t.oneOf??[]).filter(L=>!(L.type==="null"||Array.isArray(L.type)&&L.type.includes("null")));if(y.length===1)return qe({...e,schema:y[0]});const k=L=>{if(L.const!==void 0)return L.const;if(L.enum&&L.enum.length===1)return L.enum[0]},C=y.map(k),$=C.every(L=>L!==void 0);if($&&C.length>0&&C.length<=5){const L=n??t.default;return r`
        <div class="cfg-field">
          ${d?r`<label class="cfg-field__label">${p}</label>`:m}
          ${b?r`<div class="cfg-field__help">${b}</div>`:m}
          <div class="cfg-segmented">
            ${C.map(E=>r`
              <button
                type="button"
                class="cfg-segmented__btn ${E===L||String(E)===String(L)?"active":""}"
                ?disabled=${o}
                @click=${()=>l(s,E)}
              >
                ${String(E)}
              </button>
            `)}
          </div>
        </div>
      `}if($&&C.length>5)return Fo({...e,options:C,value:n??t.default});const T=new Set(y.map(L=>ke(L)).filter(Boolean)),_=new Set([...T].map(L=>L==="integer"?"number":L));if([..._].every(L=>["string","number","boolean"].includes(L))){const L=_.has("string"),E=_.has("number");if(_.has("boolean")&&_.size===1)return qe({...e,schema:{...t,type:"boolean",anyOf:void 0,oneOf:void 0}});if(L||E)return Do({...e,inputType:E&&!L?"number":"text"})}}if(t.enum){const v=t.enum;if(v.length<=5){const y=n??t.default;return r`
        <div class="cfg-field">
          ${d?r`<label class="cfg-field__label">${p}</label>`:m}
          ${b?r`<div class="cfg-field__help">${b}</div>`:m}
          <div class="cfg-segmented">
            ${v.map(k=>r`
              <button
                type="button"
                class="cfg-segmented__btn ${k===y||String(k)===String(y)?"active":""}"
                ?disabled=${o}
                @click=${()=>l(s,k)}
              >
                ${String(k)}
              </button>
            `)}
          </div>
        </div>
      `}return Fo({...e,options:v,value:n??t.default})}if(g==="object")return xb(e);if(g==="array")return $b(e);if(g==="boolean"){const v=typeof n=="boolean"?n:typeof t.default=="boolean"?t.default:!1;return r`
      <label class="cfg-toggle-row ${o?"disabled":""}">
        <div class="cfg-toggle-row__content">
          <span class="cfg-toggle-row__label">${p}</span>
          ${b?r`<span class="cfg-toggle-row__help">${b}</span>`:m}
        </div>
        <div class="cfg-toggle">
          <input
            type="checkbox"
            .checked=${v}
            ?disabled=${o}
            @change=${y=>l(s,y.target.checked)}
          />
          <span class="cfg-toggle__track"></span>
        </div>
      </label>
    `}return g==="number"||g==="integer"?yb(e):g==="string"?Do({...e,inputType:"text"}):r`
    <div class="cfg-field cfg-field--error">
      <div class="cfg-field__label">${p}</div>
      <div class="cfg-field__error">Unsupported type: ${g}. Use Raw mode.</div>
    </div>
  `}function Do(e){const{schema:t,value:n,path:s,hints:i,disabled:a,onPatch:o,inputType:l}=e,d=e.showLabel??!0,g=Ce(s,i),f=g?.label??t.title??Ge(String(s.at(-1))),p=g?.help??t.description,b=(g?.sensitive??!1)&&!/^\$\{[^}]*\}$/.test(String(n??"").trim()),u=g?.placeholder??(b?"••••":t.default!==void 0?`Default: ${String(t.default)}`:""),v=n??"";return r`
    <div class="cfg-field">
      ${d?r`<label class="cfg-field__label">${f}</label>`:m}
      ${p?r`<div class="cfg-field__help">${p}</div>`:m}
      <div class="cfg-input-wrap">
        <input
          type=${b?"password":l}
          class="cfg-input"
          placeholder=${u}
          .value=${v==null?"":String(v)}
          ?disabled=${a}
          @input=${y=>{const k=y.target.value;if(l==="number"){if(k.trim()===""){o(s,void 0);return}const C=Number(k);o(s,Number.isNaN(C)?k:C);return}o(s,k)}}
          @change=${y=>{if(l==="number")return;const k=y.target.value;o(s,k.trim())}}
        />
        ${t.default!==void 0?r`
          <button
            type="button"
            class="cfg-input__reset"
            title="Reset to default"
            ?disabled=${a}
            @click=${()=>o(s,t.default)}
          >↺</button>
        `:m}
      </div>
    </div>
  `}function yb(e){const{schema:t,value:n,path:s,hints:i,disabled:a,onPatch:o}=e,l=e.showLabel??!0,d=Ce(s,i),g=d?.label??t.title??Ge(String(s.at(-1))),f=d?.help??t.description,p=n??t.default??"",b=typeof p=="number"?p:0;return r`
    <div class="cfg-field">
      ${l?r`<label class="cfg-field__label">${g}</label>`:m}
      ${f?r`<div class="cfg-field__help">${f}</div>`:m}
      <div class="cfg-number">
        <button
          type="button"
          class="cfg-number__btn"
          ?disabled=${a}
          @click=${()=>o(s,b-1)}
        >−</button>
        <input
          type="number"
          class="cfg-number__input"
          .value=${p==null?"":String(p)}
          ?disabled=${a}
          @input=${u=>{const v=u.target.value,y=v===""?void 0:Number(v);o(s,y)}}
        />
        <button
          type="button"
          class="cfg-number__btn"
          ?disabled=${a}
          @click=${()=>o(s,b+1)}
        >+</button>
      </div>
    </div>
  `}function Fo(e){const{schema:t,value:n,path:s,hints:i,disabled:a,options:o,onPatch:l}=e,d=e.showLabel??!0,g=Ce(s,i),f=g?.label??t.title??Ge(String(s.at(-1))),p=g?.help??t.description,b=n??t.default,u=o.findIndex(y=>y===b||String(y)===String(b)),v="__unset__";return r`
    <div class="cfg-field">
      ${d?r`<label class="cfg-field__label">${f}</label>`:m}
      ${p?r`<div class="cfg-field__help">${p}</div>`:m}
      <select
        class="cfg-select"
        ?disabled=${a}
        .value=${u>=0?String(u):v}
        @change=${y=>{const k=y.target.value;l(s,k===v?void 0:o[Number(k)])}}
      >
        <option value=${v}>Select...</option>
        ${o.map((y,k)=>r`
          <option value=${String(k)}>${String(y)}</option>
        `)}
      </select>
    </div>
  `}function xb(e){const{schema:t,value:n,path:s,hints:i,unsupported:a,disabled:o,onPatch:l}=e,d=Ce(s,i),g=d?.label??t.title??Ge(String(s.at(-1))),f=d?.help??t.description,p=n??t.default,b=p&&typeof p=="object"&&!Array.isArray(p)?p:{},u=t.properties??{},y=Object.entries(u).toSorted((T,_)=>{const L=Ce([...s,T[0]],i)?.order??0,E=Ce([...s,_[0]],i)?.order??0;return L!==E?L-E:T[0].localeCompare(_[0])}),k=new Set(Object.keys(u)),C=t.additionalProperties,$=!!C&&typeof C=="object";return s.length===1?r`
      <div class="cfg-fields">
        ${y.map(([T,_])=>qe({schema:_,value:b[T],path:[...s,T],hints:i,unsupported:a,disabled:o,onPatch:l}))}
        ${$?No({schema:C,value:b,path:s,hints:i,unsupported:a,disabled:o,reservedKeys:k,onPatch:l}):m}
      </div>
    `:r`
    <details class="cfg-object" open>
      <summary class="cfg-object__header">
        <span class="cfg-object__title">${g}</span>
        <span class="cfg-object__chevron">${pn.chevronDown}</span>
      </summary>
      ${f?r`<div class="cfg-object__help">${f}</div>`:m}
      <div class="cfg-object__content">
        ${y.map(([T,_])=>qe({schema:_,value:b[T],path:[...s,T],hints:i,unsupported:a,disabled:o,onPatch:l}))}
        ${$?No({schema:C,value:b,path:s,hints:i,unsupported:a,disabled:o,reservedKeys:k,onPatch:l}):m}
      </div>
    </details>
  `}function $b(e){const{schema:t,value:n,path:s,hints:i,unsupported:a,disabled:o,onPatch:l}=e,d=e.showLabel??!0,g=Ce(s,i),f=g?.label??t.title??Ge(String(s.at(-1))),p=g?.help??t.description,b=Array.isArray(t.items)?t.items[0]:t.items;if(!b)return r`
      <div class="cfg-field cfg-field--error">
        <div class="cfg-field__label">${f}</div>
        <div class="cfg-field__error">Unsupported array schema. Use Raw mode.</div>
      </div>
    `;const u=Array.isArray(n)?n:Array.isArray(t.default)?t.default:[];return r`
    <div class="cfg-array">
      <div class="cfg-array__header">
        ${d?r`<span class="cfg-array__label">${f}</span>`:m}
        <span class="cfg-array__count">${u.length} item${u.length!==1?"s":""}</span>
        <button
          type="button"
          class="cfg-array__add"
          ?disabled=${o}
          @click=${()=>{const v=[...u,Rr(b)];l(s,v)}}
        >
          <span class="cfg-array__add-icon">${pn.plus}</span>
          Add
        </button>
      </div>
      ${p?r`<div class="cfg-array__help">${p}</div>`:m}

      ${u.length===0?r`
              <div class="cfg-array__empty">No items yet. Click "Add" to create one.</div>
            `:r`
        <div class="cfg-array__items">
          ${u.map((v,y)=>r`
            <div class="cfg-array__item">
              <div class="cfg-array__item-header">
                <span class="cfg-array__item-index">#${y+1}</span>
                <button
                  type="button"
                  class="cfg-array__item-remove"
                  title="Remove item"
                  ?disabled=${o}
                  @click=${()=>{const k=[...u];k.splice(y,1),l(s,k)}}
                >
                  ${pn.trash}
                </button>
              </div>
              <div class="cfg-array__item-content">
                ${qe({schema:b,value:v,path:[...s,y],hints:i,unsupported:a,disabled:o,showLabel:!1,onPatch:l})}
              </div>
            </div>
          `)}
        </div>
      `}
    </div>
  `}function No(e){const{schema:t,value:n,path:s,hints:i,unsupported:a,disabled:o,reservedKeys:l,onPatch:d}=e,g=mb(t),f=Object.entries(n??{}).filter(([p])=>!l.has(p));return r`
    <div class="cfg-map">
      <div class="cfg-map__header">
        <span class="cfg-map__label">Custom entries</span>
        <button
          type="button"
          class="cfg-map__add"
          ?disabled=${o}
          @click=${()=>{const p={...n};let b=1,u=`custom-${b}`;for(;u in p;)b+=1,u=`custom-${b}`;p[u]=g?{}:Rr(t),d(s,p)}}
        >
          <span class="cfg-map__add-icon">${pn.plus}</span>
          Add Entry
        </button>
      </div>

      ${f.length===0?r`
              <div class="cfg-map__empty">No custom entries.</div>
            `:r`
        <div class="cfg-map__items">
          ${f.map(([p,b])=>{const u=[...s,p],v=bb(b);return r`
              <div class="cfg-map__item">
                <div class="cfg-map__item-key">
                  <input
                    type="text"
                    class="cfg-input cfg-input--sm"
                    placeholder="Key"
                    .value=${p}
                    ?disabled=${o}
                    @change=${y=>{const k=y.target.value.trim();if(!k||k===p)return;const C={...n};k in C||(C[k]=C[p],delete C[p],d(s,C))}}
                  />
                </div>
                <div class="cfg-map__item-value">
                  ${g?r`
                        <textarea
                          class="cfg-textarea cfg-textarea--sm"
                          placeholder="JSON value"
                          rows="2"
                          .value=${v}
                          ?disabled=${o}
                          @change=${y=>{const k=y.target,C=k.value.trim();if(!C){d(u,void 0);return}try{d(u,JSON.parse(C))}catch{k.value=v}}}
                        ></textarea>
                      `:qe({schema:t,value:b,path:u,hints:i,unsupported:a,disabled:o,showLabel:!1,onPatch:d})}
                </div>
                <button
                  type="button"
                  class="cfg-map__item-remove"
                  title="Remove entry"
                  ?disabled=${o}
                  @click=${()=>{const y={...n};delete y[p],d(s,y)}}
                >
                  ${pn.trash}
                </button>
              </div>
            `})}
        </div>
      `}
    </div>
  `}const Oo={env:r`
    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
      <circle cx="12" cy="12" r="3"></circle>
      <path
        d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1 0 2.83 2 2 0 0 1-2.83 0l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-2 2 2 2 0 0 1-2-2v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83 0 2 2 0 0 1 0-2.83l.06-.06a1.65 1.65 0 0 0 .33-1.82 1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1-2-2 2 2 0 0 1 2-2h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 0-2.83 2 2 0 0 1 2.83 0l.06.06a1.65 1.65 0 0 0 1.82.33H9a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 2-2 2 2 0 0 1 2 2v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 0 2 2 0 0 1 0 2.83l-.06.06a1.65 1.65 0 0 0-.33 1.82V9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 2 2 2 2 0 0 1-2 2h-.09a1.65 1.65 0 0 0-1.51 1z"
      ></path>
    </svg>
  `,update:r`
    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
      <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"></path>
      <polyline points="7 10 12 15 17 10"></polyline>
      <line x1="12" y1="15" x2="12" y2="3"></line>
    </svg>
  `,agents:r`
    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
      <path
        d="M12 2a2 2 0 0 1 2 2c0 .74-.4 1.39-1 1.73V7h1a7 7 0 0 1 7 7h1a1 1 0 0 1 1 1v3a1 1 0 0 1-1 1h-1v1a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-1H2a1 1 0 0 1-1-1v-3a1 1 0 0 1 1-1h1a7 7 0 0 1 7-7h1V5.73c-.6-.34-1-.99-1-1.73a2 2 0 0 1 2-2z"
      ></path>
      <circle cx="8" cy="14" r="1"></circle>
      <circle cx="16" cy="14" r="1"></circle>
    </svg>
  `,auth:r`
    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
      <rect x="3" y="11" width="18" height="11" rx="2" ry="2"></rect>
      <path d="M7 11V7a5 5 0 0 1 10 0v4"></path>
    </svg>
  `,channels:r`
    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
      <path d="M21 15a2 2 0 0 1-2 2H7l-4 4V5a2 2 0 0 1 2-2h14a2 2 0 0 1 2 2z"></path>
    </svg>
  `,messages:r`
    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
      <path d="M4 4h16c1.1 0 2 .9 2 2v12c0 1.1-.9 2-2 2H4c-1.1 0-2-.9-2-2V6c0-1.1.9-2 2-2z"></path>
      <polyline points="22,6 12,13 2,6"></polyline>
    </svg>
  `,commands:r`
    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
      <polyline points="4 17 10 11 4 5"></polyline>
      <line x1="12" y1="19" x2="20" y2="19"></line>
    </svg>
  `,hooks:r`
    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
      <path d="M10 13a5 5 0 0 0 7.54.54l3-3a5 5 0 0 0-7.07-7.07l-1.72 1.71"></path>
      <path d="M14 11a5 5 0 0 0-7.54-.54l-3 3a5 5 0 0 0 7.07 7.07l1.71-1.71"></path>
    </svg>
  `,skills:r`
    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
      <polygon
        points="12 2 15.09 8.26 22 9.27 17 14.14 18.18 21.02 12 17.77 5.82 21.02 7 14.14 2 9.27 8.91 8.26 12 2"
      ></polygon>
    </svg>
  `,tools:r`
    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
      <path
        d="M14.7 6.3a1 1 0 0 0 0 1.4l1.6 1.6a1 1 0 0 0 1.4 0l3.77-3.77a6 6 0 0 1-7.94 7.94l-6.91 6.91a2.12 2.12 0 0 1-3-3l6.91-6.91a6 6 0 0 1 7.94-7.94l-3.76 3.76z"
      ></path>
    </svg>
  `,gateway:r`
    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
      <circle cx="12" cy="12" r="10"></circle>
      <line x1="2" y1="12" x2="22" y2="12"></line>
      <path
        d="M12 2a15.3 15.3 0 0 1 4 10 15.3 15.3 0 0 1-4 10 15.3 15.3 0 0 1-4-10 15.3 15.3 0 0 1 4-10z"
      ></path>
    </svg>
  `,wizard:r`
    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
      <path d="M15 4V2"></path>
      <path d="M15 16v-2"></path>
      <path d="M8 9h2"></path>
      <path d="M20 9h2"></path>
      <path d="M17.8 11.8 19 13"></path>
      <path d="M15 9h0"></path>
      <path d="M17.8 6.2 19 5"></path>
      <path d="m3 21 9-9"></path>
      <path d="M12.2 6.2 11 5"></path>
    </svg>
  `,meta:r`
    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
      <path d="M12 20h9"></path>
      <path d="M16.5 3.5a2.12 2.12 0 0 1 3 3L7 19l-4 1 1-4Z"></path>
    </svg>
  `,logging:r`
    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
      <path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"></path>
      <polyline points="14 2 14 8 20 8"></polyline>
      <line x1="16" y1="13" x2="8" y2="13"></line>
      <line x1="16" y1="17" x2="8" y2="17"></line>
      <polyline points="10 9 9 9 8 9"></polyline>
    </svg>
  `,browser:r`
    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
      <circle cx="12" cy="12" r="10"></circle>
      <circle cx="12" cy="12" r="4"></circle>
      <line x1="21.17" y1="8" x2="12" y2="8"></line>
      <line x1="3.95" y1="6.06" x2="8.54" y2="14"></line>
      <line x1="10.88" y1="21.94" x2="15.46" y2="14"></line>
    </svg>
  `,ui:r`
    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
      <rect x="3" y="3" width="18" height="18" rx="2" ry="2"></rect>
      <line x1="3" y1="9" x2="21" y2="9"></line>
      <line x1="9" y1="21" x2="9" y2="9"></line>
    </svg>
  `,models:r`
    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
      <path
        d="M21 16V8a2 2 0 0 0-1-1.73l-7-4a2 2 0 0 0-2 0l-7 4A2 2 0 0 0 3 8v8a2 2 0 0 0 1 1.73l7 4a2 2 0 0 0 2 0l7-4A2 2 0 0 0 21 16z"
      ></path>
      <polyline points="3.27 6.96 12 12.01 20.73 6.96"></polyline>
      <line x1="12" y1="22.08" x2="12" y2="12"></line>
    </svg>
  `,bindings:r`
    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
      <rect x="2" y="2" width="20" height="8" rx="2" ry="2"></rect>
      <rect x="2" y="14" width="20" height="8" rx="2" ry="2"></rect>
      <line x1="6" y1="6" x2="6.01" y2="6"></line>
      <line x1="6" y1="18" x2="6.01" y2="18"></line>
    </svg>
  `,broadcast:r`
    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
      <path d="M4.9 19.1C1 15.2 1 8.8 4.9 4.9"></path>
      <path d="M7.8 16.2c-2.3-2.3-2.3-6.1 0-8.5"></path>
      <circle cx="12" cy="12" r="2"></circle>
      <path d="M16.2 7.8c2.3 2.3 2.3 6.1 0 8.5"></path>
      <path d="M19.1 4.9C23 8.8 23 15.1 19.1 19"></path>
    </svg>
  `,audio:r`
    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
      <path d="M9 18V5l12-2v13"></path>
      <circle cx="6" cy="18" r="3"></circle>
      <circle cx="18" cy="16" r="3"></circle>
    </svg>
  `,session:r`
    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
      <path d="M17 21v-2a4 4 0 0 0-4-4H5a4 4 0 0 0-4 4v2"></path>
      <circle cx="9" cy="7" r="4"></circle>
      <path d="M23 21v-2a4 4 0 0 0-3-3.87"></path>
      <path d="M16 3.13a4 4 0 0 1 0 7.75"></path>
    </svg>
  `,cron:r`
    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
      <circle cx="12" cy="12" r="10"></circle>
      <polyline points="12 6 12 12 16 14"></polyline>
    </svg>
  `,web:r`
    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
      <circle cx="12" cy="12" r="10"></circle>
      <line x1="2" y1="12" x2="22" y2="12"></line>
      <path
        d="M12 2a15.3 15.3 0 0 1 4 10 15.3 15.3 0 0 1-4 10 15.3 15.3 0 0 1-4-10 15.3 15.3 0 0 1 4-10z"
      ></path>
    </svg>
  `,discovery:r`
    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
      <circle cx="11" cy="11" r="8"></circle>
      <line x1="21" y1="21" x2="16.65" y2="16.65"></line>
    </svg>
  `,canvasHost:r`
    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
      <rect x="3" y="3" width="18" height="18" rx="2" ry="2"></rect>
      <circle cx="8.5" cy="8.5" r="1.5"></circle>
      <polyline points="21 15 16 10 5 21"></polyline>
    </svg>
  `,talk:r`
    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
      <path d="M12 1a3 3 0 0 0-3 3v8a3 3 0 0 0 6 0V4a3 3 0 0 0-3-3z"></path>
      <path d="M19 10v2a7 7 0 0 1-14 0v-2"></path>
      <line x1="12" y1="19" x2="12" y2="23"></line>
      <line x1="8" y1="23" x2="16" y2="23"></line>
    </svg>
  `,plugins:r`
    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
      <path d="M12 2v6"></path>
      <path d="m4.93 10.93 4.24 4.24"></path>
      <path d="M2 12h6"></path>
      <path d="m4.93 13.07 4.24-4.24"></path>
      <path d="M12 22v-6"></path>
      <path d="m19.07 13.07-4.24-4.24"></path>
      <path d="M22 12h-6"></path>
      <path d="m19.07 10.93-4.24 4.24"></path>
    </svg>
  `,default:r`
    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
      <path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"></path>
      <polyline points="14 2 14 8 20 8"></polyline>
    </svg>
  `},ca={env:{label:"Environment Variables",description:"Environment variables passed to the gateway process"},update:{label:"Updates",description:"Auto-update settings and release channel"},agents:{label:"Agents",description:"Agent configurations, models, and identities"},auth:{label:"Authentication",description:"API keys and authentication profiles"},channels:{label:"Channels",description:"Messaging channels (Telegram, Discord, Slack, etc.)"},messages:{label:"Messages",description:"Message handling and routing settings"},commands:{label:"Commands",description:"Custom slash commands"},hooks:{label:"Hooks",description:"Webhooks and event hooks"},skills:{label:"Skills",description:"Skill packs and capabilities"},tools:{label:"Tools",description:"Tool configurations (browser, search, etc.)"},gateway:{label:"Gateway",description:"Gateway server settings (port, auth, binding)"},wizard:{label:"Setup Wizard",description:"Setup wizard state and history"},meta:{label:"Metadata",description:"Gateway metadata and version information"},logging:{label:"Logging",description:"Log levels and output configuration"},browser:{label:"Browser",description:"Browser automation settings"},ui:{label:"UI",description:"User interface preferences"},models:{label:"Models",description:"AI model configurations and providers"},bindings:{label:"Bindings",description:"Key bindings and shortcuts"},broadcast:{label:"Broadcast",description:"Broadcast and notification settings"},audio:{label:"Audio",description:"Audio input/output settings"},session:{label:"Session",description:"Session management and persistence"},cron:{label:"Cron",description:"Scheduled tasks and automation"},web:{label:"Web",description:"Web server and API settings"},discovery:{label:"Discovery",description:"Service discovery and networking"},canvasHost:{label:"Canvas Host",description:"Canvas rendering and display"},talk:{label:"Talk",description:"Voice and speech settings"},plugins:{label:"Plugins",description:"Plugin management and extensions"}};function Bo(e){return Oo[e]??Oo.default}function wb(e,t,n){if(!n)return!0;const s=n.toLowerCase(),i=ca[e];return e.toLowerCase().includes(s)||i&&(i.label.toLowerCase().includes(s)||i.description.toLowerCase().includes(s))?!0:Yt(t,s)}function Yt(e,t){if(e.title?.toLowerCase().includes(t)||e.description?.toLowerCase().includes(t)||e.enum?.some(s=>String(s).toLowerCase().includes(t)))return!0;if(e.properties){for(const[s,i]of Object.entries(e.properties))if(s.toLowerCase().includes(t)||Yt(i,t))return!0}if(e.items){const s=Array.isArray(e.items)?e.items:[e.items];for(const i of s)if(i&&Yt(i,t))return!0}if(e.additionalProperties&&typeof e.additionalProperties=="object"&&Yt(e.additionalProperties,t))return!0;const n=e.anyOf??e.oneOf??e.allOf;if(n){for(const s of n)if(s&&Yt(s,t))return!0}return!1}function kb(e){if(!e.schema)return r`
      <div class="muted">Schema unavailable.</div>
    `;const t=e.schema,n=e.value??{};if(ke(t)!=="object"||!t.properties)return r`
      <div class="callout danger">Unsupported schema. Use Raw.</div>
    `;const s=new Set(e.unsupportedPaths??[]),i=t.properties,a=e.searchQuery??"",o=e.activeSection,l=e.activeSubsection??null,g=Object.entries(i).toSorted((p,b)=>{const u=Ce([p[0]],e.uiHints)?.order??50,v=Ce([b[0]],e.uiHints)?.order??50;return u!==v?u-v:p[0].localeCompare(b[0])}).filter(([p,b])=>!(o&&p!==o||a&&!wb(p,b,a)));let f=null;if(o&&l&&g.length===1){const p=g[0]?.[1];p&&ke(p)==="object"&&p.properties&&p.properties[l]&&(f={sectionKey:o,subsectionKey:l,schema:p.properties[l]})}return g.length===0?r`
      <div class="config-empty">
        <div class="config-empty__icon">${de.search}</div>
        <div class="config-empty__text">
          ${a?`No settings match "${a}"`:"No settings in this section"}
        </div>
      </div>
    `:r`
    <div class="config-form config-form--modern">
      ${f?(()=>{const{sectionKey:p,subsectionKey:b,schema:u}=f,v=Ce([p,b],e.uiHints),y=v?.label??u.title??Ge(b),k=v?.help??u.description??"",C=n[p],$=C&&typeof C=="object"?C[b]:void 0,T=`config-section-${p}-${b}`;return r`
              <section class="config-section-card" id=${T}>
                <div class="config-section-card__header">
                  <span class="config-section-card__icon">${Bo(p)}</span>
                  <div class="config-section-card__titles">
                    <h3 class="config-section-card__title">${y}</h3>
                    ${k?r`<p class="config-section-card__desc">${k}</p>`:m}
                  </div>
                </div>
                <div class="config-section-card__content">
                  ${qe({schema:u,value:$,path:[p,b],hints:e.uiHints,unsupported:s,disabled:e.disabled??!1,showLabel:!1,onPatch:e.onPatch})}
                </div>
              </section>
            `})():g.map(([p,b])=>{const u=ca[p]??{label:p.charAt(0).toUpperCase()+p.slice(1),description:b.description??""};return r`
              <section class="config-section-card" id="config-section-${p}">
                <div class="config-section-card__header">
                  <span class="config-section-card__icon">${Bo(p)}</span>
                  <div class="config-section-card__titles">
                    <h3 class="config-section-card__title">${u.label}</h3>
                    ${u.description?r`<p class="config-section-card__desc">${u.description}</p>`:m}
                  </div>
                </div>
                <div class="config-section-card__content">
                  ${qe({schema:b,value:n[p],path:[p],hints:e.uiHints,unsupported:s,disabled:e.disabled??!1,showLabel:!1,onPatch:e.onPatch})}
                </div>
              </section>
            `})}
    </div>
  `}const Sb=new Set(["title","description","default","nullable"]);function Ab(e){return Object.keys(e??{}).filter(n=>!Sb.has(n)).length===0}function Hl(e){const t=e.filter(i=>i!=null),n=t.length!==e.length,s=[];for(const i of t)s.some(a=>Object.is(a,i))||s.push(i);return{enumValues:s,nullable:n}}function Kl(e){return!e||typeof e!="object"?{schema:null,unsupportedPaths:["<root>"]}:sn(e,[])}function sn(e,t){const n=new Set,s={...e},i=Fi(t)||"<root>";if(e.anyOf||e.oneOf||e.allOf){const l=_b(e,t);return l||{schema:e,unsupportedPaths:[i]}}const a=Array.isArray(e.type)&&e.type.includes("null"),o=ke(e)??(e.properties||e.additionalProperties?"object":void 0);if(s.type=o??e.type,s.nullable=a||e.nullable,s.enum){const{enumValues:l,nullable:d}=Hl(s.enum);s.enum=l,d&&(s.nullable=!0),l.length===0&&n.add(i)}if(o==="object"){const l=e.properties??{},d={};for(const[g,f]of Object.entries(l)){const p=sn(f,[...t,g]);p.schema&&(d[g]=p.schema);for(const b of p.unsupportedPaths)n.add(b)}if(s.properties=d,e.additionalProperties===!0)n.add(i);else if(e.additionalProperties===!1)s.additionalProperties=!1;else if(e.additionalProperties&&typeof e.additionalProperties=="object"&&!Ab(e.additionalProperties)){const g=sn(e.additionalProperties,[...t,"*"]);s.additionalProperties=g.schema??e.additionalProperties,g.unsupportedPaths.length>0&&n.add(i)}}else if(o==="array"){const l=Array.isArray(e.items)?e.items[0]:e.items;if(!l)n.add(i);else{const d=sn(l,[...t,"*"]);s.items=d.schema??l,d.unsupportedPaths.length>0&&n.add(i)}}else o!=="string"&&o!=="number"&&o!=="integer"&&o!=="boolean"&&!s.enum&&n.add(i);return{schema:s,unsupportedPaths:Array.from(n)}}function _b(e,t){if(e.allOf)return null;const n=e.anyOf??e.oneOf;if(!n)return null;const s=[],i=[];let a=!1;for(const l of n){if(!l||typeof l!="object")return null;if(Array.isArray(l.enum)){const{enumValues:d,nullable:g}=Hl(l.enum);s.push(...d),g&&(a=!0);continue}if("const"in l){if(l.const==null){a=!0;continue}s.push(l.const);continue}if(ke(l)==="null"){a=!0;continue}i.push(l)}if(s.length>0&&i.length===0){const l=[];for(const d of s)l.some(g=>Object.is(g,d))||l.push(d);return{schema:{...e,enum:l,nullable:a,anyOf:void 0,oneOf:void 0,allOf:void 0},unsupportedPaths:[]}}if(i.length===1){const l=sn(i[0],t);return l.schema&&(l.schema.nullable=a||l.schema.nullable),l}const o=new Set(["string","number","integer","boolean"]);return i.length>0&&s.length===0&&i.every(l=>l.type&&o.has(String(l.type)))?{schema:{...e,nullable:a},unsupportedPaths:[]}:null}function Cb(e,t){let n=e;for(const s of t){if(!n)return null;const i=ke(n);if(i==="object"){const a=n.properties??{};if(typeof s=="string"&&a[s]){n=a[s];continue}const o=n.additionalProperties;if(typeof s=="string"&&o&&typeof o=="object"){n=o;continue}return null}if(i==="array"){if(typeof s!="number")return null;n=(Array.isArray(n.items)?n.items[0]:n.items)??null;continue}return null}return n}function Tb(e,t){const s=(e.channels??{})[t],i=e[t];return(s&&typeof s=="object"?s:null)??(i&&typeof i=="object"?i:null)??{}}const Eb=["groupPolicy","streamMode","dmPolicy"];function Lb(e){if(e==null)return"n/a";if(typeof e=="string"||typeof e=="number"||typeof e=="boolean")return String(e);try{return JSON.stringify(e)}catch{return"n/a"}}function Mb(e){const t=Eb.flatMap(n=>n in e?[[n,e[n]]]:[]);return t.length===0?null:r`
    <div class="status-list" style="margin-top: 12px;">
      ${t.map(([n,s])=>r`
          <div>
            <span class="label">${n}</span>
            <span>${Lb(s)}</span>
          </div>
        `)}
    </div>
  `}function Ib(e){const t=Kl(e.schema),n=t.schema;if(!n)return r`
      <div class="callout danger">Schema unavailable. Use Raw.</div>
    `;const s=Cb(n,["channels",e.channelId]);if(!s)return r`
      <div class="callout danger">Channel config schema unavailable.</div>
    `;const i=e.configValue??{},a=Tb(i,e.channelId);return r`
    <div class="config-form">
      ${qe({schema:s,value:a,path:["channels",e.channelId],hints:e.uiHints,unsupported:new Set(t.unsupportedPaths),disabled:e.disabled,showLabel:!1,onPatch:e.onPatch})}
    </div>
    ${Mb(a)}
  `}function Ve(e){const{channelId:t,props:n}=e,s=n.configSaving||n.configSchemaLoading;return r`
    <div style="margin-top: 16px;">
      ${n.configSchemaLoading?r`
              <div class="muted">Loading config schema…</div>
            `:Ib({channelId:t,configValue:n.configForm,schema:n.configSchema,uiHints:n.configUiHints,disabled:s,onPatch:n.onConfigPatch})}
      <div class="row" style="margin-top: 12px;">
        <button
          class="btn primary"
          ?disabled=${s||!n.configFormDirty}
          @click=${()=>n.onConfigSave()}
        >
          ${n.configSaving?"Saving…":"Save"}
        </button>
        <button
          class="btn"
          ?disabled=${s}
          @click=${()=>n.onConfigReload()}
        >
          Reload
        </button>
      </div>
    </div>
  `}function Rb(e){const{props:t,discord:n,accountCountLabel:s}=e;return r`
    <div class="card">
      <div class="card-title">Discord</div>
      <div class="card-sub">Bot status and channel configuration.</div>
      ${s}

      <div class="status-list" style="margin-top: 16px;">
        <div>
          <span class="label">Configured</span>
          <span>${n?.configured?"Yes":"No"}</span>
        </div>
        <div>
          <span class="label">Running</span>
          <span>${n?.running?"Yes":"No"}</span>
        </div>
        <div>
          <span class="label">Last start</span>
          <span>${n?.lastStartAt?Y(n.lastStartAt):"n/a"}</span>
        </div>
        <div>
          <span class="label">Last probe</span>
          <span>${n?.lastProbeAt?Y(n.lastProbeAt):"n/a"}</span>
        </div>
      </div>

      ${n?.lastError?r`<div class="callout danger" style="margin-top: 12px;">
            ${n.lastError}
          </div>`:m}

      ${n?.probe?r`<div class="callout" style="margin-top: 12px;">
            Probe ${n.probe.ok?"ok":"failed"} ·
            ${n.probe.status??""} ${n.probe.error??""}
          </div>`:m}

      ${Ve({channelId:"discord",props:t})}

      <div class="row" style="margin-top: 12px;">
        <button class="btn" @click=${()=>t.onRefresh(!0)}>
          Probe
        </button>
      </div>
    </div>
  `}function Pb(e){const{props:t,googleChat:n,accountCountLabel:s}=e;return r`
    <div class="card">
      <div class="card-title">Google Chat</div>
      <div class="card-sub">Chat API webhook status and channel configuration.</div>
      ${s}

      <div class="status-list" style="margin-top: 16px;">
        <div>
          <span class="label">Configured</span>
          <span>${n?n.configured?"Yes":"No":"n/a"}</span>
        </div>
        <div>
          <span class="label">Running</span>
          <span>${n?n.running?"Yes":"No":"n/a"}</span>
        </div>
        <div>
          <span class="label">Credential</span>
          <span>${n?.credentialSource??"n/a"}</span>
        </div>
        <div>
          <span class="label">Audience</span>
          <span>
            ${n?.audienceType?`${n.audienceType}${n.audience?` · ${n.audience}`:""}`:"n/a"}
          </span>
        </div>
        <div>
          <span class="label">Last start</span>
          <span>${n?.lastStartAt?Y(n.lastStartAt):"n/a"}</span>
        </div>
        <div>
          <span class="label">Last probe</span>
          <span>${n?.lastProbeAt?Y(n.lastProbeAt):"n/a"}</span>
        </div>
      </div>

      ${n?.lastError?r`<div class="callout danger" style="margin-top: 12px;">
            ${n.lastError}
          </div>`:m}

      ${n?.probe?r`<div class="callout" style="margin-top: 12px;">
            Probe ${n.probe.ok?"ok":"failed"} ·
            ${n.probe.status??""} ${n.probe.error??""}
          </div>`:m}

      ${Ve({channelId:"googlechat",props:t})}

      <div class="row" style="margin-top: 12px;">
        <button class="btn" @click=${()=>t.onRefresh(!0)}>
          Probe
        </button>
      </div>
    </div>
  `}function Db(e){const{props:t,imessage:n,accountCountLabel:s}=e;return r`
    <div class="card">
      <div class="card-title">iMessage</div>
      <div class="card-sub">macOS bridge status and channel configuration.</div>
      ${s}

      <div class="status-list" style="margin-top: 16px;">
        <div>
          <span class="label">Configured</span>
          <span>${n?.configured?"Yes":"No"}</span>
        </div>
        <div>
          <span class="label">Running</span>
          <span>${n?.running?"Yes":"No"}</span>
        </div>
        <div>
          <span class="label">Last start</span>
          <span>${n?.lastStartAt?Y(n.lastStartAt):"n/a"}</span>
        </div>
        <div>
          <span class="label">Last probe</span>
          <span>${n?.lastProbeAt?Y(n.lastProbeAt):"n/a"}</span>
        </div>
      </div>

      ${n?.lastError?r`<div class="callout danger" style="margin-top: 12px;">
            ${n.lastError}
          </div>`:m}

      ${n?.probe?r`<div class="callout" style="margin-top: 12px;">
            Probe ${n.probe.ok?"ok":"failed"} ·
            ${n.probe.error??""}
          </div>`:m}

      ${Ve({channelId:"imessage",props:t})}

      <div class="row" style="margin-top: 12px;">
        <button class="btn" @click=${()=>t.onRefresh(!0)}>
          Probe
        </button>
      </div>
    </div>
  `}function Uo(e){return e?e.length<=20?e:`${e.slice(0,8)}...${e.slice(-8)}`:"n/a"}function Fb(e){const{props:t,nostr:n,nostrAccounts:s,accountCountLabel:i,profileFormState:a,profileFormCallbacks:o,onEditProfile:l}=e,d=s[0],g=n?.configured??d?.configured??!1,f=n?.running??d?.running??!1,p=n?.publicKey??d?.publicKey,b=n?.lastStartAt??d?.lastStartAt??null,u=n?.lastError??d?.lastError??null,v=s.length>1,y=a!=null,k=$=>{const T=$.publicKey,_=$.profile,L=_?.displayName??_?.name??$.name??$.accountId;return r`
      <div class="account-card">
        <div class="account-card-header">
          <div class="account-card-title">${L}</div>
          <div class="account-card-id">${$.accountId}</div>
        </div>
        <div class="status-list account-card-status">
          <div>
            <span class="label">Running</span>
            <span>${$.running?"Yes":"No"}</span>
          </div>
          <div>
            <span class="label">Configured</span>
            <span>${$.configured?"Yes":"No"}</span>
          </div>
          <div>
            <span class="label">Public Key</span>
            <span class="monospace" title="${T??""}">${Uo(T)}</span>
          </div>
          <div>
            <span class="label">Last inbound</span>
            <span>${$.lastInboundAt?Y($.lastInboundAt):"n/a"}</span>
          </div>
          ${$.lastError?r`
                <div class="account-card-error">${$.lastError}</div>
              `:m}
        </div>
      </div>
    `},C=()=>{if(y&&o)return Pp({state:a,callbacks:o,accountId:s[0]?.accountId??"default"});const $=d?.profile??n?.profile,{name:T,displayName:_,about:L,picture:E,nip05:P}=$??{},j=T||_||L||E||P;return r`
      <div style="margin-top: 16px; padding: 12px; background: var(--bg-secondary); border-radius: 8px;">
        <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 8px;">
          <div style="font-weight: 500;">Profile</div>
          ${g?r`
                <button
                  class="btn btn-sm"
                  @click=${l}
                  style="font-size: 12px; padding: 4px 8px;"
                >
                  Edit Profile
                </button>
              `:m}
        </div>
        ${j?r`
              <div class="status-list">
                ${E?r`
                      <div style="margin-bottom: 8px;">
                        <img
                          src=${E}
                          alt="Profile picture"
                          style="width: 48px; height: 48px; border-radius: 50%; object-fit: cover; border: 2px solid var(--border-color);"
                          @error=${Z=>{Z.target.style.display="none"}}
                        />
                      </div>
                    `:m}
                ${T?r`<div><span class="label">Name</span><span>${T}</span></div>`:m}
                ${_?r`<div><span class="label">Display Name</span><span>${_}</span></div>`:m}
                ${L?r`<div><span class="label">About</span><span style="max-width: 300px; overflow: hidden; text-overflow: ellipsis;">${L}</span></div>`:m}
                ${P?r`<div><span class="label">NIP-05</span><span>${P}</span></div>`:m}
              </div>
            `:r`
                <div style="color: var(--text-muted); font-size: 13px">
                  No profile set. Click "Edit Profile" to add your name, bio, and avatar.
                </div>
              `}
      </div>
    `};return r`
    <div class="card">
      <div class="card-title">Nostr</div>
      <div class="card-sub">Decentralized DMs via Nostr relays (NIP-04).</div>
      ${i}

      ${v?r`
            <div class="account-card-list">
              ${s.map($=>k($))}
            </div>
          `:r`
            <div class="status-list" style="margin-top: 16px;">
              <div>
                <span class="label">Configured</span>
                <span>${g?"Yes":"No"}</span>
              </div>
              <div>
                <span class="label">Running</span>
                <span>${f?"Yes":"No"}</span>
              </div>
              <div>
                <span class="label">Public Key</span>
                <span class="monospace" title="${p??""}"
                  >${Uo(p)}</span
                >
              </div>
              <div>
                <span class="label">Last start</span>
                <span>${b?Y(b):"n/a"}</span>
              </div>
            </div>
          `}

      ${u?r`<div class="callout danger" style="margin-top: 12px;">${u}</div>`:m}

      ${C()}

      ${Ve({channelId:"nostr",props:t})}

      <div class="row" style="margin-top: 12px;">
        <button class="btn" @click=${()=>t.onRefresh(!1)}>Refresh</button>
      </div>
    </div>
  `}function Nb(e,t){const n=t.snapshot,s=n?.channels;if(!n||!s)return!1;const i=s[e],a=typeof i?.configured=="boolean"&&i.configured,o=typeof i?.running=="boolean"&&i.running,l=typeof i?.connected=="boolean"&&i.connected,g=(n.channelAccounts?.[e]??[]).some(f=>f.configured||f.running||f.connected);return a||o||l||g}function Ob(e,t){return t?.[e]?.length??0}function jl(e,t){const n=Ob(e,t);return n<2?m:r`<div class="account-count">Accounts (${n})</div>`}function Bb(e){const{props:t,signal:n,accountCountLabel:s}=e;return r`
    <div class="card">
      <div class="card-title">Signal</div>
      <div class="card-sub">signal-cli status and channel configuration.</div>
      ${s}

      <div class="status-list" style="margin-top: 16px;">
        <div>
          <span class="label">Configured</span>
          <span>${n?.configured?"Yes":"No"}</span>
        </div>
        <div>
          <span class="label">Running</span>
          <span>${n?.running?"Yes":"No"}</span>
        </div>
        <div>
          <span class="label">Base URL</span>
          <span>${n?.baseUrl??"n/a"}</span>
        </div>
        <div>
          <span class="label">Last start</span>
          <span>${n?.lastStartAt?Y(n.lastStartAt):"n/a"}</span>
        </div>
        <div>
          <span class="label">Last probe</span>
          <span>${n?.lastProbeAt?Y(n.lastProbeAt):"n/a"}</span>
        </div>
      </div>

      ${n?.lastError?r`<div class="callout danger" style="margin-top: 12px;">
            ${n.lastError}
          </div>`:m}

      ${n?.probe?r`<div class="callout" style="margin-top: 12px;">
            Probe ${n.probe.ok?"ok":"failed"} ·
            ${n.probe.status??""} ${n.probe.error??""}
          </div>`:m}

      ${Ve({channelId:"signal",props:t})}

      <div class="row" style="margin-top: 12px;">
        <button class="btn" @click=${()=>t.onRefresh(!0)}>
          Probe
        </button>
      </div>
    </div>
  `}function Ub(e){const{props:t,slack:n,accountCountLabel:s}=e;return r`
    <div class="card">
      <div class="card-title">Slack</div>
      <div class="card-sub">Socket mode status and channel configuration.</div>
      ${s}

      <div class="status-list" style="margin-top: 16px;">
        <div>
          <span class="label">Configured</span>
          <span>${n?.configured?"Yes":"No"}</span>
        </div>
        <div>
          <span class="label">Running</span>
          <span>${n?.running?"Yes":"No"}</span>
        </div>
        <div>
          <span class="label">Last start</span>
          <span>${n?.lastStartAt?Y(n.lastStartAt):"n/a"}</span>
        </div>
        <div>
          <span class="label">Last probe</span>
          <span>${n?.lastProbeAt?Y(n.lastProbeAt):"n/a"}</span>
        </div>
      </div>

      ${n?.lastError?r`<div class="callout danger" style="margin-top: 12px;">
            ${n.lastError}
          </div>`:m}

      ${n?.probe?r`<div class="callout" style="margin-top: 12px;">
            Probe ${n.probe.ok?"ok":"failed"} ·
            ${n.probe.status??""} ${n.probe.error??""}
          </div>`:m}

      ${Ve({channelId:"slack",props:t})}

      <div class="row" style="margin-top: 12px;">
        <button class="btn" @click=${()=>t.onRefresh(!0)}>
          Probe
        </button>
      </div>
    </div>
  `}function zb(e){const{props:t,telegram:n,telegramAccounts:s,accountCountLabel:i}=e,a=s.length>1,o=l=>{const g=l.probe?.bot?.username,f=l.name||l.accountId;return r`
      <div class="account-card">
        <div class="account-card-header">
          <div class="account-card-title">
            ${g?`@${g}`:f}
          </div>
          <div class="account-card-id">${l.accountId}</div>
        </div>
        <div class="status-list account-card-status">
          <div>
            <span class="label">Running</span>
            <span>${l.running?"Yes":"No"}</span>
          </div>
          <div>
            <span class="label">Configured</span>
            <span>${l.configured?"Yes":"No"}</span>
          </div>
          <div>
            <span class="label">Last inbound</span>
            <span>${l.lastInboundAt?Y(l.lastInboundAt):"n/a"}</span>
          </div>
          ${l.lastError?r`
                <div class="account-card-error">
                  ${l.lastError}
                </div>
              `:m}
        </div>
      </div>
    `};return r`
    <div class="card">
      <div class="card-title">Telegram</div>
      <div class="card-sub">Bot status and channel configuration.</div>
      ${i}

      ${a?r`
            <div class="account-card-list">
              ${s.map(l=>o(l))}
            </div>
          `:r`
            <div class="status-list" style="margin-top: 16px;">
              <div>
                <span class="label">Configured</span>
                <span>${n?.configured?"Yes":"No"}</span>
              </div>
              <div>
                <span class="label">Running</span>
                <span>${n?.running?"Yes":"No"}</span>
              </div>
              <div>
                <span class="label">Mode</span>
                <span>${n?.mode??"n/a"}</span>
              </div>
              <div>
                <span class="label">Last start</span>
                <span>${n?.lastStartAt?Y(n.lastStartAt):"n/a"}</span>
              </div>
              <div>
                <span class="label">Last probe</span>
                <span>${n?.lastProbeAt?Y(n.lastProbeAt):"n/a"}</span>
              </div>
            </div>
          `}

      ${n?.lastError?r`<div class="callout danger" style="margin-top: 12px;">
            ${n.lastError}
          </div>`:m}

      ${n?.probe?r`<div class="callout" style="margin-top: 12px;">
            Probe ${n.probe.ok?"ok":"failed"} ·
            ${n.probe.status??""} ${n.probe.error??""}
          </div>`:m}

      ${Ve({channelId:"telegram",props:t})}

      <div class="row" style="margin-top: 12px;">
        <button class="btn" @click=${()=>t.onRefresh(!0)}>
          Probe
        </button>
      </div>
    </div>
  `}function Hb(e){const{props:t,whatsapp:n,accountCountLabel:s}=e;return r`
    <div class="card">
      <div class="card-title">WhatsApp</div>
      <div class="card-sub">Link WhatsApp Web and monitor connection health.</div>
      ${s}

      <div class="status-list" style="margin-top: 16px;">
        <div>
          <span class="label">Configured</span>
          <span>${n?.configured?"Yes":"No"}</span>
        </div>
        <div>
          <span class="label">Linked</span>
          <span>${n?.linked?"Yes":"No"}</span>
        </div>
        <div>
          <span class="label">Running</span>
          <span>${n?.running?"Yes":"No"}</span>
        </div>
        <div>
          <span class="label">Connected</span>
          <span>${n?.connected?"Yes":"No"}</span>
        </div>
        <div>
          <span class="label">Last connect</span>
          <span>
            ${n?.lastConnectedAt?Y(n.lastConnectedAt):"n/a"}
          </span>
        </div>
        <div>
          <span class="label">Last message</span>
          <span>
            ${n?.lastMessageAt?Y(n.lastMessageAt):"n/a"}
          </span>
        </div>
        <div>
          <span class="label">Auth age</span>
          <span>
            ${n?.authAgeMs!=null?ji(n.authAgeMs):"n/a"}
          </span>
        </div>
      </div>

      ${n?.lastError?r`<div class="callout danger" style="margin-top: 12px;">
            ${n.lastError}
          </div>`:m}

      ${t.whatsappMessage?r`<div class="callout" style="margin-top: 12px;">
            ${t.whatsappMessage}
          </div>`:m}

      ${t.whatsappQrDataUrl?r`<div class="qr-wrap">
            <img src=${t.whatsappQrDataUrl} alt="WhatsApp QR" />
          </div>`:m}

      <div class="row" style="margin-top: 14px; flex-wrap: wrap;">
        <button
          class="btn primary"
          ?disabled=${t.whatsappBusy}
          @click=${()=>t.onWhatsAppStart(!1)}
        >
          ${t.whatsappBusy?"Working…":"Show QR"}
        </button>
        <button
          class="btn"
          ?disabled=${t.whatsappBusy}
          @click=${()=>t.onWhatsAppStart(!0)}
        >
          Relink
        </button>
        <button
          class="btn"
          ?disabled=${t.whatsappBusy}
          @click=${()=>t.onWhatsAppWait()}
        >
          Wait for scan
        </button>
        <button
          class="btn danger"
          ?disabled=${t.whatsappBusy}
          @click=${()=>t.onWhatsAppLogout()}
        >
          Logout
        </button>
        <button class="btn" @click=${()=>t.onRefresh(!0)}>
          Refresh
        </button>
      </div>

      ${Ve({channelId:"whatsapp",props:t})}
    </div>
  `}function Kb(e){const t=e.snapshot?.channels,n=t?.whatsapp??void 0,s=t?.telegram??void 0,i=t?.discord??null,a=t?.googlechat??null,o=t?.slack??null,l=t?.signal??null,d=t?.imessage??null,g=t?.nostr??null,p=jb(e.snapshot).map((b,u)=>({key:b,enabled:Nb(b,e),order:u})).toSorted((b,u)=>b.enabled!==u.enabled?b.enabled?-1:1:b.order-u.order);return r`
    <section class="grid grid-cols-2">
      ${p.map(b=>Wb(b.key,e,{whatsapp:n,telegram:s,discord:i,googlechat:a,slack:o,signal:l,imessage:d,nostr:g,channelAccounts:e.snapshot?.channelAccounts??null}))}
    </section>

    <section class="card" style="margin-top: 18px;">
      <div class="row" style="justify-content: space-between;">
        <div>
          <div class="card-title">Channel health</div>
          <div class="card-sub">Channel status snapshots from the gateway.</div>
        </div>
        <div class="muted">${e.lastSuccessAt?Y(e.lastSuccessAt):"n/a"}</div>
      </div>
      ${e.lastError?r`<div class="callout danger" style="margin-top: 12px;">
            ${e.lastError}
          </div>`:m}
      <pre class="code-block" style="margin-top: 12px;">
${e.snapshot?JSON.stringify(e.snapshot,null,2):"No snapshot yet."}
      </pre>
    </section>
  `}function jb(e){return e?.channelMeta?.length?e.channelMeta.map(t=>t.id):e?.channelOrder?.length?e.channelOrder:["whatsapp","telegram","discord","googlechat","slack","signal","imessage","nostr"]}function Wb(e,t,n){const s=jl(e,n.channelAccounts);switch(e){case"whatsapp":return Hb({props:t,whatsapp:n.whatsapp,accountCountLabel:s});case"telegram":return zb({props:t,telegram:n.telegram,telegramAccounts:n.channelAccounts?.telegram??[],accountCountLabel:s});case"discord":return Rb({props:t,discord:n.discord,accountCountLabel:s});case"googlechat":return Pb({props:t,googleChat:n.googlechat,accountCountLabel:s});case"slack":return Ub({props:t,slack:n.slack,accountCountLabel:s});case"signal":return Bb({props:t,signal:n.signal,accountCountLabel:s});case"imessage":return Db({props:t,imessage:n.imessage,accountCountLabel:s});case"nostr":{const i=n.channelAccounts?.nostr??[],a=i[0],o=a?.accountId??"default",l=a?.profile??null,d=t.nostrProfileAccountId===o?t.nostrProfileFormState:null,g=d?{onFieldChange:t.onNostrProfileFieldChange,onSave:t.onNostrProfileSave,onImport:t.onNostrProfileImport,onCancel:t.onNostrProfileCancel,onToggleAdvanced:t.onNostrProfileToggleAdvanced}:null;return Fb({props:t,nostr:n.nostr,nostrAccounts:i,accountCountLabel:s,profileFormState:d,profileFormCallbacks:g,onEditProfile:()=>t.onNostrProfileEdit(o,l)})}default:return qb(e,t,n.channelAccounts??{})}}function qb(e,t,n){const s=Vb(t.snapshot,e),i=t.snapshot?.channels?.[e],a=typeof i?.configured=="boolean"?i.configured:void 0,o=typeof i?.running=="boolean"?i.running:void 0,l=typeof i?.connected=="boolean"?i.connected:void 0,d=typeof i?.lastError=="string"?i.lastError:void 0,g=n[e]??[],f=jl(e,n);return r`
    <div class="card">
      <div class="card-title">${s}</div>
      <div class="card-sub">Channel status and configuration.</div>
      ${f}

      ${g.length>0?r`
            <div class="account-card-list">
              ${g.map(p=>Zb(p))}
            </div>
          `:r`
            <div class="status-list" style="margin-top: 16px;">
              <div>
                <span class="label">Configured</span>
                <span>${a==null?"n/a":a?"Yes":"No"}</span>
              </div>
              <div>
                <span class="label">Running</span>
                <span>${o==null?"n/a":o?"Yes":"No"}</span>
              </div>
              <div>
                <span class="label">Connected</span>
                <span>${l==null?"n/a":l?"Yes":"No"}</span>
              </div>
            </div>
          `}

      ${d?r`<div class="callout danger" style="margin-top: 12px;">
            ${d}
          </div>`:m}

      ${Ve({channelId:e,props:t})}
    </div>
  `}function Gb(e){return e?.channelMeta?.length?Object.fromEntries(e.channelMeta.map(t=>[t.id,t])):{}}function Vb(e,t){return Gb(e)[t]?.label??e?.channelLabels?.[t]??t}const Qb=600*1e3;function Wl(e){return e.lastInboundAt?Date.now()-e.lastInboundAt<Qb:!1}function Yb(e){return e.running?"Yes":Wl(e)?"Active":"No"}function Jb(e){return e.connected===!0?"Yes":e.connected===!1?"No":Wl(e)?"Active":"n/a"}function Zb(e){const t=Yb(e),n=Jb(e);return r`
    <div class="account-card">
      <div class="account-card-header">
        <div class="account-card-title">${e.name||e.accountId}</div>
        <div class="account-card-id">${e.accountId}</div>
      </div>
      <div class="status-list account-card-status">
        <div>
          <span class="label">Running</span>
          <span>${t}</span>
        </div>
        <div>
          <span class="label">Configured</span>
          <span>${e.configured?"Yes":"No"}</span>
        </div>
        <div>
          <span class="label">Connected</span>
          <span>${n}</span>
        </div>
        <div>
          <span class="label">Last inbound</span>
          <span>${e.lastInboundAt?Y(e.lastInboundAt):"n/a"}</span>
        </div>
        ${e.lastError?r`
              <div class="account-card-error">
                ${e.lastError}
              </div>
            `:m}
      </div>
    </div>
  `}const an=(e,t)=>{const n=e._$AN;if(n===void 0)return!1;for(const s of n)s._$AO?.(t,!1),an(s,t);return!0},es=e=>{let t,n;do{if((t=e._$AM)===void 0)break;n=t._$AN,n.delete(e),e=t}while(n?.size===0)},ql=e=>{for(let t;t=e._$AM;e=t){let n=t._$AN;if(n===void 0)t._$AN=n=new Set;else if(n.has(e))break;n.add(e),ty(t)}};function Xb(e){this._$AN!==void 0?(es(this),this._$AM=e,ql(this)):this._$AM=e}function ey(e,t=!1,n=0){const s=this._$AH,i=this._$AN;if(i!==void 0&&i.size!==0)if(t)if(Array.isArray(s))for(let a=n;a<s.length;a++)an(s[a],!1),es(s[a]);else s!=null&&(an(s,!1),es(s));else an(this,e)}const ty=e=>{e.type==aa.CHILD&&(e._$AP??=ey,e._$AQ??=Xb)};class ny extends ra{constructor(){super(...arguments),this._$AN=void 0}_$AT(t,n,s){super._$AT(t,n,s),ql(this),this.isConnected=t._$AU}_$AO(t,n=!0){t!==this.isConnected&&(this.isConnected=t,t?this.reconnected?.():this.disconnected?.()),n&&(an(this,t),es(this))}setValue(t){if(pm(this._$Ct))this._$Ct._$AI(t,this);else{const n=[...this._$Ct._$AH];n[this._$Ci]=t,this._$Ct._$AI(n,this,0)}}disconnected(){}reconnected(){}}const qs=new WeakMap,sy=oa(class extends ny{render(e){return m}update(e,[t]){const n=t!==this.G;return n&&this.G!==void 0&&this.rt(void 0),(n||this.lt!==this.ct)&&(this.G=t,this.ht=e.options?.host,this.rt(this.ct=e.element)),m}rt(e){if(this.isConnected||(e=void 0),typeof this.G=="function"){const t=this.ht??globalThis;let n=qs.get(t);n===void 0&&(n=new WeakMap,qs.set(t,n)),n.get(this.G)!==void 0&&this.G.call(this.ht,void 0),n.set(this.G,e),e!==void 0&&this.G.call(this.ht,e)}else this.G.value=e}get lt(){return typeof this.G=="function"?qs.get(this.ht??globalThis)?.get(this.G):this.G?.value}disconnected(){this.lt===this.ct&&this.rt(void 0)}reconnected(){this.rt(this.ct)}});class mi extends ra{constructor(t){if(super(t),this.it=m,t.type!==aa.CHILD)throw Error(this.constructor.directiveName+"() can only be used in child bindings")}render(t){if(t===m||t==null)return this._t=void 0,this.it=t;if(t===et)return t;if(typeof t!="string")throw Error(this.constructor.directiveName+"() called with a non-string value");if(t===this.it)return this._t;this.it=t;const n=[t];return n.raw=n,this._t={_$litType$:this.constructor.resultType,strings:n,values:[]}}}mi.directiveName="unsafeHTML",mi.resultType=1;const bi=oa(mi);const{entries:Gl,setPrototypeOf:zo,isFrozen:iy,getPrototypeOf:ay,getOwnPropertyDescriptor:oy}=Object;let{freeze:me,seal:Te,create:yi}=Object,{apply:xi,construct:$i}=typeof Reflect<"u"&&Reflect;me||(me=function(t){return t});Te||(Te=function(t){return t});xi||(xi=function(t,n){for(var s=arguments.length,i=new Array(s>2?s-2:0),a=2;a<s;a++)i[a-2]=arguments[a];return t.apply(n,i)});$i||($i=function(t){for(var n=arguments.length,s=new Array(n>1?n-1:0),i=1;i<n;i++)s[i-1]=arguments[i];return new t(...s)});const Fn=be(Array.prototype.forEach),ry=be(Array.prototype.lastIndexOf),Ho=be(Array.prototype.pop),Kt=be(Array.prototype.push),ly=be(Array.prototype.splice),Wn=be(String.prototype.toLowerCase),Gs=be(String.prototype.toString),Vs=be(String.prototype.match),jt=be(String.prototype.replace),cy=be(String.prototype.indexOf),dy=be(String.prototype.trim),Ee=be(Object.prototype.hasOwnProperty),fe=be(RegExp.prototype.test),Wt=uy(TypeError);function be(e){return function(t){t instanceof RegExp&&(t.lastIndex=0);for(var n=arguments.length,s=new Array(n>1?n-1:0),i=1;i<n;i++)s[i-1]=arguments[i];return xi(e,t,s)}}function uy(e){return function(){for(var t=arguments.length,n=new Array(t),s=0;s<t;s++)n[s]=arguments[s];return $i(e,n)}}function q(e,t){let n=arguments.length>2&&arguments[2]!==void 0?arguments[2]:Wn;zo&&zo(e,null);let s=t.length;for(;s--;){let i=t[s];if(typeof i=="string"){const a=n(i);a!==i&&(iy(t)||(t[s]=a),i=a)}e[i]=!0}return e}function gy(e){for(let t=0;t<e.length;t++)Ee(e,t)||(e[t]=null);return e}function Fe(e){const t=yi(null);for(const[n,s]of Gl(e))Ee(e,n)&&(Array.isArray(s)?t[n]=gy(s):s&&typeof s=="object"&&s.constructor===Object?t[n]=Fe(s):t[n]=s);return t}function qt(e,t){for(;e!==null;){const s=oy(e,t);if(s){if(s.get)return be(s.get);if(typeof s.value=="function")return be(s.value)}e=ay(e)}function n(){return null}return n}const Ko=me(["a","abbr","acronym","address","area","article","aside","audio","b","bdi","bdo","big","blink","blockquote","body","br","button","canvas","caption","center","cite","code","col","colgroup","content","data","datalist","dd","decorator","del","details","dfn","dialog","dir","div","dl","dt","element","em","fieldset","figcaption","figure","font","footer","form","h1","h2","h3","h4","h5","h6","head","header","hgroup","hr","html","i","img","input","ins","kbd","label","legend","li","main","map","mark","marquee","menu","menuitem","meter","nav","nobr","ol","optgroup","option","output","p","picture","pre","progress","q","rp","rt","ruby","s","samp","search","section","select","shadow","slot","small","source","spacer","span","strike","strong","style","sub","summary","sup","table","tbody","td","template","textarea","tfoot","th","thead","time","tr","track","tt","u","ul","var","video","wbr"]),Qs=me(["svg","a","altglyph","altglyphdef","altglyphitem","animatecolor","animatemotion","animatetransform","circle","clippath","defs","desc","ellipse","enterkeyhint","exportparts","filter","font","g","glyph","glyphref","hkern","image","inputmode","line","lineargradient","marker","mask","metadata","mpath","part","path","pattern","polygon","polyline","radialgradient","rect","stop","style","switch","symbol","text","textpath","title","tref","tspan","view","vkern"]),Ys=me(["feBlend","feColorMatrix","feComponentTransfer","feComposite","feConvolveMatrix","feDiffuseLighting","feDisplacementMap","feDistantLight","feDropShadow","feFlood","feFuncA","feFuncB","feFuncG","feFuncR","feGaussianBlur","feImage","feMerge","feMergeNode","feMorphology","feOffset","fePointLight","feSpecularLighting","feSpotLight","feTile","feTurbulence"]),py=me(["animate","color-profile","cursor","discard","font-face","font-face-format","font-face-name","font-face-src","font-face-uri","foreignobject","hatch","hatchpath","mesh","meshgradient","meshpatch","meshrow","missing-glyph","script","set","solidcolor","unknown","use"]),Js=me(["math","menclose","merror","mfenced","mfrac","mglyph","mi","mlabeledtr","mmultiscripts","mn","mo","mover","mpadded","mphantom","mroot","mrow","ms","mspace","msqrt","mstyle","msub","msup","msubsup","mtable","mtd","mtext","mtr","munder","munderover","mprescripts"]),hy=me(["maction","maligngroup","malignmark","mlongdiv","mscarries","mscarry","msgroup","mstack","msline","msrow","semantics","annotation","annotation-xml","mprescripts","none"]),jo=me(["#text"]),Wo=me(["accept","action","align","alt","autocapitalize","autocomplete","autopictureinpicture","autoplay","background","bgcolor","border","capture","cellpadding","cellspacing","checked","cite","class","clear","color","cols","colspan","controls","controlslist","coords","crossorigin","datetime","decoding","default","dir","disabled","disablepictureinpicture","disableremoteplayback","download","draggable","enctype","enterkeyhint","exportparts","face","for","headers","height","hidden","high","href","hreflang","id","inert","inputmode","integrity","ismap","kind","label","lang","list","loading","loop","low","max","maxlength","media","method","min","minlength","multiple","muted","name","nonce","noshade","novalidate","nowrap","open","optimum","part","pattern","placeholder","playsinline","popover","popovertarget","popovertargetaction","poster","preload","pubdate","radiogroup","readonly","rel","required","rev","reversed","role","rows","rowspan","spellcheck","scope","selected","shape","size","sizes","slot","span","srclang","start","src","srcset","step","style","summary","tabindex","title","translate","type","usemap","valign","value","width","wrap","xmlns","slot"]),Zs=me(["accent-height","accumulate","additive","alignment-baseline","amplitude","ascent","attributename","attributetype","azimuth","basefrequency","baseline-shift","begin","bias","by","class","clip","clippathunits","clip-path","clip-rule","color","color-interpolation","color-interpolation-filters","color-profile","color-rendering","cx","cy","d","dx","dy","diffuseconstant","direction","display","divisor","dur","edgemode","elevation","end","exponent","fill","fill-opacity","fill-rule","filter","filterunits","flood-color","flood-opacity","font-family","font-size","font-size-adjust","font-stretch","font-style","font-variant","font-weight","fx","fy","g1","g2","glyph-name","glyphref","gradientunits","gradienttransform","height","href","id","image-rendering","in","in2","intercept","k","k1","k2","k3","k4","kerning","keypoints","keysplines","keytimes","lang","lengthadjust","letter-spacing","kernelmatrix","kernelunitlength","lighting-color","local","marker-end","marker-mid","marker-start","markerheight","markerunits","markerwidth","maskcontentunits","maskunits","max","mask","mask-type","media","method","mode","min","name","numoctaves","offset","operator","opacity","order","orient","orientation","origin","overflow","paint-order","path","pathlength","patterncontentunits","patterntransform","patternunits","points","preservealpha","preserveaspectratio","primitiveunits","r","rx","ry","radius","refx","refy","repeatcount","repeatdur","restart","result","rotate","scale","seed","shape-rendering","slope","specularconstant","specularexponent","spreadmethod","startoffset","stddeviation","stitchtiles","stop-color","stop-opacity","stroke-dasharray","stroke-dashoffset","stroke-linecap","stroke-linejoin","stroke-miterlimit","stroke-opacity","stroke","stroke-width","style","surfacescale","systemlanguage","tabindex","tablevalues","targetx","targety","transform","transform-origin","text-anchor","text-decoration","text-rendering","textlength","type","u1","u2","unicode","values","viewbox","visibility","version","vert-adv-y","vert-origin-x","vert-origin-y","width","word-spacing","wrap","writing-mode","xchannelselector","ychannelselector","x","x1","x2","xmlns","y","y1","y2","z","zoomandpan"]),qo=me(["accent","accentunder","align","bevelled","close","columnsalign","columnlines","columnspan","denomalign","depth","dir","display","displaystyle","encoding","fence","frame","height","href","id","largeop","length","linethickness","lspace","lquote","mathbackground","mathcolor","mathsize","mathvariant","maxsize","minsize","movablelimits","notation","numalign","open","rowalign","rowlines","rowspacing","rowspan","rspace","rquote","scriptlevel","scriptminsize","scriptsizemultiplier","selection","separator","separators","stretchy","subscriptshift","supscriptshift","symmetric","voffset","width","xmlns"]),Nn=me(["xlink:href","xml:id","xlink:title","xml:space","xmlns:xlink"]),fy=Te(/\{\{[\w\W]*|[\w\W]*\}\}/gm),vy=Te(/<%[\w\W]*|[\w\W]*%>/gm),my=Te(/\$\{[\w\W]*/gm),by=Te(/^data-[\-\w.\u00B7-\uFFFF]+$/),yy=Te(/^aria-[\-\w]+$/),Vl=Te(/^(?:(?:(?:f|ht)tps?|mailto|tel|callto|sms|cid|xmpp|matrix):|[^a-z]|[a-z+.\-]+(?:[^a-z+.\-:]|$))/i),xy=Te(/^(?:\w+script|data):/i),$y=Te(/[\u0000-\u0020\u00A0\u1680\u180E\u2000-\u2029\u205F\u3000]/g),Ql=Te(/^html$/i),wy=Te(/^[a-z][.\w]*(-[.\w]+)+$/i);var Go=Object.freeze({__proto__:null,ARIA_ATTR:yy,ATTR_WHITESPACE:$y,CUSTOM_ELEMENT:wy,DATA_ATTR:by,DOCTYPE_NAME:Ql,ERB_EXPR:vy,IS_ALLOWED_URI:Vl,IS_SCRIPT_OR_DATA:xy,MUSTACHE_EXPR:fy,TMPLIT_EXPR:my});const Gt={element:1,text:3,progressingInstruction:7,comment:8,document:9},ky=function(){return typeof window>"u"?null:window},Sy=function(t,n){if(typeof t!="object"||typeof t.createPolicy!="function")return null;let s=null;const i="data-tt-policy-suffix";n&&n.hasAttribute(i)&&(s=n.getAttribute(i));const a="dompurify"+(s?"#"+s:"");try{return t.createPolicy(a,{createHTML(o){return o},createScriptURL(o){return o}})}catch{return console.warn("TrustedTypes policy "+a+" could not be created."),null}},Vo=function(){return{afterSanitizeAttributes:[],afterSanitizeElements:[],afterSanitizeShadowDOM:[],beforeSanitizeAttributes:[],beforeSanitizeElements:[],beforeSanitizeShadowDOM:[],uponSanitizeAttribute:[],uponSanitizeElement:[],uponSanitizeShadowNode:[]}};function Yl(){let e=arguments.length>0&&arguments[0]!==void 0?arguments[0]:ky();const t=H=>Yl(H);if(t.version="3.3.1",t.removed=[],!e||!e.document||e.document.nodeType!==Gt.document||!e.Element)return t.isSupported=!1,t;let{document:n}=e;const s=n,i=s.currentScript,{DocumentFragment:a,HTMLTemplateElement:o,Node:l,Element:d,NodeFilter:g,NamedNodeMap:f=e.NamedNodeMap||e.MozNamedAttrMap,HTMLFormElement:p,DOMParser:b,trustedTypes:u}=e,v=d.prototype,y=qt(v,"cloneNode"),k=qt(v,"remove"),C=qt(v,"nextSibling"),$=qt(v,"childNodes"),T=qt(v,"parentNode");if(typeof o=="function"){const H=n.createElement("template");H.content&&H.content.ownerDocument&&(n=H.content.ownerDocument)}let _,L="";const{implementation:E,createNodeIterator:P,createDocumentFragment:j,getElementsByTagName:Z}=n,{importNode:ae}=s;let O=Vo();t.isSupported=typeof Gl=="function"&&typeof T=="function"&&E&&E.createHTMLDocument!==void 0;const{MUSTACHE_EXPR:K,ERB_EXPR:ue,TMPLIT_EXPR:M,DATA_ATTR:z,ARIA_ATTR:oe,IS_SCRIPT_OR_DATA:re,ATTR_WHITESPACE:ee,CUSTOM_ELEMENT:se}=Go;let{IS_ALLOWED_URI:R}=Go,D=null;const F=q({},[...Ko,...Qs,...Ys,...Js,...jo]);let W=null;const $e=q({},[...Wo,...Zs,...qo,...Nn]);let J=Object.seal(yi(null,{tagNameCheck:{writable:!0,configurable:!1,enumerable:!0,value:null},attributeNameCheck:{writable:!0,configurable:!1,enumerable:!0,value:null},allowCustomizedBuiltInElements:{writable:!0,configurable:!1,enumerable:!0,value:!1}})),Se=null,te=null;const he=Object.seal(yi(null,{tagCheck:{writable:!0,configurable:!1,enumerable:!0,value:null},attributeCheck:{writable:!0,configurable:!1,enumerable:!0,value:null}}));let Be=!0,Ue=!0,it=!1,Aa=!0,Tt=!1,$n=!0,at=!1,xs=!1,$s=!1,Et=!1,wn=!1,kn=!1,_a=!0,Ca=!1;const qg="user-content-";let ws=!0,Bt=!1,Lt={},Re=null;const ks=q({},["annotation-xml","audio","colgroup","desc","foreignobject","head","iframe","math","mi","mn","mo","ms","mtext","noembed","noframes","noscript","plaintext","script","style","svg","template","thead","title","video","xmp"]);let Ta=null;const Ea=q({},["audio","video","img","source","image","track"]);let Ss=null;const La=q({},["alt","class","for","id","label","name","pattern","placeholder","role","summary","title","value","style","xmlns"]),Sn="http://www.w3.org/1998/Math/MathML",An="http://www.w3.org/2000/svg",ze="http://www.w3.org/1999/xhtml";let Mt=ze,As=!1,_s=null;const Gg=q({},[Sn,An,ze],Gs);let _n=q({},["mi","mo","mn","ms","mtext"]),Cn=q({},["annotation-xml"]);const Vg=q({},["title","style","font","a","script"]);let Ut=null;const Qg=["application/xhtml+xml","text/html"],Yg="text/html";let ie=null,It=null;const Jg=n.createElement("form"),Ma=function(x){return x instanceof RegExp||x instanceof Function},Cs=function(){let x=arguments.length>0&&arguments[0]!==void 0?arguments[0]:{};if(!(It&&It===x)){if((!x||typeof x!="object")&&(x={}),x=Fe(x),Ut=Qg.indexOf(x.PARSER_MEDIA_TYPE)===-1?Yg:x.PARSER_MEDIA_TYPE,ie=Ut==="application/xhtml+xml"?Gs:Wn,D=Ee(x,"ALLOWED_TAGS")?q({},x.ALLOWED_TAGS,ie):F,W=Ee(x,"ALLOWED_ATTR")?q({},x.ALLOWED_ATTR,ie):$e,_s=Ee(x,"ALLOWED_NAMESPACES")?q({},x.ALLOWED_NAMESPACES,Gs):Gg,Ss=Ee(x,"ADD_URI_SAFE_ATTR")?q(Fe(La),x.ADD_URI_SAFE_ATTR,ie):La,Ta=Ee(x,"ADD_DATA_URI_TAGS")?q(Fe(Ea),x.ADD_DATA_URI_TAGS,ie):Ea,Re=Ee(x,"FORBID_CONTENTS")?q({},x.FORBID_CONTENTS,ie):ks,Se=Ee(x,"FORBID_TAGS")?q({},x.FORBID_TAGS,ie):Fe({}),te=Ee(x,"FORBID_ATTR")?q({},x.FORBID_ATTR,ie):Fe({}),Lt=Ee(x,"USE_PROFILES")?x.USE_PROFILES:!1,Be=x.ALLOW_ARIA_ATTR!==!1,Ue=x.ALLOW_DATA_ATTR!==!1,it=x.ALLOW_UNKNOWN_PROTOCOLS||!1,Aa=x.ALLOW_SELF_CLOSE_IN_ATTR!==!1,Tt=x.SAFE_FOR_TEMPLATES||!1,$n=x.SAFE_FOR_XML!==!1,at=x.WHOLE_DOCUMENT||!1,Et=x.RETURN_DOM||!1,wn=x.RETURN_DOM_FRAGMENT||!1,kn=x.RETURN_TRUSTED_TYPE||!1,$s=x.FORCE_BODY||!1,_a=x.SANITIZE_DOM!==!1,Ca=x.SANITIZE_NAMED_PROPS||!1,ws=x.KEEP_CONTENT!==!1,Bt=x.IN_PLACE||!1,R=x.ALLOWED_URI_REGEXP||Vl,Mt=x.NAMESPACE||ze,_n=x.MATHML_TEXT_INTEGRATION_POINTS||_n,Cn=x.HTML_INTEGRATION_POINTS||Cn,J=x.CUSTOM_ELEMENT_HANDLING||{},x.CUSTOM_ELEMENT_HANDLING&&Ma(x.CUSTOM_ELEMENT_HANDLING.tagNameCheck)&&(J.tagNameCheck=x.CUSTOM_ELEMENT_HANDLING.tagNameCheck),x.CUSTOM_ELEMENT_HANDLING&&Ma(x.CUSTOM_ELEMENT_HANDLING.attributeNameCheck)&&(J.attributeNameCheck=x.CUSTOM_ELEMENT_HANDLING.attributeNameCheck),x.CUSTOM_ELEMENT_HANDLING&&typeof x.CUSTOM_ELEMENT_HANDLING.allowCustomizedBuiltInElements=="boolean"&&(J.allowCustomizedBuiltInElements=x.CUSTOM_ELEMENT_HANDLING.allowCustomizedBuiltInElements),Tt&&(Ue=!1),wn&&(Et=!0),Lt&&(D=q({},jo),W=[],Lt.html===!0&&(q(D,Ko),q(W,Wo)),Lt.svg===!0&&(q(D,Qs),q(W,Zs),q(W,Nn)),Lt.svgFilters===!0&&(q(D,Ys),q(W,Zs),q(W,Nn)),Lt.mathMl===!0&&(q(D,Js),q(W,qo),q(W,Nn))),x.ADD_TAGS&&(typeof x.ADD_TAGS=="function"?he.tagCheck=x.ADD_TAGS:(D===F&&(D=Fe(D)),q(D,x.ADD_TAGS,ie))),x.ADD_ATTR&&(typeof x.ADD_ATTR=="function"?he.attributeCheck=x.ADD_ATTR:(W===$e&&(W=Fe(W)),q(W,x.ADD_ATTR,ie))),x.ADD_URI_SAFE_ATTR&&q(Ss,x.ADD_URI_SAFE_ATTR,ie),x.FORBID_CONTENTS&&(Re===ks&&(Re=Fe(Re)),q(Re,x.FORBID_CONTENTS,ie)),x.ADD_FORBID_CONTENTS&&(Re===ks&&(Re=Fe(Re)),q(Re,x.ADD_FORBID_CONTENTS,ie)),ws&&(D["#text"]=!0),at&&q(D,["html","head","body"]),D.table&&(q(D,["tbody"]),delete Se.tbody),x.TRUSTED_TYPES_POLICY){if(typeof x.TRUSTED_TYPES_POLICY.createHTML!="function")throw Wt('TRUSTED_TYPES_POLICY configuration option must provide a "createHTML" hook.');if(typeof x.TRUSTED_TYPES_POLICY.createScriptURL!="function")throw Wt('TRUSTED_TYPES_POLICY configuration option must provide a "createScriptURL" hook.');_=x.TRUSTED_TYPES_POLICY,L=_.createHTML("")}else _===void 0&&(_=Sy(u,i)),_!==null&&typeof L=="string"&&(L=_.createHTML(""));me&&me(x),It=x}},Ia=q({},[...Qs,...Ys,...py]),Ra=q({},[...Js,...hy]),Zg=function(x){let I=T(x);(!I||!I.tagName)&&(I={namespaceURI:Mt,tagName:"template"});const B=Wn(x.tagName),X=Wn(I.tagName);return _s[x.namespaceURI]?x.namespaceURI===An?I.namespaceURI===ze?B==="svg":I.namespaceURI===Sn?B==="svg"&&(X==="annotation-xml"||_n[X]):!!Ia[B]:x.namespaceURI===Sn?I.namespaceURI===ze?B==="math":I.namespaceURI===An?B==="math"&&Cn[X]:!!Ra[B]:x.namespaceURI===ze?I.namespaceURI===An&&!Cn[X]||I.namespaceURI===Sn&&!_n[X]?!1:!Ra[B]&&(Vg[B]||!Ia[B]):!!(Ut==="application/xhtml+xml"&&_s[x.namespaceURI]):!1},Pe=function(x){Kt(t.removed,{element:x});try{T(x).removeChild(x)}catch{k(x)}},ot=function(x,I){try{Kt(t.removed,{attribute:I.getAttributeNode(x),from:I})}catch{Kt(t.removed,{attribute:null,from:I})}if(I.removeAttribute(x),x==="is")if(Et||wn)try{Pe(I)}catch{}else try{I.setAttribute(x,"")}catch{}},Pa=function(x){let I=null,B=null;if($s)x="<remove></remove>"+x;else{const ne=Vs(x,/^[\r\n\t ]+/);B=ne&&ne[0]}Ut==="application/xhtml+xml"&&Mt===ze&&(x='<html xmlns="http://www.w3.org/1999/xhtml"><head></head><body>'+x+"</body></html>");const X=_?_.createHTML(x):x;if(Mt===ze)try{I=new b().parseFromString(X,Ut)}catch{}if(!I||!I.documentElement){I=E.createDocument(Mt,"template",null);try{I.documentElement.innerHTML=As?L:X}catch{}}const ge=I.body||I.documentElement;return x&&B&&ge.insertBefore(n.createTextNode(B),ge.childNodes[0]||null),Mt===ze?Z.call(I,at?"html":"body")[0]:at?I.documentElement:ge},Da=function(x){return P.call(x.ownerDocument||x,x,g.SHOW_ELEMENT|g.SHOW_COMMENT|g.SHOW_TEXT|g.SHOW_PROCESSING_INSTRUCTION|g.SHOW_CDATA_SECTION,null)},Ts=function(x){return x instanceof p&&(typeof x.nodeName!="string"||typeof x.textContent!="string"||typeof x.removeChild!="function"||!(x.attributes instanceof f)||typeof x.removeAttribute!="function"||typeof x.setAttribute!="function"||typeof x.namespaceURI!="string"||typeof x.insertBefore!="function"||typeof x.hasChildNodes!="function")},Fa=function(x){return typeof l=="function"&&x instanceof l};function He(H,x,I){Fn(H,B=>{B.call(t,x,I,It)})}const Na=function(x){let I=null;if(He(O.beforeSanitizeElements,x,null),Ts(x))return Pe(x),!0;const B=ie(x.nodeName);if(He(O.uponSanitizeElement,x,{tagName:B,allowedTags:D}),$n&&x.hasChildNodes()&&!Fa(x.firstElementChild)&&fe(/<[/\w!]/g,x.innerHTML)&&fe(/<[/\w!]/g,x.textContent)||x.nodeType===Gt.progressingInstruction||$n&&x.nodeType===Gt.comment&&fe(/<[/\w]/g,x.data))return Pe(x),!0;if(!(he.tagCheck instanceof Function&&he.tagCheck(B))&&(!D[B]||Se[B])){if(!Se[B]&&Ba(B)&&(J.tagNameCheck instanceof RegExp&&fe(J.tagNameCheck,B)||J.tagNameCheck instanceof Function&&J.tagNameCheck(B)))return!1;if(ws&&!Re[B]){const X=T(x)||x.parentNode,ge=$(x)||x.childNodes;if(ge&&X){const ne=ge.length;for(let ye=ne-1;ye>=0;--ye){const Ke=y(ge[ye],!0);Ke.__removalCount=(x.__removalCount||0)+1,X.insertBefore(Ke,C(x))}}}return Pe(x),!0}return x instanceof d&&!Zg(x)||(B==="noscript"||B==="noembed"||B==="noframes")&&fe(/<\/no(script|embed|frames)/i,x.innerHTML)?(Pe(x),!0):(Tt&&x.nodeType===Gt.text&&(I=x.textContent,Fn([K,ue,M],X=>{I=jt(I,X," ")}),x.textContent!==I&&(Kt(t.removed,{element:x.cloneNode()}),x.textContent=I)),He(O.afterSanitizeElements,x,null),!1)},Oa=function(x,I,B){if(_a&&(I==="id"||I==="name")&&(B in n||B in Jg))return!1;if(!(Ue&&!te[I]&&fe(z,I))){if(!(Be&&fe(oe,I))){if(!(he.attributeCheck instanceof Function&&he.attributeCheck(I,x))){if(!W[I]||te[I]){if(!(Ba(x)&&(J.tagNameCheck instanceof RegExp&&fe(J.tagNameCheck,x)||J.tagNameCheck instanceof Function&&J.tagNameCheck(x))&&(J.attributeNameCheck instanceof RegExp&&fe(J.attributeNameCheck,I)||J.attributeNameCheck instanceof Function&&J.attributeNameCheck(I,x))||I==="is"&&J.allowCustomizedBuiltInElements&&(J.tagNameCheck instanceof RegExp&&fe(J.tagNameCheck,B)||J.tagNameCheck instanceof Function&&J.tagNameCheck(B))))return!1}else if(!Ss[I]){if(!fe(R,jt(B,ee,""))){if(!((I==="src"||I==="xlink:href"||I==="href")&&x!=="script"&&cy(B,"data:")===0&&Ta[x])){if(!(it&&!fe(re,jt(B,ee,"")))){if(B)return!1}}}}}}}return!0},Ba=function(x){return x!=="annotation-xml"&&Vs(x,se)},Ua=function(x){He(O.beforeSanitizeAttributes,x,null);const{attributes:I}=x;if(!I||Ts(x))return;const B={attrName:"",attrValue:"",keepAttr:!0,allowedAttributes:W,forceKeepAttr:void 0};let X=I.length;for(;X--;){const ge=I[X],{name:ne,namespaceURI:ye,value:Ke}=ge,Rt=ie(ne),Es=Ke;let ce=ne==="value"?Es:dy(Es);if(B.attrName=Rt,B.attrValue=ce,B.keepAttr=!0,B.forceKeepAttr=void 0,He(O.uponSanitizeAttribute,x,B),ce=B.attrValue,Ca&&(Rt==="id"||Rt==="name")&&(ot(ne,x),ce=qg+ce),$n&&fe(/((--!?|])>)|<\/(style|title|textarea)/i,ce)){ot(ne,x);continue}if(Rt==="attributename"&&Vs(ce,"href")){ot(ne,x);continue}if(B.forceKeepAttr)continue;if(!B.keepAttr){ot(ne,x);continue}if(!Aa&&fe(/\/>/i,ce)){ot(ne,x);continue}Tt&&Fn([K,ue,M],Ha=>{ce=jt(ce,Ha," ")});const za=ie(x.nodeName);if(!Oa(za,Rt,ce)){ot(ne,x);continue}if(_&&typeof u=="object"&&typeof u.getAttributeType=="function"&&!ye)switch(u.getAttributeType(za,Rt)){case"TrustedHTML":{ce=_.createHTML(ce);break}case"TrustedScriptURL":{ce=_.createScriptURL(ce);break}}if(ce!==Es)try{ye?x.setAttributeNS(ye,ne,ce):x.setAttribute(ne,ce),Ts(x)?Pe(x):Ho(t.removed)}catch{ot(ne,x)}}He(O.afterSanitizeAttributes,x,null)},Xg=function H(x){let I=null;const B=Da(x);for(He(O.beforeSanitizeShadowDOM,x,null);I=B.nextNode();)He(O.uponSanitizeShadowNode,I,null),Na(I),Ua(I),I.content instanceof a&&H(I.content);He(O.afterSanitizeShadowDOM,x,null)};return t.sanitize=function(H){let x=arguments.length>1&&arguments[1]!==void 0?arguments[1]:{},I=null,B=null,X=null,ge=null;if(As=!H,As&&(H="<!-->"),typeof H!="string"&&!Fa(H))if(typeof H.toString=="function"){if(H=H.toString(),typeof H!="string")throw Wt("dirty is not a string, aborting")}else throw Wt("toString is not a function");if(!t.isSupported)return H;if(xs||Cs(x),t.removed=[],typeof H=="string"&&(Bt=!1),Bt){if(H.nodeName){const Ke=ie(H.nodeName);if(!D[Ke]||Se[Ke])throw Wt("root node is forbidden and cannot be sanitized in-place")}}else if(H instanceof l)I=Pa("<!---->"),B=I.ownerDocument.importNode(H,!0),B.nodeType===Gt.element&&B.nodeName==="BODY"||B.nodeName==="HTML"?I=B:I.appendChild(B);else{if(!Et&&!Tt&&!at&&H.indexOf("<")===-1)return _&&kn?_.createHTML(H):H;if(I=Pa(H),!I)return Et?null:kn?L:""}I&&$s&&Pe(I.firstChild);const ne=Da(Bt?H:I);for(;X=ne.nextNode();)Na(X),Ua(X),X.content instanceof a&&Xg(X.content);if(Bt)return H;if(Et){if(wn)for(ge=j.call(I.ownerDocument);I.firstChild;)ge.appendChild(I.firstChild);else ge=I;return(W.shadowroot||W.shadowrootmode)&&(ge=ae.call(s,ge,!0)),ge}let ye=at?I.outerHTML:I.innerHTML;return at&&D["!doctype"]&&I.ownerDocument&&I.ownerDocument.doctype&&I.ownerDocument.doctype.name&&fe(Ql,I.ownerDocument.doctype.name)&&(ye="<!DOCTYPE "+I.ownerDocument.doctype.name+`>
`+ye),Tt&&Fn([K,ue,M],Ke=>{ye=jt(ye,Ke," ")}),_&&kn?_.createHTML(ye):ye},t.setConfig=function(){let H=arguments.length>0&&arguments[0]!==void 0?arguments[0]:{};Cs(H),xs=!0},t.clearConfig=function(){It=null,xs=!1},t.isValidAttribute=function(H,x,I){It||Cs({});const B=ie(H),X=ie(x);return Oa(B,X,I)},t.addHook=function(H,x){typeof x=="function"&&Kt(O[H],x)},t.removeHook=function(H,x){if(x!==void 0){const I=ry(O[H],x);return I===-1?void 0:ly(O[H],I,1)[0]}return Ho(O[H])},t.removeHooks=function(H){O[H]=[]},t.removeAllHooks=function(){O=Vo()},t}var wi=Yl();function da(){return{async:!1,breaks:!1,extensions:null,gfm:!0,hooks:null,pedantic:!1,renderer:null,silent:!1,tokenizer:null,walkTokens:null}}var Ct=da();function Jl(e){Ct=e}var vt={exec:()=>null};function G(e,t=""){let n=typeof e=="string"?e:e.source,s={replace:(i,a)=>{let o=typeof a=="string"?a:a.source;return o=o.replace(ve.caret,"$1"),n=n.replace(i,o),s},getRegex:()=>new RegExp(n,t)};return s}var Ay=(()=>{try{return!!new RegExp("(?<=1)(?<!1)")}catch{return!1}})(),ve={codeRemoveIndent:/^(?: {1,4}| {0,3}\t)/gm,outputLinkReplace:/\\([\[\]])/g,indentCodeCompensation:/^(\s+)(?:```)/,beginningSpace:/^\s+/,endingHash:/#$/,startingSpaceChar:/^ /,endingSpaceChar:/ $/,nonSpaceChar:/[^ ]/,newLineCharGlobal:/\n/g,tabCharGlobal:/\t/g,multipleSpaceGlobal:/\s+/g,blankLine:/^[ \t]*$/,doubleBlankLine:/\n[ \t]*\n[ \t]*$/,blockquoteStart:/^ {0,3}>/,blockquoteSetextReplace:/\n {0,3}((?:=+|-+) *)(?=\n|$)/g,blockquoteSetextReplace2:/^ {0,3}>[ \t]?/gm,listReplaceNesting:/^ {1,4}(?=( {4})*[^ ])/g,listIsTask:/^\[[ xX]\] +\S/,listReplaceTask:/^\[[ xX]\] +/,listTaskCheckbox:/\[[ xX]\]/,anyLine:/\n.*\n/,hrefBrackets:/^<(.*)>$/,tableDelimiter:/[:|]/,tableAlignChars:/^\||\| *$/g,tableRowBlankLine:/\n[ \t]*$/,tableAlignRight:/^ *-+: *$/,tableAlignCenter:/^ *:-+: *$/,tableAlignLeft:/^ *:-+ *$/,startATag:/^<a /i,endATag:/^<\/a>/i,startPreScriptTag:/^<(pre|code|kbd|script)(\s|>)/i,endPreScriptTag:/^<\/(pre|code|kbd|script)(\s|>)/i,startAngleBracket:/^</,endAngleBracket:/>$/,pedanticHrefTitle:/^([^'"]*[^\s])\s+(['"])(.*)\2/,unicodeAlphaNumeric:/[\p{L}\p{N}]/u,escapeTest:/[&<>"']/,escapeReplace:/[&<>"']/g,escapeTestNoEncode:/[<>"']|&(?!(#\d{1,7}|#[Xx][a-fA-F0-9]{1,6}|\w+);)/,escapeReplaceNoEncode:/[<>"']|&(?!(#\d{1,7}|#[Xx][a-fA-F0-9]{1,6}|\w+);)/g,unescapeTest:/&(#(?:\d+)|(?:#x[0-9A-Fa-f]+)|(?:\w+));?/ig,caret:/(^|[^\[])\^/g,percentDecode:/%25/g,findPipe:/\|/g,splitPipe:/ \|/,slashPipe:/\\\|/g,carriageReturn:/\r\n|\r/g,spaceLine:/^ +$/gm,notSpaceStart:/^\S*/,endingNewline:/\n$/,listItemRegex:e=>new RegExp(`^( {0,3}${e})((?:[	 ][^\\n]*)?(?:\\n|$))`),nextBulletRegex:e=>new RegExp(`^ {0,${Math.min(3,e-1)}}(?:[*+-]|\\d{1,9}[.)])((?:[ 	][^\\n]*)?(?:\\n|$))`),hrRegex:e=>new RegExp(`^ {0,${Math.min(3,e-1)}}((?:- *){3,}|(?:_ *){3,}|(?:\\* *){3,})(?:\\n+|$)`),fencesBeginRegex:e=>new RegExp(`^ {0,${Math.min(3,e-1)}}(?:\`\`\`|~~~)`),headingBeginRegex:e=>new RegExp(`^ {0,${Math.min(3,e-1)}}#`),htmlBeginRegex:e=>new RegExp(`^ {0,${Math.min(3,e-1)}}<(?:[a-z].*>|!--)`,"i"),blockquoteBeginRegex:e=>new RegExp(`^ {0,${Math.min(3,e-1)}}>`)},_y=/^(?:[ \t]*(?:\n|$))+/,Cy=/^((?: {4}| {0,3}\t)[^\n]+(?:\n(?:[ \t]*(?:\n|$))*)?)+/,Ty=/^ {0,3}(`{3,}(?=[^`\n]*(?:\n|$))|~{3,})([^\n]*)(?:\n|$)(?:|([\s\S]*?)(?:\n|$))(?: {0,3}\1[~`]* *(?=\n|$)|$)/,bn=/^ {0,3}((?:-[\t ]*){3,}|(?:_[ \t]*){3,}|(?:\*[ \t]*){3,})(?:\n+|$)/,Ey=/^ {0,3}(#{1,6})(?=\s|$)(.*)(?:\n+|$)/,ua=/ {0,3}(?:[*+-]|\d{1,9}[.)])/,Zl=/^(?!bull |blockCode|fences|blockquote|heading|html|table)((?:.|\n(?!\s*?\n|bull |blockCode|fences|blockquote|heading|html|table))+?)\n {0,3}(=+|-+) *(?:\n+|$)/,Xl=G(Zl).replace(/bull/g,ua).replace(/blockCode/g,/(?: {4}| {0,3}\t)/).replace(/fences/g,/ {0,3}(?:`{3,}|~{3,})/).replace(/blockquote/g,/ {0,3}>/).replace(/heading/g,/ {0,3}#{1,6}/).replace(/html/g,/ {0,3}<[^\n>]+>\n/).replace(/\|table/g,"").getRegex(),Ly=G(Zl).replace(/bull/g,ua).replace(/blockCode/g,/(?: {4}| {0,3}\t)/).replace(/fences/g,/ {0,3}(?:`{3,}|~{3,})/).replace(/blockquote/g,/ {0,3}>/).replace(/heading/g,/ {0,3}#{1,6}/).replace(/html/g,/ {0,3}<[^\n>]+>\n/).replace(/table/g,/ {0,3}\|?(?:[:\- ]*\|)+[\:\- ]*\n/).getRegex(),ga=/^([^\n]+(?:\n(?!hr|heading|lheading|blockquote|fences|list|html|table| +\n)[^\n]+)*)/,My=/^[^\n]+/,pa=/(?!\s*\])(?:\\[\s\S]|[^\[\]\\])+/,Iy=G(/^ {0,3}\[(label)\]: *(?:\n[ \t]*)?([^<\s][^\s]*|<.*?>)(?:(?: +(?:\n[ \t]*)?| *\n[ \t]*)(title))? *(?:\n+|$)/).replace("label",pa).replace("title",/(?:"(?:\\"?|[^"\\])*"|'[^'\n]*(?:\n[^'\n]+)*\n?'|\([^()]*\))/).getRegex(),Ry=G(/^(bull)([ \t][^\n]+?)?(?:\n|$)/).replace(/bull/g,ua).getRegex(),ms="address|article|aside|base|basefont|blockquote|body|caption|center|col|colgroup|dd|details|dialog|dir|div|dl|dt|fieldset|figcaption|figure|footer|form|frame|frameset|h[1-6]|head|header|hr|html|iframe|legend|li|link|main|menu|menuitem|meta|nav|noframes|ol|optgroup|option|p|param|search|section|summary|table|tbody|td|tfoot|th|thead|title|tr|track|ul",ha=/<!--(?:-?>|[\s\S]*?(?:-->|$))/,Py=G("^ {0,3}(?:<(script|pre|style|textarea)[\\s>][\\s\\S]*?(?:</\\1>[^\\n]*\\n+|$)|comment[^\\n]*(\\n+|$)|<\\?[\\s\\S]*?(?:\\?>\\n*|$)|<![A-Z][\\s\\S]*?(?:>\\n*|$)|<!\\[CDATA\\[[\\s\\S]*?(?:\\]\\]>\\n*|$)|</?(tag)(?: +|\\n|/?>)[\\s\\S]*?(?:(?:\\n[ 	]*)+\\n|$)|<(?!script|pre|style|textarea)([a-z][\\w-]*)(?:attribute)*? */?>(?=[ \\t]*(?:\\n|$))[\\s\\S]*?(?:(?:\\n[ 	]*)+\\n|$)|</(?!script|pre|style|textarea)[a-z][\\w-]*\\s*>(?=[ \\t]*(?:\\n|$))[\\s\\S]*?(?:(?:\\n[ 	]*)+\\n|$))","i").replace("comment",ha).replace("tag",ms).replace("attribute",/ +[a-zA-Z:_][\w.:-]*(?: *= *"[^"\n]*"| *= *'[^'\n]*'| *= *[^\s"'=<>`]+)?/).getRegex(),ec=G(ga).replace("hr",bn).replace("heading"," {0,3}#{1,6}(?:\\s|$)").replace("|lheading","").replace("|table","").replace("blockquote"," {0,3}>").replace("fences"," {0,3}(?:`{3,}(?=[^`\\n]*\\n)|~{3,})[^\\n]*\\n").replace("list"," {0,3}(?:[*+-]|1[.)])[ \\t]").replace("html","</?(?:tag)(?: +|\\n|/?>)|<(?:script|pre|style|textarea|!--)").replace("tag",ms).getRegex(),Dy=G(/^( {0,3}> ?(paragraph|[^\n]*)(?:\n|$))+/).replace("paragraph",ec).getRegex(),fa={blockquote:Dy,code:Cy,def:Iy,fences:Ty,heading:Ey,hr:bn,html:Py,lheading:Xl,list:Ry,newline:_y,paragraph:ec,table:vt,text:My},Qo=G("^ *([^\\n ].*)\\n {0,3}((?:\\| *)?:?-+:? *(?:\\| *:?-+:? *)*(?:\\| *)?)(?:\\n((?:(?! *\\n|hr|heading|blockquote|code|fences|list|html).*(?:\\n|$))*)\\n*|$)").replace("hr",bn).replace("heading"," {0,3}#{1,6}(?:\\s|$)").replace("blockquote"," {0,3}>").replace("code","(?: {4}| {0,3}	)[^\\n]").replace("fences"," {0,3}(?:`{3,}(?=[^`\\n]*\\n)|~{3,})[^\\n]*\\n").replace("list"," {0,3}(?:[*+-]|1[.)])[ \\t]").replace("html","</?(?:tag)(?: +|\\n|/?>)|<(?:script|pre|style|textarea|!--)").replace("tag",ms).getRegex(),Fy={...fa,lheading:Ly,table:Qo,paragraph:G(ga).replace("hr",bn).replace("heading"," {0,3}#{1,6}(?:\\s|$)").replace("|lheading","").replace("table",Qo).replace("blockquote"," {0,3}>").replace("fences"," {0,3}(?:`{3,}(?=[^`\\n]*\\n)|~{3,})[^\\n]*\\n").replace("list"," {0,3}(?:[*+-]|1[.)])[ \\t]").replace("html","</?(?:tag)(?: +|\\n|/?>)|<(?:script|pre|style|textarea|!--)").replace("tag",ms).getRegex()},Ny={...fa,html:G(`^ *(?:comment *(?:\\n|\\s*$)|<(tag)[\\s\\S]+?</\\1> *(?:\\n{2,}|\\s*$)|<tag(?:"[^"]*"|'[^']*'|\\s[^'"/>\\s]*)*?/?> *(?:\\n{2,}|\\s*$))`).replace("comment",ha).replace(/tag/g,"(?!(?:a|em|strong|small|s|cite|q|dfn|abbr|data|time|code|var|samp|kbd|sub|sup|i|b|u|mark|ruby|rt|rp|bdi|bdo|span|br|wbr|ins|del|img)\\b)\\w+(?!:|[^\\w\\s@]*@)\\b").getRegex(),def:/^ *\[([^\]]+)\]: *<?([^\s>]+)>?(?: +(["(][^\n]+[")]))? *(?:\n+|$)/,heading:/^(#{1,6})(.*)(?:\n+|$)/,fences:vt,lheading:/^(.+?)\n {0,3}(=+|-+) *(?:\n+|$)/,paragraph:G(ga).replace("hr",bn).replace("heading",` *#{1,6} *[^
]`).replace("lheading",Xl).replace("|table","").replace("blockquote"," {0,3}>").replace("|fences","").replace("|list","").replace("|html","").replace("|tag","").getRegex()},Oy=/^\\([!"#$%&'()*+,\-./:;<=>?@\[\]\\^_`{|}~])/,By=/^(`+)([^`]|[^`][\s\S]*?[^`])\1(?!`)/,tc=/^( {2,}|\\)\n(?!\s*$)/,Uy=/^(`+|[^`])(?:(?= {2,}\n)|[\s\S]*?(?:(?=[\\<!\[`*_]|\b_|$)|[^ ](?= {2,}\n)))/,bs=/[\p{P}\p{S}]/u,va=/[\s\p{P}\p{S}]/u,nc=/[^\s\p{P}\p{S}]/u,zy=G(/^((?![*_])punctSpace)/,"u").replace(/punctSpace/g,va).getRegex(),sc=/(?!~)[\p{P}\p{S}]/u,Hy=/(?!~)[\s\p{P}\p{S}]/u,Ky=/(?:[^\s\p{P}\p{S}]|~)/u,ic=/(?![*_])[\p{P}\p{S}]/u,jy=/(?![*_])[\s\p{P}\p{S}]/u,Wy=/(?:[^\s\p{P}\p{S}]|[*_])/u,qy=G(/link|precode-code|html/,"g").replace("link",/\[(?:[^\[\]`]|(?<a>`+)[^`]+\k<a>(?!`))*?\]\((?:\\[\s\S]|[^\\\(\)]|\((?:\\[\s\S]|[^\\\(\)])*\))*\)/).replace("precode-",Ay?"(?<!`)()":"(^^|[^`])").replace("code",/(?<b>`+)[^`]+\k<b>(?!`)/).replace("html",/<(?! )[^<>]*?>/).getRegex(),ac=/^(?:\*+(?:((?!\*)punct)|[^\s*]))|^_+(?:((?!_)punct)|([^\s_]))/,Gy=G(ac,"u").replace(/punct/g,bs).getRegex(),Vy=G(ac,"u").replace(/punct/g,sc).getRegex(),oc="^[^_*]*?__[^_*]*?\\*[^_*]*?(?=__)|[^*]+(?=[^*])|(?!\\*)punct(\\*+)(?=[\\s]|$)|notPunctSpace(\\*+)(?!\\*)(?=punctSpace|$)|(?!\\*)punctSpace(\\*+)(?=notPunctSpace)|[\\s](\\*+)(?!\\*)(?=punct)|(?!\\*)punct(\\*+)(?!\\*)(?=punct)|notPunctSpace(\\*+)(?=notPunctSpace)",Qy=G(oc,"gu").replace(/notPunctSpace/g,nc).replace(/punctSpace/g,va).replace(/punct/g,bs).getRegex(),Yy=G(oc,"gu").replace(/notPunctSpace/g,Ky).replace(/punctSpace/g,Hy).replace(/punct/g,sc).getRegex(),Jy=G("^[^_*]*?\\*\\*[^_*]*?_[^_*]*?(?=\\*\\*)|[^_]+(?=[^_])|(?!_)punct(_+)(?=[\\s]|$)|notPunctSpace(_+)(?!_)(?=punctSpace|$)|(?!_)punctSpace(_+)(?=notPunctSpace)|[\\s](_+)(?!_)(?=punct)|(?!_)punct(_+)(?!_)(?=punct)","gu").replace(/notPunctSpace/g,nc).replace(/punctSpace/g,va).replace(/punct/g,bs).getRegex(),Zy=G(/^~~?(?:((?!~)punct)|[^\s~])/,"u").replace(/punct/g,ic).getRegex(),Xy="^[^~]+(?=[^~])|(?!~)punct(~~?)(?=[\\s]|$)|notPunctSpace(~~?)(?!~)(?=punctSpace|$)|(?!~)punctSpace(~~?)(?=notPunctSpace)|[\\s](~~?)(?!~)(?=punct)|(?!~)punct(~~?)(?!~)(?=punct)|notPunctSpace(~~?)(?=notPunctSpace)",e0=G(Xy,"gu").replace(/notPunctSpace/g,Wy).replace(/punctSpace/g,jy).replace(/punct/g,ic).getRegex(),t0=G(/\\(punct)/,"gu").replace(/punct/g,bs).getRegex(),n0=G(/^<(scheme:[^\s\x00-\x1f<>]*|email)>/).replace("scheme",/[a-zA-Z][a-zA-Z0-9+.-]{1,31}/).replace("email",/[a-zA-Z0-9.!#$%&'*+/=?^_`{|}~-]+(@)[a-zA-Z0-9](?:[a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?(?:\.[a-zA-Z0-9](?:[a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?)+(?![-_])/).getRegex(),s0=G(ha).replace("(?:-->|$)","-->").getRegex(),i0=G("^comment|^</[a-zA-Z][\\w:-]*\\s*>|^<[a-zA-Z][\\w-]*(?:attribute)*?\\s*/?>|^<\\?[\\s\\S]*?\\?>|^<![a-zA-Z]+\\s[\\s\\S]*?>|^<!\\[CDATA\\[[\\s\\S]*?\\]\\]>").replace("comment",s0).replace("attribute",/\s+[a-zA-Z:_][\w.:-]*(?:\s*=\s*"[^"]*"|\s*=\s*'[^']*'|\s*=\s*[^\s"'=<>`]+)?/).getRegex(),ts=/(?:\[(?:\\[\s\S]|[^\[\]\\])*\]|\\[\s\S]|`+[^`]*?`+(?!`)|[^\[\]\\`])*?/,a0=G(/^!?\[(label)\]\(\s*(href)(?:(?:[ \t]*(?:\n[ \t]*)?)(title))?\s*\)/).replace("label",ts).replace("href",/<(?:\\.|[^\n<>\\])+>|[^ \t\n\x00-\x1f]*/).replace("title",/"(?:\\"?|[^"\\])*"|'(?:\\'?|[^'\\])*'|\((?:\\\)?|[^)\\])*\)/).getRegex(),rc=G(/^!?\[(label)\]\[(ref)\]/).replace("label",ts).replace("ref",pa).getRegex(),lc=G(/^!?\[(ref)\](?:\[\])?/).replace("ref",pa).getRegex(),o0=G("reflink|nolink(?!\\()","g").replace("reflink",rc).replace("nolink",lc).getRegex(),Yo=/[hH][tT][tT][pP][sS]?|[fF][tT][pP]/,ma={_backpedal:vt,anyPunctuation:t0,autolink:n0,blockSkip:qy,br:tc,code:By,del:vt,delLDelim:vt,delRDelim:vt,emStrongLDelim:Gy,emStrongRDelimAst:Qy,emStrongRDelimUnd:Jy,escape:Oy,link:a0,nolink:lc,punctuation:zy,reflink:rc,reflinkSearch:o0,tag:i0,text:Uy,url:vt},r0={...ma,link:G(/^!?\[(label)\]\((.*?)\)/).replace("label",ts).getRegex(),reflink:G(/^!?\[(label)\]\s*\[([^\]]*)\]/).replace("label",ts).getRegex()},ki={...ma,emStrongRDelimAst:Yy,emStrongLDelim:Vy,delLDelim:Zy,delRDelim:e0,url:G(/^((?:protocol):\/\/|www\.)(?:[a-zA-Z0-9\-]+\.?)+[^\s<]*|^email/).replace("protocol",Yo).replace("email",/[A-Za-z0-9._+-]+(@)[a-zA-Z0-9-_]+(?:\.[a-zA-Z0-9-_]*[a-zA-Z0-9])+(?![-_])/).getRegex(),_backpedal:/(?:[^?!.,:;*_'"~()&]+|\([^)]*\)|&(?![a-zA-Z0-9]+;$)|[?!.,:;*_'"~)]+(?!$))+/,del:/^(~~?)(?=[^\s~])((?:\\[\s\S]|[^\\])*?(?:\\[\s\S]|[^\s~\\]))\1(?=[^~]|$)/,text:G(/^([`~]+|[^`~])(?:(?= {2,}\n)|(?=[a-zA-Z0-9.!#$%&'*+\/=?_`{\|}~-]+@)|[\s\S]*?(?:(?=[\\<!\[`*~_]|\b_|protocol:\/\/|www\.|$)|[^ ](?= {2,}\n)|[^a-zA-Z0-9.!#$%&'*+\/=?_`{\|}~-](?=[a-zA-Z0-9.!#$%&'*+\/=?_`{\|}~-]+@)))/).replace("protocol",Yo).getRegex()},l0={...ki,br:G(tc).replace("{2,}","*").getRegex(),text:G(ki.text).replace("\\b_","\\b_| {2,}\\n").replace(/\{2,\}/g,"*").getRegex()},On={normal:fa,gfm:Fy,pedantic:Ny},Vt={normal:ma,gfm:ki,breaks:l0,pedantic:r0},c0={"&":"&amp;","<":"&lt;",">":"&gt;",'"':"&quot;","'":"&#39;"},Jo=e=>c0[e];function Ne(e,t){if(t){if(ve.escapeTest.test(e))return e.replace(ve.escapeReplace,Jo)}else if(ve.escapeTestNoEncode.test(e))return e.replace(ve.escapeReplaceNoEncode,Jo);return e}function Zo(e){try{e=encodeURI(e).replace(ve.percentDecode,"%")}catch{return null}return e}function Xo(e,t){let n=e.replace(ve.findPipe,(a,o,l)=>{let d=!1,g=o;for(;--g>=0&&l[g]==="\\";)d=!d;return d?"|":" |"}),s=n.split(ve.splitPipe),i=0;if(s[0].trim()||s.shift(),s.length>0&&!s.at(-1)?.trim()&&s.pop(),t)if(s.length>t)s.splice(t);else for(;s.length<t;)s.push("");for(;i<s.length;i++)s[i]=s[i].trim().replace(ve.slashPipe,"|");return s}function Qt(e,t,n){let s=e.length;if(s===0)return"";let i=0;for(;i<s&&e.charAt(s-i-1)===t;)i++;return e.slice(0,s-i)}function d0(e,t){if(e.indexOf(t[1])===-1)return-1;let n=0;for(let s=0;s<e.length;s++)if(e[s]==="\\")s++;else if(e[s]===t[0])n++;else if(e[s]===t[1]&&(n--,n<0))return s;return n>0?-2:-1}function u0(e,t=0){let n=t,s="";for(let i of e)if(i==="	"){let a=4-n%4;s+=" ".repeat(a),n+=a}else s+=i,n++;return s}function er(e,t,n,s,i){let a=t.href,o=t.title||null,l=e[1].replace(i.other.outputLinkReplace,"$1");s.state.inLink=!0;let d={type:e[0].charAt(0)==="!"?"image":"link",raw:n,href:a,title:o,text:l,tokens:s.inlineTokens(l)};return s.state.inLink=!1,d}function g0(e,t,n){let s=e.match(n.other.indentCodeCompensation);if(s===null)return t;let i=s[1];return t.split(`
`).map(a=>{let o=a.match(n.other.beginningSpace);if(o===null)return a;let[l]=o;return l.length>=i.length?a.slice(i.length):a}).join(`
`)}var ns=class{options;rules;lexer;constructor(e){this.options=e||Ct}space(e){let t=this.rules.block.newline.exec(e);if(t&&t[0].length>0)return{type:"space",raw:t[0]}}code(e){let t=this.rules.block.code.exec(e);if(t){let n=t[0].replace(this.rules.other.codeRemoveIndent,"");return{type:"code",raw:t[0],codeBlockStyle:"indented",text:this.options.pedantic?n:Qt(n,`
`)}}}fences(e){let t=this.rules.block.fences.exec(e);if(t){let n=t[0],s=g0(n,t[3]||"",this.rules);return{type:"code",raw:n,lang:t[2]?t[2].trim().replace(this.rules.inline.anyPunctuation,"$1"):t[2],text:s}}}heading(e){let t=this.rules.block.heading.exec(e);if(t){let n=t[2].trim();if(this.rules.other.endingHash.test(n)){let s=Qt(n,"#");(this.options.pedantic||!s||this.rules.other.endingSpaceChar.test(s))&&(n=s.trim())}return{type:"heading",raw:t[0],depth:t[1].length,text:n,tokens:this.lexer.inline(n)}}}hr(e){let t=this.rules.block.hr.exec(e);if(t)return{type:"hr",raw:Qt(t[0],`
`)}}blockquote(e){let t=this.rules.block.blockquote.exec(e);if(t){let n=Qt(t[0],`
`).split(`
`),s="",i="",a=[];for(;n.length>0;){let o=!1,l=[],d;for(d=0;d<n.length;d++)if(this.rules.other.blockquoteStart.test(n[d]))l.push(n[d]),o=!0;else if(!o)l.push(n[d]);else break;n=n.slice(d);let g=l.join(`
`),f=g.replace(this.rules.other.blockquoteSetextReplace,`
    $1`).replace(this.rules.other.blockquoteSetextReplace2,"");s=s?`${s}
${g}`:g,i=i?`${i}
${f}`:f;let p=this.lexer.state.top;if(this.lexer.state.top=!0,this.lexer.blockTokens(f,a,!0),this.lexer.state.top=p,n.length===0)break;let b=a.at(-1);if(b?.type==="code")break;if(b?.type==="blockquote"){let u=b,v=u.raw+`
`+n.join(`
`),y=this.blockquote(v);a[a.length-1]=y,s=s.substring(0,s.length-u.raw.length)+y.raw,i=i.substring(0,i.length-u.text.length)+y.text;break}else if(b?.type==="list"){let u=b,v=u.raw+`
`+n.join(`
`),y=this.list(v);a[a.length-1]=y,s=s.substring(0,s.length-b.raw.length)+y.raw,i=i.substring(0,i.length-u.raw.length)+y.raw,n=v.substring(a.at(-1).raw.length).split(`
`);continue}}return{type:"blockquote",raw:s,tokens:a,text:i}}}list(e){let t=this.rules.block.list.exec(e);if(t){let n=t[1].trim(),s=n.length>1,i={type:"list",raw:"",ordered:s,start:s?+n.slice(0,-1):"",loose:!1,items:[]};n=s?`\\d{1,9}\\${n.slice(-1)}`:`\\${n}`,this.options.pedantic&&(n=s?n:"[*+-]");let a=this.rules.other.listItemRegex(n),o=!1;for(;e;){let d=!1,g="",f="";if(!(t=a.exec(e))||this.rules.block.hr.test(e))break;g=t[0],e=e.substring(g.length);let p=u0(t[2].split(`
`,1)[0],t[1].length),b=e.split(`
`,1)[0],u=!p.trim(),v=0;if(this.options.pedantic?(v=2,f=p.trimStart()):u?v=t[1].length+1:(v=p.search(this.rules.other.nonSpaceChar),v=v>4?1:v,f=p.slice(v),v+=t[1].length),u&&this.rules.other.blankLine.test(b)&&(g+=b+`
`,e=e.substring(b.length+1),d=!0),!d){let y=this.rules.other.nextBulletRegex(v),k=this.rules.other.hrRegex(v),C=this.rules.other.fencesBeginRegex(v),$=this.rules.other.headingBeginRegex(v),T=this.rules.other.htmlBeginRegex(v),_=this.rules.other.blockquoteBeginRegex(v);for(;e;){let L=e.split(`
`,1)[0],E;if(b=L,this.options.pedantic?(b=b.replace(this.rules.other.listReplaceNesting,"  "),E=b):E=b.replace(this.rules.other.tabCharGlobal,"    "),C.test(b)||$.test(b)||T.test(b)||_.test(b)||y.test(b)||k.test(b))break;if(E.search(this.rules.other.nonSpaceChar)>=v||!b.trim())f+=`
`+E.slice(v);else{if(u||p.replace(this.rules.other.tabCharGlobal,"    ").search(this.rules.other.nonSpaceChar)>=4||C.test(p)||$.test(p)||k.test(p))break;f+=`
`+b}u=!b.trim(),g+=L+`
`,e=e.substring(L.length+1),p=E.slice(v)}}i.loose||(o?i.loose=!0:this.rules.other.doubleBlankLine.test(g)&&(o=!0)),i.items.push({type:"list_item",raw:g,task:!!this.options.gfm&&this.rules.other.listIsTask.test(f),loose:!1,text:f,tokens:[]}),i.raw+=g}let l=i.items.at(-1);if(l)l.raw=l.raw.trimEnd(),l.text=l.text.trimEnd();else return;i.raw=i.raw.trimEnd();for(let d of i.items){if(this.lexer.state.top=!1,d.tokens=this.lexer.blockTokens(d.text,[]),d.task){if(d.text=d.text.replace(this.rules.other.listReplaceTask,""),d.tokens[0]?.type==="text"||d.tokens[0]?.type==="paragraph"){d.tokens[0].raw=d.tokens[0].raw.replace(this.rules.other.listReplaceTask,""),d.tokens[0].text=d.tokens[0].text.replace(this.rules.other.listReplaceTask,"");for(let f=this.lexer.inlineQueue.length-1;f>=0;f--)if(this.rules.other.listIsTask.test(this.lexer.inlineQueue[f].src)){this.lexer.inlineQueue[f].src=this.lexer.inlineQueue[f].src.replace(this.rules.other.listReplaceTask,"");break}}let g=this.rules.other.listTaskCheckbox.exec(d.raw);if(g){let f={type:"checkbox",raw:g[0]+" ",checked:g[0]!=="[ ]"};d.checked=f.checked,i.loose?d.tokens[0]&&["paragraph","text"].includes(d.tokens[0].type)&&"tokens"in d.tokens[0]&&d.tokens[0].tokens?(d.tokens[0].raw=f.raw+d.tokens[0].raw,d.tokens[0].text=f.raw+d.tokens[0].text,d.tokens[0].tokens.unshift(f)):d.tokens.unshift({type:"paragraph",raw:f.raw,text:f.raw,tokens:[f]}):d.tokens.unshift(f)}}if(!i.loose){let g=d.tokens.filter(p=>p.type==="space"),f=g.length>0&&g.some(p=>this.rules.other.anyLine.test(p.raw));i.loose=f}}if(i.loose)for(let d of i.items){d.loose=!0;for(let g of d.tokens)g.type==="text"&&(g.type="paragraph")}return i}}html(e){let t=this.rules.block.html.exec(e);if(t)return{type:"html",block:!0,raw:t[0],pre:t[1]==="pre"||t[1]==="script"||t[1]==="style",text:t[0]}}def(e){let t=this.rules.block.def.exec(e);if(t){let n=t[1].toLowerCase().replace(this.rules.other.multipleSpaceGlobal," "),s=t[2]?t[2].replace(this.rules.other.hrefBrackets,"$1").replace(this.rules.inline.anyPunctuation,"$1"):"",i=t[3]?t[3].substring(1,t[3].length-1).replace(this.rules.inline.anyPunctuation,"$1"):t[3];return{type:"def",tag:n,raw:t[0],href:s,title:i}}}table(e){let t=this.rules.block.table.exec(e);if(!t||!this.rules.other.tableDelimiter.test(t[2]))return;let n=Xo(t[1]),s=t[2].replace(this.rules.other.tableAlignChars,"").split("|"),i=t[3]?.trim()?t[3].replace(this.rules.other.tableRowBlankLine,"").split(`
`):[],a={type:"table",raw:t[0],header:[],align:[],rows:[]};if(n.length===s.length){for(let o of s)this.rules.other.tableAlignRight.test(o)?a.align.push("right"):this.rules.other.tableAlignCenter.test(o)?a.align.push("center"):this.rules.other.tableAlignLeft.test(o)?a.align.push("left"):a.align.push(null);for(let o=0;o<n.length;o++)a.header.push({text:n[o],tokens:this.lexer.inline(n[o]),header:!0,align:a.align[o]});for(let o of i)a.rows.push(Xo(o,a.header.length).map((l,d)=>({text:l,tokens:this.lexer.inline(l),header:!1,align:a.align[d]})));return a}}lheading(e){let t=this.rules.block.lheading.exec(e);if(t)return{type:"heading",raw:t[0],depth:t[2].charAt(0)==="="?1:2,text:t[1],tokens:this.lexer.inline(t[1])}}paragraph(e){let t=this.rules.block.paragraph.exec(e);if(t){let n=t[1].charAt(t[1].length-1)===`
`?t[1].slice(0,-1):t[1];return{type:"paragraph",raw:t[0],text:n,tokens:this.lexer.inline(n)}}}text(e){let t=this.rules.block.text.exec(e);if(t)return{type:"text",raw:t[0],text:t[0],tokens:this.lexer.inline(t[0])}}escape(e){let t=this.rules.inline.escape.exec(e);if(t)return{type:"escape",raw:t[0],text:t[1]}}tag(e){let t=this.rules.inline.tag.exec(e);if(t)return!this.lexer.state.inLink&&this.rules.other.startATag.test(t[0])?this.lexer.state.inLink=!0:this.lexer.state.inLink&&this.rules.other.endATag.test(t[0])&&(this.lexer.state.inLink=!1),!this.lexer.state.inRawBlock&&this.rules.other.startPreScriptTag.test(t[0])?this.lexer.state.inRawBlock=!0:this.lexer.state.inRawBlock&&this.rules.other.endPreScriptTag.test(t[0])&&(this.lexer.state.inRawBlock=!1),{type:"html",raw:t[0],inLink:this.lexer.state.inLink,inRawBlock:this.lexer.state.inRawBlock,block:!1,text:t[0]}}link(e){let t=this.rules.inline.link.exec(e);if(t){let n=t[2].trim();if(!this.options.pedantic&&this.rules.other.startAngleBracket.test(n)){if(!this.rules.other.endAngleBracket.test(n))return;let a=Qt(n.slice(0,-1),"\\");if((n.length-a.length)%2===0)return}else{let a=d0(t[2],"()");if(a===-2)return;if(a>-1){let o=(t[0].indexOf("!")===0?5:4)+t[1].length+a;t[2]=t[2].substring(0,a),t[0]=t[0].substring(0,o).trim(),t[3]=""}}let s=t[2],i="";if(this.options.pedantic){let a=this.rules.other.pedanticHrefTitle.exec(s);a&&(s=a[1],i=a[3])}else i=t[3]?t[3].slice(1,-1):"";return s=s.trim(),this.rules.other.startAngleBracket.test(s)&&(this.options.pedantic&&!this.rules.other.endAngleBracket.test(n)?s=s.slice(1):s=s.slice(1,-1)),er(t,{href:s&&s.replace(this.rules.inline.anyPunctuation,"$1"),title:i&&i.replace(this.rules.inline.anyPunctuation,"$1")},t[0],this.lexer,this.rules)}}reflink(e,t){let n;if((n=this.rules.inline.reflink.exec(e))||(n=this.rules.inline.nolink.exec(e))){let s=(n[2]||n[1]).replace(this.rules.other.multipleSpaceGlobal," "),i=t[s.toLowerCase()];if(!i){let a=n[0].charAt(0);return{type:"text",raw:a,text:a}}return er(n,i,n[0],this.lexer,this.rules)}}emStrong(e,t,n=""){let s=this.rules.inline.emStrongLDelim.exec(e);if(!(!s||s[3]&&n.match(this.rules.other.unicodeAlphaNumeric))&&(!(s[1]||s[2])||!n||this.rules.inline.punctuation.exec(n))){let i=[...s[0]].length-1,a,o,l=i,d=0,g=s[0][0]==="*"?this.rules.inline.emStrongRDelimAst:this.rules.inline.emStrongRDelimUnd;for(g.lastIndex=0,t=t.slice(-1*e.length+i);(s=g.exec(t))!=null;){if(a=s[1]||s[2]||s[3]||s[4]||s[5]||s[6],!a)continue;if(o=[...a].length,s[3]||s[4]){l+=o;continue}else if((s[5]||s[6])&&i%3&&!((i+o)%3)){d+=o;continue}if(l-=o,l>0)continue;o=Math.min(o,o+l+d);let f=[...s[0]][0].length,p=e.slice(0,i+s.index+f+o);if(Math.min(i,o)%2){let u=p.slice(1,-1);return{type:"em",raw:p,text:u,tokens:this.lexer.inlineTokens(u)}}let b=p.slice(2,-2);return{type:"strong",raw:p,text:b,tokens:this.lexer.inlineTokens(b)}}}}codespan(e){let t=this.rules.inline.code.exec(e);if(t){let n=t[2].replace(this.rules.other.newLineCharGlobal," "),s=this.rules.other.nonSpaceChar.test(n),i=this.rules.other.startingSpaceChar.test(n)&&this.rules.other.endingSpaceChar.test(n);return s&&i&&(n=n.substring(1,n.length-1)),{type:"codespan",raw:t[0],text:n}}}br(e){let t=this.rules.inline.br.exec(e);if(t)return{type:"br",raw:t[0]}}del(e,t,n=""){let s=this.rules.inline.delLDelim.exec(e);if(s&&(!s[1]||!n||this.rules.inline.punctuation.exec(n))){let i=[...s[0]].length-1,a,o,l=i,d=this.rules.inline.delRDelim;for(d.lastIndex=0,t=t.slice(-1*e.length+i);(s=d.exec(t))!=null;){if(a=s[1]||s[2]||s[3]||s[4]||s[5]||s[6],!a||(o=[...a].length,o!==i))continue;if(s[3]||s[4]){l+=o;continue}if(l-=o,l>0)continue;o=Math.min(o,o+l);let g=[...s[0]][0].length,f=e.slice(0,i+s.index+g+o),p=f.slice(i,-i);return{type:"del",raw:f,text:p,tokens:this.lexer.inlineTokens(p)}}}}autolink(e){let t=this.rules.inline.autolink.exec(e);if(t){let n,s;return t[2]==="@"?(n=t[1],s="mailto:"+n):(n=t[1],s=n),{type:"link",raw:t[0],text:n,href:s,tokens:[{type:"text",raw:n,text:n}]}}}url(e){let t;if(t=this.rules.inline.url.exec(e)){let n,s;if(t[2]==="@")n=t[0],s="mailto:"+n;else{let i;do i=t[0],t[0]=this.rules.inline._backpedal.exec(t[0])?.[0]??"";while(i!==t[0]);n=t[0],t[1]==="www."?s="http://"+t[0]:s=t[0]}return{type:"link",raw:t[0],text:n,href:s,tokens:[{type:"text",raw:n,text:n}]}}}inlineText(e){let t=this.rules.inline.text.exec(e);if(t){let n=this.lexer.state.inRawBlock;return{type:"text",raw:t[0],text:t[0],escaped:n}}}},Le=class Si{tokens;options;state;inlineQueue;tokenizer;constructor(t){this.tokens=[],this.tokens.links=Object.create(null),this.options=t||Ct,this.options.tokenizer=this.options.tokenizer||new ns,this.tokenizer=this.options.tokenizer,this.tokenizer.options=this.options,this.tokenizer.lexer=this,this.inlineQueue=[],this.state={inLink:!1,inRawBlock:!1,top:!0};let n={other:ve,block:On.normal,inline:Vt.normal};this.options.pedantic?(n.block=On.pedantic,n.inline=Vt.pedantic):this.options.gfm&&(n.block=On.gfm,this.options.breaks?n.inline=Vt.breaks:n.inline=Vt.gfm),this.tokenizer.rules=n}static get rules(){return{block:On,inline:Vt}}static lex(t,n){return new Si(n).lex(t)}static lexInline(t,n){return new Si(n).inlineTokens(t)}lex(t){t=t.replace(ve.carriageReturn,`
`),this.blockTokens(t,this.tokens);for(let n=0;n<this.inlineQueue.length;n++){let s=this.inlineQueue[n];this.inlineTokens(s.src,s.tokens)}return this.inlineQueue=[],this.tokens}blockTokens(t,n=[],s=!1){for(this.options.pedantic&&(t=t.replace(ve.tabCharGlobal,"    ").replace(ve.spaceLine,""));t;){let i;if(this.options.extensions?.block?.some(o=>(i=o.call({lexer:this},t,n))?(t=t.substring(i.raw.length),n.push(i),!0):!1))continue;if(i=this.tokenizer.space(t)){t=t.substring(i.raw.length);let o=n.at(-1);i.raw.length===1&&o!==void 0?o.raw+=`
`:n.push(i);continue}if(i=this.tokenizer.code(t)){t=t.substring(i.raw.length);let o=n.at(-1);o?.type==="paragraph"||o?.type==="text"?(o.raw+=(o.raw.endsWith(`
`)?"":`
`)+i.raw,o.text+=`
`+i.text,this.inlineQueue.at(-1).src=o.text):n.push(i);continue}if(i=this.tokenizer.fences(t)){t=t.substring(i.raw.length),n.push(i);continue}if(i=this.tokenizer.heading(t)){t=t.substring(i.raw.length),n.push(i);continue}if(i=this.tokenizer.hr(t)){t=t.substring(i.raw.length),n.push(i);continue}if(i=this.tokenizer.blockquote(t)){t=t.substring(i.raw.length),n.push(i);continue}if(i=this.tokenizer.list(t)){t=t.substring(i.raw.length),n.push(i);continue}if(i=this.tokenizer.html(t)){t=t.substring(i.raw.length),n.push(i);continue}if(i=this.tokenizer.def(t)){t=t.substring(i.raw.length);let o=n.at(-1);o?.type==="paragraph"||o?.type==="text"?(o.raw+=(o.raw.endsWith(`
`)?"":`
`)+i.raw,o.text+=`
`+i.raw,this.inlineQueue.at(-1).src=o.text):this.tokens.links[i.tag]||(this.tokens.links[i.tag]={href:i.href,title:i.title},n.push(i));continue}if(i=this.tokenizer.table(t)){t=t.substring(i.raw.length),n.push(i);continue}if(i=this.tokenizer.lheading(t)){t=t.substring(i.raw.length),n.push(i);continue}let a=t;if(this.options.extensions?.startBlock){let o=1/0,l=t.slice(1),d;this.options.extensions.startBlock.forEach(g=>{d=g.call({lexer:this},l),typeof d=="number"&&d>=0&&(o=Math.min(o,d))}),o<1/0&&o>=0&&(a=t.substring(0,o+1))}if(this.state.top&&(i=this.tokenizer.paragraph(a))){let o=n.at(-1);s&&o?.type==="paragraph"?(o.raw+=(o.raw.endsWith(`
`)?"":`
`)+i.raw,o.text+=`
`+i.text,this.inlineQueue.pop(),this.inlineQueue.at(-1).src=o.text):n.push(i),s=a.length!==t.length,t=t.substring(i.raw.length);continue}if(i=this.tokenizer.text(t)){t=t.substring(i.raw.length);let o=n.at(-1);o?.type==="text"?(o.raw+=(o.raw.endsWith(`
`)?"":`
`)+i.raw,o.text+=`
`+i.text,this.inlineQueue.pop(),this.inlineQueue.at(-1).src=o.text):n.push(i);continue}if(t){let o="Infinite loop on byte: "+t.charCodeAt(0);if(this.options.silent){console.error(o);break}else throw new Error(o)}}return this.state.top=!0,n}inline(t,n=[]){return this.inlineQueue.push({src:t,tokens:n}),n}inlineTokens(t,n=[]){let s=t,i=null;if(this.tokens.links){let d=Object.keys(this.tokens.links);if(d.length>0)for(;(i=this.tokenizer.rules.inline.reflinkSearch.exec(s))!=null;)d.includes(i[0].slice(i[0].lastIndexOf("[")+1,-1))&&(s=s.slice(0,i.index)+"["+"a".repeat(i[0].length-2)+"]"+s.slice(this.tokenizer.rules.inline.reflinkSearch.lastIndex))}for(;(i=this.tokenizer.rules.inline.anyPunctuation.exec(s))!=null;)s=s.slice(0,i.index)+"++"+s.slice(this.tokenizer.rules.inline.anyPunctuation.lastIndex);let a;for(;(i=this.tokenizer.rules.inline.blockSkip.exec(s))!=null;)a=i[2]?i[2].length:0,s=s.slice(0,i.index+a)+"["+"a".repeat(i[0].length-a-2)+"]"+s.slice(this.tokenizer.rules.inline.blockSkip.lastIndex);s=this.options.hooks?.emStrongMask?.call({lexer:this},s)??s;let o=!1,l="";for(;t;){o||(l=""),o=!1;let d;if(this.options.extensions?.inline?.some(f=>(d=f.call({lexer:this},t,n))?(t=t.substring(d.raw.length),n.push(d),!0):!1))continue;if(d=this.tokenizer.escape(t)){t=t.substring(d.raw.length),n.push(d);continue}if(d=this.tokenizer.tag(t)){t=t.substring(d.raw.length),n.push(d);continue}if(d=this.tokenizer.link(t)){t=t.substring(d.raw.length),n.push(d);continue}if(d=this.tokenizer.reflink(t,this.tokens.links)){t=t.substring(d.raw.length);let f=n.at(-1);d.type==="text"&&f?.type==="text"?(f.raw+=d.raw,f.text+=d.text):n.push(d);continue}if(d=this.tokenizer.emStrong(t,s,l)){t=t.substring(d.raw.length),n.push(d);continue}if(d=this.tokenizer.codespan(t)){t=t.substring(d.raw.length),n.push(d);continue}if(d=this.tokenizer.br(t)){t=t.substring(d.raw.length),n.push(d);continue}if(d=this.tokenizer.del(t,s,l)){t=t.substring(d.raw.length),n.push(d);continue}if(d=this.tokenizer.autolink(t)){t=t.substring(d.raw.length),n.push(d);continue}if(!this.state.inLink&&(d=this.tokenizer.url(t))){t=t.substring(d.raw.length),n.push(d);continue}let g=t;if(this.options.extensions?.startInline){let f=1/0,p=t.slice(1),b;this.options.extensions.startInline.forEach(u=>{b=u.call({lexer:this},p),typeof b=="number"&&b>=0&&(f=Math.min(f,b))}),f<1/0&&f>=0&&(g=t.substring(0,f+1))}if(d=this.tokenizer.inlineText(g)){t=t.substring(d.raw.length),d.raw.slice(-1)!=="_"&&(l=d.raw.slice(-1)),o=!0;let f=n.at(-1);f?.type==="text"?(f.raw+=d.raw,f.text+=d.text):n.push(d);continue}if(t){let f="Infinite loop on byte: "+t.charCodeAt(0);if(this.options.silent){console.error(f);break}else throw new Error(f)}}return n}},ss=class{options;parser;constructor(e){this.options=e||Ct}space(e){return""}code({text:e,lang:t,escaped:n}){let s=(t||"").match(ve.notSpaceStart)?.[0],i=e.replace(ve.endingNewline,"")+`
`;return s?'<pre><code class="language-'+Ne(s)+'">'+(n?i:Ne(i,!0))+`</code></pre>
`:"<pre><code>"+(n?i:Ne(i,!0))+`</code></pre>
`}blockquote({tokens:e}){return`<blockquote>
${this.parser.parse(e)}</blockquote>
`}html({text:e}){return e}def(e){return""}heading({tokens:e,depth:t}){return`<h${t}>${this.parser.parseInline(e)}</h${t}>
`}hr(e){return`<hr>
`}list(e){let t=e.ordered,n=e.start,s="";for(let o=0;o<e.items.length;o++){let l=e.items[o];s+=this.listitem(l)}let i=t?"ol":"ul",a=t&&n!==1?' start="'+n+'"':"";return"<"+i+a+`>
`+s+"</"+i+`>
`}listitem(e){return`<li>${this.parser.parse(e.tokens)}</li>
`}checkbox({checked:e}){return"<input "+(e?'checked="" ':"")+'disabled="" type="checkbox"> '}paragraph({tokens:e}){return`<p>${this.parser.parseInline(e)}</p>
`}table(e){let t="",n="";for(let i=0;i<e.header.length;i++)n+=this.tablecell(e.header[i]);t+=this.tablerow({text:n});let s="";for(let i=0;i<e.rows.length;i++){let a=e.rows[i];n="";for(let o=0;o<a.length;o++)n+=this.tablecell(a[o]);s+=this.tablerow({text:n})}return s&&(s=`<tbody>${s}</tbody>`),`<table>
<thead>
`+t+`</thead>
`+s+`</table>
`}tablerow({text:e}){return`<tr>
${e}</tr>
`}tablecell(e){let t=this.parser.parseInline(e.tokens),n=e.header?"th":"td";return(e.align?`<${n} align="${e.align}">`:`<${n}>`)+t+`</${n}>
`}strong({tokens:e}){return`<strong>${this.parser.parseInline(e)}</strong>`}em({tokens:e}){return`<em>${this.parser.parseInline(e)}</em>`}codespan({text:e}){return`<code>${Ne(e,!0)}</code>`}br(e){return"<br>"}del({tokens:e}){return`<del>${this.parser.parseInline(e)}</del>`}link({href:e,title:t,tokens:n}){let s=this.parser.parseInline(n),i=Zo(e);if(i===null)return s;e=i;let a='<a href="'+e+'"';return t&&(a+=' title="'+Ne(t)+'"'),a+=">"+s+"</a>",a}image({href:e,title:t,text:n,tokens:s}){s&&(n=this.parser.parseInline(s,this.parser.textRenderer));let i=Zo(e);if(i===null)return Ne(n);e=i;let a=`<img src="${e}" alt="${Ne(n)}"`;return t&&(a+=` title="${Ne(t)}"`),a+=">",a}text(e){return"tokens"in e&&e.tokens?this.parser.parseInline(e.tokens):"escaped"in e&&e.escaped?e.text:Ne(e.text)}},ba=class{strong({text:e}){return e}em({text:e}){return e}codespan({text:e}){return e}del({text:e}){return e}html({text:e}){return e}text({text:e}){return e}link({text:e}){return""+e}image({text:e}){return""+e}br(){return""}checkbox({raw:e}){return e}},Me=class Ai{options;renderer;textRenderer;constructor(t){this.options=t||Ct,this.options.renderer=this.options.renderer||new ss,this.renderer=this.options.renderer,this.renderer.options=this.options,this.renderer.parser=this,this.textRenderer=new ba}static parse(t,n){return new Ai(n).parse(t)}static parseInline(t,n){return new Ai(n).parseInline(t)}parse(t){let n="";for(let s=0;s<t.length;s++){let i=t[s];if(this.options.extensions?.renderers?.[i.type]){let o=i,l=this.options.extensions.renderers[o.type].call({parser:this},o);if(l!==!1||!["space","hr","heading","code","table","blockquote","list","html","def","paragraph","text"].includes(o.type)){n+=l||"";continue}}let a=i;switch(a.type){case"space":{n+=this.renderer.space(a);break}case"hr":{n+=this.renderer.hr(a);break}case"heading":{n+=this.renderer.heading(a);break}case"code":{n+=this.renderer.code(a);break}case"table":{n+=this.renderer.table(a);break}case"blockquote":{n+=this.renderer.blockquote(a);break}case"list":{n+=this.renderer.list(a);break}case"checkbox":{n+=this.renderer.checkbox(a);break}case"html":{n+=this.renderer.html(a);break}case"def":{n+=this.renderer.def(a);break}case"paragraph":{n+=this.renderer.paragraph(a);break}case"text":{n+=this.renderer.text(a);break}default:{let o='Token with "'+a.type+'" type was not found.';if(this.options.silent)return console.error(o),"";throw new Error(o)}}}return n}parseInline(t,n=this.renderer){let s="";for(let i=0;i<t.length;i++){let a=t[i];if(this.options.extensions?.renderers?.[a.type]){let l=this.options.extensions.renderers[a.type].call({parser:this},a);if(l!==!1||!["escape","html","link","image","strong","em","codespan","br","del","text"].includes(a.type)){s+=l||"";continue}}let o=a;switch(o.type){case"escape":{s+=n.text(o);break}case"html":{s+=n.html(o);break}case"link":{s+=n.link(o);break}case"image":{s+=n.image(o);break}case"checkbox":{s+=n.checkbox(o);break}case"strong":{s+=n.strong(o);break}case"em":{s+=n.em(o);break}case"codespan":{s+=n.codespan(o);break}case"br":{s+=n.br(o);break}case"del":{s+=n.del(o);break}case"text":{s+=n.text(o);break}default:{let l='Token with "'+o.type+'" type was not found.';if(this.options.silent)return console.error(l),"";throw new Error(l)}}}return s}},Jt=class{options;block;constructor(e){this.options=e||Ct}static passThroughHooks=new Set(["preprocess","postprocess","processAllTokens","emStrongMask"]);static passThroughHooksRespectAsync=new Set(["preprocess","postprocess","processAllTokens"]);preprocess(e){return e}postprocess(e){return e}processAllTokens(e){return e}emStrongMask(e){return e}provideLexer(){return this.block?Le.lex:Le.lexInline}provideParser(){return this.block?Me.parse:Me.parseInline}},p0=class{defaults=da();options=this.setOptions;parse=this.parseMarkdown(!0);parseInline=this.parseMarkdown(!1);Parser=Me;Renderer=ss;TextRenderer=ba;Lexer=Le;Tokenizer=ns;Hooks=Jt;constructor(...e){this.use(...e)}walkTokens(e,t){let n=[];for(let s of e)switch(n=n.concat(t.call(this,s)),s.type){case"table":{let i=s;for(let a of i.header)n=n.concat(this.walkTokens(a.tokens,t));for(let a of i.rows)for(let o of a)n=n.concat(this.walkTokens(o.tokens,t));break}case"list":{let i=s;n=n.concat(this.walkTokens(i.items,t));break}default:{let i=s;this.defaults.extensions?.childTokens?.[i.type]?this.defaults.extensions.childTokens[i.type].forEach(a=>{let o=i[a].flat(1/0);n=n.concat(this.walkTokens(o,t))}):i.tokens&&(n=n.concat(this.walkTokens(i.tokens,t)))}}return n}use(...e){let t=this.defaults.extensions||{renderers:{},childTokens:{}};return e.forEach(n=>{let s={...n};if(s.async=this.defaults.async||s.async||!1,n.extensions&&(n.extensions.forEach(i=>{if(!i.name)throw new Error("extension name required");if("renderer"in i){let a=t.renderers[i.name];a?t.renderers[i.name]=function(...o){let l=i.renderer.apply(this,o);return l===!1&&(l=a.apply(this,o)),l}:t.renderers[i.name]=i.renderer}if("tokenizer"in i){if(!i.level||i.level!=="block"&&i.level!=="inline")throw new Error("extension level must be 'block' or 'inline'");let a=t[i.level];a?a.unshift(i.tokenizer):t[i.level]=[i.tokenizer],i.start&&(i.level==="block"?t.startBlock?t.startBlock.push(i.start):t.startBlock=[i.start]:i.level==="inline"&&(t.startInline?t.startInline.push(i.start):t.startInline=[i.start]))}"childTokens"in i&&i.childTokens&&(t.childTokens[i.name]=i.childTokens)}),s.extensions=t),n.renderer){let i=this.defaults.renderer||new ss(this.defaults);for(let a in n.renderer){if(!(a in i))throw new Error(`renderer '${a}' does not exist`);if(["options","parser"].includes(a))continue;let o=a,l=n.renderer[o],d=i[o];i[o]=(...g)=>{let f=l.apply(i,g);return f===!1&&(f=d.apply(i,g)),f||""}}s.renderer=i}if(n.tokenizer){let i=this.defaults.tokenizer||new ns(this.defaults);for(let a in n.tokenizer){if(!(a in i))throw new Error(`tokenizer '${a}' does not exist`);if(["options","rules","lexer"].includes(a))continue;let o=a,l=n.tokenizer[o],d=i[o];i[o]=(...g)=>{let f=l.apply(i,g);return f===!1&&(f=d.apply(i,g)),f}}s.tokenizer=i}if(n.hooks){let i=this.defaults.hooks||new Jt;for(let a in n.hooks){if(!(a in i))throw new Error(`hook '${a}' does not exist`);if(["options","block"].includes(a))continue;let o=a,l=n.hooks[o],d=i[o];Jt.passThroughHooks.has(a)?i[o]=g=>{if(this.defaults.async&&Jt.passThroughHooksRespectAsync.has(a))return(async()=>{let p=await l.call(i,g);return d.call(i,p)})();let f=l.call(i,g);return d.call(i,f)}:i[o]=(...g)=>{if(this.defaults.async)return(async()=>{let p=await l.apply(i,g);return p===!1&&(p=await d.apply(i,g)),p})();let f=l.apply(i,g);return f===!1&&(f=d.apply(i,g)),f}}s.hooks=i}if(n.walkTokens){let i=this.defaults.walkTokens,a=n.walkTokens;s.walkTokens=function(o){let l=[];return l.push(a.call(this,o)),i&&(l=l.concat(i.call(this,o))),l}}this.defaults={...this.defaults,...s}}),this}setOptions(e){return this.defaults={...this.defaults,...e},this}lexer(e,t){return Le.lex(e,t??this.defaults)}parser(e,t){return Me.parse(e,t??this.defaults)}parseMarkdown(e){return(t,n)=>{let s={...n},i={...this.defaults,...s},a=this.onError(!!i.silent,!!i.async);if(this.defaults.async===!0&&s.async===!1)return a(new Error("marked(): The async option was set to true by an extension. Remove async: false from the parse options object to return a Promise."));if(typeof t>"u"||t===null)return a(new Error("marked(): input parameter is undefined or null"));if(typeof t!="string")return a(new Error("marked(): input parameter is of type "+Object.prototype.toString.call(t)+", string expected"));if(i.hooks&&(i.hooks.options=i,i.hooks.block=e),i.async)return(async()=>{let o=i.hooks?await i.hooks.preprocess(t):t,l=await(i.hooks?await i.hooks.provideLexer():e?Le.lex:Le.lexInline)(o,i),d=i.hooks?await i.hooks.processAllTokens(l):l;i.walkTokens&&await Promise.all(this.walkTokens(d,i.walkTokens));let g=await(i.hooks?await i.hooks.provideParser():e?Me.parse:Me.parseInline)(d,i);return i.hooks?await i.hooks.postprocess(g):g})().catch(a);try{i.hooks&&(t=i.hooks.preprocess(t));let o=(i.hooks?i.hooks.provideLexer():e?Le.lex:Le.lexInline)(t,i);i.hooks&&(o=i.hooks.processAllTokens(o)),i.walkTokens&&this.walkTokens(o,i.walkTokens);let l=(i.hooks?i.hooks.provideParser():e?Me.parse:Me.parseInline)(o,i);return i.hooks&&(l=i.hooks.postprocess(l)),l}catch(o){return a(o)}}}onError(e,t){return n=>{if(n.message+=`
Please report this to https://github.com/markedjs/marked.`,e){let s="<p>An error occurred:</p><pre>"+Ne(n.message+"",!0)+"</pre>";return t?Promise.resolve(s):s}if(t)return Promise.reject(n);throw n}}},St=new p0;function V(e,t){return St.parse(e,t)}V.options=V.setOptions=function(e){return St.setOptions(e),V.defaults=St.defaults,Jl(V.defaults),V};V.getDefaults=da;V.defaults=Ct;V.use=function(...e){return St.use(...e),V.defaults=St.defaults,Jl(V.defaults),V};V.walkTokens=function(e,t){return St.walkTokens(e,t)};V.parseInline=St.parseInline;V.Parser=Me;V.parser=Me.parse;V.Renderer=ss;V.TextRenderer=ba;V.Lexer=Le;V.lexer=Le.lex;V.Tokenizer=ns;V.Hooks=Jt;V.parse=V;V.options;V.setOptions;V.use;V.walkTokens;V.parseInline;Me.parse;Le.lex;V.setOptions({gfm:!0,breaks:!0});const h0=["a","b","blockquote","br","code","del","em","h1","h2","h3","h4","hr","i","li","ol","p","pre","strong","table","tbody","td","th","thead","tr","ul","img"],f0=["class","href","rel","target","title","start","src","alt"],tr={ALLOWED_TAGS:h0,ALLOWED_ATTR:f0,ADD_DATA_URI_TAGS:["img"]};let nr=!1;const v0=14e4,m0=4e4,b0=200,Xs=5e4,bt=new Map;function y0(e){const t=bt.get(e);return t===void 0?null:(bt.delete(e),bt.set(e,t),t)}function sr(e,t){if(bt.set(e,t),bt.size<=b0)return;const n=bt.keys().next().value;n&&bt.delete(n)}function x0(){nr||(nr=!0,wi.addHook("afterSanitizeAttributes",e=>{!(e instanceof HTMLAnchorElement)||!e.getAttribute("href")||(e.setAttribute("rel","noreferrer noopener"),e.setAttribute("target","_blank"))}))}function _i(e){const t=e.trim();if(!t)return"";if(x0(),t.length<=Xs){const o=y0(t);if(o!==null)return o}const n=Wr(t,v0),s=n.truncated?`

… truncated (${n.total} chars, showing first ${n.text.length}).`:"";if(n.text.length>m0){const l=`<pre class="code-block">${dc(`${n.text}${s}`)}</pre>`,d=wi.sanitize(l,tr);return t.length<=Xs&&sr(t,d),d}const i=V.parse(`${n.text}${s}`,{renderer:cc}),a=wi.sanitize(i,tr);return t.length<=Xs&&sr(t,a),a}const cc=new V.Renderer;cc.html=({text:e})=>dc(e);function dc(e){return e.replace(/&/g,"&amp;").replace(/</g,"&lt;").replace(/>/g,"&gt;").replace(/"/g,"&quot;").replace(/'/g,"&#39;")}const $0=new RegExp("\\p{Script=Hebrew}|\\p{Script=Arabic}|\\p{Script=Syriac}|\\p{Script=Thaana}|\\p{Script=Nko}|\\p{Script=Samaritan}|\\p{Script=Mandaic}|\\p{Script=Adlam}|\\p{Script=Phoenician}|\\p{Script=Lydian}","u");function uc(e,t=/[\s\p{P}\p{S}]/u){if(!e)return"ltr";for(const n of e)if(!t.test(n))return $0.test(n)?"rtl":"ltr";return"ltr"}const w0=1500,k0=2e3,gc="Copy as markdown",S0="Copied",A0="Copy failed";async function _0(e){if(!e)return!1;try{return await navigator.clipboard.writeText(e),!0}catch{return!1}}function Bn(e,t){e.title=t,e.setAttribute("aria-label",t)}function C0(e){const t=e.label??gc;return r`
    <button
      class="chat-copy-btn"
      type="button"
      title=${t}
      aria-label=${t}
      @click=${async n=>{const s=n.currentTarget;if(!s||s.dataset.copying==="1")return;s.dataset.copying="1",s.setAttribute("aria-busy","true"),s.disabled=!0;const i=await _0(e.text());if(s.isConnected){if(delete s.dataset.copying,s.removeAttribute("aria-busy"),s.disabled=!1,!i){s.dataset.error="1",Bn(s,A0),window.setTimeout(()=>{s.isConnected&&(delete s.dataset.error,Bn(s,t))},k0);return}s.dataset.copied="1",Bn(s,S0),window.setTimeout(()=>{s.isConnected&&(delete s.dataset.copied,Bn(s,t))},w0)}}}
    >
      <span class="chat-copy-btn__icon" aria-hidden="true">
        <span class="chat-copy-btn__icon-copy">${de.copy}</span>
        <span class="chat-copy-btn__icon-check">${de.check}</span>
      </span>
    </button>
  `}function T0(e){return C0({text:()=>e,label:gc})}function pc(e){const t=e;let n=typeof t.role=="string"?t.role:"unknown";const s=typeof t.toolCallId=="string"||typeof t.tool_call_id=="string",i=t.content,a=Array.isArray(i)?i:null,o=Array.isArray(a)&&a.some(p=>{const b=p,u=(typeof b.type=="string"?b.type:"").toLowerCase();return u==="toolresult"||u==="tool_result"}),l=typeof t.toolName=="string"||typeof t.tool_name=="string";(s||o||l)&&(n="toolResult");let d=[];typeof t.content=="string"?d=[{type:"text",text:t.content}]:Array.isArray(t.content)?d=t.content.map(p=>({type:p.type||"text",text:p.text,name:p.name,args:p.args||p.arguments})):typeof t.text=="string"&&(d=[{type:"text",text:t.text}]);const g=typeof t.timestamp=="number"?t.timestamp:Date.now(),f=typeof t.id=="string"?t.id:void 0;return{role:n,content:d,timestamp:g,id:f}}function ya(e){const t=e.toLowerCase();return e==="user"||e==="User"?e:e==="assistant"?"assistant":e==="system"?"system":t==="toolresult"||t==="tool_result"||t==="tool"||t==="function"?"tool":e}function hc(e){const t=e,n=typeof t.role=="string"?t.role.toLowerCase():"";return n==="toolresult"||n==="tool_result"}function E0(e){return(e??"tool").trim()}function L0(e){const t=e.replace(/_/g," ").trim();return t?t.split(/\s+/).map(n=>n.length<=2&&n.toUpperCase()===n?n:`${n.at(0)?.toUpperCase()??""}${n.slice(1)}`).join(" "):"Tool"}function M0(e){const t=e?.trim();if(t)return t.replace(/_/g," ")}function fc(e,t={}){const n=t.maxStringChars??160,s=t.maxArrayEntries??3;if(e!=null){if(typeof e=="string"){const i=e.trim();if(!i)return;const a=i.split(/\r?\n/)[0]?.trim()??"";return a?a.length>n?`${a.slice(0,Math.max(0,n-3))}…`:a:void 0}if(typeof e=="boolean")return!e&&!t.includeFalse?void 0:e?"true":"false";if(typeof e=="number")return Number.isFinite(e)?e===0&&!t.includeZero?void 0:String(e):t.includeNonFinite?String(e):void 0;if(Array.isArray(e)){const i=e.map(o=>fc(o,t)).filter(o=>!!o);if(i.length===0)return;const a=i.slice(0,s).join(", ");return i.length>s?`${a}…`:a}}}function I0(e,t){if(!e||typeof e!="object")return;let n=e;for(const s of t.split(".")){if(!s||!n||typeof n!="object")return;n=n[s]}return n}function R0(e){if(!e||typeof e!="object")return;const t=e,n=typeof t.path=="string"?t.path:void 0;if(!n)return;const s=typeof t.offset=="number"?t.offset:void 0,i=typeof t.limit=="number"?t.limit:void 0;return s!==void 0&&i!==void 0?`${n}:${s}-${s+i}`:n}function P0(e){if(!e||typeof e!="object")return;const t=e;return typeof t.path=="string"?t.path:void 0}function D0(e,t){if(!(!e||!t))return e.actions?.[t]??void 0}function F0(e,t,n){{for(const s of t){const i=I0(e,s),a=fc(i,n.coerce);if(a)return a}return}}const N0={icon:"puzzle",detailKeys:["command","path","url","targetUrl","targetId","ref","element","node","nodeId","id","requestId","to","channelId","guildId","userId","name","query","pattern","messageId"]},O0={bash:{icon:"wrench",title:"Bash",detailKeys:["command"]},process:{icon:"wrench",title:"Process",detailKeys:["sessionId"]},read:{icon:"fileText",title:"Read",detailKeys:["path"]},write:{icon:"edit",title:"Write",detailKeys:["path"]},edit:{icon:"penLine",title:"Edit",detailKeys:["path"]},attach:{icon:"paperclip",title:"Attach",detailKeys:["path","url","fileName"]},browser:{icon:"globe",title:"Browser",actions:{status:{label:"status"},start:{label:"start"},stop:{label:"stop"},tabs:{label:"tabs"},open:{label:"open",detailKeys:["targetUrl"]},focus:{label:"focus",detailKeys:["targetId"]},close:{label:"close",detailKeys:["targetId"]},snapshot:{label:"snapshot",detailKeys:["targetUrl","targetId","ref","element","format"]},screenshot:{label:"screenshot",detailKeys:["targetUrl","targetId","ref","element"]},navigate:{label:"navigate",detailKeys:["targetUrl","targetId"]},console:{label:"console",detailKeys:["level","targetId"]},pdf:{label:"pdf",detailKeys:["targetId"]},upload:{label:"upload",detailKeys:["paths","ref","inputRef","element","targetId"]},dialog:{label:"dialog",detailKeys:["accept","promptText","targetId"]},act:{label:"act",detailKeys:["request.kind","request.ref","request.selector","request.text","request.value"]}}},canvas:{icon:"image",title:"Canvas",actions:{present:{label:"present",detailKeys:["target","node","nodeId"]},hide:{label:"hide",detailKeys:["node","nodeId"]},navigate:{label:"navigate",detailKeys:["url","node","nodeId"]},eval:{label:"eval",detailKeys:["javaScript","node","nodeId"]},snapshot:{label:"snapshot",detailKeys:["format","node","nodeId"]},a2ui_push:{label:"A2UI push",detailKeys:["jsonlPath","node","nodeId"]},a2ui_reset:{label:"A2UI reset",detailKeys:["node","nodeId"]}}},nodes:{icon:"smartphone",title:"Nodes",actions:{status:{label:"status"},describe:{label:"describe",detailKeys:["node","nodeId"]},pending:{label:"pending"},approve:{label:"approve",detailKeys:["requestId"]},reject:{label:"reject",detailKeys:["requestId"]},notify:{label:"notify",detailKeys:["node","nodeId","title","body"]},camera_snap:{label:"camera snap",detailKeys:["node","nodeId","facing","deviceId"]},camera_list:{label:"camera list",detailKeys:["node","nodeId"]},camera_clip:{label:"camera clip",detailKeys:["node","nodeId","facing","duration","durationMs"]},screen_record:{label:"screen record",detailKeys:["node","nodeId","duration","durationMs","fps","screenIndex"]}}},cron:{icon:"loader",title:"Cron",actions:{status:{label:"status"},list:{label:"list"},add:{label:"add",detailKeys:["job.name","job.id","job.schedule","job.cron"]},update:{label:"update",detailKeys:["id"]},remove:{label:"remove",detailKeys:["id"]},run:{label:"run",detailKeys:["id"]},runs:{label:"runs",detailKeys:["id"]},wake:{label:"wake",detailKeys:["text","mode"]}}},gateway:{icon:"plug",title:"Gateway",actions:{restart:{label:"restart",detailKeys:["reason","delayMs"]},"config.get":{label:"config get"},"config.schema":{label:"config schema"},"config.apply":{label:"config apply",detailKeys:["restartDelayMs"]},"update.run":{label:"update run",detailKeys:["restartDelayMs"]}}},whatsapp_login:{icon:"circle",title:"WhatsApp Login",actions:{start:{label:"start"},wait:{label:"wait"}}},discord:{icon:"messageSquare",title:"Discord",actions:{react:{label:"react",detailKeys:["channelId","messageId","emoji"]},reactions:{label:"reactions",detailKeys:["channelId","messageId"]},sticker:{label:"sticker",detailKeys:["to","stickerIds"]},poll:{label:"poll",detailKeys:["question","to"]},permissions:{label:"permissions",detailKeys:["channelId"]},readMessages:{label:"read messages",detailKeys:["channelId","limit"]},sendMessage:{label:"send",detailKeys:["to","content"]},editMessage:{label:"edit",detailKeys:["channelId","messageId"]},deleteMessage:{label:"delete",detailKeys:["channelId","messageId"]},threadCreate:{label:"thread create",detailKeys:["channelId","name"]},threadList:{label:"thread list",detailKeys:["guildId","channelId"]},threadReply:{label:"thread reply",detailKeys:["channelId","content"]},pinMessage:{label:"pin",detailKeys:["channelId","messageId"]},unpinMessage:{label:"unpin",detailKeys:["channelId","messageId"]},listPins:{label:"list pins",detailKeys:["channelId"]},searchMessages:{label:"search",detailKeys:["guildId","content"]},memberInfo:{label:"member",detailKeys:["guildId","userId"]},roleInfo:{label:"roles",detailKeys:["guildId"]},emojiList:{label:"emoji list",detailKeys:["guildId"]},roleAdd:{label:"role add",detailKeys:["guildId","userId","roleId"]},roleRemove:{label:"role remove",detailKeys:["guildId","userId","roleId"]},channelInfo:{label:"channel",detailKeys:["channelId"]},channelList:{label:"channels",detailKeys:["guildId"]},voiceStatus:{label:"voice",detailKeys:["guildId","userId"]},eventList:{label:"events",detailKeys:["guildId"]},eventCreate:{label:"event create",detailKeys:["guildId","name"]},timeout:{label:"timeout",detailKeys:["guildId","userId"]},kick:{label:"kick",detailKeys:["guildId","userId"]},ban:{label:"ban",detailKeys:["guildId","userId"]}}},slack:{icon:"messageSquare",title:"Slack",actions:{react:{label:"react",detailKeys:["channelId","messageId","emoji"]},reactions:{label:"reactions",detailKeys:["channelId","messageId"]},sendMessage:{label:"send",detailKeys:["to","content"]},editMessage:{label:"edit",detailKeys:["channelId","messageId"]},deleteMessage:{label:"delete",detailKeys:["channelId","messageId"]},readMessages:{label:"read messages",detailKeys:["channelId","limit"]},pinMessage:{label:"pin",detailKeys:["channelId","messageId"]},unpinMessage:{label:"unpin",detailKeys:["channelId","messageId"]},listPins:{label:"list pins",detailKeys:["channelId"]},memberInfo:{label:"member",detailKeys:["userId"]},emojiList:{label:"emoji list"}}}},B0={fallback:N0,tools:O0},vc=B0,ir=vc.fallback??{icon:"puzzle"},U0=vc.tools??{};function z0(e){if(!e)return e;const t=[{re:/^\/Users\/[^/]+(\/|$)/,replacement:"~$1"},{re:/^\/home\/[^/]+(\/|$)/,replacement:"~$1"},{re:/^C:\\Users\\[^\\]+(\\|$)/i,replacement:"~$1"}];for(const n of t)if(n.re.test(e))return e.replace(n.re,n.replacement);return e}function H0(e){const t=E0(e.name),n=t.toLowerCase(),s=U0[n],i=s?.icon??ir.icon??"puzzle",a=s?.title??L0(t),o=s?.label??t,l=e.args&&typeof e.args=="object"?e.args.action:void 0,d=typeof l=="string"?l.trim():void 0,g=D0(s,d),f=M0(g?.label??d);let p;n==="read"&&(p=R0(e.args)),!p&&(n==="write"||n==="edit"||n==="attach")&&(p=P0(e.args));const b=g?.detailKeys??s?.detailKeys??ir.detailKeys??[];return!p&&b.length>0&&(p=F0(e.args,b,{coerce:{includeFalse:!0,includeZero:!0}})),!p&&e.meta&&(p=e.meta),p&&(p=z0(p)),{name:t,icon:i,title:a,label:o,verb:f,detail:p}}function K0(e){const t=[];if(e.verb&&t.push(e.verb),e.detail&&t.push(e.detail),t.length!==0)return t.join(" · ")}const j0=80,W0=2,ar=100;function q0(e){const t=e.trim();if(t.startsWith("{")||t.startsWith("["))try{const n=JSON.parse(t);return"```json\n"+JSON.stringify(n,null,2)+"\n```"}catch{}return e}function G0(e){const t=e.split(`
`),n=t.slice(0,W0),s=n.join(`
`);return s.length>ar?s.slice(0,ar)+"…":n.length<t.length?s+"…":s}function V0(e){const t=e,n=Q0(t.content),s=[];for(const i of n){const a=(typeof i.type=="string"?i.type:"").toLowerCase();(["toolcall","tool_call","tooluse","tool_use"].includes(a)||typeof i.name=="string"&&i.arguments!=null)&&s.push({kind:"call",name:i.name??"tool",args:Y0(i.arguments??i.args)})}for(const i of n){const a=(typeof i.type=="string"?i.type:"").toLowerCase();if(a!=="toolresult"&&a!=="tool_result")continue;const o=J0(i),l=typeof i.name=="string"?i.name:"tool";s.push({kind:"result",name:l,text:o})}if(hc(e)&&!s.some(i=>i.kind==="result")){const i=typeof t.toolName=="string"&&t.toolName||typeof t.tool_name=="string"&&t.tool_name||"tool",a=yl(e)??void 0;s.push({kind:"result",name:i,text:a})}return s}function or(e,t){const n=H0({name:e.name,args:e.args}),s=K0(n),i=!!e.text?.trim(),a=!!t,o=a?()=>{if(i){t(q0(e.text));return}const p=`## ${n.label}

${s?`**Command:** \`${s}\`

`:""}*No output — tool completed successfully.*`;t(p)}:void 0,l=i&&(e.text?.length??0)<=j0,d=i&&!l,g=i&&l,f=!i;return r`
    <div
      class="chat-tool-card ${a?"chat-tool-card--clickable":""}"
      @click=${o}
      role=${a?"button":m}
      tabindex=${a?"0":m}
      @keydown=${a?p=>{p.key!=="Enter"&&p.key!==" "||(p.preventDefault(),o?.())}:m}
    >
      <div class="chat-tool-card__header">
        <div class="chat-tool-card__title">
          <span class="chat-tool-card__icon">${de[n.icon]}</span>
          <span>${n.label}</span>
        </div>
        ${a?r`<span class="chat-tool-card__action">${i?"View":""} ${de.check}</span>`:m}
        ${f&&!a?r`<span class="chat-tool-card__status">${de.check}</span>`:m}
      </div>
      ${s?r`<div class="chat-tool-card__detail">${s}</div>`:m}
      ${f?r`
              <div class="chat-tool-card__status-text muted">Completed</div>
            `:m}
      ${d?r`<div class="chat-tool-card__preview mono">${G0(e.text)}</div>`:m}
      ${g?r`<div class="chat-tool-card__inline mono">${e.text}</div>`:m}
    </div>
  `}function Q0(e){return Array.isArray(e)?e.filter(Boolean):[]}function Y0(e){if(typeof e!="string")return e;const t=e.trim();if(!t||!t.startsWith("{")&&!t.startsWith("["))return e;try{return JSON.parse(t)}catch{return e}}function J0(e){if(typeof e.text=="string")return e.text;if(typeof e.content=="string")return e.content}function Z0(e){const n=e.content,s=[];if(Array.isArray(n))for(const i of n){if(typeof i!="object"||i===null)continue;const a=i;if(a.type==="image"){const o=a.source;if(o?.type==="base64"&&typeof o.data=="string"){const l=o.data,d=o.media_type||"image/png",g=l.startsWith("data:")?l:`data:${d};base64,${l}`;s.push({url:g})}else typeof a.url=="string"&&s.push({url:a.url})}else if(a.type==="image_url"){const o=a.image_url;typeof o?.url=="string"&&s.push({url:o.url})}}return s}function X0(e){return r`
    <div class="chat-group assistant">
      ${xa("assistant",e)}
      <div class="chat-group-messages">
        <div class="chat-bubble chat-reading-indicator" aria-hidden="true">
          <span class="chat-reading-indicator__dots">
            <span></span><span></span><span></span>
          </span>
        </div>
      </div>
    </div>
  `}function e1(e,t,n,s){const i=new Date(t).toLocaleTimeString([],{hour:"numeric",minute:"2-digit"}),a=s?.name??"Assistant";return r`
    <div class="chat-group assistant">
      ${xa("assistant",s)}
      <div class="chat-group-messages">
        ${mc({role:"assistant",content:[{type:"text",text:e}],timestamp:t},{isStreaming:!0,showReasoning:!1},n)}
        <div class="chat-group-footer">
          <span class="chat-sender-name">${a}</span>
          <span class="chat-group-timestamp">${i}</span>
        </div>
      </div>
    </div>
  `}function t1(e,t){const n=ya(e.role),s=t.assistantName??"Assistant",i=n==="user"?"You":n==="assistant"?s:n,a=n==="user"?"user":n==="assistant"?"assistant":"other",o=new Date(e.timestamp).toLocaleTimeString([],{hour:"numeric",minute:"2-digit"});return r`
    <div class="chat-group ${a}">
      ${xa(e.role,{name:s,avatar:t.assistantAvatar??null})}
      <div class="chat-group-messages">
        ${e.messages.map((l,d)=>mc(l.message,{isStreaming:e.isStreaming&&d===e.messages.length-1,showReasoning:t.showReasoning},t.onOpenSidebar))}
        <div class="chat-group-footer">
          <span class="chat-sender-name">${i}</span>
          <span class="chat-group-timestamp">${o}</span>
        </div>
      </div>
    </div>
  `}function xa(e,t){const n=ya(e),s=t?.name?.trim()||"Assistant",i=t?.avatar?.trim()||"",a=n==="user"?"U":n==="assistant"?s.charAt(0).toUpperCase()||"A":n==="tool"?"⚙":"?",o=n==="user"?"user":n==="assistant"?"assistant":n==="tool"?"tool":"other";return i&&n==="assistant"?n1(i)?r`<img
        class="chat-avatar ${o}"
        src="${i}"
        alt="${s}"
      />`:r`<div class="chat-avatar ${o}">${i}</div>`:r`<div class="chat-avatar ${o}">${a}</div>`}function n1(e){return/^https?:\/\//i.test(e)||/^data:image\//i.test(e)||e.startsWith("/")}function s1(e){return e.length===0?m:r`
    <div class="chat-message-images">
      ${e.map(t=>r`
          <img
            src=${t.url}
            alt=${t.alt??"Attached image"}
            class="chat-message-image"
            @click=${()=>window.open(t.url,"_blank")}
          />
        `)}
    </div>
  `}function mc(e,t,n){const s=e,i=typeof s.role=="string"?s.role:"unknown",a=hc(e)||i.toLowerCase()==="toolresult"||i.toLowerCase()==="tool_result"||typeof s.toolCallId=="string"||typeof s.tool_call_id=="string",o=V0(e),l=o.length>0,d=Z0(e),g=d.length>0,f=yl(e),p=t.showReasoning&&i==="assistant"?Bf(e):null,b=f?.trim()?f:null,u=p?zf(p):null,v=b,y=i==="assistant"&&!!v?.trim(),k=["chat-bubble",y?"has-copy":"",t.isStreaming?"streaming":"","fade-in"].filter(Boolean).join(" ");return!v&&l&&a?r`${o.map(C=>or(C,n))}`:!v&&!l&&!g?m:r`
    <div class="${k}">
      ${y?T0(v):m}
      ${s1(d)}
      ${u?r`<div class="chat-thinking">${bi(_i(u))}</div>`:m}
      ${v?r`<div class="chat-text" dir="${uc(v)}">${bi(_i(v))}</div>`:m}
      ${o.map(C=>or(C,n))}
    </div>
  `}function i1(e){return r`
    <div class="sidebar-panel">
      <div class="sidebar-header">
        <div class="sidebar-title">Tool Output</div>
        <button @click=${e.onClose} class="btn" title="Close sidebar">
          ${de.x}
        </button>
      </div>
      <div class="sidebar-content">
        ${e.error?r`
              <div class="callout danger">${e.error}</div>
              <button @click=${e.onViewRawText} class="btn" style="margin-top: 12px;">
                View Raw Text
              </button>
            `:e.content?r`<div class="sidebar-markdown">${bi(_i(e.content))}</div>`:r`
                  <div class="muted">No content available</div>
                `}
      </div>
    </div>
  `}var a1=Object.create,$a=Object.defineProperty,o1=Object.getOwnPropertyDescriptor,bc=(e,t)=>(t=Symbol[e])?t:Symbol.for("Symbol."+e),yn=e=>{throw TypeError(e)},r1=(e,t,n)=>t in e?$a(e,t,{enumerable:!0,configurable:!0,writable:!0,value:n}):e[t]=n,rr=(e,t)=>$a(e,"name",{value:t,configurable:!0}),l1=e=>[,,,a1(e?.[bc("metadata")]??null)],yc=["class","method","getter","setter","accessor","field","value","get","set"],Zt=e=>e!==void 0&&typeof e!="function"?yn("Function expected"):e,c1=(e,t,n,s,i)=>({kind:yc[e],name:t,metadata:s,addInitializer:a=>n._?yn("Already initialized"):i.push(Zt(a||null))}),d1=(e,t)=>r1(t,bc("metadata"),e[3]),pt=(e,t,n,s)=>{for(var i=0,a=e[t>>1],o=a&&a.length;i<o;i++)t&1?a[i].call(n):s=a[i].call(n,s);return s},ys=(e,t,n,s,i,a)=>{var o,l,d,g,f,p=t&7,b=!!(t&8),u=!!(t&16),v=p>3?e.length+1:p?b?1:2:0,y=yc[p+5],k=p>3&&(e[v-1]=[]),C=e[v]||(e[v]=[]),$=p&&(!u&&!b&&(i=i.prototype),p<5&&(p>3||!u)&&o1(p<4?i:{get[n](){return lr(this,a)},set[n](_){return cr(this,a,_)}},n));p?u&&p<4&&rr(a,(p>2?"set ":p>1?"get ":"")+n):rr(i,n);for(var T=s.length-1;T>=0;T--)g=c1(p,n,d={},e[3],C),p&&(g.static=b,g.private=u,f=g.access={has:u?_=>u1(i,_):_=>n in _},p^3&&(f.get=u?_=>(p^1?lr:g1)(_,i,p^4?a:$.get):_=>_[n]),p>2&&(f.set=u?(_,L)=>cr(_,i,L,p^4?a:$.set):(_,L)=>_[n]=L)),l=(0,s[T])(p?p<4?u?a:$[y]:p>4?void 0:{get:$.get,set:$.set}:i,g),d._=1,p^4||l===void 0?Zt(l)&&(p>4?k.unshift(l):p?u?a=l:$[y]=l:i=l):typeof l!="object"||l===null?yn("Object expected"):(Zt(o=l.get)&&($.get=o),Zt(o=l.set)&&($.set=o),Zt(o=l.init)&&k.unshift(o));return p||d1(e,i),$&&$a(i,n,$),u?p^4?a:$:i},wa=(e,t,n)=>t.has(e)||yn("Cannot "+n),u1=(e,t)=>Object(t)!==t?yn('Cannot use the "in" operator on this value'):e.has(t),lr=(e,t,n)=>(wa(e,t,"read from private field"),n?n.call(e):t.get(e)),cr=(e,t,n,s)=>(wa(e,t,"write to private field"),s?s.call(e,n):t.set(e,n),n),g1=(e,t,n)=>(wa(e,t,"access private method"),n),xc,$c,wc,Ci,kc,_e;kc=[Ir("resizable-divider")];class At extends(Ci=Ft,wc=[zn({type:Number})],$c=[zn({type:Number})],xc=[zn({type:Number})],Ci){constructor(){super(...arguments),this.splitRatio=pt(_e,8,this,.6),pt(_e,11,this),this.minRatio=pt(_e,12,this,.4),pt(_e,15,this),this.maxRatio=pt(_e,16,this,.7),pt(_e,19,this),this.isDragging=!1,this.startX=0,this.startRatio=0,this.handleMouseDown=t=>{this.isDragging=!0,this.startX=t.clientX,this.startRatio=this.splitRatio,this.classList.add("dragging"),document.addEventListener("mousemove",this.handleMouseMove),document.addEventListener("mouseup",this.handleMouseUp),t.preventDefault()},this.handleMouseMove=t=>{if(!this.isDragging)return;const n=this.parentElement;if(!n)return;const s=n.getBoundingClientRect().width,a=(t.clientX-this.startX)/s;let o=this.startRatio+a;o=Math.max(this.minRatio,Math.min(this.maxRatio,o)),this.dispatchEvent(new CustomEvent("resize",{detail:{splitRatio:o},bubbles:!0,composed:!0}))},this.handleMouseUp=()=>{this.isDragging=!1,this.classList.remove("dragging"),document.removeEventListener("mousemove",this.handleMouseMove),document.removeEventListener("mouseup",this.handleMouseUp)}}render(){return m}connectedCallback(){super.connectedCallback(),this.addEventListener("mousedown",this.handleMouseDown)}disconnectedCallback(){super.disconnectedCallback(),this.removeEventListener("mousedown",this.handleMouseDown),document.removeEventListener("mousemove",this.handleMouseMove),document.removeEventListener("mouseup",this.handleMouseUp)}}_e=l1(Ci);ys(_e,5,"splitRatio",wc,At);ys(_e,5,"minRatio",$c,At);ys(_e,5,"maxRatio",xc,At);At=ys(_e,0,"ResizableDivider",kc,At);At.styles=tp`
    :host {
      width: 4px;
      cursor: col-resize;
      background: var(--border, #333);
      transition: background 150ms ease-out;
      flex-shrink: 0;
      position: relative;
    }
    :host::before {
      content: "";
      position: absolute;
      top: 0;
      left: -4px;
      right: -4px;
      bottom: 0;
    }
    :host(:hover) {
      background: var(--accent, #007bff);
    }
    :host(.dragging) {
      background: var(--accent, #007bff);
    }
  `;pt(_e,1,At);const p1=5e3;function dr(e){e.style.height="auto",e.style.height=`${e.scrollHeight}px`}function h1(e){return e?e.active?r`
      <div class="compaction-indicator compaction-indicator--active" role="status" aria-live="polite">
        ${de.loader} Compacting context...
      </div>
    `:e.completedAt&&Date.now()-e.completedAt<p1?r`
        <div class="compaction-indicator compaction-indicator--complete" role="status" aria-live="polite">
          ${de.check} Context compacted
        </div>
      `:m:m}function f1(){return`att-${Date.now()}-${Math.random().toString(36).slice(2,9)}`}function v1(e,t){const n=e.clipboardData?.items;if(!n||!t.onAttachmentsChange)return;const s=[];for(let i=0;i<n.length;i++){const a=n[i];a.type.startsWith("image/")&&s.push(a)}if(s.length!==0){e.preventDefault();for(const i of s){const a=i.getAsFile();if(!a)continue;const o=new FileReader;o.addEventListener("load",()=>{const l=o.result,d={id:f1(),dataUrl:l,mimeType:a.type},g=t.attachments??[];t.onAttachmentsChange?.([...g,d])}),o.readAsDataURL(a)}}}function m1(e){const t=e.attachments??[];return t.length===0?m:r`
    <div class="chat-attachments">
      ${t.map(n=>r`
          <div class="chat-attachment">
            <img
              src=${n.dataUrl}
              alt="Attachment preview"
              class="chat-attachment__img"
            />
            <button
              class="chat-attachment__remove"
              type="button"
              aria-label="Remove attachment"
              @click=${()=>{const s=(e.attachments??[]).filter(i=>i.id!==n.id);e.onAttachmentsChange?.(s)}}
            >
              ${de.x}
            </button>
          </div>
        `)}
    </div>
  `}function b1(e){const t=e.connected,n=e.sending||e.stream!==null,s=!!(e.canAbort&&e.onAbort),a=e.sessions?.sessions?.find(u=>u.key===e.sessionKey)?.reasoningLevel??"off",o=e.showThinking&&a!=="off",l={name:e.assistantName,avatar:e.assistantAvatar??e.assistantAvatarUrl??null},d=(e.attachments?.length??0)>0,g=e.connected?d?"Add a message or paste more images...":"Message (↩ to send, Shift+↩ for line breaks, paste images)":"Connect to the gateway to start chatting…",f=e.splitRatio??.6,p=!!(e.sidebarOpen&&e.onCloseSidebar),b=r`
    <div
      class="chat-thread"
      role="log"
      aria-live="polite"
      @scroll=${e.onChatScroll}
    >
      ${e.loading?r`
              <div class="muted">Loading chat…</div>
            `:m}
      ${Rl(x1(e),u=>u.key,u=>u.kind==="divider"?r`
              <div class="chat-divider" role="separator" data-ts=${String(u.timestamp)}>
                <span class="chat-divider__line"></span>
                <span class="chat-divider__label">${u.label}</span>
                <span class="chat-divider__line"></span>
              </div>
            `:u.kind==="reading-indicator"?X0(l):u.kind==="stream"?e1(u.text,u.startedAt,e.onOpenSidebar,l):u.kind==="group"?t1(u,{onOpenSidebar:e.onOpenSidebar,showReasoning:o,assistantName:e.assistantName,assistantAvatar:l.avatar}):m)}
    </div>
  `;return r`
    <section class="card chat">
      ${e.disabledReason?r`<div class="callout">${e.disabledReason}</div>`:m}

      ${e.error?r`<div class="callout danger">${e.error}</div>`:m}

      ${e.focusMode?r`
            <button
              class="chat-focus-exit"
              type="button"
              @click=${e.onToggleFocusMode}
              aria-label="Exit focus mode"
              title="Exit focus mode"
            >
              ${de.x}
            </button>
          `:m}

      <div
        class="chat-split-container ${p?"chat-split-container--open":""}"
      >
        <div
          class="chat-main"
          style="flex: ${p?`0 0 ${f*100}%`:"1 1 100%"}"
        >
          ${b}
        </div>

        ${p?r`
              <resizable-divider
                .splitRatio=${f}
                @resize=${u=>e.onSplitRatioChange?.(u.detail.splitRatio)}
              ></resizable-divider>
              <div class="chat-sidebar">
                ${i1({content:e.sidebarContent??null,error:e.sidebarError??null,onClose:e.onCloseSidebar,onViewRawText:()=>{!e.sidebarContent||!e.onOpenSidebar||e.onOpenSidebar(`\`\`\`
${e.sidebarContent}
\`\`\``)}})}
              </div>
            `:m}
      </div>

      ${e.queue.length?r`
            <div class="chat-queue" role="status" aria-live="polite">
              <div class="chat-queue__title">Queued (${e.queue.length})</div>
              <div class="chat-queue__list">
                ${e.queue.map(u=>r`
                    <div class="chat-queue__item">
                      <div class="chat-queue__text">
                        ${u.text||(u.attachments?.length?`Image (${u.attachments.length})`:"")}
                      </div>
                      <button
                        class="btn chat-queue__remove"
                        type="button"
                        aria-label="Remove queued message"
                        @click=${()=>e.onQueueRemove(u.id)}
                      >
                        ${de.x}
                      </button>
                    </div>
                  `)}
              </div>
            </div>
          `:m}

      ${h1(e.compactionStatus)}

      ${e.showNewMessages?r`
            <button
              class="btn chat-new-messages"
              type="button"
              @click=${e.onScrollToBottom}
            >
              New messages ${de.arrowDown}
            </button>
          `:m}

      <div class="chat-compose">
        ${m1(e)}
        <div class="chat-compose__row">
          <label class="field chat-compose__field">
            <span>Message</span>
            <textarea
              ${sy(u=>u&&dr(u))}
              .value=${e.draft}
              dir=${uc(e.draft)}
              ?disabled=${!e.connected}
              @keydown=${u=>{u.key==="Enter"&&(u.isComposing||u.keyCode===229||u.shiftKey||e.connected&&(u.preventDefault(),t&&e.onSend()))}}
              @input=${u=>{const v=u.target;dr(v),e.onDraftChange(v.value)}}
              @paste=${u=>v1(u,e)}
              placeholder=${g}
            ></textarea>
          </label>
          <div class="chat-compose__actions">
            <button
              class="btn"
              ?disabled=${!e.connected||!s&&e.sending}
              @click=${s?e.onAbort:e.onNewSession}
            >
              ${s?"Stop":"New session"}
            </button>
            <button
              class="btn primary"
              ?disabled=${!e.connected}
              @click=${e.onSend}
            >
              ${n?"Queue":"Send"}<kbd class="btn-kbd">↵</kbd>
            </button>
          </div>
        </div>
      </div>
    </section>
  `}const ur=200;function y1(e){const t=[];let n=null;for(const s of e){if(s.kind!=="message"){n&&(t.push(n),n=null),t.push(s);continue}const i=pc(s.message),a=ya(i.role),o=i.timestamp||Date.now();!n||n.role!==a?(n&&t.push(n),n={kind:"group",key:`group:${a}:${s.key}`,role:a,messages:[{message:s.message,key:s.key}],timestamp:o,isStreaming:!1}):n.messages.push({message:s.message,key:s.key})}return n&&t.push(n),t}function x1(e){const t=[],n=Array.isArray(e.messages)?e.messages:[],s=Array.isArray(e.toolMessages)?e.toolMessages:[],i=Math.max(0,n.length-ur);i>0&&t.push({kind:"message",key:"chat:history:notice",message:{role:"system",content:`Showing last ${ur} messages (${i} hidden).`,timestamp:Date.now()}});for(let a=i;a<n.length;a++){const o=n[a],l=pc(o),g=o.__aisopod;if(g&&g.kind==="compaction"){t.push({kind:"divider",key:typeof g.id=="string"?`divider:compaction:${g.id}`:`divider:compaction:${l.timestamp}:${a}`,label:"Compaction",timestamp:l.timestamp??Date.now()});continue}!e.showThinking&&l.role.toLowerCase()==="toolresult"||t.push({kind:"message",key:gr(o,a),message:o})}if(e.showThinking)for(let a=0;a<s.length;a++)t.push({kind:"message",key:gr(s[a],a+n.length),message:s[a]});if(e.stream!==null){const a=`stream:${e.sessionKey}:${e.streamStartedAt??"live"}`;e.stream.trim().length>0?t.push({kind:"stream",key:a,text:e.stream,startedAt:e.streamStartedAt??Date.now()}):t.push({kind:"reading-indicator",key:a})}return y1(t)}function gr(e,t){const n=e,s=typeof n.toolCallId=="string"?n.toolCallId:"";if(s)return`tool:${s}`;const i=typeof n.id=="string"?n.id:"";if(i)return`msg:${i}`;const a=typeof n.messageId=="string"?n.messageId:"";if(a)return`msg:${a}`;const o=typeof n.timestamp=="number"?n.timestamp:null,l=typeof n.role=="string"?n.role:"unknown";return o!=null?`msg:${l}:${o}:${t}`:`msg:${l}:${t}`}const Ti={all:r`
    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
      <rect x="3" y="3" width="7" height="7"></rect>
      <rect x="14" y="3" width="7" height="7"></rect>
      <rect x="14" y="14" width="7" height="7"></rect>
      <rect x="3" y="14" width="7" height="7"></rect>
    </svg>
  `,env:r`
    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
      <circle cx="12" cy="12" r="3"></circle>
      <path
        d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1 0 2.83 2 2 0 0 1-2.83 0l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-2 2 2 2 0 0 1-2-2v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83 0 2 2 0 0 1 0-2.83l.06-.06a1.65 1.65 0 0 0 .33-1.82 1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1-2-2 2 2 0 0 1 2-2h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 0-2.83 2 2 0 0 1 2.83 0l.06.06a1.65 1.65 0 0 0 1.82.33H9a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 2-2 2 2 0 0 1 2 2v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 0 2 2 0 0 1 0 2.83l-.06.06a1.65 1.65 0 0 0-.33 1.82V9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 2 2 2 2 0 0 1-2 2h-.09a1.65 1.65 0 0 0-1.51 1z"
      ></path>
    </svg>
  `,update:r`
    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
      <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"></path>
      <polyline points="7 10 12 15 17 10"></polyline>
      <line x1="12" y1="15" x2="12" y2="3"></line>
    </svg>
  `,agents:r`
    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
      <path
        d="M12 2a2 2 0 0 1 2 2c0 .74-.4 1.39-1 1.73V7h1a7 7 0 0 1 7 7h1a1 1 0 0 1 1 1v3a1 1 0 0 1-1 1h-1v1a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-1H2a1 1 0 0 1-1-1v-3a1 1 0 0 1 1-1h1a7 7 0 0 1 7-7h1V5.73c-.6-.34-1-.99-1-1.73a2 2 0 0 1 2-2z"
      ></path>
      <circle cx="8" cy="14" r="1"></circle>
      <circle cx="16" cy="14" r="1"></circle>
    </svg>
  `,auth:r`
    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
      <rect x="3" y="11" width="18" height="11" rx="2" ry="2"></rect>
      <path d="M7 11V7a5 5 0 0 1 10 0v4"></path>
    </svg>
  `,channels:r`
    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
      <path d="M21 15a2 2 0 0 1-2 2H7l-4 4V5a2 2 0 0 1 2-2h14a2 2 0 0 1 2 2z"></path>
    </svg>
  `,messages:r`
    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
      <path d="M4 4h16c1.1 0 2 .9 2 2v12c0 1.1-.9 2-2 2H4c-1.1 0-2-.9-2-2V6c0-1.1.9-2 2-2z"></path>
      <polyline points="22,6 12,13 2,6"></polyline>
    </svg>
  `,commands:r`
    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
      <polyline points="4 17 10 11 4 5"></polyline>
      <line x1="12" y1="19" x2="20" y2="19"></line>
    </svg>
  `,hooks:r`
    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
      <path d="M10 13a5 5 0 0 0 7.54.54l3-3a5 5 0 0 0-7.07-7.07l-1.72 1.71"></path>
      <path d="M14 11a5 5 0 0 0-7.54-.54l-3 3a5 5 0 0 0 7.07 7.07l1.71-1.71"></path>
    </svg>
  `,skills:r`
    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
      <polygon
        points="12 2 15.09 8.26 22 9.27 17 14.14 18.18 21.02 12 17.77 5.82 21.02 7 14.14 2 9.27 8.91 8.26 12 2"
      ></polygon>
    </svg>
  `,tools:r`
    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
      <path
        d="M14.7 6.3a1 1 0 0 0 0 1.4l1.6 1.6a1 1 0 0 0 1.4 0l3.77-3.77a6 6 0 0 1-7.94 7.94l-6.91 6.91a2.12 2.12 0 0 1-3-3l6.91-6.91a6 6 0 0 1 7.94-7.94l-3.76 3.76z"
      ></path>
    </svg>
  `,gateway:r`
    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
      <circle cx="12" cy="12" r="10"></circle>
      <line x1="2" y1="12" x2="22" y2="12"></line>
      <path
        d="M12 2a15.3 15.3 0 0 1 4 10 15.3 15.3 0 0 1-4 10 15.3 15.3 0 0 1-4-10 15.3 15.3 0 0 1 4-10z"
      ></path>
    </svg>
  `,wizard:r`
    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
      <path d="M15 4V2"></path>
      <path d="M15 16v-2"></path>
      <path d="M8 9h2"></path>
      <path d="M20 9h2"></path>
      <path d="M17.8 11.8 19 13"></path>
      <path d="M15 9h0"></path>
      <path d="M17.8 6.2 19 5"></path>
      <path d="m3 21 9-9"></path>
      <path d="M12.2 6.2 11 5"></path>
    </svg>
  `,meta:r`
    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
      <path d="M12 20h9"></path>
      <path d="M16.5 3.5a2.12 2.12 0 0 1 3 3L7 19l-4 1 1-4Z"></path>
    </svg>
  `,logging:r`
    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
      <path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"></path>
      <polyline points="14 2 14 8 20 8"></polyline>
      <line x1="16" y1="13" x2="8" y2="13"></line>
      <line x1="16" y1="17" x2="8" y2="17"></line>
      <polyline points="10 9 9 9 8 9"></polyline>
    </svg>
  `,browser:r`
    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
      <circle cx="12" cy="12" r="10"></circle>
      <circle cx="12" cy="12" r="4"></circle>
      <line x1="21.17" y1="8" x2="12" y2="8"></line>
      <line x1="3.95" y1="6.06" x2="8.54" y2="14"></line>
      <line x1="10.88" y1="21.94" x2="15.46" y2="14"></line>
    </svg>
  `,ui:r`
    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
      <rect x="3" y="3" width="18" height="18" rx="2" ry="2"></rect>
      <line x1="3" y1="9" x2="21" y2="9"></line>
      <line x1="9" y1="21" x2="9" y2="9"></line>
    </svg>
  `,models:r`
    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
      <path
        d="M21 16V8a2 2 0 0 0-1-1.73l-7-4a2 2 0 0 0-2 0l-7 4A2 2 0 0 0 3 8v8a2 2 0 0 0 1 1.73l7 4a2 2 0 0 0 2 0l7-4A2 2 0 0 0 21 16z"
      ></path>
      <polyline points="3.27 6.96 12 12.01 20.73 6.96"></polyline>
      <line x1="12" y1="22.08" x2="12" y2="12"></line>
    </svg>
  `,bindings:r`
    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
      <rect x="2" y="2" width="20" height="8" rx="2" ry="2"></rect>
      <rect x="2" y="14" width="20" height="8" rx="2" ry="2"></rect>
      <line x1="6" y1="6" x2="6.01" y2="6"></line>
      <line x1="6" y1="18" x2="6.01" y2="18"></line>
    </svg>
  `,broadcast:r`
    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
      <path d="M4.9 19.1C1 15.2 1 8.8 4.9 4.9"></path>
      <path d="M7.8 16.2c-2.3-2.3-2.3-6.1 0-8.5"></path>
      <circle cx="12" cy="12" r="2"></circle>
      <path d="M16.2 7.8c2.3 2.3 2.3 6.1 0 8.5"></path>
      <path d="M19.1 4.9C23 8.8 23 15.1 19.1 19"></path>
    </svg>
  `,audio:r`
    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
      <path d="M9 18V5l12-2v13"></path>
      <circle cx="6" cy="18" r="3"></circle>
      <circle cx="18" cy="16" r="3"></circle>
    </svg>
  `,session:r`
    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
      <path d="M17 21v-2a4 4 0 0 0-4-4H5a4 4 0 0 0-4 4v2"></path>
      <circle cx="9" cy="7" r="4"></circle>
      <path d="M23 21v-2a4 4 0 0 0-3-3.87"></path>
      <path d="M16 3.13a4 4 0 0 1 0 7.75"></path>
    </svg>
  `,cron:r`
    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
      <circle cx="12" cy="12" r="10"></circle>
      <polyline points="12 6 12 12 16 14"></polyline>
    </svg>
  `,web:r`
    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
      <circle cx="12" cy="12" r="10"></circle>
      <line x1="2" y1="12" x2="22" y2="12"></line>
      <path
        d="M12 2a15.3 15.3 0 0 1 4 10 15.3 15.3 0 0 1-4 10 15.3 15.3 0 0 1-4-10 15.3 15.3 0 0 1 4-10z"
      ></path>
    </svg>
  `,discovery:r`
    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
      <circle cx="11" cy="11" r="8"></circle>
      <line x1="21" y1="21" x2="16.65" y2="16.65"></line>
    </svg>
  `,canvasHost:r`
    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
      <rect x="3" y="3" width="18" height="18" rx="2" ry="2"></rect>
      <circle cx="8.5" cy="8.5" r="1.5"></circle>
      <polyline points="21 15 16 10 5 21"></polyline>
    </svg>
  `,talk:r`
    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
      <path d="M12 1a3 3 0 0 0-3 3v8a3 3 0 0 0 6 0V4a3 3 0 0 0-3-3z"></path>
      <path d="M19 10v2a7 7 0 0 1-14 0v-2"></path>
      <line x1="12" y1="19" x2="12" y2="23"></line>
      <line x1="8" y1="23" x2="16" y2="23"></line>
    </svg>
  `,plugins:r`
    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
      <path d="M12 2v6"></path>
      <path d="m4.93 10.93 4.24 4.24"></path>
      <path d="M2 12h6"></path>
      <path d="m4.93 13.07 4.24-4.24"></path>
      <path d="M12 22v-6"></path>
      <path d="m19.07 13.07-4.24-4.24"></path>
      <path d="M22 12h-6"></path>
      <path d="m19.07 10.93-4.24 4.24"></path>
    </svg>
  `,default:r`
    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
      <path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"></path>
      <polyline points="14 2 14 8 20 8"></polyline>
    </svg>
  `},pr=[{key:"env",label:"Environment"},{key:"update",label:"Updates"},{key:"agents",label:"Agents"},{key:"auth",label:"Authentication"},{key:"channels",label:"Channels"},{key:"messages",label:"Messages"},{key:"commands",label:"Commands"},{key:"hooks",label:"Hooks"},{key:"skills",label:"Skills"},{key:"tools",label:"Tools"},{key:"gateway",label:"Gateway"},{key:"wizard",label:"Setup Wizard"}],hr="__all__";function fr(e){return Ti[e]??Ti.default}function $1(e,t){const n=ca[e];return n||{label:t?.title??Ge(e),description:t?.description??""}}function w1(e){const{key:t,schema:n,uiHints:s}=e;if(!n||ke(n)!=="object"||!n.properties)return[];const i=Object.entries(n.properties).map(([a,o])=>{const l=Ce([t,a],s),d=l?.label??o.title??Ge(a),g=l?.help??o.description??"",f=l?.order??50;return{key:a,label:d,description:g,order:f}});return i.sort((a,o)=>a.order!==o.order?a.order-o.order:a.key.localeCompare(o.key)),i}function k1(e,t){if(!e||!t)return[];const n=[];function s(i,a,o){if(i===a)return;if(typeof i!=typeof a){n.push({path:o,from:i,to:a});return}if(typeof i!="object"||i===null||a===null){i!==a&&n.push({path:o,from:i,to:a});return}if(Array.isArray(i)&&Array.isArray(a)){JSON.stringify(i)!==JSON.stringify(a)&&n.push({path:o,from:i,to:a});return}const l=i,d=a,g=new Set([...Object.keys(l),...Object.keys(d)]);for(const f of g)s(l[f],d[f],o?`${o}.${f}`:f)}return s(e,t,""),n}function vr(e,t=40){let n;try{n=JSON.stringify(e)??String(e)}catch{n=String(e)}return n.length<=t?n:n.slice(0,t-3)+"..."}function S1(e){const t=e.valid==null?"unknown":e.valid?"valid":"invalid",n=Kl(e.schema),s=n.schema?n.unsupportedPaths.length>0:!1,i=n.schema?.properties??{},a=pr.filter(E=>E.key in i),o=new Set(pr.map(E=>E.key)),l=Object.keys(i).filter(E=>!o.has(E)).map(E=>({key:E,label:E.charAt(0).toUpperCase()+E.slice(1)})),d=[...a,...l],g=e.activeSection&&n.schema&&ke(n.schema)==="object"?n.schema.properties?.[e.activeSection]:void 0,f=e.activeSection?$1(e.activeSection,g):null,p=e.activeSection?w1({key:e.activeSection,schema:g,uiHints:e.uiHints}):[],b=e.formMode==="form"&&!!e.activeSection&&p.length>0,u=e.activeSubsection===hr,v=e.searchQuery||u?null:e.activeSubsection??p[0]?.key??null,y=e.formMode==="form"?k1(e.originalValue,e.formValue):[],k=e.formMode==="raw"&&e.raw!==e.originalRaw,C=e.formMode==="form"?y.length>0:k,$=!!e.formValue&&!e.loading&&!!n.schema,T=e.connected&&!e.saving&&C&&(e.formMode==="raw"?!0:$),_=e.connected&&!e.applying&&!e.updating&&C&&(e.formMode==="raw"?!0:$),L=e.connected&&!e.applying&&!e.updating;return r`
    <div class="config-layout">
      <!-- Sidebar -->
      <aside class="config-sidebar">
        <div class="config-sidebar__header">
          <div class="config-sidebar__title">Settings</div>
          <span
            class="pill pill--sm ${t==="valid"?"pill--ok":t==="invalid"?"pill--danger":""}"
            >${t}</span
          >
        </div>

        <!-- Search -->
        <div class="config-search">
          <svg
            class="config-search__icon"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="2"
          >
            <circle cx="11" cy="11" r="8"></circle>
            <path d="M21 21l-4.35-4.35"></path>
          </svg>
          <input
            type="text"
            class="config-search__input"
            placeholder="Search settings..."
            .value=${e.searchQuery}
            @input=${E=>e.onSearchChange(E.target.value)}
          />
          ${e.searchQuery?r`
                <button
                  class="config-search__clear"
                  @click=${()=>e.onSearchChange("")}
                >
                  ×
                </button>
              `:m}
        </div>

        <!-- Section nav -->
        <nav class="config-nav">
          <button
            class="config-nav__item ${e.activeSection===null?"active":""}"
            @click=${()=>e.onSectionChange(null)}
          >
            <span class="config-nav__icon">${Ti.all}</span>
            <span class="config-nav__label">All Settings</span>
          </button>
          ${d.map(E=>r`
              <button
                class="config-nav__item ${e.activeSection===E.key?"active":""}"
                @click=${()=>e.onSectionChange(E.key)}
              >
                <span class="config-nav__icon"
                  >${fr(E.key)}</span
                >
                <span class="config-nav__label">${E.label}</span>
              </button>
            `)}
        </nav>

        <!-- Mode toggle at bottom -->
        <div class="config-sidebar__footer">
          <div class="config-mode-toggle">
            <button
              class="config-mode-toggle__btn ${e.formMode==="form"?"active":""}"
              ?disabled=${e.schemaLoading||!e.schema}
              @click=${()=>e.onFormModeChange("form")}
            >
              Form
            </button>
            <button
              class="config-mode-toggle__btn ${e.formMode==="raw"?"active":""}"
              @click=${()=>e.onFormModeChange("raw")}
            >
              Raw
            </button>
          </div>
        </div>
      </aside>

      <!-- Main content -->
      <main class="config-main">
        <!-- Action bar -->
        <div class="config-actions">
          <div class="config-actions__left">
            ${C?r`
                  <span class="config-changes-badge"
                    >${e.formMode==="raw"?"Unsaved changes":`${y.length} unsaved change${y.length!==1?"s":""}`}</span
                  >
                `:r`
                    <span class="config-status muted">No changes</span>
                  `}
          </div>
          <div class="config-actions__right">
            <button
              class="btn btn--sm"
              ?disabled=${e.loading}
              @click=${e.onReload}
            >
              ${e.loading?"Loading…":"Reload"}
            </button>
            <button
              class="btn btn--sm primary"
              ?disabled=${!T}
              @click=${e.onSave}
            >
              ${e.saving?"Saving…":"Save"}
            </button>
            <button
              class="btn btn--sm"
              ?disabled=${!_}
              @click=${e.onApply}
            >
              ${e.applying?"Applying…":"Apply"}
            </button>
            <button
              class="btn btn--sm"
              ?disabled=${!L}
              @click=${e.onUpdate}
            >
              ${e.updating?"Updating…":"Update"}
            </button>
          </div>
        </div>

        <!-- Diff panel (form mode only - raw mode doesn't have granular diff) -->
        ${C&&e.formMode==="form"?r`
              <details class="config-diff">
                <summary class="config-diff__summary">
                  <span
                    >View ${y.length} pending
                    change${y.length!==1?"s":""}</span
                  >
                  <svg
                    class="config-diff__chevron"
                    viewBox="0 0 24 24"
                    fill="none"
                    stroke="currentColor"
                    stroke-width="2"
                  >
                    <polyline points="6 9 12 15 18 9"></polyline>
                  </svg>
                </summary>
                <div class="config-diff__content">
                  ${y.map(E=>r`
                      <div class="config-diff__item">
                        <div class="config-diff__path">${E.path}</div>
                        <div class="config-diff__values">
                          <span class="config-diff__from"
                            >${vr(E.from)}</span
                          >
                          <span class="config-diff__arrow">→</span>
                          <span class="config-diff__to"
                            >${vr(E.to)}</span
                          >
                        </div>
                      </div>
                    `)}
                </div>
              </details>
            `:m}
        ${f&&e.formMode==="form"?r`
              <div class="config-section-hero">
                <div class="config-section-hero__icon">
                  ${fr(e.activeSection??"")}
                </div>
                <div class="config-section-hero__text">
                  <div class="config-section-hero__title">
                    ${f.label}
                  </div>
                  ${f.description?r`<div class="config-section-hero__desc">
                        ${f.description}
                      </div>`:m}
                </div>
              </div>
            `:m}
        ${b?r`
              <div class="config-subnav">
                <button
                  class="config-subnav__item ${v===null?"active":""}"
                  @click=${()=>e.onSubsectionChange(hr)}
                >
                  All
                </button>
                ${p.map(E=>r`
                    <button
                      class="config-subnav__item ${v===E.key?"active":""}"
                      title=${E.description||E.label}
                      @click=${()=>e.onSubsectionChange(E.key)}
                    >
                      ${E.label}
                    </button>
                  `)}
              </div>
            `:m}

        <!-- Form content -->
        <div class="config-content">
          ${e.formMode==="form"?r`
                ${e.schemaLoading?r`
                        <div class="config-loading">
                          <div class="config-loading__spinner"></div>
                          <span>Loading schema…</span>
                        </div>
                      `:kb({schema:n.schema,uiHints:e.uiHints,value:e.formValue,disabled:e.loading||!e.formValue,unsupportedPaths:n.unsupportedPaths,onPatch:e.onFormPatch,searchQuery:e.searchQuery,activeSection:e.activeSection,activeSubsection:v})}
                ${s?r`
                        <div class="callout danger" style="margin-top: 12px">
                          Form view can't safely edit some fields. Use Raw to avoid losing config entries.
                        </div>
                      `:m}
              `:r`
                <label class="field config-raw-field">
                  <span>Raw JSON5</span>
                  <textarea
                    .value=${e.raw}
                    @input=${E=>e.onRawChange(E.target.value)}
                  ></textarea>
                </label>
              `}
        </div>

        ${e.issues.length>0?r`<div class="callout danger" style="margin-top: 12px;">
              <pre class="code-block">
${JSON.stringify(e.issues,null,2)}</pre
              >
            </div>`:m}
      </main>
    </div>
  `}function A1(e){const t=["last",...e.channels.filter(Boolean)],n=e.form.deliveryChannel?.trim();n&&!t.includes(n)&&t.push(n);const s=new Set;return t.filter(i=>s.has(i)?!1:(s.add(i),!0))}function _1(e,t){if(t==="last")return"last";const n=e.channelMeta?.find(s=>s.id===t);return n?.label?n.label:e.channelLabels?.[t]??t}function C1(e){const t=A1(e),s=(e.runsJobId==null?void 0:e.jobs.find(a=>a.id===e.runsJobId))?.name??e.runsJobId??"(select a job)",i=e.runs.toSorted((a,o)=>o.ts-a.ts);return r`
    <section class="grid grid-cols-2">
      <div class="card">
        <div class="card-title">Scheduler</div>
        <div class="card-sub">Gateway-owned cron scheduler status.</div>
        <div class="stat-grid" style="margin-top: 16px;">
          <div class="stat">
            <div class="stat-label">Enabled</div>
            <div class="stat-value">
              ${e.status?e.status.enabled?"Yes":"No":"n/a"}
            </div>
          </div>
          <div class="stat">
            <div class="stat-label">Jobs</div>
            <div class="stat-value">${e.status?.jobs??"n/a"}</div>
          </div>
          <div class="stat">
            <div class="stat-label">Next wake</div>
            <div class="stat-value">${la(e.status?.nextWakeAtMs??null)}</div>
          </div>
        </div>
        <div class="row" style="margin-top: 12px;">
          <button class="btn" ?disabled=${e.loading} @click=${e.onRefresh}>
            ${e.loading?"Refreshing…":"Refresh"}
          </button>
          ${e.error?r`<span class="muted">${e.error}</span>`:m}
        </div>
      </div>

      <div class="card">
        <div class="card-title">New Job</div>
        <div class="card-sub">Create a scheduled wakeup or agent run.</div>
        <div class="form-grid" style="margin-top: 16px;">
          <label class="field">
            <span>Name</span>
            <input
              .value=${e.form.name}
              @input=${a=>e.onFormChange({name:a.target.value})}
            />
          </label>
          <label class="field">
            <span>Description</span>
            <input
              .value=${e.form.description}
              @input=${a=>e.onFormChange({description:a.target.value})}
            />
          </label>
          <label class="field">
            <span>Agent ID</span>
            <input
              .value=${e.form.agentId}
              @input=${a=>e.onFormChange({agentId:a.target.value})}
              placeholder="default"
            />
          </label>
          <label class="field checkbox">
            <span>Enabled</span>
            <input
              type="checkbox"
              .checked=${e.form.enabled}
              @change=${a=>e.onFormChange({enabled:a.target.checked})}
            />
          </label>
          <label class="field">
            <span>Schedule</span>
            <select
              .value=${e.form.scheduleKind}
              @change=${a=>e.onFormChange({scheduleKind:a.target.value})}
            >
              <option value="every">Every</option>
              <option value="at">At</option>
              <option value="cron">Cron</option>
            </select>
          </label>
        </div>
        ${T1(e)}
        <div class="form-grid" style="margin-top: 12px;">
          <label class="field">
            <span>Session</span>
            <select
              .value=${e.form.sessionTarget}
              @change=${a=>e.onFormChange({sessionTarget:a.target.value})}
            >
              <option value="main">Main</option>
              <option value="isolated">Isolated</option>
            </select>
          </label>
          <label class="field">
            <span>Wake mode</span>
            <select
              .value=${e.form.wakeMode}
              @change=${a=>e.onFormChange({wakeMode:a.target.value})}
            >
              <option value="now">Now</option>
              <option value="next-heartbeat">Next heartbeat</option>
            </select>
          </label>
          <label class="field">
            <span>Payload</span>
            <select
              .value=${e.form.payloadKind}
              @change=${a=>e.onFormChange({payloadKind:a.target.value})}
            >
              <option value="systemEvent">System event</option>
              <option value="agentTurn">Agent turn</option>
            </select>
          </label>
        </div>
        <label class="field" style="margin-top: 12px;">
          <span>${e.form.payloadKind==="systemEvent"?"System text":"Agent message"}</span>
          <textarea
            .value=${e.form.payloadText}
            @input=${a=>e.onFormChange({payloadText:a.target.value})}
            rows="4"
          ></textarea>
        </label>
        ${e.form.payloadKind==="agentTurn"?r`
                <div class="form-grid" style="margin-top: 12px;">
                  <label class="field">
                    <span>Delivery</span>
                    <select
                      .value=${e.form.deliveryMode}
                      @change=${a=>e.onFormChange({deliveryMode:a.target.value})}
                    >
                      <option value="announce">Announce summary (default)</option>
                      <option value="none">None (internal)</option>
                    </select>
                  </label>
                  <label class="field">
                    <span>Timeout (seconds)</span>
                    <input
                      .value=${e.form.timeoutSeconds}
                      @input=${a=>e.onFormChange({timeoutSeconds:a.target.value})}
                    />
                  </label>
                  ${e.form.deliveryMode==="announce"?r`
                          <label class="field">
                            <span>Channel</span>
                            <select
                              .value=${e.form.deliveryChannel||"last"}
                              @change=${a=>e.onFormChange({deliveryChannel:a.target.value})}
                            >
                              ${t.map(a=>r`<option value=${a}>
                                    ${_1(e,a)}
                                  </option>`)}
                            </select>
                          </label>
                          <label class="field">
                            <span>To</span>
                            <input
                              .value=${e.form.deliveryTo}
                              @input=${a=>e.onFormChange({deliveryTo:a.target.value})}
                              placeholder="+1555… or chat id"
                            />
                          </label>
                        `:m}
                </div>
              `:m}
        <div class="row" style="margin-top: 14px;">
          <button class="btn primary" ?disabled=${e.busy} @click=${e.onAdd}>
            ${e.busy?"Saving…":"Add job"}
          </button>
        </div>
      </div>
    </section>

    <section class="card" style="margin-top: 18px;">
      <div class="card-title">Jobs</div>
      <div class="card-sub">All scheduled jobs stored in the gateway.</div>
      ${e.jobs.length===0?r`
              <div class="muted" style="margin-top: 12px">No jobs yet.</div>
            `:r`
            <div class="list" style="margin-top: 12px;">
              ${e.jobs.map(a=>E1(a,e))}
            </div>
          `}
    </section>

    <section class="card" style="margin-top: 18px;">
      <div class="card-title">Run history</div>
      <div class="card-sub">Latest runs for ${s}.</div>
      ${e.runsJobId==null?r`
              <div class="muted" style="margin-top: 12px">Select a job to inspect run history.</div>
            `:i.length===0?r`
                <div class="muted" style="margin-top: 12px">No runs yet.</div>
              `:r`
              <div class="list" style="margin-top: 12px;">
                ${i.map(a=>I1(a,e.basePath))}
              </div>
            `}
    </section>
  `}function T1(e){const t=e.form;return t.scheduleKind==="at"?r`
      <label class="field" style="margin-top: 12px;">
        <span>Run at</span>
        <input
          type="datetime-local"
          .value=${t.scheduleAt}
          @input=${n=>e.onFormChange({scheduleAt:n.target.value})}
        />
      </label>
    `:t.scheduleKind==="every"?r`
      <div class="form-grid" style="margin-top: 12px;">
        <label class="field">
          <span>Every</span>
          <input
            .value=${t.everyAmount}
            @input=${n=>e.onFormChange({everyAmount:n.target.value})}
          />
        </label>
        <label class="field">
          <span>Unit</span>
          <select
            .value=${t.everyUnit}
            @change=${n=>e.onFormChange({everyUnit:n.target.value})}
          >
            <option value="minutes">Minutes</option>
            <option value="hours">Hours</option>
            <option value="days">Days</option>
          </select>
        </label>
      </div>
    `:r`
    <div class="form-grid" style="margin-top: 12px;">
      <label class="field">
        <span>Expression</span>
        <input
          .value=${t.cronExpr}
          @input=${n=>e.onFormChange({cronExpr:n.target.value})}
        />
      </label>
      <label class="field">
        <span>Timezone (optional)</span>
        <input
          .value=${t.cronTz}
          @input=${n=>e.onFormChange({cronTz:n.target.value})}
        />
      </label>
    </div>
  `}function E1(e,t){const s=`list-item list-item-clickable cron-job${t.runsJobId===e.id?" list-item-selected":""}`;return r`
    <div class=${s} @click=${()=>t.onLoadRuns(e.id)}>
      <div class="list-main">
        <div class="list-title">${e.name}</div>
        <div class="list-sub">${Dl(e)}</div>
        ${L1(e)}
        ${e.agentId?r`<div class="muted cron-job-agent">Agent: ${e.agentId}</div>`:m}
      </div>
      <div class="list-meta">
        ${M1(e)}
      </div>
      <div class="cron-job-footer">
        <div class="chip-row cron-job-chips">
          <span class=${`chip ${e.enabled?"chip-ok":"chip-danger"}`}>
            ${e.enabled?"enabled":"disabled"}
          </span>
          <span class="chip">${e.sessionTarget}</span>
          <span class="chip">${e.wakeMode}</span>
        </div>
        <div class="row cron-job-actions">
          <button
            class="btn"
            ?disabled=${t.busy}
            @click=${i=>{i.stopPropagation(),t.onToggle(e,!e.enabled)}}
          >
            ${e.enabled?"Disable":"Enable"}
          </button>
          <button
            class="btn"
            ?disabled=${t.busy}
            @click=${i=>{i.stopPropagation(),t.onRun(e)}}
          >
            Run
          </button>
          <button
            class="btn"
            ?disabled=${t.busy}
            @click=${i=>{i.stopPropagation(),t.onLoadRuns(e.id)}}
          >
            History
          </button>
          <button
            class="btn danger"
            ?disabled=${t.busy}
            @click=${i=>{i.stopPropagation(),t.onRemove(e)}}
          >
            Remove
          </button>
        </div>
      </div>
    </div>
  `}function L1(e){if(e.payload.kind==="systemEvent")return r`<div class="cron-job-detail">
      <span class="cron-job-detail-label">System</span>
      <span class="muted cron-job-detail-value">${e.payload.text}</span>
    </div>`;const t=e.delivery,n=t?.channel||t?.to?` (${t.channel??"last"}${t.to?` -> ${t.to}`:""})`:"";return r`
    <div class="cron-job-detail">
      <span class="cron-job-detail-label">Prompt</span>
      <span class="muted cron-job-detail-value">${e.payload.message}</span>
    </div>
    ${t?r`<div class="cron-job-detail">
            <span class="cron-job-detail-label">Delivery</span>
            <span class="muted cron-job-detail-value">${t.mode}${n}</span>
          </div>`:m}
  `}function mr(e){return typeof e!="number"||!Number.isFinite(e)?"n/a":Y(e)}function M1(e){const t=e.state?.lastStatus??"n/a",n=t==="ok"?"cron-job-status-ok":t==="error"?"cron-job-status-error":t==="skipped"?"cron-job-status-skipped":"cron-job-status-na",s=e.state?.nextRunAtMs,i=e.state?.lastRunAtMs;return r`
    <div class="cron-job-state">
      <div class="cron-job-state-row">
        <span class="cron-job-state-key">Status</span>
        <span class=${`cron-job-status-pill ${n}`}>${t}</span>
      </div>
      <div class="cron-job-state-row">
        <span class="cron-job-state-key">Next</span>
        <span class="cron-job-state-value" title=${$t(s)}>
          ${mr(s)}
        </span>
      </div>
      <div class="cron-job-state-row">
        <span class="cron-job-state-key">Last</span>
        <span class="cron-job-state-value" title=${$t(i)}>
          ${mr(i)}
        </span>
      </div>
    </div>
  `}function I1(e,t){const n=typeof e.sessionKey=="string"&&e.sessionKey.trim().length>0?`${gs("chat",t)}?session=${encodeURIComponent(e.sessionKey)}`:null;return r`
    <div class="list-item">
      <div class="list-main">
        <div class="list-title">${e.status}</div>
        <div class="list-sub">${e.summary??""}</div>
      </div>
      <div class="list-meta">
        <div>${$t(e.ts)}</div>
        <div class="muted">${e.durationMs??0}ms</div>
        ${n?r`<div><a class="session-link" href=${n}>Open run chat</a></div>`:m}
        ${e.error?r`<div class="muted">${e.error}</div>`:m}
      </div>
    </div>
  `}function R1(e){const n=(e.status&&typeof e.status=="object"?e.status.securityAudit:null)?.summary??null,s=n?.critical??0,i=n?.warn??0,a=n?.info??0,o=s>0?"danger":i>0?"warn":"success",l=s>0?`${s} critical`:i>0?`${i} warnings`:"No critical issues";return r`
    <section class="grid grid-cols-2">
      <div class="card">
        <div class="row" style="justify-content: space-between;">
          <div>
            <div class="card-title">Snapshots</div>
            <div class="card-sub">Status, health, and heartbeat data.</div>
          </div>
          <button class="btn" ?disabled=${e.loading} @click=${e.onRefresh}>
            ${e.loading?"Refreshing…":"Refresh"}
          </button>
        </div>
        <div class="stack" style="margin-top: 12px;">
          <div>
            <div class="muted">Status</div>
            ${n?r`<div class="callout ${o}" style="margin-top: 8px;">
                  Security audit: ${l}${a>0?` · ${a} info`:""}. Run
                  <span class="mono">aisopod security audit --deep</span> for details.
                </div>`:m}
            <pre class="code-block">${JSON.stringify(e.status??{},null,2)}</pre>
          </div>
          <div>
            <div class="muted">Health</div>
            <pre class="code-block">${JSON.stringify(e.health??{},null,2)}</pre>
          </div>
          <div>
            <div class="muted">Last heartbeat</div>
            <pre class="code-block">${JSON.stringify(e.heartbeat??{},null,2)}</pre>
          </div>
        </div>
      </div>

      <div class="card">
        <div class="card-title">Manual RPC</div>
        <div class="card-sub">Send a raw gateway method with JSON params.</div>
        <div class="form-grid" style="margin-top: 16px;">
          <label class="field">
            <span>Method</span>
            <input
              .value=${e.callMethod}
              @input=${d=>e.onCallMethodChange(d.target.value)}
              placeholder="system-presence"
            />
          </label>
          <label class="field">
            <span>Params (JSON)</span>
            <textarea
              .value=${e.callParams}
              @input=${d=>e.onCallParamsChange(d.target.value)}
              rows="6"
            ></textarea>
          </label>
        </div>
        <div class="row" style="margin-top: 12px;">
          <button class="btn primary" @click=${e.onCall}>Call</button>
        </div>
        ${e.callError?r`<div class="callout danger" style="margin-top: 12px;">
              ${e.callError}
            </div>`:m}
        ${e.callResult?r`<pre class="code-block" style="margin-top: 12px;">${e.callResult}</pre>`:m}
      </div>
    </section>

    <section class="card" style="margin-top: 18px;">
      <div class="card-title">Models</div>
      <div class="card-sub">Catalog from models.list.</div>
      <pre class="code-block" style="margin-top: 12px;">${JSON.stringify(e.models??[],null,2)}</pre>
    </section>

    <section class="card" style="margin-top: 18px;">
      <div class="card-title">Event Log</div>
      <div class="card-sub">Latest gateway events.</div>
      ${e.eventLog.length===0?r`
              <div class="muted" style="margin-top: 12px">No events yet.</div>
            `:r`
            <div class="list" style="margin-top: 12px;">
              ${e.eventLog.map(d=>r`
                  <div class="list-item">
                    <div class="list-main">
                      <div class="list-title">${d.event}</div>
                      <div class="list-sub">${new Date(d.ts).toLocaleTimeString()}</div>
                    </div>
                    <div class="list-meta">
                      <pre class="code-block">${Rm(d.payload)}</pre>
                    </div>
                  </div>
                `)}
            </div>
          `}
    </section>
  `}function P1(e){const t=Math.max(0,e),n=Math.floor(t/1e3);if(n<60)return`${n}s`;const s=Math.floor(n/60);return s<60?`${s}m`:`${Math.floor(s/60)}h`}function dt(e,t){return t?r`<div class="exec-approval-meta-row"><span>${e}</span><span>${t}</span></div>`:m}function D1(e){const t=e.execApprovalQueue[0];if(!t)return m;const n=t.request,s=t.expiresAtMs-Date.now(),i=s>0?`expires in ${P1(s)}`:"expired",a=e.execApprovalQueue.length;return r`
    <div class="exec-approval-overlay" role="dialog" aria-live="polite">
      <div class="exec-approval-card">
        <div class="exec-approval-header">
          <div>
            <div class="exec-approval-title">Exec approval needed</div>
            <div class="exec-approval-sub">${i}</div>
          </div>
          ${a>1?r`<div class="exec-approval-queue">${a} pending</div>`:m}
        </div>
        <div class="exec-approval-command mono">${n.command}</div>
        <div class="exec-approval-meta">
          ${dt("Host",n.host)}
          ${dt("Agent",n.agentId)}
          ${dt("Session",n.sessionKey)}
          ${dt("CWD",n.cwd)}
          ${dt("Resolved",n.resolvedPath)}
          ${dt("Security",n.security)}
          ${dt("Ask",n.ask)}
        </div>
        ${e.execApprovalError?r`<div class="exec-approval-error">${e.execApprovalError}</div>`:m}
        <div class="exec-approval-actions">
          <button
            class="btn primary"
            ?disabled=${e.execApprovalBusy}
            @click=${()=>e.handleExecApprovalDecision("allow-once")}
          >
            Allow once
          </button>
          <button
            class="btn"
            ?disabled=${e.execApprovalBusy}
            @click=${()=>e.handleExecApprovalDecision("allow-always")}
          >
            Always allow
          </button>
          <button
            class="btn danger"
            ?disabled=${e.execApprovalBusy}
            @click=${()=>e.handleExecApprovalDecision("deny")}
          >
            Deny
          </button>
        </div>
      </div>
    </div>
  `}function F1(e){const{pendingGatewayUrl:t}=e;return t?r`
    <div class="exec-approval-overlay" role="dialog" aria-modal="true" aria-live="polite">
      <div class="exec-approval-card">
        <div class="exec-approval-header">
          <div>
            <div class="exec-approval-title">Change Gateway URL</div>
            <div class="exec-approval-sub">This will reconnect to a different gateway server</div>
          </div>
        </div>
        <div class="exec-approval-command mono">${t}</div>
        <div class="callout danger" style="margin-top: 12px;">
          Only confirm if you trust this URL. Malicious URLs can compromise your system.
        </div>
        <div class="exec-approval-actions">
          <button
            class="btn primary"
            @click=${()=>e.handleGatewayUrlConfirm()}
          >
            Confirm
          </button>
          <button
            class="btn"
            @click=${()=>e.handleGatewayUrlCancel()}
          >
            Cancel
          </button>
        </div>
      </div>
    </div>
  `:m}function N1(e){return r`
    <section class="card">
      <div class="row" style="justify-content: space-between;">
        <div>
          <div class="card-title">Connected Instances</div>
          <div class="card-sub">Presence beacons from the gateway and clients.</div>
        </div>
        <button class="btn" ?disabled=${e.loading} @click=${e.onRefresh}>
          ${e.loading?"Loading…":"Refresh"}
        </button>
      </div>
      ${e.lastError?r`<div class="callout danger" style="margin-top: 12px;">
            ${e.lastError}
          </div>`:m}
      ${e.statusMessage?r`<div class="callout" style="margin-top: 12px;">
            ${e.statusMessage}
          </div>`:m}
      <div class="list" style="margin-top: 16px;">
        ${e.entries.length===0?r`
                <div class="muted">No instances reported yet.</div>
              `:e.entries.map(t=>O1(t))}
      </div>
    </section>
  `}function O1(e){const t=e.lastInputSeconds!=null?`${e.lastInputSeconds}s ago`:"n/a",n=e.mode??"unknown",s=Array.isArray(e.roles)?e.roles.filter(Boolean):[],i=Array.isArray(e.scopes)?e.scopes.filter(Boolean):[],a=i.length>0?i.length>3?`${i.length} scopes`:`scopes: ${i.join(", ")}`:null;return r`
    <div class="list-item">
      <div class="list-main">
        <div class="list-title">${e.host??"unknown host"}</div>
        <div class="list-sub">${Lm(e)}</div>
        <div class="chip-row">
          <span class="chip">${n}</span>
          ${s.map(o=>r`<span class="chip">${o}</span>`)}
          ${a?r`<span class="chip">${a}</span>`:m}
          ${e.platform?r`<span class="chip">${e.platform}</span>`:m}
          ${e.deviceFamily?r`<span class="chip">${e.deviceFamily}</span>`:m}
          ${e.modelIdentifier?r`<span class="chip">${e.modelIdentifier}</span>`:m}
          ${e.version?r`<span class="chip">${e.version}</span>`:m}
        </div>
      </div>
      <div class="list-meta">
        <div>${Mm(e)}</div>
        <div class="muted">Last input ${t}</div>
        <div class="muted">Reason ${e.reason??""}</div>
      </div>
    </div>
  `}const br=["trace","debug","info","warn","error","fatal"];function B1(e){if(!e)return"";const t=new Date(e);return Number.isNaN(t.getTime())?e:t.toLocaleTimeString()}function U1(e,t){return t?[e.message,e.subsystem,e.raw].filter(Boolean).join(" ").toLowerCase().includes(t):!0}function z1(e){const t=e.filterText.trim().toLowerCase(),n=br.some(a=>!e.levelFilters[a]),s=e.entries.filter(a=>a.level&&!e.levelFilters[a.level]?!1:U1(a,t)),i=t||n?"filtered":"visible";return r`
    <section class="card">
      <div class="row" style="justify-content: space-between;">
        <div>
          <div class="card-title">Logs</div>
          <div class="card-sub">Gateway file logs (JSONL).</div>
        </div>
        <div class="row" style="gap: 8px;">
          <button class="btn" ?disabled=${e.loading} @click=${e.onRefresh}>
            ${e.loading?"Loading…":"Refresh"}
          </button>
          <button
            class="btn"
            ?disabled=${s.length===0}
            @click=${()=>e.onExport(s.map(a=>a.raw),i)}
          >
            Export ${i}
          </button>
        </div>
      </div>

      <div class="filters" style="margin-top: 14px;">
        <label class="field" style="min-width: 220px;">
          <span>Filter</span>
          <input
            .value=${e.filterText}
            @input=${a=>e.onFilterTextChange(a.target.value)}
            placeholder="Search logs"
          />
        </label>
        <label class="field checkbox">
          <span>Auto-follow</span>
          <input
            type="checkbox"
            .checked=${e.autoFollow}
            @change=${a=>e.onToggleAutoFollow(a.target.checked)}
          />
        </label>
      </div>

      <div class="chip-row" style="margin-top: 12px;">
        ${br.map(a=>r`
            <label class="chip log-chip ${a}">
              <input
                type="checkbox"
                .checked=${e.levelFilters[a]}
                @change=${o=>e.onLevelToggle(a,o.target.checked)}
              />
              <span>${a}</span>
            </label>
          `)}
      </div>

      ${e.file?r`<div class="muted" style="margin-top: 10px;">File: ${e.file}</div>`:m}
      ${e.truncated?r`
              <div class="callout" style="margin-top: 10px">Log output truncated; showing latest chunk.</div>
            `:m}
      ${e.error?r`<div class="callout danger" style="margin-top: 10px;">${e.error}</div>`:m}

      <div class="log-stream" style="margin-top: 12px;" @scroll=${e.onScroll}>
        ${s.length===0?r`
                <div class="muted" style="padding: 12px">No log entries.</div>
              `:s.map(a=>r`
                <div class="log-row">
                  <div class="log-time mono">${B1(a.time)}</div>
                  <div class="log-level ${a.level??""}">${a.level??""}</div>
                  <div class="log-subsystem mono">${a.subsystem??""}</div>
                  <div class="log-message mono">${a.message??a.raw}</div>
                </div>
              `)}
      </div>
    </section>
  `}const Xe="__defaults__",yr=[{value:"deny",label:"Deny"},{value:"allowlist",label:"Allowlist"},{value:"full",label:"Full"}],H1=[{value:"off",label:"Off"},{value:"on-miss",label:"On miss"},{value:"always",label:"Always"}];function xr(e){return e==="allowlist"||e==="full"||e==="deny"?e:"deny"}function K1(e){return e==="always"||e==="off"||e==="on-miss"?e:"on-miss"}function j1(e){const t=e?.defaults??{};return{security:xr(t.security),ask:K1(t.ask),askFallback:xr(t.askFallback??"deny"),autoAllowSkills:!!(t.autoAllowSkills??!1)}}function W1(e){const t=e?.agents??{},n=Array.isArray(t.list)?t.list:[],s=[];return n.forEach(i=>{if(!i||typeof i!="object")return;const a=i,o=typeof a.id=="string"?a.id.trim():"";if(!o)return;const l=typeof a.name=="string"?a.name.trim():void 0,d=a.default===!0;s.push({id:o,name:l||void 0,isDefault:d})}),s}function q1(e,t){const n=W1(e),s=Object.keys(t?.agents??{}),i=new Map;n.forEach(o=>i.set(o.id,o)),s.forEach(o=>{i.has(o)||i.set(o,{id:o})});const a=Array.from(i.values());return a.length===0&&a.push({id:"main",isDefault:!0}),a.sort((o,l)=>{if(o.isDefault&&!l.isDefault)return-1;if(!o.isDefault&&l.isDefault)return 1;const d=o.name?.trim()?o.name:o.id,g=l.name?.trim()?l.name:l.id;return d.localeCompare(g)}),a}function G1(e,t){return e===Xe?Xe:e&&t.some(n=>n.id===e)?e:Xe}function V1(e){const t=e.execApprovalsForm??e.execApprovalsSnapshot?.file??null,n=!!t,s=j1(t),i=q1(e.configForm,t),a=tx(e.nodes),o=e.execApprovalsTarget;let l=o==="node"&&e.execApprovalsTargetNodeId?e.execApprovalsTargetNodeId:null;o==="node"&&l&&!a.some(p=>p.id===l)&&(l=null);const d=G1(e.execApprovalsSelectedAgent,i),g=d!==Xe?(t?.agents??{})[d]??null:null,f=Array.isArray(g?.allowlist)?g.allowlist??[]:[];return{ready:n,disabled:e.execApprovalsSaving||e.execApprovalsLoading,dirty:e.execApprovalsDirty,loading:e.execApprovalsLoading,saving:e.execApprovalsSaving,form:t,defaults:s,selectedScope:d,selectedAgent:g,agents:i,allowlist:f,target:o,targetNodeId:l,targetNodes:a,onSelectScope:e.onExecApprovalsSelectAgent,onSelectTarget:e.onExecApprovalsTargetChange,onPatch:e.onExecApprovalsPatch,onRemove:e.onExecApprovalsRemove,onLoad:e.onLoadExecApprovals,onSave:e.onSaveExecApprovals}}function Q1(e){const t=e.ready,n=e.target!=="node"||!!e.targetNodeId;return r`
    <section class="card">
      <div class="row" style="justify-content: space-between; align-items: center;">
        <div>
          <div class="card-title">Exec approvals</div>
          <div class="card-sub">
            Allowlist and approval policy for <span class="mono">exec host=gateway/node</span>.
          </div>
        </div>
        <button
          class="btn"
          ?disabled=${e.disabled||!e.dirty||!n}
          @click=${e.onSave}
        >
          ${e.saving?"Saving…":"Save"}
        </button>
      </div>

      ${Y1(e)}

      ${t?r`
            ${J1(e)}
            ${Z1(e)}
            ${e.selectedScope===Xe?m:X1(e)}
          `:r`<div class="row" style="margin-top: 12px; gap: 12px;">
            <div class="muted">Load exec approvals to edit allowlists.</div>
            <button class="btn" ?disabled=${e.loading||!n} @click=${e.onLoad}>
              ${e.loading?"Loading…":"Load approvals"}
            </button>
          </div>`}
    </section>
  `}function Y1(e){const t=e.targetNodes.length>0,n=e.targetNodeId??"";return r`
    <div class="list" style="margin-top: 12px;">
      <div class="list-item">
        <div class="list-main">
          <div class="list-title">Target</div>
          <div class="list-sub">
            Gateway edits local approvals; node edits the selected node.
          </div>
        </div>
        <div class="list-meta">
          <label class="field">
            <span>Host</span>
            <select
              ?disabled=${e.disabled}
              @change=${s=>{if(s.target.value==="node"){const o=e.targetNodes[0]?.id??null;e.onSelectTarget("node",n||o)}else e.onSelectTarget("gateway",null)}}
            >
              <option value="gateway" ?selected=${e.target==="gateway"}>Gateway</option>
              <option value="node" ?selected=${e.target==="node"}>Node</option>
            </select>
          </label>
          ${e.target==="node"?r`
                <label class="field">
                  <span>Node</span>
                  <select
                    ?disabled=${e.disabled||!t}
                    @change=${s=>{const a=s.target.value.trim();e.onSelectTarget("node",a||null)}}
                  >
                    <option value="" ?selected=${n===""}>Select node</option>
                    ${e.targetNodes.map(s=>r`<option
                          value=${s.id}
                          ?selected=${n===s.id}
                        >
                          ${s.label}
                        </option>`)}
                  </select>
                </label>
              `:m}
        </div>
      </div>
      ${e.target==="node"&&!t?r`
              <div class="muted">No nodes advertise exec approvals yet.</div>
            `:m}
    </div>
  `}function J1(e){return r`
    <div class="row" style="margin-top: 12px; gap: 8px; flex-wrap: wrap;">
      <span class="label">Scope</span>
      <div class="row" style="gap: 8px; flex-wrap: wrap;">
        <button
          class="btn btn--sm ${e.selectedScope===Xe?"active":""}"
          @click=${()=>e.onSelectScope(Xe)}
        >
          Defaults
        </button>
        ${e.agents.map(t=>{const n=t.name?.trim()?`${t.name} (${t.id})`:t.id;return r`
            <button
              class="btn btn--sm ${e.selectedScope===t.id?"active":""}"
              @click=${()=>e.onSelectScope(t.id)}
            >
              ${n}
            </button>
          `})}
      </div>
    </div>
  `}function Z1(e){const t=e.selectedScope===Xe,n=e.defaults,s=e.selectedAgent??{},i=t?["defaults"]:["agents",e.selectedScope],a=typeof s.security=="string"?s.security:void 0,o=typeof s.ask=="string"?s.ask:void 0,l=typeof s.askFallback=="string"?s.askFallback:void 0,d=t?n.security:a??"__default__",g=t?n.ask:o??"__default__",f=t?n.askFallback:l??"__default__",p=typeof s.autoAllowSkills=="boolean"?s.autoAllowSkills:void 0,b=p??n.autoAllowSkills,u=p==null;return r`
    <div class="list" style="margin-top: 16px;">
      <div class="list-item">
        <div class="list-main">
          <div class="list-title">Security</div>
          <div class="list-sub">
            ${t?"Default security mode.":`Default: ${n.security}.`}
          </div>
        </div>
        <div class="list-meta">
          <label class="field">
            <span>Mode</span>
            <select
              ?disabled=${e.disabled}
              @change=${v=>{const k=v.target.value;!t&&k==="__default__"?e.onRemove([...i,"security"]):e.onPatch([...i,"security"],k)}}
            >
              ${t?m:r`<option value="__default__" ?selected=${d==="__default__"}>
                    Use default (${n.security})
                  </option>`}
              ${yr.map(v=>r`<option
                    value=${v.value}
                    ?selected=${d===v.value}
                  >
                    ${v.label}
                  </option>`)}
            </select>
          </label>
        </div>
      </div>

      <div class="list-item">
        <div class="list-main">
          <div class="list-title">Ask</div>
          <div class="list-sub">
            ${t?"Default prompt policy.":`Default: ${n.ask}.`}
          </div>
        </div>
        <div class="list-meta">
          <label class="field">
            <span>Mode</span>
            <select
              ?disabled=${e.disabled}
              @change=${v=>{const k=v.target.value;!t&&k==="__default__"?e.onRemove([...i,"ask"]):e.onPatch([...i,"ask"],k)}}
            >
              ${t?m:r`<option value="__default__" ?selected=${g==="__default__"}>
                    Use default (${n.ask})
                  </option>`}
              ${H1.map(v=>r`<option
                    value=${v.value}
                    ?selected=${g===v.value}
                  >
                    ${v.label}
                  </option>`)}
            </select>
          </label>
        </div>
      </div>

      <div class="list-item">
        <div class="list-main">
          <div class="list-title">Ask fallback</div>
          <div class="list-sub">
            ${t?"Applied when the UI prompt is unavailable.":`Default: ${n.askFallback}.`}
          </div>
        </div>
        <div class="list-meta">
          <label class="field">
            <span>Fallback</span>
            <select
              ?disabled=${e.disabled}
              @change=${v=>{const k=v.target.value;!t&&k==="__default__"?e.onRemove([...i,"askFallback"]):e.onPatch([...i,"askFallback"],k)}}
            >
              ${t?m:r`<option value="__default__" ?selected=${f==="__default__"}>
                    Use default (${n.askFallback})
                  </option>`}
              ${yr.map(v=>r`<option
                    value=${v.value}
                    ?selected=${f===v.value}
                  >
                    ${v.label}
                  </option>`)}
            </select>
          </label>
        </div>
      </div>

      <div class="list-item">
        <div class="list-main">
          <div class="list-title">Auto-allow skill CLIs</div>
          <div class="list-sub">
            ${t?"Allow skill executables listed by the Gateway.":u?`Using default (${n.autoAllowSkills?"on":"off"}).`:`Override (${b?"on":"off"}).`}
          </div>
        </div>
        <div class="list-meta">
          <label class="field">
            <span>Enabled</span>
            <input
              type="checkbox"
              ?disabled=${e.disabled}
              .checked=${b}
              @change=${v=>{const y=v.target;e.onPatch([...i,"autoAllowSkills"],y.checked)}}
            />
          </label>
          ${!t&&!u?r`<button
                class="btn btn--sm"
                ?disabled=${e.disabled}
                @click=${()=>e.onRemove([...i,"autoAllowSkills"])}
              >
                Use default
              </button>`:m}
        </div>
      </div>
    </div>
  `}function X1(e){const t=["agents",e.selectedScope,"allowlist"],n=e.allowlist;return r`
    <div class="row" style="margin-top: 18px; justify-content: space-between;">
      <div>
        <div class="card-title">Allowlist</div>
        <div class="card-sub">Case-insensitive glob patterns.</div>
      </div>
      <button
        class="btn btn--sm"
        ?disabled=${e.disabled}
        @click=${()=>{const s=[...n,{pattern:""}];e.onPatch(t,s)}}
      >
        Add pattern
      </button>
    </div>
    <div class="list" style="margin-top: 12px;">
      ${n.length===0?r`
              <div class="muted">No allowlist entries yet.</div>
            `:n.map((s,i)=>ex(e,s,i))}
    </div>
  `}function ex(e,t,n){const s=t.lastUsedAt?Y(t.lastUsedAt):"never",i=t.lastUsedCommand?si(t.lastUsedCommand,120):null,a=t.lastResolvedPath?si(t.lastResolvedPath,120):null;return r`
    <div class="list-item">
      <div class="list-main">
        <div class="list-title">${t.pattern?.trim()?t.pattern:"New pattern"}</div>
        <div class="list-sub">Last used: ${s}</div>
        ${i?r`<div class="list-sub mono">${i}</div>`:m}
        ${a?r`<div class="list-sub mono">${a}</div>`:m}
      </div>
      <div class="list-meta">
        <label class="field">
          <span>Pattern</span>
          <input
            type="text"
            .value=${t.pattern??""}
            ?disabled=${e.disabled}
            @input=${o=>{const l=o.target;e.onPatch(["agents",e.selectedScope,"allowlist",n,"pattern"],l.value)}}
          />
        </label>
        <button
          class="btn btn--sm danger"
          ?disabled=${e.disabled}
          @click=${()=>{if(e.allowlist.length<=1){e.onRemove(["agents",e.selectedScope,"allowlist"]);return}e.onRemove(["agents",e.selectedScope,"allowlist",n])}}
        >
          Remove
        </button>
      </div>
    </div>
  `}function tx(e){const t=[];for(const n of e){if(!(Array.isArray(n.commands)?n.commands:[]).some(l=>String(l)==="system.execApprovals.get"||String(l)==="system.execApprovals.set"))continue;const a=typeof n.nodeId=="string"?n.nodeId.trim():"";if(!a)continue;const o=typeof n.displayName=="string"&&n.displayName.trim()?n.displayName.trim():a;t.push({id:a,label:o===a?a:`${o} · ${a}`})}return t.sort((n,s)=>n.label.localeCompare(s.label)),t}function nx(e){const t=rx(e),n=V1(e);return r`
    ${Q1(n)}
    ${lx(t)}
    ${sx(e)}
    <section class="card">
      <div class="row" style="justify-content: space-between;">
        <div>
          <div class="card-title">Nodes</div>
          <div class="card-sub">Paired devices and live links.</div>
        </div>
        <button class="btn" ?disabled=${e.loading} @click=${e.onRefresh}>
          ${e.loading?"Loading…":"Refresh"}
        </button>
      </div>
      <div class="list" style="margin-top: 16px;">
        ${e.nodes.length===0?r`
                <div class="muted">No nodes found.</div>
              `:e.nodes.map(s=>gx(s))}
      </div>
    </section>
  `}function sx(e){const t=e.devicesList??{pending:[],paired:[]},n=Array.isArray(t.pending)?t.pending:[],s=Array.isArray(t.paired)?t.paired:[];return r`
    <section class="card">
      <div class="row" style="justify-content: space-between;">
        <div>
          <div class="card-title">Devices</div>
          <div class="card-sub">Pairing requests + role tokens.</div>
        </div>
        <button class="btn" ?disabled=${e.devicesLoading} @click=${e.onDevicesRefresh}>
          ${e.devicesLoading?"Loading…":"Refresh"}
        </button>
      </div>
      ${e.devicesError?r`<div class="callout danger" style="margin-top: 12px;">${e.devicesError}</div>`:m}
      <div class="list" style="margin-top: 16px;">
        ${n.length>0?r`
              <div class="muted" style="margin-bottom: 8px;">Pending</div>
              ${n.map(i=>ix(i,e))}
            `:m}
        ${s.length>0?r`
              <div class="muted" style="margin-top: 12px; margin-bottom: 8px;">Paired</div>
              ${s.map(i=>ax(i,e))}
            `:m}
        ${n.length===0&&s.length===0?r`
                <div class="muted">No paired devices.</div>
              `:m}
      </div>
    </section>
  `}function ix(e,t){const n=e.displayName?.trim()||e.deviceId,s=typeof e.ts=="number"?Y(e.ts):"n/a",i=e.role?.trim()?`role: ${e.role}`:"role: -",a=e.isRepair?" · repair":"",o=e.remoteIp?` · ${e.remoteIp}`:"";return r`
    <div class="list-item">
      <div class="list-main">
        <div class="list-title">${n}</div>
        <div class="list-sub">${e.deviceId}${o}</div>
        <div class="muted" style="margin-top: 6px;">
          ${i} · requested ${s}${a}
        </div>
      </div>
      <div class="list-meta">
        <div class="row" style="justify-content: flex-end; gap: 8px; flex-wrap: wrap;">
          <button class="btn btn--sm primary" @click=${()=>t.onDeviceApprove(e.requestId)}>
            Approve
          </button>
          <button class="btn btn--sm" @click=${()=>t.onDeviceReject(e.requestId)}>
            Reject
          </button>
        </div>
      </div>
    </div>
  `}function ax(e,t){const n=e.displayName?.trim()||e.deviceId,s=e.remoteIp?` · ${e.remoteIp}`:"",i=`roles: ${ni(e.roles)}`,a=`scopes: ${ni(e.scopes)}`,o=Array.isArray(e.tokens)?e.tokens:[];return r`
    <div class="list-item">
      <div class="list-main">
        <div class="list-title">${n}</div>
        <div class="list-sub">${e.deviceId}${s}</div>
        <div class="muted" style="margin-top: 6px;">${i} · ${a}</div>
        ${o.length===0?r`
                <div class="muted" style="margin-top: 6px">Tokens: none</div>
              `:r`
              <div class="muted" style="margin-top: 10px;">Tokens</div>
              <div style="display: flex; flex-direction: column; gap: 8px; margin-top: 6px;">
                ${o.map(l=>ox(e.deviceId,l,t))}
              </div>
            `}
      </div>
    </div>
  `}function ox(e,t,n){const s=t.revokedAtMs?"revoked":"active",i=`scopes: ${ni(t.scopes)}`,a=Y(t.rotatedAtMs??t.createdAtMs??t.lastUsedAtMs??null);return r`
    <div class="row" style="justify-content: space-between; gap: 8px;">
      <div class="list-sub">${t.role} · ${s} · ${i} · ${a}</div>
      <div class="row" style="justify-content: flex-end; gap: 6px; flex-wrap: wrap;">
        <button
          class="btn btn--sm"
          @click=${()=>n.onDeviceRotate(e,t.role,t.scopes)}
        >
          Rotate
        </button>
        ${t.revokedAtMs?m:r`
              <button
                class="btn btn--sm danger"
                @click=${()=>n.onDeviceRevoke(e,t.role)}
              >
                Revoke
              </button>
            `}
      </div>
    </div>
  `}function rx(e){const t=e.configForm,n=dx(e.nodes),{defaultBinding:s,agents:i}=ux(t),a=!!t,o=e.configSaving||e.configFormMode==="raw";return{ready:a,disabled:o,configDirty:e.configDirty,configLoading:e.configLoading,configSaving:e.configSaving,defaultBinding:s,agents:i,nodes:n,onBindDefault:e.onBindDefault,onBindAgent:e.onBindAgent,onSave:e.onSaveBindings,onLoadConfig:e.onLoadConfig,formMode:e.configFormMode}}function lx(e){const t=e.nodes.length>0,n=e.defaultBinding??"";return r`
    <section class="card">
      <div class="row" style="justify-content: space-between; align-items: center;">
        <div>
          <div class="card-title">Exec node binding</div>
          <div class="card-sub">
            Pin agents to a specific node when using <span class="mono">exec host=node</span>.
          </div>
        </div>
        <button
          class="btn"
          ?disabled=${e.disabled||!e.configDirty}
          @click=${e.onSave}
        >
          ${e.configSaving?"Saving…":"Save"}
        </button>
      </div>

      ${e.formMode==="raw"?r`
              <div class="callout warn" style="margin-top: 12px">
                Switch the Config tab to <strong>Form</strong> mode to edit bindings here.
              </div>
            `:m}

      ${e.ready?r`
            <div class="list" style="margin-top: 16px;">
              <div class="list-item">
                <div class="list-main">
                  <div class="list-title">Default binding</div>
                  <div class="list-sub">Used when agents do not override a node binding.</div>
                </div>
                <div class="list-meta">
                  <label class="field">
                    <span>Node</span>
                    <select
                      ?disabled=${e.disabled||!t}
                      @change=${s=>{const a=s.target.value.trim();e.onBindDefault(a||null)}}
                    >
                      <option value="" ?selected=${n===""}>Any node</option>
                      ${e.nodes.map(s=>r`<option
                            value=${s.id}
                            ?selected=${n===s.id}
                          >
                            ${s.label}
                          </option>`)}
                    </select>
                  </label>
                  ${t?m:r`
                          <div class="muted">No nodes with system.run available.</div>
                        `}
                </div>
              </div>

              ${e.agents.length===0?r`
                      <div class="muted">No agents found.</div>
                    `:e.agents.map(s=>cx(s,e))}
            </div>
          `:r`<div class="row" style="margin-top: 12px; gap: 12px;">
            <div class="muted">Load config to edit bindings.</div>
            <button class="btn" ?disabled=${e.configLoading} @click=${e.onLoadConfig}>
              ${e.configLoading?"Loading…":"Load config"}
            </button>
          </div>`}
    </section>
  `}function cx(e,t){const n=e.binding??"__default__",s=e.name?.trim()?`${e.name} (${e.id})`:e.id,i=t.nodes.length>0;return r`
    <div class="list-item">
      <div class="list-main">
        <div class="list-title">${s}</div>
        <div class="list-sub">
          ${e.isDefault?"default agent":"agent"} ·
          ${n==="__default__"?`uses default (${t.defaultBinding??"any"})`:`override: ${e.binding}`}
        </div>
      </div>
      <div class="list-meta">
        <label class="field">
          <span>Binding</span>
          <select
            ?disabled=${t.disabled||!i}
            @change=${a=>{const l=a.target.value.trim();t.onBindAgent(e.index,l==="__default__"?null:l)}}
          >
            <option value="__default__" ?selected=${n==="__default__"}>
              Use default
            </option>
            ${t.nodes.map(a=>r`<option
                  value=${a.id}
                  ?selected=${n===a.id}
                >
                  ${a.label}
                </option>`)}
          </select>
        </label>
      </div>
    </div>
  `}function dx(e){const t=[];for(const n of e){if(!(Array.isArray(n.commands)?n.commands:[]).some(l=>String(l)==="system.run"))continue;const a=typeof n.nodeId=="string"?n.nodeId.trim():"";if(!a)continue;const o=typeof n.displayName=="string"&&n.displayName.trim()?n.displayName.trim():a;t.push({id:a,label:o===a?a:`${o} · ${a}`})}return t.sort((n,s)=>n.label.localeCompare(s.label)),t}function ux(e){const t={id:"main",name:void 0,index:0,isDefault:!0,binding:null};if(!e||typeof e!="object")return{defaultBinding:null,agents:[t]};const s=(e.tools??{}).exec??{},i=typeof s.node=="string"&&s.node.trim()?s.node.trim():null,a=e.agents??{},o=Array.isArray(a.list)?a.list:[];if(o.length===0)return{defaultBinding:i,agents:[t]};const l=[];return o.forEach((d,g)=>{if(!d||typeof d!="object")return;const f=d,p=typeof f.id=="string"?f.id.trim():"";if(!p)return;const b=typeof f.name=="string"?f.name.trim():void 0,u=f.default===!0,y=(f.tools??{}).exec??{},k=typeof y.node=="string"&&y.node.trim()?y.node.trim():null;l.push({id:p,name:b||void 0,index:g,isDefault:u,binding:k})}),l.length===0&&l.push(t),{defaultBinding:i,agents:l}}function gx(e){const t=!!e.connected,n=!!e.paired,s=typeof e.displayName=="string"&&e.displayName.trim()||(typeof e.nodeId=="string"?e.nodeId:"unknown"),i=Array.isArray(e.caps)?e.caps:[],a=Array.isArray(e.commands)?e.commands:[];return r`
    <div class="list-item">
      <div class="list-main">
        <div class="list-title">${s}</div>
        <div class="list-sub">
          ${typeof e.nodeId=="string"?e.nodeId:""}
          ${typeof e.remoteIp=="string"?` · ${e.remoteIp}`:""}
          ${typeof e.version=="string"?` · ${e.version}`:""}
        </div>
        <div class="chip-row" style="margin-top: 6px;">
          <span class="chip">${n?"paired":"unpaired"}</span>
          <span class="chip ${t?"chip-ok":"chip-warn"}">
            ${t?"connected":"offline"}
          </span>
          ${i.slice(0,12).map(o=>r`<span class="chip">${String(o)}</span>`)}
          ${a.slice(0,8).map(o=>r`<span class="chip">${String(o)}</span>`)}
        </div>
      </div>
    </div>
  `}function px(e){const t=e.hello?.snapshot,n=t?.uptimeMs?ji(t.uptimeMs):"n/a",s=t?.policy?.tickIntervalMs?`${t.policy.tickIntervalMs}ms`:"n/a",a=t?.authMode==="trusted-proxy",o=(()=>{if(e.connected||!e.lastError)return null;const d=e.lastError.toLowerCase();if(!(d.includes("unauthorized")||d.includes("connect failed")))return null;const f=!!e.settings.token.trim(),p=!!e.password.trim();return!f&&!p?r`
        <div class="muted" style="margin-top: 8px">
          This gateway requires auth. Add a token or password, then click Connect.
          <div style="margin-top: 6px">
            <span class="mono">aisopod dashboard --no-open</span> → open the Control UI<br />
            <span class="mono">aisopod doctor --generate-gateway-token</span> → set token
          </div>
          <div style="margin-top: 6px">
            <a
              class="session-link"
              href="https://docs.aisopod.ai/web/dashboard"
              target="_blank"
              rel="noreferrer"
              title="Control UI auth docs (opens in new tab)"
              >Docs: Control UI auth</a
            >
          </div>
        </div>
      `:r`
      <div class="muted" style="margin-top: 8px">
        Auth failed. Update the token or password in Control UI settings, then click Connect.
        <div style="margin-top: 6px">
          <a
            class="session-link"
            href="https://docs.aisopod.ai/web/dashboard"
            target="_blank"
            rel="noreferrer"
            title="Control UI auth docs (opens in new tab)"
            >Docs: Control UI auth</a
          >
        </div>
      </div>
    `})(),l=(()=>{if(e.connected||!e.lastError||(typeof window<"u"?window.isSecureContext:!0))return null;const g=e.lastError.toLowerCase();return!g.includes("secure context")&&!g.includes("device identity required")?null:r`
      <div class="muted" style="margin-top: 8px">
        This page is HTTP, so the browser blocks device identity. Use HTTPS (Tailscale Serve) or open
        <span class="mono">http://127.0.0.1:18789</span> on the gateway host.
        <div style="margin-top: 6px">
          If you must stay on HTTP, set
          <span class="mono">gateway.controlUi.allowInsecureAuth: true</span> (token-only).
        </div>
        <div style="margin-top: 6px">
          <a
            class="session-link"
            href="https://docs.aisopod.ai/gateway/tailscale"
            target="_blank"
            rel="noreferrer"
            title="Tailscale Serve docs (opens in new tab)"
            >Docs: Tailscale Serve</a
          >
          <span class="muted"> · </span>
          <a
            class="session-link"
            href="https://docs.aisopod.ai/web/control-ui#insecure-http"
            target="_blank"
            rel="noreferrer"
            title="Insecure HTTP docs (opens in new tab)"
            >Docs: Insecure HTTP</a
          >
        </div>
      </div>
    `})();return r`
    <section class="grid grid-cols-2">
      <div class="card">
        <div class="card-title">Gateway Access</div>
        <div class="card-sub">Where the dashboard connects and how it authenticates.</div>
        <div class="form-grid" style="margin-top: 16px;">
          <label class="field">
            <span>WebSocket URL</span>
            <input
              .value=${e.settings.gatewayUrl}
              @input=${d=>{const g=d.target.value;e.onSettingsChange({...e.settings,gatewayUrl:g})}}
              placeholder="ws://100.x.y.z:18789"
            />
          </label>
          ${a?"":r`
                <label class="field">
                  <span>Gateway Token</span>
                  <input
                    .value=${e.settings.token}
                    @input=${d=>{const g=d.target.value;e.onSettingsChange({...e.settings,token:g})}}
                    placeholder="AISOPOD_GATEWAY_TOKEN"
                  />
                </label>
                <label class="field">
                  <span>Password (not stored)</span>
                  <input
                    type="password"
                    .value=${e.password}
                    @input=${d=>{const g=d.target.value;e.onPasswordChange(g)}}
                    placeholder="system or shared password"
                  />
                </label>
              `}
          <label class="field">
            <span>Default Session Key</span>
            <input
              .value=${e.settings.sessionKey}
              @input=${d=>{const g=d.target.value;e.onSessionKeyChange(g)}}
            />
          </label>
        </div>
        <div class="row" style="margin-top: 14px;">
          <button class="btn" @click=${()=>e.onConnect()}>Connect</button>
          <button class="btn" @click=${()=>e.onRefresh()}>Refresh</button>
          <span class="muted">${a?"Authenticated via trusted proxy.":"Click Connect to apply connection changes."}</span>
        </div>
      </div>

      <div class="card">
        <div class="card-title">Snapshot</div>
        <div class="card-sub">Latest gateway handshake information.</div>
        <div class="stat-grid" style="margin-top: 16px;">
          <div class="stat">
            <div class="stat-label">Status</div>
            <div class="stat-value ${e.connected?"ok":"warn"}">
              ${e.connected?"Connected":"Disconnected"}
            </div>
          </div>
          <div class="stat">
            <div class="stat-label">Uptime</div>
            <div class="stat-value">${n}</div>
          </div>
          <div class="stat">
            <div class="stat-label">Tick Interval</div>
            <div class="stat-value">${s}</div>
          </div>
          <div class="stat">
            <div class="stat-label">Last Channels Refresh</div>
            <div class="stat-value">
              ${e.lastChannelsRefresh?Y(e.lastChannelsRefresh):"n/a"}
            </div>
          </div>
        </div>
        ${e.lastError?r`<div class="callout danger" style="margin-top: 14px;">
              <div>${e.lastError}</div>
              ${o??""}
              ${l??""}
            </div>`:r`
                <div class="callout" style="margin-top: 14px">
                  Use Channels to link WhatsApp, Telegram, Discord, Signal, or iMessage.
                </div>
              `}
      </div>
    </section>

    <section class="grid grid-cols-3" style="margin-top: 18px;">
      <div class="card stat-card">
        <div class="stat-label">Instances</div>
        <div class="stat-value">${e.presenceCount}</div>
        <div class="muted">Presence beacons in the last 5 minutes.</div>
      </div>
      <div class="card stat-card">
        <div class="stat-label">Sessions</div>
        <div class="stat-value">${e.sessionsCount??"n/a"}</div>
        <div class="muted">Recent session keys tracked by the gateway.</div>
      </div>
      <div class="card stat-card">
        <div class="stat-label">Cron</div>
        <div class="stat-value">
          ${e.cronEnabled==null?"n/a":e.cronEnabled?"Enabled":"Disabled"}
        </div>
        <div class="muted">Next wake ${la(e.cronNext)}</div>
      </div>
    </section>

    <section class="card" style="margin-top: 18px;">
      <div class="card-title">Notes</div>
      <div class="card-sub">Quick reminders for remote control setups.</div>
      <div class="note-grid" style="margin-top: 14px;">
        <div>
          <div class="note-title">Tailscale serve</div>
          <div class="muted">
            Prefer serve mode to keep the gateway on loopback with tailnet auth.
          </div>
        </div>
        <div>
          <div class="note-title">Session hygiene</div>
          <div class="muted">Use /new or sessions.patch to reset context.</div>
        </div>
        <div>
          <div class="note-title">Cron reminders</div>
          <div class="muted">Use isolated sessions for recurring runs.</div>
        </div>
      </div>
    </section>
  `}const hx=["","off","minimal","low","medium","high","xhigh"],fx=["","off","on"],vx=[{value:"",label:"inherit"},{value:"off",label:"off (explicit)"},{value:"on",label:"on"},{value:"full",label:"full"}],mx=["","off","on","stream"];function bx(e){if(!e)return"";const t=e.trim().toLowerCase();return t==="z.ai"||t==="z-ai"?"zai":t}function Sc(e){return bx(e)==="zai"}function yx(e){return Sc(e)?fx:hx}function $r(e,t){return t?e.includes(t)?[...e]:[...e,t]:[...e]}function xx(e,t){return t?e.some(n=>n.value===t)?[...e]:[...e,{value:t,label:`${t} (custom)`}]:[...e]}function $x(e,t){return!t||!e||e==="off"?e:"on"}function wx(e,t){return e?t&&e==="on"?"low":e:null}function kx(e){const t=e.result?.sessions??[];return r`
    <section class="card">
      <div class="row" style="justify-content: space-between;">
        <div>
          <div class="card-title">Sessions</div>
          <div class="card-sub">Active session keys and per-session overrides.</div>
        </div>
        <button class="btn" ?disabled=${e.loading} @click=${e.onRefresh}>
          ${e.loading?"Loading…":"Refresh"}
        </button>
      </div>

      <div class="filters" style="margin-top: 14px;">
        <label class="field">
          <span>Active within (minutes)</span>
          <input
            .value=${e.activeMinutes}
            @input=${n=>e.onFiltersChange({activeMinutes:n.target.value,limit:e.limit,includeGlobal:e.includeGlobal,includeUnknown:e.includeUnknown})}
          />
        </label>
        <label class="field">
          <span>Limit</span>
          <input
            .value=${e.limit}
            @input=${n=>e.onFiltersChange({activeMinutes:e.activeMinutes,limit:n.target.value,includeGlobal:e.includeGlobal,includeUnknown:e.includeUnknown})}
          />
        </label>
        <label class="field checkbox">
          <span>Include global</span>
          <input
            type="checkbox"
            .checked=${e.includeGlobal}
            @change=${n=>e.onFiltersChange({activeMinutes:e.activeMinutes,limit:e.limit,includeGlobal:n.target.checked,includeUnknown:e.includeUnknown})}
          />
        </label>
        <label class="field checkbox">
          <span>Include unknown</span>
          <input
            type="checkbox"
            .checked=${e.includeUnknown}
            @change=${n=>e.onFiltersChange({activeMinutes:e.activeMinutes,limit:e.limit,includeGlobal:e.includeGlobal,includeUnknown:n.target.checked})}
          />
        </label>
      </div>

      ${e.error?r`<div class="callout danger" style="margin-top: 12px;">${e.error}</div>`:m}

      <div class="muted" style="margin-top: 12px;">
        ${e.result?`Store: ${e.result.path}`:""}
      </div>

      <div class="table" style="margin-top: 16px;">
        <div class="table-head">
          <div>Key</div>
          <div>Label</div>
          <div>Kind</div>
          <div>Updated</div>
          <div>Tokens</div>
          <div>Thinking</div>
          <div>Verbose</div>
          <div>Reasoning</div>
          <div>Actions</div>
        </div>
        ${t.length===0?r`
                <div class="muted">No sessions found.</div>
              `:t.map(n=>Sx(n,e.basePath,e.onPatch,e.onDelete,e.loading))}
      </div>
    </section>
  `}function Sx(e,t,n,s,i){const a=e.updatedAt?Y(e.updatedAt):"n/a",o=e.thinkingLevel??"",l=Sc(e.modelProvider),d=$x(o,l),g=$r(yx(e.modelProvider),d),f=e.verboseLevel??"",p=xx(vx,f),b=e.reasoningLevel??"",u=$r(mx,b),v=typeof e.displayName=="string"&&e.displayName.trim().length>0?e.displayName.trim():null,y=typeof e.label=="string"?e.label.trim():"",k=!!(v&&v!==e.key&&v!==y),C=e.kind!=="global",$=C?`${gs("chat",t)}?session=${encodeURIComponent(e.key)}`:null;return r`
    <div class="table-row">
      <div class="mono session-key-cell">
        ${C?r`<a href=${$} class="session-link">${e.key}</a>`:e.key}
        ${k?r`<span class="muted session-key-display-name">${v}</span>`:m}
      </div>
      <div>
        <input
          .value=${e.label??""}
          ?disabled=${i}
          placeholder="(optional)"
          @change=${T=>{const _=T.target.value.trim();n(e.key,{label:_||null})}}
        />
      </div>
      <div>${e.kind}</div>
      <div>${a}</div>
      <div>${Im(e)}</div>
      <div>
        <select
          ?disabled=${i}
          @change=${T=>{const _=T.target.value;n(e.key,{thinkingLevel:wx(_,l)})}}
        >
          ${g.map(T=>r`<option value=${T} ?selected=${d===T}>
                ${T||"inherit"}
              </option>`)}
        </select>
      </div>
      <div>
        <select
          ?disabled=${i}
          @change=${T=>{const _=T.target.value;n(e.key,{verboseLevel:_||null})}}
        >
          ${p.map(T=>r`<option value=${T.value} ?selected=${f===T.value}>
                ${T.label}
              </option>`)}
        </select>
      </div>
      <div>
        <select
          ?disabled=${i}
          @change=${T=>{const _=T.target.value;n(e.key,{reasoningLevel:_||null})}}
        >
          ${u.map(T=>r`<option value=${T} ?selected=${b===T}>
                ${T||"inherit"}
              </option>`)}
        </select>
      </div>
      <div>
        <button class="btn danger" ?disabled=${i} @click=${()=>s(e.key)}>
          Delete
        </button>
      </div>
    </div>
  `}function Ax(e){const t=e.report?.skills??[],n=e.filter.trim().toLowerCase(),s=n?t.filter(a=>[a.name,a.description,a.source].join(" ").toLowerCase().includes(n)):t,i=Ol(s);return r`
    <section class="card">
      <div class="row" style="justify-content: space-between;">
        <div>
          <div class="card-title">Skills</div>
          <div class="card-sub">Bundled, managed, and workspace skills.</div>
        </div>
        <button class="btn" ?disabled=${e.loading} @click=${e.onRefresh}>
          ${e.loading?"Loading…":"Refresh"}
        </button>
      </div>

      <div class="filters" style="margin-top: 14px;">
        <label class="field" style="flex: 1;">
          <span>Filter</span>
          <input
            .value=${e.filter}
            @input=${a=>e.onFilterChange(a.target.value)}
            placeholder="Search skills"
          />
        </label>
        <div class="muted">${s.length} shown</div>
      </div>

      ${e.error?r`<div class="callout danger" style="margin-top: 12px;">${e.error}</div>`:m}

      ${s.length===0?r`
              <div class="muted" style="margin-top: 16px">No skills found.</div>
            `:r`
            <div class="agent-skills-groups" style="margin-top: 16px;">
              ${i.map(a=>{const o=a.id==="workspace"||a.id==="built-in";return r`
                  <details class="agent-skills-group" ?open=${!o}>
                    <summary class="agent-skills-header">
                      <span>${a.label}</span>
                      <span class="muted">${a.skills.length}</span>
                    </summary>
                    <div class="list skills-grid">
                      ${a.skills.map(l=>_x(l,e))}
                    </div>
                  </details>
                `})}
            </div>
          `}
    </section>
  `}function _x(e,t){const n=t.busyKey===e.skillKey,s=t.edits[e.skillKey]??"",i=t.messages[e.skillKey]??null,a=e.install.length>0&&e.missing.bins.length>0,o=!!(e.bundled&&e.source!=="aisopod-bundled"),l=Bl(e),d=Ul(e);return r`
    <div class="list-item">
      <div class="list-main">
        <div class="list-title">
          ${e.emoji?`${e.emoji} `:""}${e.name}
        </div>
        <div class="list-sub">${si(e.description,140)}</div>
        ${zl({skill:e,showBundledBadge:o})}
        ${l.length>0?r`
              <div class="muted" style="margin-top: 6px;">
                Missing: ${l.join(", ")}
              </div>
            `:m}
        ${d.length>0?r`
              <div class="muted" style="margin-top: 6px;">
                Reason: ${d.join(", ")}
              </div>
            `:m}
      </div>
      <div class="list-meta">
        <div class="row" style="justify-content: flex-end; flex-wrap: wrap;">
          <button
            class="btn"
            ?disabled=${n}
            @click=${()=>t.onToggle(e.skillKey,e.disabled)}
          >
            ${e.disabled?"Enable":"Disable"}
          </button>
          ${a?r`<button
                class="btn"
                ?disabled=${n}
                @click=${()=>t.onInstall(e.skillKey,e.name,e.install[0].id)}
              >
                ${n?"Installing…":e.install[0].label}
              </button>`:m}
        </div>
        ${i?r`<div
              class="muted"
              style="margin-top: 8px; color: ${i.kind==="error"?"var(--danger-color, #d14343)":"var(--success-color, #0a7f5a)"};"
            >
              ${i.message}
            </div>`:m}
        ${e.primaryEnv?r`
              <div class="field" style="margin-top: 10px;">
                <span>API key</span>
                <input
                  type="password"
                  .value=${s}
                  @input=${g=>t.onEdit(e.skillKey,g.target.value)}
                />
              </div>
              <button
                class="btn primary"
                style="margin-top: 8px;"
                ?disabled=${n}
                @click=${()=>t.onSaveKey(e.skillKey)}
              >
                Save key
              </button>
            `:m}
      </div>
    </div>
  `}const Cx=/^data:/i,Tx=/^https?:\/\//i;function Ex(e){const t=e.agentsList?.agents??[],s=zr(e.sessionKey)?.agentId??e.agentsList?.defaultId??"main",a=t.find(l=>l.id===s)?.identity,o=a?.avatarUrl??a?.avatar;if(o)return Cx.test(o)||Tx.test(o)?o:a?.avatarUrl}function Lx(e){const t=e.presenceEntries.length,n=e.sessionsResult?.count??null,s=e.cronStatus?.nextWakeAtMs??null,i=e.connected?null:"Disconnected from gateway.",a=e.tab==="chat",o=a&&(e.settings.chatFocusMode||e.onboarding),l=e.onboarding?!1:e.settings.chatShowThinking,d=Ex(e),g=e.chatAvatarUrl??d??null,f=e.configForm??e.configSnapshot?.config,p=mn(e.basePath??""),b=e.agentsSelectedId??e.agentsList?.defaultId??e.agentsList?.agents?.[0]?.id??null;return r`
    <div class="shell ${a?"shell--chat":""} ${o?"shell--chat-focus":""} ${e.settings.navCollapsed?"shell--nav-collapsed":""} ${e.onboarding?"shell--onboarding":""}">
      <header class="topbar">
        <div class="topbar-left">
          <button
            class="nav-collapse-toggle"
            @click=${()=>e.applySettings({...e.settings,navCollapsed:!e.settings.navCollapsed})}
            title="${e.settings.navCollapsed?"Expand sidebar":"Collapse sidebar"}"
            aria-label="${e.settings.navCollapsed?"Expand sidebar":"Collapse sidebar"}"
          >
            <span class="nav-collapse-toggle__icon">${de.menu}</span>
          </button>
          <div class="brand">
            <div class="brand-logo">
              <img src=${p?`${p}/favicon.svg`:"/favicon.svg"} alt="Aisopod" />
            </div>
            <div class="brand-text">
              <div class="brand-title">AISOPOD</div>
              <div class="brand-sub">Gateway Dashboard</div>
            </div>
          </div>
        </div>
        <div class="topbar-status">
          <div class="pill">
            <span class="statusDot ${e.connected?"ok":""}"></span>
            <span>Health</span>
            <span class="mono">${e.connected?"OK":"Offline"}</span>
          </div>
          ${Sm(e)}
        </div>
      </header>
      <aside class="nav ${e.settings.navCollapsed?"nav--collapsed":""}">
        ${of.map(u=>{const v=e.settings.navGroupsCollapsed[u.label]??!1,y=u.tabs.some(k=>k===e.tab);return r`
            <div class="nav-group ${v&&!y?"nav-group--collapsed":""}">
              <button
                class="nav-label"
                @click=${()=>{const k={...e.settings.navGroupsCollapsed};k[u.label]=!v,e.applySettings({...e.settings,navGroupsCollapsed:k})}}
                aria-expanded=${!v}
              >
                <span class="nav-label__text">${u.label}</span>
                <span class="nav-label__chevron">${v?"+":"−"}</span>
              </button>
              <div class="nav-group__items">
                ${u.tabs.map(k=>ym(e,k))}
              </div>
            </div>
          `})}
        <div class="nav-group nav-group--links">
          <div class="nav-label nav-label--static">
            <span class="nav-label__text">Resources</span>
          </div>
          <div class="nav-group__items">
            <a
              class="nav-item nav-item--external"
              href="https://docs.aisopod.ai"
              target="_blank"
              rel="noreferrer"
              title="Docs (opens in new tab)"
            >
              <span class="nav-item__icon" aria-hidden="true">${de.book}</span>
              <span class="nav-item__text">Docs</span>
            </a>
          </div>
        </div>
      </aside>
      <main class="content ${a?"content--chat":""}">
        <section class="content-header">
          <div>
            ${e.tab==="usage"?m:r`<div class="page-title">${li(e.tab)}</div>`}
            ${e.tab==="usage"?m:r`<div class="page-sub">${cf(e.tab)}</div>`}
          </div>
          <div class="page-meta">
            ${e.lastError?r`<div class="pill danger">${e.lastError}</div>`:m}
            ${a?xm(e):m}
          </div>
        </section>

        ${e.tab==="overview"?px({connected:e.connected,hello:e.hello,settings:e.settings,password:e.password,lastError:e.lastError,presenceCount:t,sessionsCount:n,cronEnabled:e.cronStatus?.enabled??null,cronNext:s,lastChannelsRefresh:e.channelsLastSuccess,onSettingsChange:u=>e.applySettings(u),onPasswordChange:u=>e.password=u,onSessionKeyChange:u=>{e.sessionKey=u,e.chatMessage="",e.resetToolStream(),e.applySettings({...e.settings,sessionKey:u,lastActiveSessionKey:u}),e.loadAssistantIdentity()},onConnect:()=>e.connect(),onRefresh:()=>e.loadOverview()}):m}

        ${e.tab==="channels"?Kb({connected:e.connected,loading:e.channelsLoading,snapshot:e.channelsSnapshot,lastError:e.channelsError,lastSuccessAt:e.channelsLastSuccess,whatsappMessage:e.whatsappLoginMessage,whatsappQrDataUrl:e.whatsappLoginQrDataUrl,whatsappConnected:e.whatsappLoginConnected,whatsappBusy:e.whatsappBusy,configSchema:e.configSchema,configSchemaLoading:e.configSchemaLoading,configForm:e.configForm,configUiHints:e.configUiHints,configSaving:e.configSaving,configFormDirty:e.configFormDirty,nostrProfileFormState:e.nostrProfileFormState,nostrProfileAccountId:e.nostrProfileAccountId,onRefresh:u=>xe(e,u),onWhatsAppStart:u=>e.handleWhatsAppStart(u),onWhatsAppWait:()=>e.handleWhatsAppWait(),onWhatsAppLogout:()=>e.handleWhatsAppLogout(),onConfigPatch:(u,v)=>we(e,u,v),onConfigSave:()=>e.handleChannelConfigSave(),onConfigReload:()=>e.handleChannelConfigReload(),onNostrProfileEdit:(u,v)=>e.handleNostrProfileEdit(u,v),onNostrProfileCancel:()=>e.handleNostrProfileCancel(),onNostrProfileFieldChange:(u,v)=>e.handleNostrProfileFieldChange(u,v),onNostrProfileSave:()=>e.handleNostrProfileSave(),onNostrProfileImport:()=>e.handleNostrProfileImport(),onNostrProfileToggleAdvanced:()=>e.handleNostrProfileToggleAdvanced()}):m}

        ${e.tab==="instances"?N1({loading:e.presenceLoading,entries:e.presenceEntries,lastError:e.presenceError,statusMessage:e.presenceStatus,onRefresh:()=>Zi(e)}):m}

        ${e.tab==="sessions"?kx({loading:e.sessionsLoading,result:e.sessionsResult,error:e.sessionsError,activeMinutes:e.sessionsFilterActive,limit:e.sessionsFilterLimit,includeGlobal:e.sessionsIncludeGlobal,includeUnknown:e.sessionsIncludeUnknown,basePath:e.basePath,onFiltersChange:u=>{e.sessionsFilterActive=u.activeMinutes,e.sessionsFilterLimit=u.limit,e.sessionsIncludeGlobal=u.includeGlobal,e.sessionsIncludeUnknown=u.includeUnknown},onRefresh:()=>_t(e),onPatch:(u,v)=>Xh(e,u,v),onDelete:u=>ef(e,u)}):m}

        ${um(e)}

        ${e.tab==="cron"?C1({basePath:e.basePath,loading:e.cronLoading,status:e.cronStatus,jobs:e.cronJobs,error:e.cronError,busy:e.cronBusy,form:e.cronForm,channels:e.channelsSnapshot?.channelMeta?.length?e.channelsSnapshot.channelMeta.map(u=>u.id):e.channelsSnapshot?.channelOrder??[],channelLabels:e.channelsSnapshot?.channelLabels??{},channelMeta:e.channelsSnapshot?.channelMeta??[],runsJobId:e.cronRunsJobId,runs:e.cronRuns,onFormChange:u=>e.cronForm={...e.cronForm,...u},onRefresh:()=>e.loadCron(),onAdd:()=>gh(e),onToggle:(u,v)=>ph(e,u,v),onRun:u=>hh(e,u),onRemove:u=>fh(e,u),onLoadRuns:u=>qr(e,u)}):m}

        ${e.tab==="agents"?gb({loading:e.agentsLoading,error:e.agentsError,agentsList:e.agentsList,selectedAgentId:b,activePanel:e.agentsPanel,configForm:f,configLoading:e.configLoading,configSaving:e.configSaving,configDirty:e.configFormDirty,channelsLoading:e.channelsLoading,channelsError:e.channelsError,channelsSnapshot:e.channelsSnapshot,channelsLastSuccess:e.channelsLastSuccess,cronLoading:e.cronLoading,cronStatus:e.cronStatus,cronJobs:e.cronJobs,cronError:e.cronError,agentFilesLoading:e.agentFilesLoading,agentFilesError:e.agentFilesError,agentFilesList:e.agentFilesList,agentFileActive:e.agentFileActive,agentFileContents:e.agentFileContents,agentFileDrafts:e.agentFileDrafts,agentFileSaving:e.agentFileSaving,agentIdentityLoading:e.agentIdentityLoading,agentIdentityError:e.agentIdentityError,agentIdentityById:e.agentIdentityById,agentSkillsLoading:e.agentSkillsLoading,agentSkillsReport:e.agentSkillsReport,agentSkillsError:e.agentSkillsError,agentSkillsAgentId:e.agentSkillsAgentId,skillsFilter:e.skillsFilter,onRefresh:async()=>{await Hi(e);const u=e.agentsList?.agents?.map(v=>v.id)??[];u.length>0&&jr(e,u)},onSelectAgent:u=>{e.agentsSelectedId!==u&&(e.agentsSelectedId=u,e.agentFilesList=null,e.agentFilesError=null,e.agentFilesLoading=!1,e.agentFileActive=null,e.agentFileContents={},e.agentFileDrafts={},e.agentSkillsReport=null,e.agentSkillsError=null,e.agentSkillsAgentId=null,Kr(e,u),e.agentsPanel==="files"&&Ws(e,u),e.agentsPanel==="skills"&&Kn(e,u))},onSelectPanel:u=>{e.agentsPanel=u,u==="files"&&b&&e.agentFilesList?.agentId!==b&&(e.agentFilesList=null,e.agentFilesError=null,e.agentFileActive=null,e.agentFileContents={},e.agentFileDrafts={},Ws(e,b)),u==="skills"&&b&&Kn(e,b),u==="channels"&&xe(e,!1),u==="cron"&&e.loadCron()},onLoadFiles:u=>Ws(e,u),onSelectFile:u=>{e.agentFileActive=u,b&&Tm(e,b,u)},onFileDraftChange:(u,v)=>{e.agentFileDrafts={...e.agentFileDrafts,[u]:v}},onFileReset:u=>{const v=e.agentFileContents[u]??"";e.agentFileDrafts={...e.agentFileDrafts,[u]:v}},onFileSave:u=>{if(!b)return;const v=e.agentFileDrafts[u]??e.agentFileContents[u]??"";Em(e,b,u,v)},onToolsProfileChange:(u,v,y)=>{if(!f)return;const k=f.agents?.list;if(!Array.isArray(k))return;const C=k.findIndex(T=>T&&typeof T=="object"&&"id"in T&&T.id===u);if(C<0)return;const $=["agents","list",C,"tools"];v?we(e,[...$,"profile"],v):je(e,[...$,"profile"]),y&&je(e,[...$,"allow"])},onToolsOverridesChange:(u,v,y)=>{if(!f)return;const k=f.agents?.list;if(!Array.isArray(k))return;const C=k.findIndex(T=>T&&typeof T=="object"&&"id"in T&&T.id===u);if(C<0)return;const $=["agents","list",C,"tools"];v.length>0?we(e,[...$,"alsoAllow"],v):je(e,[...$,"alsoAllow"]),y.length>0?we(e,[...$,"deny"],y):je(e,[...$,"deny"])},onConfigReload:()=>Ie(e),onConfigSave:()=>Hn(e),onChannelsRefresh:()=>xe(e,!1),onCronRefresh:()=>e.loadCron(),onSkillsFilterChange:u=>e.skillsFilter=u,onSkillsRefresh:()=>{b&&Kn(e,b)},onAgentSkillToggle:(u,v,y)=>{if(!f)return;const k=f.agents?.list;if(!Array.isArray(k))return;const C=k.findIndex(j=>j&&typeof j=="object"&&"id"in j&&j.id===u);if(C<0)return;const $=k[C],T=v.trim();if(!T)return;const _=e.agentSkillsReport?.skills?.map(j=>j.name).filter(Boolean)??[],E=(Array.isArray($.skills)?$.skills.map(j=>String(j).trim()).filter(Boolean):void 0)??_,P=new Set(E);y?P.add(T):P.delete(T),we(e,["agents","list",C,"skills"],[...P])},onAgentSkillsClear:u=>{if(!f)return;const v=f.agents?.list;if(!Array.isArray(v))return;const y=v.findIndex(k=>k&&typeof k=="object"&&"id"in k&&k.id===u);y<0||je(e,["agents","list",y,"skills"])},onAgentSkillsDisableAll:u=>{if(!f)return;const v=f.agents?.list;if(!Array.isArray(v))return;const y=v.findIndex(k=>k&&typeof k=="object"&&"id"in k&&k.id===u);y<0||we(e,["agents","list",y,"skills"],[])},onModelChange:(u,v)=>{if(!f)return;const y=f.agents?.list;if(!Array.isArray(y))return;const k=y.findIndex(_=>_&&typeof _=="object"&&"id"in _&&_.id===u);if(k<0)return;const C=["agents","list",k,"model"];if(!v){je(e,C);return}const T=y[k]?.model;if(T&&typeof T=="object"&&!Array.isArray(T)){const _=T.fallbacks,L={primary:v,...Array.isArray(_)?{fallbacks:_}:{}};we(e,C,L)}else we(e,C,v)},onModelFallbacksChange:(u,v)=>{if(!f)return;const y=f.agents?.list;if(!Array.isArray(y))return;const k=y.findIndex(j=>j&&typeof j=="object"&&"id"in j&&j.id===u);if(k<0)return;const C=["agents","list",k,"model"],$=y[k],T=v.map(j=>j.trim()).filter(Boolean),_=$.model,E=(()=>{if(typeof _=="string")return _.trim()||null;if(_&&typeof _=="object"&&!Array.isArray(_)){const j=_.primary;if(typeof j=="string")return j.trim()||null}return null})();if(T.length===0){E?we(e,C,E):je(e,C);return}we(e,C,E?{primary:E,fallbacks:T}:{fallbacks:T})}}):m}

        ${e.tab==="skills"?Ax({loading:e.skillsLoading,report:e.skillsReport,error:e.skillsError,filter:e.skillsFilter,edits:e.skillEdits,messages:e.skillMessages,busyKey:e.skillsBusyKey,onFilterChange:u=>e.skillsFilter=u,onRefresh:()=>vn(e,{clearMessages:!0}),onToggle:(u,v)=>nf(e,u,v),onEdit:(u,v)=>tf(e,u,v),onSaveKey:u=>sf(e,u),onInstall:(u,v,y)=>af(e,u,v,y)}):m}

        ${e.tab==="nodes"?nx({loading:e.nodesLoading,nodes:e.nodes,devicesLoading:e.devicesLoading,devicesError:e.devicesError,devicesList:e.devicesList,configForm:e.configForm??e.configSnapshot?.config,configLoading:e.configLoading,configSaving:e.configSaving,configDirty:e.configFormDirty,configFormMode:e.configFormMode,execApprovalsLoading:e.execApprovalsLoading,execApprovalsSaving:e.execApprovalsSaving,execApprovalsDirty:e.execApprovalsDirty,execApprovalsSnapshot:e.execApprovalsSnapshot,execApprovalsForm:e.execApprovalsForm,execApprovalsSelectedAgent:e.execApprovalsSelectedAgent,execApprovalsTarget:e.execApprovalsTarget,execApprovalsTargetNodeId:e.execApprovalsTargetNodeId,onRefresh:()=>ls(e),onDevicesRefresh:()=>st(e),onDeviceApprove:u=>Kh(e,u),onDeviceReject:u=>jh(e,u),onDeviceRotate:(u,v,y)=>Wh(e,{deviceId:u,role:v,scopes:y}),onDeviceRevoke:(u,v)=>qh(e,{deviceId:u,role:v}),onLoadConfig:()=>Ie(e),onLoadExecApprovals:()=>{const u=e.execApprovalsTarget==="node"&&e.execApprovalsTargetNodeId?{kind:"node",nodeId:e.execApprovalsTargetNodeId}:{kind:"gateway"};return Ji(e,u)},onBindDefault:u=>{u?we(e,["tools","exec","node"],u):je(e,["tools","exec","node"])},onBindAgent:(u,v)=>{const y=["agents","list",u,"tools","exec","node"];v?we(e,y,v):je(e,y)},onSaveBindings:()=>Hn(e),onExecApprovalsTargetChange:(u,v)=>{e.execApprovalsTarget=u,e.execApprovalsTargetNodeId=v,e.execApprovalsSnapshot=null,e.execApprovalsForm=null,e.execApprovalsDirty=!1,e.execApprovalsSelectedAgent=null},onExecApprovalsSelectAgent:u=>{e.execApprovalsSelectedAgent=u},onExecApprovalsPatch:(u,v)=>Jh(e,u,v),onExecApprovalsRemove:u=>Zh(e,u),onSaveExecApprovals:()=>{const u=e.execApprovalsTarget==="node"&&e.execApprovalsTargetNodeId?{kind:"node",nodeId:e.execApprovalsTargetNodeId}:{kind:"gateway"};return Yh(e,u)}}):m}

        ${e.tab==="chat"?b1({sessionKey:e.sessionKey,onSessionKeyChange:u=>{e.sessionKey=u,e.chatMessage="",e.chatAttachments=[],e.chatStream=null,e.chatStreamStartedAt=null,e.chatRunId=null,e.chatQueue=[],e.resetToolStream(),e.resetChatScroll(),e.applySettings({...e.settings,sessionKey:u,lastActiveSessionKey:u}),e.loadAssistantIdentity(),gn(e),ui(e)},thinkingLevel:e.chatThinkingLevel,showThinking:l,loading:e.chatLoading,sending:e.chatSending,compactionStatus:e.compactionStatus,assistantAvatarUrl:g,messages:e.chatMessages,toolMessages:e.chatToolMessages,stream:e.chatStream,streamStartedAt:e.chatStreamStartedAt,draft:e.chatMessage,queue:e.chatQueue,connected:e.connected,canSend:e.connected,disabledReason:i,error:e.lastError,sessions:e.sessionsResult,focusMode:o,onRefresh:()=>(e.resetToolStream(),Promise.all([gn(e),ui(e)])),onToggleFocusMode:()=>{e.onboarding||e.applySettings({...e.settings,chatFocusMode:!e.settings.chatFocusMode})},onChatScroll:u=>e.handleChatScroll(u),onDraftChange:u=>e.chatMessage=u,attachments:e.chatAttachments,onAttachmentsChange:u=>e.chatAttachments=u,onSend:()=>e.handleSendChat(),canAbort:!!e.chatRunId,onAbort:()=>{e.handleAbortChat()},onQueueRemove:u=>e.removeQueuedMessage(u),onNewSession:()=>e.handleSendChat("/new",{restoreDraft:!0}),showNewMessages:e.chatNewMessagesBelow&&!e.chatManualRefreshInFlight,onScrollToBottom:()=>e.scrollToBottom(),sidebarOpen:e.sidebarOpen,sidebarContent:e.sidebarContent,sidebarError:e.sidebarError,splitRatio:e.splitRatio,onOpenSidebar:u=>e.handleOpenSidebar(u),onCloseSidebar:()=>e.handleCloseSidebar(),onSplitRatioChange:u=>e.handleSplitRatioChange(u),assistantName:e.assistantName,assistantAvatar:e.assistantAvatar}):m}

        ${e.tab==="config"?S1({raw:e.configRaw,originalRaw:e.configRawOriginal,valid:e.configValid,issues:e.configIssues,loading:e.configLoading,saving:e.configSaving,applying:e.configApplying,updating:e.updateRunning,connected:e.connected,schema:e.configSchema,schemaLoading:e.configSchemaLoading,uiHints:e.configUiHints,formMode:e.configFormMode,formValue:e.configForm,originalValue:e.configFormOriginal,searchQuery:e.configSearchQuery,activeSection:e.configActiveSection,activeSubsection:e.configActiveSubsection,onRawChange:u=>{e.configRaw=u},onFormModeChange:u=>e.configFormMode=u,onFormPatch:(u,v)=>we(e,u,v),onSearchChange:u=>e.configSearchQuery=u,onSectionChange:u=>{e.configActiveSection=u,e.configActiveSubsection=null},onSubsectionChange:u=>e.configActiveSubsection=u,onReload:()=>Ie(e),onSave:()=>Hn(e),onApply:()=>Mp(e),onUpdate:()=>Ip(e)}):m}

        ${e.tab==="debug"?R1({loading:e.debugLoading,status:e.debugStatus,health:e.debugHealth,models:e.debugModels,heartbeat:e.debugHeartbeat,eventLog:e.eventLog,callMethod:e.debugCallMethod,callParams:e.debugCallParams,callResult:e.debugCallResult,callError:e.debugCallError,onCallMethodChange:u=>e.debugCallMethod=u,onCallParamsChange:u=>e.debugCallParams=u,onRefresh:()=>rs(e),onCall:()=>Xp(e)}):m}

        ${e.tab==="logs"?z1({loading:e.logsLoading,error:e.logsError,file:e.logsFile,entries:e.logsEntries,filterText:e.logsFilterText,levelFilters:e.logsLevelFilters,autoFollow:e.logsAutoFollow,truncated:e.logsTruncated,onFilterTextChange:u=>e.logsFilterText=u,onLevelToggle:(u,v)=>{e.logsLevelFilters={...e.logsLevelFilters,[u]:v}},onToggleAutoFollow:u=>e.logsAutoFollow=u,onRefresh:()=>Ni(e,{reset:!0}),onExport:(u,v)=>e.exportLogs(u,v),onScroll:u=>e.handleLogsScroll(u)}):m}
      </main>
      ${D1(e)}
      ${F1(e)}
    </div>
  `}var Mx=Object.create,ka=Object.defineProperty,Ix=Object.getOwnPropertyDescriptor,Ac=(e,t)=>(t=Symbol[e])?t:Symbol.for("Symbol."+e),xn=e=>{throw TypeError(e)},Rx=(e,t,n)=>t in e?ka(e,t,{enumerable:!0,configurable:!0,writable:!0,value:n}):e[t]=n,wr=(e,t)=>ka(e,"name",{value:t,configurable:!0}),Px=e=>[,,,Mx(e?.[Ac("metadata")]??null)],_c=["class","method","getter","setter","accessor","field","value","get","set"],Xt=e=>e!==void 0&&typeof e!="function"?xn("Function expected"):e,Dx=(e,t,n,s,i)=>({kind:_c[e],name:t,metadata:s,addInitializer:a=>n._?xn("Already initialized"):i.push(Xt(a||null))}),Fx=(e,t)=>Rx(t,Ac("metadata"),e[3]),h=(e,t,n,s)=>{for(var i=0,a=e[t>>1],o=a&&a.length;i<o;i++)t&1?a[i].call(n):s=a[i].call(n,s);return s},S=(e,t,n,s,i,a)=>{var o,l,d,g,f,p=t&7,b=!!(t&8),u=!!(t&16),v=p>3?e.length+1:p?b?1:2:0,y=_c[p+5],k=p>3&&(e[v-1]=[]),C=e[v]||(e[v]=[]),$=p&&(!u&&!b&&(i=i.prototype),p<5&&(p>3||!u)&&Ix(p<4?i:{get[n](){return kr(this,a)},set[n](_){return Sr(this,a,_)}},n));p?u&&p<4&&wr(a,(p>2?"set ":p>1?"get ":"")+n):wr(i,n);for(var T=s.length-1;T>=0;T--)g=Dx(p,n,d={},e[3],C),p&&(g.static=b,g.private=u,f=g.access={has:u?_=>Nx(i,_):_=>n in _},p^3&&(f.get=u?_=>(p^1?kr:Ox)(_,i,p^4?a:$.get):_=>_[n]),p>2&&(f.set=u?(_,L)=>Sr(_,i,L,p^4?a:$.set):(_,L)=>_[n]=L)),l=(0,s[T])(p?p<4?u?a:$[y]:p>4?void 0:{get:$.get,set:$.set}:i,g),d._=1,p^4||l===void 0?Xt(l)&&(p>4?k.unshift(l):p?u?a=l:$[y]=l:i=l):typeof l!="object"||l===null?xn("Object expected"):(Xt(o=l.get)&&($.get=o),Xt(o=l.set)&&($.set=o),Xt(o=l.init)&&k.unshift(o));return p||Fx(e,i),$&&ka(i,n,$),u?p^4?a:$:i},Sa=(e,t,n)=>t.has(e)||xn("Cannot "+n),Nx=(e,t)=>Object(t)!==t?xn('Cannot use the "in" operator on this value'):e.has(t),kr=(e,t,n)=>(Sa(e,t,"read from private field"),n?n.call(e):t.get(e)),Sr=(e,t,n,s)=>(Sa(e,t,"write to private field"),s?s.call(e,n):t.set(e,n),n),Ox=(e,t,n)=>(Sa(e,t,"access private method"),n),Cc,Tc,Ec,Lc,Mc,Ic,Rc,Pc,Dc,Fc,Nc,Oc,Bc,Uc,zc,Hc,Kc,jc,Wc,qc,Gc,Vc,Qc,Yc,Jc,Zc,Xc,ed,td,nd,sd,id,ad,od,rd,ld,cd,dd,ud,gd,pd,hd,fd,vd,md,bd,yd,xd,$d,wd,kd,Sd,Ad,_d,Cd,Td,Ed,Ld,Md,Id,Rd,Pd,Dd,Fd,Nd,Od,Bd,Ud,zd,Hd,Kd,jd,Wd,qd,Gd,Vd,Qd,Yd,Jd,Zd,Xd,eu,tu,nu,su,iu,au,ou,ru,lu,cu,du,uu,gu,pu,hu,fu,vu,mu,bu,yu,xu,$u,wu,ku,Su,Au,_u,Cu,Tu,Eu,Lu,Mu,Iu,Ru,Pu,Du,Fu,Nu,Ou,Bu,Uu,zu,Hu,Ku,ju,Wu,qu,Gu,Vu,Qu,Yu,Ju,Zu,Xu,eg,tg,ng,sg,ig,ag,og,rg,lg,cg,dg,ug,gg,pg,hg,fg,vg,mg,bg,yg,xg,$g,wg,kg,Sg,Ag,_g,Cg,Tg,Eg,Lg,Mg,Ig,Rg,Pg,Dg,Fg,Ng,Og,Bg,Ug,zg,Hg,Kg,jg,Ei,Wg,c;const ei=rv();function Bx(){if(!window.location.search)return!1;const t=new URLSearchParams(window.location.search).get("onboarding");if(!t)return!1;const n=t.trim().toLowerCase();return n==="1"||n==="true"||n==="yes"||n==="on"}Wg=[Ir("aisopod-app")];class w extends(Ei=Ft,jg=[A()],Kg=[A()],Hg=[A()],zg=[A()],Ug=[A()],Bg=[A()],Og=[A()],Ng=[A()],Fg=[A()],Dg=[A()],Pg=[A()],Rg=[A()],Ig=[A()],Mg=[A()],Lg=[A()],Eg=[A()],Tg=[A()],Cg=[A()],_g=[A()],Ag=[A()],Sg=[A()],kg=[A()],wg=[A()],$g=[A()],xg=[A()],yg=[A()],bg=[A()],mg=[A()],vg=[A()],fg=[A()],hg=[A()],pg=[A()],gg=[A()],ug=[A()],dg=[A()],cg=[A()],lg=[A()],rg=[A()],og=[A()],ag=[A()],ig=[A()],sg=[A()],ng=[A()],tg=[A()],eg=[A()],Xu=[A()],Zu=[A()],Ju=[A()],Yu=[A()],Qu=[A()],Vu=[A()],Gu=[A()],qu=[A()],Wu=[A()],ju=[A()],Ku=[A()],Hu=[A()],zu=[A()],Uu=[A()],Bu=[A()],Ou=[A()],Nu=[A()],Fu=[A()],Du=[A()],Pu=[A()],Ru=[A()],Iu=[A()],Mu=[A()],Lu=[A()],Eu=[A()],Tu=[A()],Cu=[A()],_u=[A()],Au=[A()],Su=[A()],ku=[A()],wu=[A()],$u=[A()],xu=[A()],yu=[A()],bu=[A()],mu=[A()],vu=[A()],fu=[A()],hu=[A()],pu=[A()],gu=[A()],uu=[A()],du=[A()],cu=[A()],lu=[A()],ru=[A()],ou=[A()],au=[A()],iu=[A()],su=[A()],nu=[A()],tu=[A()],eu=[A()],Xd=[A()],Zd=[A()],Jd=[A()],Yd=[A()],Qd=[A()],Vd=[A()],Gd=[A()],qd=[A()],Wd=[A()],jd=[A()],Kd=[A()],Hd=[A()],zd=[A()],Ud=[A()],Bd=[A()],Od=[A()],Nd=[A()],Fd=[A()],Dd=[A()],Pd=[A()],Rd=[A()],Id=[A()],Md=[A()],Ld=[A()],Ed=[A()],Td=[A()],Cd=[A()],_d=[A()],Ad=[A()],Sd=[A()],kd=[A()],wd=[A()],$d=[A()],xd=[A()],yd=[A()],bd=[A()],md=[A()],vd=[A()],fd=[A()],hd=[A()],pd=[A()],gd=[A()],ud=[A()],dd=[A()],cd=[A()],ld=[A()],rd=[A()],od=[A()],ad=[A()],id=[A()],sd=[A()],nd=[A()],td=[A()],ed=[A()],Xc=[A()],Zc=[A()],Jc=[A()],Yc=[A()],Qc=[A()],Vc=[A()],Gc=[A()],qc=[A()],Wc=[A()],jc=[A()],Kc=[A()],Hc=[A()],zc=[A()],Uc=[A()],Bc=[A()],Oc=[A()],Nc=[A()],Fc=[A()],Dc=[A()],Pc=[A()],Rc=[A()],Ic=[A()],Mc=[A()],Lc=[A()],Ec=[A()],Tc=[A()],Cc=[A()],Ei){constructor(){super(...arguments),this.settings=h(c,8,this,df()),h(c,11,this),this.password=h(c,12,this,""),h(c,15,this),this.tab=h(c,16,this,"chat"),h(c,19,this),this.onboarding=h(c,20,this,Bx()),h(c,23,this),this.connected=h(c,24,this,!1),h(c,27,this),this.theme=h(c,28,this,this.settings.theme??"system"),h(c,31,this),this.themeResolved=h(c,32,this,"dark"),h(c,35,this),this.hello=h(c,36,this,null),h(c,39,this),this.lastError=h(c,40,this,null),h(c,43,this),this.eventLog=h(c,44,this,[]),h(c,47,this),this.eventLogBuffer=[],this.toolStreamSyncTimer=null,this.sidebarCloseTimer=null,this.assistantName=h(c,48,this,ei.name),h(c,51,this),this.assistantAvatar=h(c,52,this,ei.avatar),h(c,55,this),this.assistantAgentId=h(c,56,this,ei.agentId??null),h(c,59,this),this.sessionKey=h(c,60,this,this.settings.sessionKey),h(c,63,this),this.chatLoading=h(c,64,this,!1),h(c,67,this),this.chatSending=h(c,68,this,!1),h(c,71,this),this.chatMessage=h(c,72,this,""),h(c,75,this),this.chatMessages=h(c,76,this,[]),h(c,79,this),this.chatToolMessages=h(c,80,this,[]),h(c,83,this),this.chatStream=h(c,84,this,null),h(c,87,this),this.chatStreamStartedAt=h(c,88,this,null),h(c,91,this),this.chatRunId=h(c,92,this,null),h(c,95,this),this.compactionStatus=h(c,96,this,null),h(c,99,this),this.chatAvatarUrl=h(c,100,this,null),h(c,103,this),this.chatThinkingLevel=h(c,104,this,null),h(c,107,this),this.chatQueue=h(c,108,this,[]),h(c,111,this),this.chatAttachments=h(c,112,this,[]),h(c,115,this),this.chatManualRefreshInFlight=h(c,116,this,!1),h(c,119,this),this.sidebarOpen=h(c,120,this,!1),h(c,123,this),this.sidebarContent=h(c,124,this,null),h(c,127,this),this.sidebarError=h(c,128,this,null),h(c,131,this),this.splitRatio=h(c,132,this,this.settings.splitRatio),h(c,135,this),this.nodesLoading=h(c,136,this,!1),h(c,139,this),this.nodes=h(c,140,this,[]),h(c,143,this),this.devicesLoading=h(c,144,this,!1),h(c,147,this),this.devicesError=h(c,148,this,null),h(c,151,this),this.devicesList=h(c,152,this,null),h(c,155,this),this.execApprovalsLoading=h(c,156,this,!1),h(c,159,this),this.execApprovalsSaving=h(c,160,this,!1),h(c,163,this),this.execApprovalsDirty=h(c,164,this,!1),h(c,167,this),this.execApprovalsSnapshot=h(c,168,this,null),h(c,171,this),this.execApprovalsForm=h(c,172,this,null),h(c,175,this),this.execApprovalsSelectedAgent=h(c,176,this,null),h(c,179,this),this.execApprovalsTarget=h(c,180,this,"gateway"),h(c,183,this),this.execApprovalsTargetNodeId=h(c,184,this,null),h(c,187,this),this.execApprovalQueue=h(c,188,this,[]),h(c,191,this),this.execApprovalBusy=h(c,192,this,!1),h(c,195,this),this.execApprovalError=h(c,196,this,null),h(c,199,this),this.pendingGatewayUrl=h(c,200,this,null),h(c,203,this),this.configLoading=h(c,204,this,!1),h(c,207,this),this.configRaw=h(c,208,this,`{
}
`),h(c,211,this),this.configRawOriginal=h(c,212,this,""),h(c,215,this),this.configValid=h(c,216,this,null),h(c,219,this),this.configIssues=h(c,220,this,[]),h(c,223,this),this.configSaving=h(c,224,this,!1),h(c,227,this),this.configApplying=h(c,228,this,!1),h(c,231,this),this.updateRunning=h(c,232,this,!1),h(c,235,this),this.applySessionKey=h(c,236,this,this.settings.lastActiveSessionKey),h(c,239,this),this.configSnapshot=h(c,240,this,null),h(c,243,this),this.configSchema=h(c,244,this,null),h(c,247,this),this.configSchemaVersion=h(c,248,this,null),h(c,251,this),this.configSchemaLoading=h(c,252,this,!1),h(c,255,this),this.configUiHints=h(c,256,this,{}),h(c,259,this),this.configForm=h(c,260,this,null),h(c,263,this),this.configFormOriginal=h(c,264,this,null),h(c,267,this),this.configFormDirty=h(c,268,this,!1),h(c,271,this),this.configFormMode=h(c,272,this,"form"),h(c,275,this),this.configSearchQuery=h(c,276,this,""),h(c,279,this),this.configActiveSection=h(c,280,this,null),h(c,283,this),this.configActiveSubsection=h(c,284,this,null),h(c,287,this),this.channelsLoading=h(c,288,this,!1),h(c,291,this),this.channelsSnapshot=h(c,292,this,null),h(c,295,this),this.channelsError=h(c,296,this,null),h(c,299,this),this.channelsLastSuccess=h(c,300,this,null),h(c,303,this),this.whatsappLoginMessage=h(c,304,this,null),h(c,307,this),this.whatsappLoginQrDataUrl=h(c,308,this,null),h(c,311,this),this.whatsappLoginConnected=h(c,312,this,null),h(c,315,this),this.whatsappBusy=h(c,316,this,!1),h(c,319,this),this.nostrProfileFormState=h(c,320,this,null),h(c,323,this),this.nostrProfileAccountId=h(c,324,this,null),h(c,327,this),this.presenceLoading=h(c,328,this,!1),h(c,331,this),this.presenceEntries=h(c,332,this,[]),h(c,335,this),this.presenceError=h(c,336,this,null),h(c,339,this),this.presenceStatus=h(c,340,this,null),h(c,343,this),this.agentsLoading=h(c,344,this,!1),h(c,347,this),this.agentsList=h(c,348,this,null),h(c,351,this),this.agentsError=h(c,352,this,null),h(c,355,this),this.agentsSelectedId=h(c,356,this,null),h(c,359,this),this.agentsPanel=h(c,360,this,"overview"),h(c,363,this),this.agentFilesLoading=h(c,364,this,!1),h(c,367,this),this.agentFilesError=h(c,368,this,null),h(c,371,this),this.agentFilesList=h(c,372,this,null),h(c,375,this),this.agentFileContents=h(c,376,this,{}),h(c,379,this),this.agentFileDrafts=h(c,380,this,{}),h(c,383,this),this.agentFileActive=h(c,384,this,null),h(c,387,this),this.agentFileSaving=h(c,388,this,!1),h(c,391,this),this.agentIdentityLoading=h(c,392,this,!1),h(c,395,this),this.agentIdentityError=h(c,396,this,null),h(c,399,this),this.agentIdentityById=h(c,400,this,{}),h(c,403,this),this.agentSkillsLoading=h(c,404,this,!1),h(c,407,this),this.agentSkillsError=h(c,408,this,null),h(c,411,this),this.agentSkillsReport=h(c,412,this,null),h(c,415,this),this.agentSkillsAgentId=h(c,416,this,null),h(c,419,this),this.sessionsLoading=h(c,420,this,!1),h(c,423,this),this.sessionsResult=h(c,424,this,null),h(c,427,this),this.sessionsError=h(c,428,this,null),h(c,431,this),this.sessionsFilterActive=h(c,432,this,""),h(c,435,this),this.sessionsFilterLimit=h(c,436,this,"120"),h(c,439,this),this.sessionsIncludeGlobal=h(c,440,this,!0),h(c,443,this),this.sessionsIncludeUnknown=h(c,444,this,!1),h(c,447,this),this.usageLoading=h(c,448,this,!1),h(c,451,this),this.usageResult=h(c,452,this,null),h(c,455,this),this.usageCostSummary=h(c,456,this,null),h(c,459,this),this.usageError=h(c,460,this,null),h(c,463,this),this.usageStartDate=h(c,464,this,(()=>{const t=new Date;return`${t.getFullYear()}-${String(t.getMonth()+1).padStart(2,"0")}-${String(t.getDate()).padStart(2,"0")}`})()),h(c,467,this),this.usageEndDate=h(c,468,this,(()=>{const t=new Date;return`${t.getFullYear()}-${String(t.getMonth()+1).padStart(2,"0")}-${String(t.getDate()).padStart(2,"0")}`})()),h(c,471,this),this.usageSelectedSessions=h(c,472,this,[]),h(c,475,this),this.usageSelectedDays=h(c,476,this,[]),h(c,479,this),this.usageSelectedHours=h(c,480,this,[]),h(c,483,this),this.usageChartMode=h(c,484,this,"tokens"),h(c,487,this),this.usageDailyChartMode=h(c,488,this,"by-type"),h(c,491,this),this.usageTimeSeriesMode=h(c,492,this,"per-turn"),h(c,495,this),this.usageTimeSeriesBreakdownMode=h(c,496,this,"by-type"),h(c,499,this),this.usageTimeSeries=h(c,500,this,null),h(c,503,this),this.usageTimeSeriesLoading=h(c,504,this,!1),h(c,507,this),this.usageSessionLogs=h(c,508,this,null),h(c,511,this),this.usageSessionLogsLoading=h(c,512,this,!1),h(c,515,this),this.usageSessionLogsExpanded=h(c,516,this,!1),h(c,519,this),this.usageQuery=h(c,520,this,""),h(c,523,this),this.usageQueryDraft=h(c,524,this,""),h(c,527,this),this.usageSessionSort=h(c,528,this,"recent"),h(c,531,this),this.usageSessionSortDir=h(c,532,this,"desc"),h(c,535,this),this.usageRecentSessions=h(c,536,this,[]),h(c,539,this),this.usageTimeZone=h(c,540,this,"local"),h(c,543,this),this.usageContextExpanded=h(c,544,this,!1),h(c,547,this),this.usageHeaderPinned=h(c,548,this,!1),h(c,551,this),this.usageSessionsTab=h(c,552,this,"all"),h(c,555,this),this.usageVisibleColumns=h(c,556,this,["channel","agent","provider","model","messages","tools","errors","duration"]),h(c,559,this),this.usageLogFilterRoles=h(c,560,this,[]),h(c,563,this),this.usageLogFilterTools=h(c,564,this,[]),h(c,567,this),this.usageLogFilterHasTools=h(c,568,this,!1),h(c,571,this),this.usageLogFilterQuery=h(c,572,this,""),h(c,575,this),this.usageQueryDebounceTimer=null,this.cronLoading=h(c,576,this,!1),h(c,579,this),this.cronJobs=h(c,580,this,[]),h(c,583,this),this.cronStatus=h(c,584,this,null),h(c,587,this),this.cronError=h(c,588,this,null),h(c,591,this),this.cronForm=h(c,592,this,{...sv}),h(c,595,this),this.cronRunsJobId=h(c,596,this,null),h(c,599,this),this.cronRuns=h(c,600,this,[]),h(c,603,this),this.cronBusy=h(c,604,this,!1),h(c,607,this),this.skillsLoading=h(c,608,this,!1),h(c,611,this),this.skillsReport=h(c,612,this,null),h(c,615,this),this.skillsError=h(c,616,this,null),h(c,619,this),this.skillsFilter=h(c,620,this,""),h(c,623,this),this.skillEdits=h(c,624,this,{}),h(c,627,this),this.skillsBusyKey=h(c,628,this,null),h(c,631,this),this.skillMessages=h(c,632,this,{}),h(c,635,this),this.debugLoading=h(c,636,this,!1),h(c,639,this),this.debugStatus=h(c,640,this,null),h(c,643,this),this.debugHealth=h(c,644,this,null),h(c,647,this),this.debugModels=h(c,648,this,[]),h(c,651,this),this.debugHeartbeat=h(c,652,this,null),h(c,655,this),this.debugCallMethod=h(c,656,this,""),h(c,659,this),this.debugCallParams=h(c,660,this,"{}"),h(c,663,this),this.debugCallResult=h(c,664,this,null),h(c,667,this),this.debugCallError=h(c,668,this,null),h(c,671,this),this.logsLoading=h(c,672,this,!1),h(c,675,this),this.logsError=h(c,676,this,null),h(c,679,this),this.logsFile=h(c,680,this,null),h(c,683,this),this.logsEntries=h(c,684,this,[]),h(c,687,this),this.logsFilterText=h(c,688,this,""),h(c,691,this),this.logsLevelFilters=h(c,692,this,{...nv}),h(c,695,this),this.logsAutoFollow=h(c,696,this,!0),h(c,699,this),this.logsTruncated=h(c,700,this,!1),h(c,703,this),this.logsCursor=h(c,704,this,null),h(c,707,this),this.logsLastFetchAt=h(c,708,this,null),h(c,711,this),this.logsLimit=h(c,712,this,500),h(c,715,this),this.logsMaxBytes=h(c,716,this,25e4),h(c,719,this),this.logsAtBottom=h(c,720,this,!0),h(c,723,this),this.client=null,this.chatScrollFrame=null,this.chatScrollTimeout=null,this.chatHasAutoScrolled=!1,this.chatUserNearBottom=!0,this.chatNewMessagesBelow=h(c,724,this,!1),h(c,727,this),this.nodesPollInterval=null,this.logsPollInterval=null,this.debugPollInterval=null,this.logsScrollFrame=null,this.toolStreamById=new Map,this.toolStreamOrder=[],this.refreshSessionsAfterChat=new Set,this.basePath="",this.popStateHandler=()=>kf(this),this.themeMedia=null,this.themeMediaHandler=null,this.topbarObserver=null}createRenderRoot(){return this}connectedCallback(){super.connectedCallback(),bv(this)}firstUpdated(){yv(this)}disconnectedCallback(){xv(this),super.disconnectedCallback()}updated(t){$v(this,t)}connect(){El(this)}handleChatScroll(t){Qp(this,t)}handleLogsScroll(t){Yp(this,t)}exportLogs(t,n){Jp(t,n)}resetToolStream(){hs(this)}resetChatScroll(){no(this)}scrollToBottom(t){no(this),hn(this,!0,!!t?.smooth)}async loadAssistantIdentity(){await _l(this)}applySettings(t){tt(this,t)}setTab(t){vf(this,t)}setTheme(t,n){mf(this,t,n)}async loadOverview(){await bl(this)}async loadCron(){await Jn(this)}async handleAbortChat(){await wl(this)}removeQueuedMessage(t){Jf(this,t)}async handleSendChat(t,n){await Zf(this,t,n)}async handleWhatsAppStart(t){await Fp(this,t)}async handleWhatsAppWait(){await Np(this)}async handleWhatsAppLogout(){await Op(this)}async handleChannelConfigSave(){await Bp(this)}async handleChannelConfigReload(){await Up(this)}handleNostrProfileEdit(t,n){Kp(this,t,n)}handleNostrProfileCancel(){jp(this)}handleNostrProfileFieldChange(t,n){Wp(this,t,n)}async handleNostrProfileSave(){await Gp(this)}async handleNostrProfileImport(){await Vp(this)}handleNostrProfileToggleAdvanced(){qp(this)}async handleExecApprovalDecision(t){const n=this.execApprovalQueue[0];if(!(!n||!this.client||this.execApprovalBusy)){this.execApprovalBusy=!0,this.execApprovalError=null;try{await this.client.request("exec.approval.resolve",{id:n.id,decision:t}),this.execApprovalQueue=this.execApprovalQueue.filter(s=>s.id!==n.id)}catch(s){this.execApprovalError=`Exec approval failed: ${String(s)}`}finally{this.execApprovalBusy=!1}}}handleGatewayUrlConfirm(){const t=this.pendingGatewayUrl;t&&(this.pendingGatewayUrl=null,tt(this,{...this.settings,gatewayUrl:t}),this.connect())}handleGatewayUrlCancel(){this.pendingGatewayUrl=null}handleOpenSidebar(t){this.sidebarCloseTimer!=null&&(window.clearTimeout(this.sidebarCloseTimer),this.sidebarCloseTimer=null),this.sidebarContent=t,this.sidebarError=null,this.sidebarOpen=!0}handleCloseSidebar(){this.sidebarOpen=!1,this.sidebarCloseTimer!=null&&window.clearTimeout(this.sidebarCloseTimer),this.sidebarCloseTimer=window.setTimeout(()=>{this.sidebarOpen||(this.sidebarContent=null,this.sidebarError=null,this.sidebarCloseTimer=null)},200)}handleSplitRatioChange(t){const n=Math.max(.4,Math.min(.7,t));this.splitRatio=n,this.applySettings({...this.settings,splitRatio:n})}render(){return Lx(this)}}c=Px(Ei);S(c,5,"settings",jg,w);S(c,5,"password",Kg,w);S(c,5,"tab",Hg,w);S(c,5,"onboarding",zg,w);S(c,5,"connected",Ug,w);S(c,5,"theme",Bg,w);S(c,5,"themeResolved",Og,w);S(c,5,"hello",Ng,w);S(c,5,"lastError",Fg,w);S(c,5,"eventLog",Dg,w);S(c,5,"assistantName",Pg,w);S(c,5,"assistantAvatar",Rg,w);S(c,5,"assistantAgentId",Ig,w);S(c,5,"sessionKey",Mg,w);S(c,5,"chatLoading",Lg,w);S(c,5,"chatSending",Eg,w);S(c,5,"chatMessage",Tg,w);S(c,5,"chatMessages",Cg,w);S(c,5,"chatToolMessages",_g,w);S(c,5,"chatStream",Ag,w);S(c,5,"chatStreamStartedAt",Sg,w);S(c,5,"chatRunId",kg,w);S(c,5,"compactionStatus",wg,w);S(c,5,"chatAvatarUrl",$g,w);S(c,5,"chatThinkingLevel",xg,w);S(c,5,"chatQueue",yg,w);S(c,5,"chatAttachments",bg,w);S(c,5,"chatManualRefreshInFlight",mg,w);S(c,5,"sidebarOpen",vg,w);S(c,5,"sidebarContent",fg,w);S(c,5,"sidebarError",hg,w);S(c,5,"splitRatio",pg,w);S(c,5,"nodesLoading",gg,w);S(c,5,"nodes",ug,w);S(c,5,"devicesLoading",dg,w);S(c,5,"devicesError",cg,w);S(c,5,"devicesList",lg,w);S(c,5,"execApprovalsLoading",rg,w);S(c,5,"execApprovalsSaving",og,w);S(c,5,"execApprovalsDirty",ag,w);S(c,5,"execApprovalsSnapshot",ig,w);S(c,5,"execApprovalsForm",sg,w);S(c,5,"execApprovalsSelectedAgent",ng,w);S(c,5,"execApprovalsTarget",tg,w);S(c,5,"execApprovalsTargetNodeId",eg,w);S(c,5,"execApprovalQueue",Xu,w);S(c,5,"execApprovalBusy",Zu,w);S(c,5,"execApprovalError",Ju,w);S(c,5,"pendingGatewayUrl",Yu,w);S(c,5,"configLoading",Qu,w);S(c,5,"configRaw",Vu,w);S(c,5,"configRawOriginal",Gu,w);S(c,5,"configValid",qu,w);S(c,5,"configIssues",Wu,w);S(c,5,"configSaving",ju,w);S(c,5,"configApplying",Ku,w);S(c,5,"updateRunning",Hu,w);S(c,5,"applySessionKey",zu,w);S(c,5,"configSnapshot",Uu,w);S(c,5,"configSchema",Bu,w);S(c,5,"configSchemaVersion",Ou,w);S(c,5,"configSchemaLoading",Nu,w);S(c,5,"configUiHints",Fu,w);S(c,5,"configForm",Du,w);S(c,5,"configFormOriginal",Pu,w);S(c,5,"configFormDirty",Ru,w);S(c,5,"configFormMode",Iu,w);S(c,5,"configSearchQuery",Mu,w);S(c,5,"configActiveSection",Lu,w);S(c,5,"configActiveSubsection",Eu,w);S(c,5,"channelsLoading",Tu,w);S(c,5,"channelsSnapshot",Cu,w);S(c,5,"channelsError",_u,w);S(c,5,"channelsLastSuccess",Au,w);S(c,5,"whatsappLoginMessage",Su,w);S(c,5,"whatsappLoginQrDataUrl",ku,w);S(c,5,"whatsappLoginConnected",wu,w);S(c,5,"whatsappBusy",$u,w);S(c,5,"nostrProfileFormState",xu,w);S(c,5,"nostrProfileAccountId",yu,w);S(c,5,"presenceLoading",bu,w);S(c,5,"presenceEntries",mu,w);S(c,5,"presenceError",vu,w);S(c,5,"presenceStatus",fu,w);S(c,5,"agentsLoading",hu,w);S(c,5,"agentsList",pu,w);S(c,5,"agentsError",gu,w);S(c,5,"agentsSelectedId",uu,w);S(c,5,"agentsPanel",du,w);S(c,5,"agentFilesLoading",cu,w);S(c,5,"agentFilesError",lu,w);S(c,5,"agentFilesList",ru,w);S(c,5,"agentFileContents",ou,w);S(c,5,"agentFileDrafts",au,w);S(c,5,"agentFileActive",iu,w);S(c,5,"agentFileSaving",su,w);S(c,5,"agentIdentityLoading",nu,w);S(c,5,"agentIdentityError",tu,w);S(c,5,"agentIdentityById",eu,w);S(c,5,"agentSkillsLoading",Xd,w);S(c,5,"agentSkillsError",Zd,w);S(c,5,"agentSkillsReport",Jd,w);S(c,5,"agentSkillsAgentId",Yd,w);S(c,5,"sessionsLoading",Qd,w);S(c,5,"sessionsResult",Vd,w);S(c,5,"sessionsError",Gd,w);S(c,5,"sessionsFilterActive",qd,w);S(c,5,"sessionsFilterLimit",Wd,w);S(c,5,"sessionsIncludeGlobal",jd,w);S(c,5,"sessionsIncludeUnknown",Kd,w);S(c,5,"usageLoading",Hd,w);S(c,5,"usageResult",zd,w);S(c,5,"usageCostSummary",Ud,w);S(c,5,"usageError",Bd,w);S(c,5,"usageStartDate",Od,w);S(c,5,"usageEndDate",Nd,w);S(c,5,"usageSelectedSessions",Fd,w);S(c,5,"usageSelectedDays",Dd,w);S(c,5,"usageSelectedHours",Pd,w);S(c,5,"usageChartMode",Rd,w);S(c,5,"usageDailyChartMode",Id,w);S(c,5,"usageTimeSeriesMode",Md,w);S(c,5,"usageTimeSeriesBreakdownMode",Ld,w);S(c,5,"usageTimeSeries",Ed,w);S(c,5,"usageTimeSeriesLoading",Td,w);S(c,5,"usageSessionLogs",Cd,w);S(c,5,"usageSessionLogsLoading",_d,w);S(c,5,"usageSessionLogsExpanded",Ad,w);S(c,5,"usageQuery",Sd,w);S(c,5,"usageQueryDraft",kd,w);S(c,5,"usageSessionSort",wd,w);S(c,5,"usageSessionSortDir",$d,w);S(c,5,"usageRecentSessions",xd,w);S(c,5,"usageTimeZone",yd,w);S(c,5,"usageContextExpanded",bd,w);S(c,5,"usageHeaderPinned",md,w);S(c,5,"usageSessionsTab",vd,w);S(c,5,"usageVisibleColumns",fd,w);S(c,5,"usageLogFilterRoles",hd,w);S(c,5,"usageLogFilterTools",pd,w);S(c,5,"usageLogFilterHasTools",gd,w);S(c,5,"usageLogFilterQuery",ud,w);S(c,5,"cronLoading",dd,w);S(c,5,"cronJobs",cd,w);S(c,5,"cronStatus",ld,w);S(c,5,"cronError",rd,w);S(c,5,"cronForm",od,w);S(c,5,"cronRunsJobId",ad,w);S(c,5,"cronRuns",id,w);S(c,5,"cronBusy",sd,w);S(c,5,"skillsLoading",nd,w);S(c,5,"skillsReport",td,w);S(c,5,"skillsError",ed,w);S(c,5,"skillsFilter",Xc,w);S(c,5,"skillEdits",Zc,w);S(c,5,"skillsBusyKey",Jc,w);S(c,5,"skillMessages",Yc,w);S(c,5,"debugLoading",Qc,w);S(c,5,"debugStatus",Vc,w);S(c,5,"debugHealth",Gc,w);S(c,5,"debugModels",qc,w);S(c,5,"debugHeartbeat",Wc,w);S(c,5,"debugCallMethod",jc,w);S(c,5,"debugCallParams",Kc,w);S(c,5,"debugCallResult",Hc,w);S(c,5,"debugCallError",zc,w);S(c,5,"logsLoading",Uc,w);S(c,5,"logsError",Bc,w);S(c,5,"logsFile",Oc,w);S(c,5,"logsEntries",Nc,w);S(c,5,"logsFilterText",Fc,w);S(c,5,"logsLevelFilters",Dc,w);S(c,5,"logsAutoFollow",Pc,w);S(c,5,"logsTruncated",Rc,w);S(c,5,"logsCursor",Ic,w);S(c,5,"logsLastFetchAt",Mc,w);S(c,5,"logsLimit",Lc,w);S(c,5,"logsMaxBytes",Ec,w);S(c,5,"logsAtBottom",Tc,w);S(c,5,"chatNewMessagesBelow",Cc,w);w=S(c,0,"AisopodApp",Wg,w);h(c,1,w);
//# sourceMappingURL=index-BoczOECE.js.map
