(function(){const e=document.createElement("link").relList;if(e&&e.supports&&e.supports("modulepreload"))return;for(const i of document.querySelectorAll('link[rel="modulepreload"]'))t(i);new MutationObserver(i=>{for(const n of i)if(n.type==="childList")for(const r of n.addedNodes)r.tagName==="LINK"&&r.rel==="modulepreload"&&t(r)}).observe(document,{childList:!0,subtree:!0});function s(i){const n={};return i.integrity&&(n.integrity=i.integrity),i.referrerPolicy&&(n.referrerPolicy=i.referrerPolicy),i.crossOrigin==="use-credentials"?n.credentials="include":i.crossOrigin==="anonymous"?n.credentials="omit":n.credentials="same-origin",n}function t(i){if(i.ep)return;i.ep=!0;const n=s(i);fetch(i.href,n)}})();const ee=globalThis,he=ee.ShadowRoot&&(ee.ShadyCSS===void 0||ee.ShadyCSS.nativeShadow)&&"adoptedStyleSheets"in Document.prototype&&"replace"in CSSStyleSheet.prototype,De=Symbol(),$e=new WeakMap;let Fe=class{constructor(e,s,t){if(this._$cssResult$=!0,t!==De)throw Error("CSSResult is not constructable. Use `unsafeCSS` or `css` instead.");this.cssText=e,this.t=s}get styleSheet(){let e=this.o;const s=this.t;if(he&&e===void 0){const t=s!==void 0&&s.length===1;t&&(e=$e.get(s)),e===void 0&&((this.o=e=new CSSStyleSheet).replaceSync(this.cssText),t&&$e.set(s,e))}return e}toString(){return this.cssText}};const Ve=o=>new Fe(typeof o=="string"?o:o+"",void 0,De),We=(o,e)=>{if(he)o.adoptedStyleSheets=e.map(s=>s instanceof CSSStyleSheet?s:s.styleSheet);else for(const s of e){const t=document.createElement("style"),i=ee.litNonce;i!==void 0&&t.setAttribute("nonce",i),t.textContent=s.cssText,o.appendChild(t)}},ge=he?o=>o:o=>o instanceof CSSStyleSheet?(e=>{let s="";for(const t of e.cssRules)s+=t.cssText;return Ve(s)})(o):o;const{is:Je,defineProperty:Ke,getOwnPropertyDescriptor:Ge,getOwnPropertyNames:Ye,getOwnPropertySymbols:Ze,getPrototypeOf:Qe}=Object,ie=globalThis,ve=ie.trustedTypes,Xe=ve?ve.emptyScript:"",es=ie.reactiveElementPolyfillSupport,B=(o,e)=>o,ce={toAttribute(o,e){switch(e){case Boolean:o=o?Xe:null;break;case Object:case Array:o=o==null?o:JSON.stringify(o)}return o},fromAttribute(o,e){let s=o;switch(e){case Boolean:s=o!==null;break;case Number:s=o===null?null:Number(o);break;case Object:case Array:try{s=JSON.parse(o)}catch{s=null}}return s}},Te=(o,e)=>!Je(o,e),me={attribute:!0,type:String,converter:ce,reflect:!1,useDefault:!1,hasChanged:Te};Symbol.metadata??=Symbol("metadata"),ie.litPropertyMetadata??=new WeakMap;let O=class extends HTMLElement{static addInitializer(e){this._$Ei(),(this.l??=[]).push(e)}static get observedAttributes(){return this.finalize(),this._$Eh&&[...this._$Eh.keys()]}static createProperty(e,s=me){if(s.state&&(s.attribute=!1),this._$Ei(),this.prototype.hasOwnProperty(e)&&((s=Object.create(s)).wrapped=!0),this.elementProperties.set(e,s),!s.noAccessor){const t=Symbol(),i=this.getPropertyDescriptor(e,t,s);i!==void 0&&Ke(this.prototype,e,i)}}static getPropertyDescriptor(e,s,t){const{get:i,set:n}=Ge(this.prototype,e)??{get(){return this[s]},set(r){this[s]=r}};return{get:i,set(r){const c=i?.call(this);n?.call(this,r),this.requestUpdate(e,c,t)},configurable:!0,enumerable:!0}}static getPropertyOptions(e){return this.elementProperties.get(e)??me}static _$Ei(){if(this.hasOwnProperty(B("elementProperties")))return;const e=Qe(this);e.finalize(),e.l!==void 0&&(this.l=[...e.l]),this.elementProperties=new Map(e.elementProperties)}static finalize(){if(this.hasOwnProperty(B("finalized")))return;if(this.finalized=!0,this._$Ei(),this.hasOwnProperty(B("properties"))){const s=this.properties,t=[...Ye(s),...Ze(s)];for(const i of t)this.createProperty(i,s[i])}const e=this[Symbol.metadata];if(e!==null){const s=litPropertyMetadata.get(e);if(s!==void 0)for(const[t,i]of s)this.elementProperties.set(t,i)}this._$Eh=new Map;for(const[s,t]of this.elementProperties){const i=this._$Eu(s,t);i!==void 0&&this._$Eh.set(i,s)}this.elementStyles=this.finalizeStyles(this.styles)}static finalizeStyles(e){const s=[];if(Array.isArray(e)){const t=new Set(e.flat(1/0).reverse());for(const i of t)s.unshift(ge(i))}else e!==void 0&&s.push(ge(e));return s}static _$Eu(e,s){const t=s.attribute;return t===!1?void 0:typeof t=="string"?t:typeof e=="string"?e.toLowerCase():void 0}constructor(){super(),this._$Ep=void 0,this.isUpdatePending=!1,this.hasUpdated=!1,this._$Em=null,this._$Ev()}_$Ev(){this._$ES=new Promise(e=>this.enableUpdating=e),this._$AL=new Map,this._$E_(),this.requestUpdate(),this.constructor.l?.forEach(e=>e(this))}addController(e){(this._$EO??=new Set).add(e),this.renderRoot!==void 0&&this.isConnected&&e.hostConnected?.()}removeController(e){this._$EO?.delete(e)}_$E_(){const e=new Map,s=this.constructor.elementProperties;for(const t of s.keys())this.hasOwnProperty(t)&&(e.set(t,this[t]),delete this[t]);e.size>0&&(this._$Ep=e)}createRenderRoot(){const e=this.shadowRoot??this.attachShadow(this.constructor.shadowRootOptions);return We(e,this.constructor.elementStyles),e}connectedCallback(){this.renderRoot??=this.createRenderRoot(),this.enableUpdating(!0),this._$EO?.forEach(e=>e.hostConnected?.())}enableUpdating(e){}disconnectedCallback(){this._$EO?.forEach(e=>e.hostDisconnected?.())}attributeChangedCallback(e,s,t){this._$AK(e,t)}_$ET(e,s){const t=this.constructor.elementProperties.get(e),i=this.constructor._$Eu(e,t);if(i!==void 0&&t.reflect===!0){const n=(t.converter?.toAttribute!==void 0?t.converter:ce).toAttribute(s,t.type);this._$Em=e,n==null?this.removeAttribute(i):this.setAttribute(i,n),this._$Em=null}}_$AK(e,s){const t=this.constructor,i=t._$Eh.get(e);if(i!==void 0&&this._$Em!==i){const n=t.getPropertyOptions(i),r=typeof n.converter=="function"?{fromAttribute:n.converter}:n.converter?.fromAttribute!==void 0?n.converter:ce;this._$Em=i;const c=r.fromAttribute(s,n.type);this[i]=c??this._$Ej?.get(i)??c,this._$Em=null}}requestUpdate(e,s,t,i=!1,n){if(e!==void 0){const r=this.constructor;if(i===!1&&(n=this[e]),t??=r.getPropertyOptions(e),!((t.hasChanged??Te)(n,s)||t.useDefault&&t.reflect&&n===this._$Ej?.get(e)&&!this.hasAttribute(r._$Eu(e,t))))return;this.C(e,s,t)}this.isUpdatePending===!1&&(this._$ES=this._$EP())}C(e,s,{useDefault:t,reflect:i,wrapped:n},r){t&&!(this._$Ej??=new Map).has(e)&&(this._$Ej.set(e,r??s??this[e]),n!==!0||r!==void 0)||(this._$AL.has(e)||(this.hasUpdated||t||(s=void 0),this._$AL.set(e,s)),i===!0&&this._$Em!==e&&(this._$Eq??=new Set).add(e))}async _$EP(){this.isUpdatePending=!0;try{await this._$ES}catch(s){Promise.reject(s)}const e=this.scheduleUpdate();return e!=null&&await e,!this.isUpdatePending}scheduleUpdate(){return this.performUpdate()}performUpdate(){if(!this.isUpdatePending)return;if(!this.hasUpdated){if(this.renderRoot??=this.createRenderRoot(),this._$Ep){for(const[i,n]of this._$Ep)this[i]=n;this._$Ep=void 0}const t=this.constructor.elementProperties;if(t.size>0)for(const[i,n]of t){const{wrapped:r}=n,c=this[i];r!==!0||this._$AL.has(i)||c===void 0||this.C(i,void 0,n,c)}}let e=!1;const s=this._$AL;try{e=this.shouldUpdate(s),e?(this.willUpdate(s),this._$EO?.forEach(t=>t.hostUpdate?.()),this.update(s)):this._$EM()}catch(t){throw e=!1,this._$EM(),t}e&&this._$AE(s)}willUpdate(e){}_$AE(e){this._$EO?.forEach(s=>s.hostUpdated?.()),this.hasUpdated||(this.hasUpdated=!0,this.firstUpdated(e)),this.updated(e)}_$EM(){this._$AL=new Map,this.isUpdatePending=!1}get updateComplete(){return this.getUpdateComplete()}getUpdateComplete(){return this._$ES}shouldUpdate(e){return!0}update(e){this._$Eq&&=this._$Eq.forEach(s=>this._$ET(s,this[s])),this._$EM()}updated(e){}firstUpdated(e){}};O.elementStyles=[],O.shadowRootOptions={mode:"open"},O[B("elementProperties")]=new Map,O[B("finalized")]=new Map,es?.({ReactiveElement:O}),(ie.reactiveElementVersions??=[]).push("2.1.2");const pe=globalThis,be=o=>o,te=pe.trustedTypes,we=te?te.createPolicy("lit-html",{createHTML:o=>o}):void 0,Oe="$lit$",E=`lit$${Math.random().toFixed(9).slice(2)}$`,Me="?"+E,ss=`<${Me}>`,U=document,W=()=>U.createComment(""),J=o=>o===null||typeof o!="object"&&typeof o!="function",ye=Array.isArray,ts=o=>ye(o)||typeof o?.[Symbol.iterator]=="function",ne=`[ 	
\f\r]`,I=/<(?:(!--|\/[^a-zA-Z])|(\/?[a-zA-Z][^>\s]*)|(\/?$))/g,qe=/-->/g,Se=/>/g,x=RegExp(`>|${ne}(?:([^\\s"'>=/]+)(${ne}*=${ne}*(?:[^ 	
\f\r"'\`<>=]|("|')|))|$)`,"g"),ke=/'/g,Ae=/"/g,ze=/^(?:script|style|textarea|title)$/i,is=o=>(e,...s)=>({_$litType$:o,strings:e,values:s}),a=is(1),z=Symbol.for("lit-noChange"),u=Symbol.for("lit-nothing"),Re=new WeakMap,L=U.createTreeWalker(U,129);function He(o,e){if(!ye(o)||!o.hasOwnProperty("raw"))throw Error("invalid template strings array");return we!==void 0?we.createHTML(e):e}const os=(o,e)=>{const s=o.length-1,t=[];let i,n=e===2?"<svg>":e===3?"<math>":"",r=I;for(let c=0;c<s;c++){const l=o[c];let d,_,h=-1,p=0;for(;p<l.length&&(r.lastIndex=p,_=r.exec(l),_!==null);)p=r.lastIndex,r===I?_[1]==="!--"?r=qe:_[1]!==void 0?r=Se:_[2]!==void 0?(ze.test(_[2])&&(i=RegExp("</"+_[2],"g")),r=x):_[3]!==void 0&&(r=x):r===x?_[0]===">"?(r=i??I,h=-1):_[1]===void 0?h=-2:(h=r.lastIndex-_[2].length,d=_[1],r=_[3]===void 0?x:_[3]==='"'?Ae:ke):r===Ae||r===ke?r=x:r===qe||r===Se?r=I:(r=x,i=void 0);const y=r===x&&o[c+1].startsWith("/>")?" ":"";n+=r===I?l+ss:h>=0?(t.push(d),l.slice(0,h)+Oe+l.slice(h)+E+y):l+E+(h===-2?c:y)}return[He(o,n+(o[s]||"<?>")+(e===2?"</svg>":e===3?"</math>":"")),t]};class K{constructor({strings:e,_$litType$:s},t){let i;this.parts=[];let n=0,r=0;const c=e.length-1,l=this.parts,[d,_]=os(e,s);if(this.el=K.createElement(d,t),L.currentNode=this.el.content,s===2||s===3){const h=this.el.content.firstChild;h.replaceWith(...h.childNodes)}for(;(i=L.nextNode())!==null&&l.length<c;){if(i.nodeType===1){if(i.hasAttributes())for(const h of i.getAttributeNames())if(h.endsWith(Oe)){const p=_[r++],y=i.getAttribute(h).split(E),$=/([.?@])?(.*)/.exec(p);l.push({type:1,index:n,name:$[2],strings:y,ctor:$[1]==="."?rs:$[1]==="?"?as:$[1]==="@"?ds:oe}),i.removeAttribute(h)}else h.startsWith(E)&&(l.push({type:6,index:n}),i.removeAttribute(h));if(ze.test(i.tagName)){const h=i.textContent.split(E),p=h.length-1;if(p>0){i.textContent=te?te.emptyScript:"";for(let y=0;y<p;y++)i.append(h[y],W()),L.nextNode(),l.push({type:2,index:++n});i.append(h[p],W())}}}else if(i.nodeType===8)if(i.data===Me)l.push({type:2,index:n});else{let h=-1;for(;(h=i.data.indexOf(E,h+1))!==-1;)l.push({type:7,index:n}),h+=E.length-1}n++}}static createElement(e,s){const t=U.createElement("template");return t.innerHTML=e,t}}function H(o,e,s=o,t){if(e===z)return e;let i=t!==void 0?s._$Co?.[t]:s._$Cl;const n=J(e)?void 0:e._$litDirective$;return i?.constructor!==n&&(i?._$AO?.(!1),n===void 0?i=void 0:(i=new n(o),i._$AT(o,s,t)),t!==void 0?(s._$Co??=[])[t]=i:s._$Cl=i),i!==void 0&&(e=H(o,i._$AS(o,e.values),i,t)),e}class ns{constructor(e,s){this._$AV=[],this._$AN=void 0,this._$AD=e,this._$AM=s}get parentNode(){return this._$AM.parentNode}get _$AU(){return this._$AM._$AU}u(e){const{el:{content:s},parts:t}=this._$AD,i=(e?.creationScope??U).importNode(s,!0);L.currentNode=i;let n=L.nextNode(),r=0,c=0,l=t[0];for(;l!==void 0;){if(r===l.index){let d;l.type===2?d=new G(n,n.nextSibling,this,e):l.type===1?d=new l.ctor(n,l.name,l.strings,this,e):l.type===6&&(d=new ls(n,this,e)),this._$AV.push(d),l=t[++c]}r!==l?.index&&(n=L.nextNode(),r++)}return L.currentNode=U,i}p(e){let s=0;for(const t of this._$AV)t!==void 0&&(t.strings!==void 0?(t._$AI(e,t,s),s+=t.strings.length-2):t._$AI(e[s])),s++}}class G{get _$AU(){return this._$AM?._$AU??this._$Cv}constructor(e,s,t,i){this.type=2,this._$AH=u,this._$AN=void 0,this._$AA=e,this._$AB=s,this._$AM=t,this.options=i,this._$Cv=i?.isConnected??!0}get parentNode(){let e=this._$AA.parentNode;const s=this._$AM;return s!==void 0&&e?.nodeType===11&&(e=s.parentNode),e}get startNode(){return this._$AA}get endNode(){return this._$AB}_$AI(e,s=this){e=H(this,e,s),J(e)?e===u||e==null||e===""?(this._$AH!==u&&this._$AR(),this._$AH=u):e!==this._$AH&&e!==z&&this._(e):e._$litType$!==void 0?this.$(e):e.nodeType!==void 0?this.T(e):ts(e)?this.k(e):this._(e)}O(e){return this._$AA.parentNode.insertBefore(e,this._$AB)}T(e){this._$AH!==e&&(this._$AR(),this._$AH=this.O(e))}_(e){this._$AH!==u&&J(this._$AH)?this._$AA.nextSibling.data=e:this.T(U.createTextNode(e)),this._$AH=e}$(e){const{values:s,_$litType$:t}=e,i=typeof t=="number"?this._$AC(e):(t.el===void 0&&(t.el=K.createElement(He(t.h,t.h[0]),this.options)),t);if(this._$AH?._$AD===i)this._$AH.p(s);else{const n=new ns(i,this),r=n.u(this.options);n.p(s),this.T(r),this._$AH=n}}_$AC(e){let s=Re.get(e.strings);return s===void 0&&Re.set(e.strings,s=new K(e)),s}k(e){ye(this._$AH)||(this._$AH=[],this._$AR());const s=this._$AH;let t,i=0;for(const n of e)i===s.length?s.push(t=new G(this.O(W()),this.O(W()),this,this.options)):t=s[i],t._$AI(n),i++;i<s.length&&(this._$AR(t&&t._$AB.nextSibling,i),s.length=i)}_$AR(e=this._$AA.nextSibling,s){for(this._$AP?.(!1,!0,s);e!==this._$AB;){const t=be(e).nextSibling;be(e).remove(),e=t}}setConnected(e){this._$AM===void 0&&(this._$Cv=e,this._$AP?.(e))}}class oe{get tagName(){return this.element.tagName}get _$AU(){return this._$AM._$AU}constructor(e,s,t,i,n){this.type=1,this._$AH=u,this._$AN=void 0,this.element=e,this.name=s,this._$AM=i,this.options=n,t.length>2||t[0]!==""||t[1]!==""?(this._$AH=Array(t.length-1).fill(new String),this.strings=t):this._$AH=u}_$AI(e,s=this,t,i){const n=this.strings;let r=!1;if(n===void 0)e=H(this,e,s,0),r=!J(e)||e!==this._$AH&&e!==z,r&&(this._$AH=e);else{const c=e;let l,d;for(e=n[0],l=0;l<n.length-1;l++)d=H(this,c[t+l],s,l),d===z&&(d=this._$AH[l]),r||=!J(d)||d!==this._$AH[l],d===u?e=u:e!==u&&(e+=(d??"")+n[l+1]),this._$AH[l]=d}r&&!i&&this.j(e)}j(e){e===u?this.element.removeAttribute(this.name):this.element.setAttribute(this.name,e??"")}}class rs extends oe{constructor(){super(...arguments),this.type=3}j(e){this.element[this.name]=e===u?void 0:e}}class as extends oe{constructor(){super(...arguments),this.type=4}j(e){this.element.toggleAttribute(this.name,!!e&&e!==u)}}class ds extends oe{constructor(e,s,t,i,n){super(e,s,t,i,n),this.type=5}_$AI(e,s=this){if((e=H(this,e,s,0)??u)===z)return;const t=this._$AH,i=e===u&&t!==u||e.capture!==t.capture||e.once!==t.once||e.passive!==t.passive,n=e!==u&&(t===u||i);i&&this.element.removeEventListener(this.name,this,t),n&&this.element.addEventListener(this.name,this,e),this._$AH=e}handleEvent(e){typeof this._$AH=="function"?this._$AH.call(this.options?.host??this.element,e):this._$AH.handleEvent(e)}}class ls{constructor(e,s,t){this.element=e,this.type=6,this._$AN=void 0,this._$AM=s,this.options=t}get _$AU(){return this._$AM._$AU}_$AI(e){H(this,e)}}const cs=pe.litHtmlPolyfillSupport;cs?.(K,G),(pe.litHtmlVersions??=[]).push("3.3.3");const _s=(o,e,s)=>{const t=s?.renderBefore??e;let i=t._$litPart$;if(i===void 0){const n=s?.renderBefore??null;t._$litPart$=i=new G(e.insertBefore(W(),n),n,void 0,s??{})}return i._$AI(o),i};const fe=globalThis;class w extends O{constructor(){super(...arguments),this.renderOptions={host:this},this._$Do=void 0}createRenderRoot(){const e=super.createRenderRoot();return this.renderOptions.renderBefore??=e.firstChild,e}update(e){const s=this.render();this.hasUpdated||(this.renderOptions.isConnected=this.isConnected),super.update(e),this._$Do=_s(s,this.renderRoot,this.renderOptions)}connectedCallback(){super.connectedCallback(),this._$Do?.setConnected(!0)}disconnectedCallback(){super.disconnectedCallback(),this._$Do?.setConnected(!1)}render(){return z}}w._$litElement$=!0,w.finalized=!0,fe.litElementHydrateSupport?.({LitElement:w});const us=fe.litElementPolyfillSupport;us?.({LitElement:w});(fe.litElementVersions??=[]).push("4.2.2");class Ie extends Error{status;constructor(e,s){super(s),this.name="HttpError",this.status=e}}async function m(o,e){const s=await fetch(o,{cache:"no-store",signal:e});if(!s.ok){const t=await s.json().catch(()=>({}));throw new Ie(s.status,t.error??`Request failed (${s.status})`)}return s.json()}function R(o){return o instanceof Error&&o.name==="AbortError"}function j(o,e,s=!1){const t=s?{hour:"2-digit",minute:"2-digit",second:"2-digit"}:{dateStyle:"medium",timeStyle:"medium"};return e==="utc"&&(t.timeZone="UTC"),new Intl.DateTimeFormat(void 0,t).format(new Date(o))}function hs(o,e){const s=new Date(o),t=new Date,i=e==="utc"?s.getUTCFullYear():s.getFullYear(),n=e==="utc"?t.getUTCFullYear():t.getFullYear(),r={month:"short",day:"numeric",hour:"2-digit",minute:"2-digit"};return i!==n&&(r.year="numeric"),e==="utc"&&(r.timeZone="UTC"),new Intl.DateTimeFormat(void 0,r).format(s)}function ps(o,e){const s=Math.max(0,e-o);if(s<1e3)return`${s.toLocaleString()} ms`;const t=Math.floor(s/1e3);if(t<60)return`${t}s`;const i=Math.floor(t/60);if(i<60)return`${i}m ${t%60}s`;const n=Math.floor(i/60);return n<24?`${n}h ${i%60}m`:`${Math.floor(n/24)}d ${n%24}h`}function M(o){return`${o.day}:${o.row_id}`}function b(o,e=10){return o?o.length>e?`…${o.slice(-e)}`:o:"—"}function ys(o){const e=o.inbound_req_url??o.endpoint;return F(e)}function Ee(o){const e=o.toLowerCase().replaceAll("_","-");return e==="authorization"||e==="password"||e==="code"||e==="signature"||e==="sig"||e.includes("api-key")||e.includes("access-key")||e.includes("token")||e.includes("secret")||e.includes("credential")}function F(o){if(!o)return"unknown endpoint";try{const e=new URL(o,window.location.origin);for(const s of new Set(e.searchParams.keys()))Ee(s)&&e.searchParams.set(s,"REDACTED");return`${e.pathname}${e.search}`}catch{return o.replace(/([?&]([^=&]+)=)([^&]*)/g,(e,s,t)=>{let i=t;try{i=decodeURIComponent(t)}catch{}return Ee(i)?`${s}REDACTED`:e})}}function fs(o){if(o.request_error)return{label:"ERR",tone:"error",title:o.request_error};const e=o.inbound_resp_status??o.outbound_resp_status??o.status;if(e===null)return{label:"—",tone:"neutral",title:"No response status persisted"};const s=o.inbound_resp_status!==null?"Client response":o.outbound_resp_status!==null?"Provider response":"Request";return e>=400?{label:String(e),tone:"error",title:`${s}: ${e}`}:e>=300?{label:String(e),tone:"warning",title:`${s}: ${e}`}:{label:String(e),tone:"success",title:`${s}: ${e}`}}function $s(o){const e=o.status;return e===null?{label:"—",tone:"neutral",title:"No status stored for the current session head"}:e>=400?{label:String(e),tone:"error",title:`Current head status: ${e}`}:e>=300?{label:String(e),tone:"warning",title:`Current head status: ${e}`}:{label:String(e),tone:"success",title:`Current head status: ${e}`}}function P(o){return o.detail}function v(o,e){const s=o[e];return typeof s=="string"?s:void 0}function Q(o,e){const s=o[e];return typeof s=="number"?s:void 0}const re="••••••••";function ae(o){const e=o.toLowerCase().replaceAll("_","-");return e==="authorization"||e==="proxy-authorization"||e==="cookie"||e==="set-cookie"||e.includes("api-key")||e.includes("token")||e.includes("secret")}function V(o){if(Array.isArray(o))return o.length===2&&typeof o[0]=="string"&&ae(o[0])?[o[0],re]:o.map(e=>V(e));if(o!==null&&typeof o=="object")return Object.fromEntries(Object.entries(o).map(([e,s])=>[e,ae(e)?re:V(s)]));if(typeof o=="string")try{return V(JSON.parse(o))}catch{return o.replace(/^([^:\r\n]+)(:\s*)(.*)$/gm,(e,s,t)=>ae(s.trim())?`${s}${t}${re}`:e)}return o}function _e(o){return Array.isArray(o)?o.map(e=>_e(e)):o!==null&&typeof o=="object"?Object.fromEntries(Object.entries(o).map(([e,s])=>[e,gs(e)?V(s):_e(s)])):o}function gs(o){const e=o.replace(/([a-z0-9])([A-Z])/g,"$1_$2").toLowerCase().replace(/[-\s]+/g,"_");return e==="headers"||e.endsWith("_headers")}function ue(o){return Array.isArray(o)?o.map(e=>ue(e)):o!==null&&typeof o=="object"?Object.fromEntries(Object.entries(o).map(([e,s])=>[e,e.toLowerCase().endsWith("_url")&&typeof s=="string"?F(s):ue(s)])):o}function vs(o){if(typeof o=="string")try{return JSON.stringify(JSON.parse(o),null,2)}catch{return o}return JSON.stringify(o,null,2)??String(o)}function ms(o){if(Array.isArray(o))return`${o.length} item${o.length===1?"":"s"}`;if(o!==null&&typeof o=="object"){const e=Object.keys(o).length;return`${e} field${e===1?"":"s"}`}return typeof o=="string"?`${new Blob([o]).size.toLocaleString()} bytes`:typeof o}class bs extends w{static properties={label:{type:String},value:{attribute:!1},load_url:{type:String},is_headers:{type:Boolean},redact_record_headers:{type:Boolean},open:{type:Boolean,state:!0},wrap:{type:Boolean,state:!0},revealed:{type:Boolean,state:!0},copy_state:{type:String,state:!0},load_state:{type:String,state:!0},loaded_value:{attribute:!1,state:!0},error_message:{type:String,state:!0}};load_controller;copy_timeout;constructor(){super(),this.label="Payload",this.is_headers=!1,this.redact_record_headers=!1,this.open=!1,this.wrap=!0,this.revealed=!1,this.copy_state="idle",this.load_state="idle"}createRenderRoot(){return this}disconnectedCallback(){this.load_controller?.abort(),this.copy_timeout!==void 0&&window.clearTimeout(this.copy_timeout),super.disconnectedCallback()}willUpdate(e){!e.has("value")&&!e.has("load_url")||(this.load_controller?.abort(),this.load_controller=void 0,this.copy_timeout!==void 0&&(window.clearTimeout(this.copy_timeout),this.copy_timeout=void 0),this.open=!1,this.revealed=!1,this.copy_state="idle",this.load_state="idle",this.loaded_value=void 0,this.error_message=void 0)}effectiveValue(){return this.load_state==="ready"?this.loaded_value:this.value}displayedValue(){const e=this.effectiveValue(),s=this.redact_record_headers?ue(e):e,t=this.revealed?s:this.redact_record_headers?_e(s):this.is_headers?V(s):s;return vs(t)}toggleOpen(e){this.open=e.currentTarget.open,this.open&&this.value===void 0&&this.load_url&&this.load_state==="idle"&&this.loadPayload()}async loadPayload(){const e=this.load_url;if(!e)return;this.load_controller?.abort();const s=new AbortController;this.load_controller=s,this.load_state="loading",this.error_message=void 0;try{const t=await m(e,s.signal);if(this.load_controller!==s||this.load_url!==e)return;const i=new URL(e,window.location.origin).searchParams.get("field");if(!i||t.field!==i)throw new Error("Payload response did not match the requested field");this.loaded_value=t.value,this.load_state="ready"}catch(t){if(this.load_controller!==s||R(t))return;this.load_state="error",this.error_message=t instanceof Error?t.message:"Unable to load payload"}finally{this.load_controller===s&&(this.load_controller=void 0)}}async copyValue(){try{await navigator.clipboard.writeText(this.displayedValue()),this.copy_state="copied",this.copy_timeout!==void 0&&window.clearTimeout(this.copy_timeout),this.copy_timeout=window.setTimeout(()=>{this.copy_state="idle",this.copy_timeout=void 0},1500)}catch{this.copy_state="error"}}render(){if(!this.load_url&&(this.value===null||this.value===void 0||this.value===""))return u;const e=this.effectiveValue(),s=this.is_headers||this.redact_record_headers,t=this.load_state==="loading"?"Loading…":this.load_state==="error"?"Load failed":e===null?"No payload":e===void 0?"Load on open":ms(e);return a`
      <details class="payload-panel" ?open=${this.open} @toggle=${this.toggleOpen}>
        <summary>
          <span>${this.label}</span>
          <span class="payload-summary">${t}</span>
        </summary>
        ${this.open?this.load_state==="loading"?a`<div class="payload-state" role="status"><span class="spinner" aria-hidden="true"></span>Loading payload…</div>`:this.load_state==="error"?a`
                  <div class="payload-state payload-error" role="alert">
                    <span>${this.error_message}</span>
                    <button type="button" @click=${()=>{this.loadPayload()}}>Retry</button>
                  </div>
                `:e==null||e===""?a`<div class="payload-state">No payload was persisted.</div>`:a`
                    <div class="payload-toolbar">
                      <button type="button" @click=${()=>{this.copyValue()}}>
                        ${this.copy_state==="copied"?"Copied":this.copy_state==="error"?"Copy failed":"Copy"}
                      </button>
                      <button type="button" aria-pressed=${String(this.wrap)} @click=${()=>this.wrap=!this.wrap}>
                        ${this.wrap?"No wrap":"Wrap"}
                      </button>
                      ${s?a`
                            <button
                              type="button"
                              class=${this.revealed?"danger-button":""}
                              aria-pressed=${String(this.revealed)}
                              @click=${()=>this.revealed=!this.revealed}
                            >
                              ${this.revealed?"Hide sensitive":"Reveal sensitive"}
                            </button>
                          `:u}
                      <span class="payload-security-note">
                        ${s&&!this.revealed?"Sensitive headers redacted":""}
                      </span>
                    </div>
                    <pre class=${this.wrap?"wrap":"nowrap"}><code>${this.displayedValue()}</code></pre>
                  `:u}
      </details>
    `}}customElements.define("payload-panel",bs);const C=[{id:"overview",label:"Overview"},{id:"client",label:"Client"},{id:"provider",label:"Provider"},{id:"raw",label:"Raw"}];function N(o){return o==null||o===""?"—":typeof o=="boolean"?o?"Yes":"No":String(o)}function ws(o){if(o!==null&&typeof o=="object"&&!Array.isArray(o))return o;if(typeof o=="string")try{const e=JSON.parse(o);return e!==null&&typeof e=="object"&&!Array.isArray(e)?e:void 0}catch{return}}function xe(o,e,s){return ws(o[e])?.[s]??o[s]}function A(o,e,s,t){return`/api/request-payload?${new URLSearchParams({day:o,request_id:e,row_id:s,field:t}).toString()}`}function Ce(o){return o===void 0?"neutral":o>=400?"error":o>=300?"warning":"success"}class qs extends w{static properties={detail:{attribute:!1},summary:{attribute:!1},state:{type:String},error_message:{type:String},active_tab:{type:String},timezone:{type:String}};createRenderRoot(){return this}openSession(e){this.dispatchEvent(new CustomEvent("open-session",{detail:e,bubbles:!0,composed:!0}))}retry(){this.dispatchEvent(new CustomEvent("detail-retry",{bubbles:!0,composed:!0}))}close(){this.dispatchEvent(new CustomEvent("detail-close",{bubbles:!0,composed:!0}))}selectTab(e){this.dispatchEvent(new CustomEvent("detail-tab-change",{detail:e,bubbles:!0,composed:!0}))}tabKeydown(e){const s=C.findIndex(r=>r.id===this.active_tab);let t;if(e.key==="ArrowRight"?t=(s+1)%C.length:e.key==="ArrowLeft"?t=(s-1+C.length)%C.length:e.key==="Home"?t=0:e.key==="End"&&(t=C.length-1),t===void 0)return;e.preventDefault();const i=C[t];this.selectTab(i.id),this.querySelectorAll("[role=tab]")[t]?.focus()}renderOverview(e){const s=Q(e,"ts"),t=xe(e,"ctx_json","latency_ms"),i=xe(e,"params_json","stream"),n=[["Timestamp",s===void 0?void 0:j(s,this.timezone)],["Storage day",this.detail?.day],["Endpoint",e.endpoint],["Model",e.model],["Provider",e.provider_id],["Account",e.account_id],["Latency",typeof t=="number"?`${t} ms`:t],["Streaming",i]],r=Q(e,"inbound_resp_status"),c=Q(e,"outbound_resp_status"),l=Q(e,"status");return a`
      <section class="flow-grid" aria-label="Request flow">
        <div>
          <span>Client request</span>
          <strong>${v(e,"inbound_req_method")??"—"}</strong>
        </div>
        <span class="flow-arrow" aria-hidden="true">→</span>
        <div>
          <span>Provider response</span>
          <strong class="status-text ${Ce(c)}">${N(c)}</strong>
        </div>
        <span class="flow-arrow" aria-hidden="true">→</span>
        <div>
          <span>Client response</span>
          <strong class="status-text ${Ce(r??l)}">
            ${N(r??l)}
          </strong>
        </div>
      </section>
      <dl class="metadata-grid">
        ${n.map(([d,_])=>a`
            <div>
              <dt>${d}</dt>
              <dd title=${N(_)}>${N(_)}</dd>
            </div>
          `)}
      </dl>
      <div class="payload-stack">
        <payload-panel label="Request parameters" .value=${e.params_json}></payload-panel>
        <payload-panel label="Usage" .value=${e.usage_json}></payload-panel>
        <payload-panel label="Request context" .value=${e.ctx_json}></payload-panel>
      </div>
    `}renderClient(e,s,t,i){return a`
      <section class="payload-group">
        <div class="payload-group-heading">
          <div><span class="direction-label">Incoming</span><h3>Client request</h3></div>
          <span>${v(e,"inbound_req_method")??"—"} ${F(v(e,"inbound_req_url"))}</span>
        </div>
        <payload-panel
          label="Request headers"
          .value=${e.inbound_req_headers}
          .load_url=${A(s,t,i,"inbound_req_headers")}
          .is_headers=${!0}
        ></payload-panel>
        <payload-panel
          label="Request body"
          .value=${e.inbound_req_body}
          .load_url=${A(s,t,i,"inbound_req_body")}
        ></payload-panel>
      </section>
      <section class="payload-group">
        <div class="payload-group-heading">
          <div><span class="direction-label">Outgoing</span><h3>Client response</h3></div>
          <span>Status ${N(e.inbound_resp_status??e.status)}</span>
        </div>
        <payload-panel
          label="Response headers"
          .value=${e.inbound_resp_headers}
          .load_url=${A(s,t,i,"inbound_resp_headers")}
          .is_headers=${!0}
        ></payload-panel>
        <payload-panel
          label="Response body"
          .value=${e.inbound_resp_body}
          .load_url=${A(s,t,i,"inbound_resp_body")}
        ></payload-panel>
      </section>
    `}renderProvider(e,s,t,i){return a`
      <section class="payload-group">
        <div class="payload-group-heading">
          <div><span class="direction-label">Outgoing</span><h3>Provider request</h3></div>
          <span>${v(e,"outbound_req_method")??"—"} ${F(v(e,"outbound_req_url"))}</span>
        </div>
        <payload-panel
          label="Request headers"
          .value=${e.outbound_req_headers}
          .load_url=${A(s,t,i,"outbound_req_headers")}
          .is_headers=${!0}
        ></payload-panel>
        <payload-panel
          label="Request body"
          .value=${e.outbound_req_body}
          .load_url=${A(s,t,i,"outbound_req_body")}
        ></payload-panel>
      </section>
      <section class="payload-group">
        <div class="payload-group-heading">
          <div><span class="direction-label">Incoming</span><h3>Provider response</h3></div>
          <span>Status ${N(e.outbound_resp_status)}</span>
        </div>
        <payload-panel
          label="Response headers"
          .value=${e.outbound_resp_headers}
          .load_url=${A(s,t,i,"outbound_resp_headers")}
          .is_headers=${!0}
        ></payload-panel>
        <payload-panel
          label="Response body"
          .value=${e.outbound_resp_body}
          .load_url=${A(s,t,i,"outbound_resp_body")}
        ></payload-panel>
      </section>
    `}renderTab(e,s,t,i){switch(this.active_tab){case"client":return this.renderClient(e,s,t,i);case"provider":return this.renderProvider(e,s,t,i);case"raw":return a`
          <p class="raw-note">Network headers and bodies remain lazy and are not included in this overview record.</p>
          <payload-panel
            label="Persisted overview record"
            .value=${e}
            .redact_record_headers=${!0}
          ></payload-panel>
        `;default:return this.renderOverview(e)}}render(){if(!this.detail)return this.state==="loading"?a`
          <section class="detail-state" aria-live="polite">
            <button type="button" class="mobile-back-button" @click=${this.close}>← Requests</button>
            <span class="spinner" aria-hidden="true"></span>
            <p>Loading request detail…</p>
          </section>
        `:this.state==="error"?a`
          <section class="detail-state error-state" role="alert">
            <button type="button" class="mobile-back-button" @click=${this.close}>← Requests</button>
            <strong>Request detail could not be loaded</strong>
            <p>${this.error_message}</p>
            <button type="button" class="primary-button" @click=${this.retry}>Retry</button>
          </section>
        `:a`<section class="detail-state"><p>Select a request to inspect its route, payloads, and responses.</p></section>`;const e=this.detail.request,s=v(e,"request_id")??this.summary?.request_id??"unknown id",t=v(e,"session_id")??this.summary?.session_id??void 0,i=v(e,"inbound_req_method")??this.summary?.inbound_req_method??"REQUEST",n=F(v(e,"inbound_req_url")??this.summary?.inbound_req_url??v(e,"endpoint"));return a`
      <section class="detail-content">
        <header class="detail-header">
          <button type="button" class="mobile-back-button" @click=${this.close}>← Requests</button>
          <div class="detail-title">
            <p class="eyebrow">request · ${b(s)}</p>
            <h2><span>${i}</span> ${n}</h2>
            <p class="muted" title=${s}>${s}</p>
          </div>
          <div class="detail-actions">
            ${t?a`<button type="button" class="secondary-button" @click=${()=>this.openSession(t)}>Open session</button>`:u}
            <button
              type="button"
              class="icon-button"
              aria-label="Refresh request detail"
              title="Refresh request detail"
              @click=${this.retry}
            >
              ↻
            </button>
          </div>
        </header>
        ${this.state==="loading"?a`<div class="inline-state" role="status"><span class="spinner" aria-hidden="true"></span>Refreshing detail…</div>`:u}
        ${this.state==="error"?a`
              <div class="inline-error" role="alert">
                <span>${this.error_message}</span>
                <button type="button" @click=${this.retry}>Retry</button>
              </div>
            `:u}
        ${e.request_error?a`<div class="request-error" role="alert">${String(e.request_error)}</div>`:u}
        <div class="detail-tabs" role="tablist" aria-label="Request detail sections" @keydown=${this.tabKeydown}>
          ${C.map(r=>a`
              <button
                id="request-tab-${r.id}"
                type="button"
                role="tab"
                aria-selected=${String(this.active_tab===r.id)}
                aria-controls="request-panel-${r.id}"
                tabindex=${this.active_tab===r.id?"0":"-1"}
                @click=${()=>this.selectTab(r.id)}
              >
                ${r.label}
              </button>
            `)}
        </div>
        <section
          id="request-panel-${this.active_tab}"
          class="detail-tab-panel"
          role="tabpanel"
          aria-labelledby="request-tab-${this.active_tab}"
          tabindex="0"
        >
          ${this.renderTab(e,this.detail.day,s,this.detail.row_id)}
        </section>
      </section>
    `}}customElements.define("request-detail-view",qs);class Ss extends w{static properties={requests:{attribute:!1},selected_key:{type:String},timezone:{type:String}};createRenderRoot(){return this}selectRequest(e){this.dispatchEvent(new CustomEvent("request-select",{detail:e,bubbles:!0,composed:!0}))}render(){const e=this.requests??[];return e.length===0?a`<p class="empty">No persisted requests match these filters.</p>`:a`
      <ul class="request-list" aria-label="Requests">
        ${e.map(s=>{const t=fs(s),i=this.selected_key===M(s),n=s.inbound_req_method??"REQUEST",r=ys(s);return a`
            <li>
              <button
                type="button"
                class="request-row ${i?"selected":""}"
                data-request-key=${M(s)}
                aria-current=${i?"true":"false"}
                @click=${()=>this.selectRequest(s)}
              >
                <span class="request-row-time">${j(s.ts,this.timezone,!0)}</span>
                <span class="status ${t.tone}" title=${t.title}>${t.label}</span>
                <span class="request-row-main">
                  <span class="request-route"><strong>${n}</strong><span>${r}</span></span>
                  <span class="request-context">
                    <span>${s.model??"unknown model"}</span>
                    <span aria-hidden="true">·</span>
                    <span>${s.provider_id??"unknown provider"}</span>
                  </span>
                  <span class="request-identifiers">
                    <span title=${s.request_id}>req ${b(s.request_id)}</span>
                    ${s.session_id?a`<span title=${s.session_id}>session ${b(s.session_id)}</span>`:a`<span>no session</span>`}
                  </span>
                </span>
              </button>
            </li>
          `})}
      </ul>
    `}}customElements.define("request-list",Ss);function ks(o,e){const s=new Set,t=new Set;for(const i of o){if(t.has(i.node_id))continue;const n=[],r=new Map;let c=i;for(;c&&!t.has(c.node_id);){const l=r.get(c.node_id);if(l!==void 0){for(const d of n.slice(l))s.add(d);break}r.set(c.node_id,n.length),n.push(c.node_id),c=c.parent_node_id?e.get(c.parent_node_id):void 0}for(const l of n)t.add(l)}return s}function As(o,e,s){const t=Number(s.has(e.node_id))-Number(s.has(o.node_id));return t!==0?t:o.ts!==e.ts?e.ts-o.ts:o.node_id.localeCompare(e.node_id)}function Rs(o,e,s){const t=[...o].filter(r=>r.is_head).sort((r,c)=>c.ts-r.ts||r.node_id.localeCompare(c.node_id))[0],i=new Set;let n=t;for(;n;){if(i.has(n.node_id)){s.add(n.node_id);break}i.add(n.node_id),n=n.parent_node_id?e.get(n.parent_node_id):void 0}return i}function Le(o,e,s,t,i){const n=[{node:o,next_child:0}];for(;n.length>0;){const r=n[n.length-1],c=s.get(r.node.node_id);if(c==="done"){n.pop();continue}c===void 0&&s.set(r.node.node_id,"visiting");const l=e.get(r.node.node_id)??[];if(r.next_child<l.length){const d=l[r.next_child];r.next_child+=1;const _=s.get(d.node_id);_===void 0?n.push({node:d,next_child:0}):_==="visiting"&&(t.add(r.node.node_id),t.add(d.node_id));continue}s.set(r.node.node_id,"done"),i.push(r.node),n.pop()}}function Es(o,e,s,t,i){const n=(d,_)=>As(d,_,t);for(const d of s.values())d.sort(n);const r=o.filter(d=>d.parent_node_id===null||!e.has(d.parent_node_id)||i.has(d.node_id)).sort(n),c=new Map,l=[];for(const d of r)Le(d,s,c,i,l);for(const d of[...o].sort(n))c.has(d.node_id)||(i.add(d.node_id),Le(d,s,c,i,l));return l}function xs(o,e,s,t,i){const n=[],r=[],c=new Set;let l=0;for(const d of o){let _=r.indexOf(d.node_id);const h=_===-1;h&&(_=r.length,r.push(d.node_id));const p=[...r],y=[];let $;const f=d.parent_node_id,q=f&&i.has(d.node_id)&&i.has(f)?null:f;if(q&&!c.has(q)){const g=r.findIndex((Z,je)=>je!==_&&Z===q);g===-1?(r[_]=q,$=_):(r.splice(_,1),$=g-+(_<g))}else q&&c.has(q)&&(i.add(d.node_id),i.add(q)),r.splice(_,1);const Y=[...r];for(let g=0;g<p.length;g+=1){if(g===_)continue;const Z=Y.indexOf(p[g]);Z!==-1&&y.push({from_lane:g,to_lane:Z,kind:"continuation",active:s.has(p[g])})}$!==void 0&&y.push({from_lane:_,to_lane:$,kind:"parent",active:s.has(d.node_id)}),l=Math.max(l,p.length,Y.length),n.push({node:d,top_lanes:p,bottom_lanes:Y,node_lane:_,starts_here:h,connections:y,bottom_lane_is_active:Y.map(g=>s.has(g)),child_count:e.get(d.node_id)?.length??0,parent_is_missing:!!(q&&t.has(q)),is_on_head_path:s.has(d.node_id),has_topology_warning:i.has(d.node_id)}),c.add(d.node_id)}return{rows:n,max_lane_count:l,remaining_lanes:[...r]}}function Ue(o){const e=new Map;for(const d of o)e.has(d.node_id)||e.set(d.node_id,d);const s=[...e.values()],t=new Map(s.map(d=>[d.node_id,[]])),i=new Set,n=ks(s,e);for(const d of s){const _=d.parent_node_id;_&&(e.has(_)&&!(n.has(d.node_id)&&n.has(_))?t.get(_)?.push(d):e.has(_)||i.add(_))}const r=Rs(s,e,n),c=Es(s,e,t,r,n),l=xs(c,t,r,i,n);for(const d of l.rows)d.has_topology_warning=n.has(d.node.node_id);return{...l,missing_parent_ids:[...i].sort(),remaining_lanes:l.remaining_lanes.filter(d=>i.has(d)),cycle_node_ids:[...n].sort()}}const Be=6,se=16,de=25;function Cs(o){return o===null?{label:"—",tone:"neutral",title:"No response status stored"}:o>=400?{label:String(o),tone:"error",title:`Response status: ${o}`}:o>=300?{label:String(o),tone:"warning",title:`Response status: ${o}`}:{label:String(o),tone:"success",title:`Response status: ${o}`}}function Ls(o){switch(o.toLowerCase()){case"assistant":return"assistant";case"system":case"developer":return"system";case"tool":case"function":return"tool";case"compaction":return"compaction";default:return"user"}}function Us(o){try{return JSON.stringify(o,null,2)??String(o)}catch{return String(o)}}function D(o){if(o<1024)return`${o.toLocaleString()} B`;const e=["KiB","MiB","GiB"];let s=o/1024,t=e[0];for(const i of e.slice(1)){if(s<1024)break;s/=1024,t=i}return`${s>=10?s.toFixed(0):s.toFixed(1)} ${t}`}function T(o){return o===null?"—":o.toLocaleString()}function le(o){return o===null?"—":new Intl.NumberFormat(void 0,{notation:"compact",maximumFractionDigits:o>=1e4?1:0}).format(o)}function Ps(o){switch(o){case"message_tree":return{direction:"New",title:"Input delta",empty_message:"No new semantic input was stored for this observation."};case"suffix_append":return{direction:"Appended",title:"Input delta",empty_message:"No new semantic input was stored for this node."};case"root_snapshot":return{direction:"Initial",title:"Input snapshot",empty_message:"No semantic input was stored for this root snapshot."};case"conflict_snapshot":return{direction:"Replaced",title:"Replacement snapshot",empty_message:"No semantic input was stored for this replacement snapshot."};default:return{direction:"Stored",title:"Node input",empty_message:"No semantic input was stored for this node."}}}function S(o){return(o+.5)*se}function Pe(o){return`session-tree-lanes-${Math.min(o,Be)}`}class Ns extends w{static properties={sessions:{attribute:!1},selected_session_id:{type:String},timezone:{type:String}};createRenderRoot(){return this}selectSession(e){this.dispatchEvent(new CustomEvent("session-select",{detail:e,bubbles:!0,composed:!0}))}render(){const e=this.sessions??[];return a`
      <ul class="session-list" aria-label="Sessions">
        ${e.map(s=>{const t=this.selected_session_id===s.session_id,i=$s(s);return a`
            <li>
              <button
                type="button"
                class="session-row ${t?"selected":""}"
                data-session-id=${s.session_id}
                aria-current=${t?"true":"false"}
                @click=${()=>this.selectSession(s)}
              >
                <time datetime=${new Date(s.last_ts).toISOString()}>
                  ${hs(s.last_ts,this.timezone)}
                </time>
                <span class="status ${i.tone}" title=${i.title}>${i.label}</span>
                <span class="session-row-main">
                  <span class="session-row-title">
                    <strong>${s.model??"Unknown model"}</strong>
                    <span>${s.endpoint??"unknown endpoint"}</span>
                  </span>
                  <span class="session-row-context">
                    <span>${s.provider_id??"unknown provider"}</span>
                    <span aria-hidden="true">·</span>
                    <span>${s.request_count.toLocaleString()} ${s.request_count===1?"node":"nodes"}</span>
                  </span>
                  <span class="session-row-id" title=${s.session_id}>
                    session ${b(s.session_id)}
                  </span>
                </span>
                <span class="session-row-chevron" aria-hidden="true">›</span>
              </button>
            </li>
          `})}
      </ul>
    `}}class Ds extends w{static properties={detail:{attribute:!1},node_detail:{attribute:!1},state:{type:String},error_message:{type:String},node_state:{type:String},node_error_message:{type:String},selected_node_id:{type:String},usage:{attribute:!1},usage_state:{type:String},usage_error_message:{type:String},timezone:{type:String}};createRenderRoot(){return this}close(){this.dispatchEvent(new CustomEvent("session-close",{bubbles:!0,composed:!0}))}retryDetail(){this.dispatchEvent(new CustomEvent("session-retry",{bubbles:!0,composed:!0}))}retryNode(){this.dispatchEvent(new CustomEvent("session-node-retry",{bubbles:!0,composed:!0}))}retryUsage(){this.dispatchEvent(new CustomEvent("session-usage-retry",{bubbles:!0,composed:!0}))}selectNode(e){this.dispatchEvent(new CustomEvent("session-node-select",{detail:e,bubbles:!0,composed:!0}))}openRequest(e){this.dispatchEvent(new CustomEvent("open-request",{detail:e,bubbles:!0,composed:!0}))}renderPart(e){switch(e.content.encoding){case"text":{const s=e.content.value||a`<span class="faint">Empty text part</span>`,t=e.content.truncated?a`<p class="session-part-note">Preview truncated · ${D(e.byte_length)} stored</p>`:u;return a`<div class="session-part-text">${s}${t}</div>`}case"json":return a`
          <details class="session-structured-part">
            <summary>${e.part_type.replaceAll("_"," ")}</summary>
            <pre>${Us(e.content.value)}</pre>
          </details>
        `;case"encrypted":return a`
          <details class="session-structured-part">
            <summary>${e.part_type.replaceAll("_"," ")} · encrypted</summary>
            <p>
              ${D(e.content.byte_length)} encrypted payload stored. Plaintext is unavailable and the
              encrypted content is not returned to the viewer.
            </p>
          </details>
        `;case"binary":return a`
          <details class="session-structured-part">
            <summary>${e.part_type.replaceAll("_"," ")} · binary</summary>
            <p>${D(e.content.byte_length)} stored. Binary bytes are not returned to the viewer.</p>
          </details>
        `;case"omitted":return a`
          <details class="session-structured-part">
            <summary>${e.part_type.replaceAll("_"," ")} · omitted</summary>
            <p>
              ${D(e.byte_length)} ${e.content.original_encoding} content omitted after reaching the
              ${e.content.reason==="part_limit"?"per-part byte preview":"node content-size"} limit.
            </p>
          </details>
        `}}renderMessages(e,s){return e.length===0?a`<p class="session-message-empty">${s}</p>`:a`
      <div class="session-message-stack">
        ${e.map(t=>a`
          <article class="session-message ${Ls(t.role)}">
            <header>
              <span>${t.role}</span>
              <span>
                ${t.parts.length.toLocaleString()}${t.parts.length===t.parts_total?"":` of ${t.parts_total.toLocaleString()}`} parts
                ${t.status===null?u:a` · status ${t.status}`}
              </span>
            </header>
            <div class="session-message-parts">
              ${t.parts.length>0?t.parts.map(i=>this.renderPart(i)):t.parts_total>0?a`
                      <p class="session-message-empty">
                        ${t.parts_total.toLocaleString()} stored parts were omitted from this bounded preview.
                      </p>
                    `:a`<p class="session-message-empty">No stored parts in this message.</p>`}
            </div>
          </article>
        `)}
      </div>
    `}renderUsage(){if(this.usage_state==="loading")return a`
        <section class="session-usage-panel" aria-busy="true">
          <header>
            <div>
              <p class="eyebrow">usage.db</p>
              <h3>Token usage</h3>
            </div>
          </header>
          <div class="inline-state"><span class="spinner" aria-hidden="true"></span>Loading usage…</div>
        </section>
      `;if(this.usage_state==="error")return a`
        <section class="session-usage-panel">
          <header>
            <div>
              <p class="eyebrow">usage.db</p>
              <h3>Token usage</h3>
            </div>
          </header>
          <div class="inline-error" role="alert">
            <span>${this.usage_error_message}</span>
            <button type="button" @click=${this.retryUsage}>Retry</button>
          </div>
        </section>
      `;if(!this.usage)return a`
        <section class="session-usage-panel">
          <header>
            <div>
              <p class="eyebrow">usage.db</p>
              <h3>Token usage</h3>
            </div>
            <span>No usage recorded</span>
          </header>
        </section>
      `;const e=this.usage;return a`
      <section class="session-usage-panel">
        <header>
          <div>
            <p class="eyebrow">usage.db</p>
            <h3>Token usage</h3>
          </div>
          <span>
            ${e.requests_with_usage.toLocaleString()} of ${e.request_count.toLocaleString()} requests reported
          </span>
        </header>
        <dl class="session-usage-grid">
          <div><dt>Input</dt><dd>${T(e.input_tokens)}</dd></div>
          <div><dt>Output</dt><dd>${T(e.output_tokens)}</dd></div>
          <div><dt>Total</dt><dd>${T(e.total_tokens)}</dd></div>
          <div><dt>Cache read</dt><dd>${T(e.cache_read_tokens)}</dd></div>
          <div><dt>Cache write</dt><dd>${T(e.cache_write_tokens)}</dd></div>
          <div><dt>Reasoning</dt><dd>${T(e.reasoning_tokens)}</dd></div>
        </dl>
      </section>
    `}nodeDomId(e,s){return`session-node-${e}-${encodeURIComponent(s)}`}renderNodeGraph(e,s){const t=s*se,i=S(e.node_lane),n=`M ${i} ${de} l 0 0.001`,r=e.connections.map(l=>{const d=S(l.from_lane),_=S(l.to_lane),h=l.kind==="parent"?de:0;return a`
        <path
          class="session-tree-edge ${l.kind} ${l.active?"active":""}"
          d=${`M ${d} ${h} L ${_} 100`}
        ></path>
      `}),c=["session-tree-dot",e.node.is_head?"head":"",e.child_count>1?"branch":"",e.has_topology_warning?"warning":""].filter(Boolean).join(" ");return a`
      <svg
        viewBox=${`0 0 ${t} 100`}
        preserveAspectRatio="none"
        focusable="false"
        aria-hidden="true"
      >
        ${e.starts_here?u:a`
              <path
                class="session-tree-edge incoming ${e.is_on_head_path?"active":""}"
                d=${`M ${i} 0 L ${i} ${de}`}
              ></path>
            `}
        ${r}
        <path class="${c} outline" d=${n}></path>
        <path class="${c} fill" d=${n}></path>
      </svg>
    `}renderNodeGraphContinuation(e,s){const t=s*se;return a`
      <svg
        viewBox=${`0 0 ${t} 100`}
        preserveAspectRatio="none"
        focusable="false"
        aria-hidden="true"
      >
        ${e.bottom_lanes.map((i,n)=>a`
          <path
            class="session-tree-edge continuation ${e.bottom_lane_is_active[n]?"active":""}"
            d=${`M ${S(n)} 0 L ${S(n)} 100`}
          ></path>
        `)}
      </svg>
    `}renderTreeBoundary(e,s,t,i,n){if(e.missing_parent_ids.length===0)return u;const r=s*se,c=e.remaining_lanes.length>0?e.remaining_lanes.map((p,y)=>y):e.missing_parent_ids.map((p,y)=>y),l=[...new Set(c)],d=n?"Connects to loaded tree":t?"Earlier ancestry omitted":"Parent nodes unavailable",_=n?`Parent ${b(n.node_id)} appears in the session tree below.`:t?`${i.toLocaleString()} ${i===1?"node falls":"nodes fall"} outside this bounded tree snapshot.`:"The stored parent links point outside the returned session tree.",h=n?"Parent link resolved in the loaded snapshot":`${e.missing_parent_ids.length.toLocaleString()} parent ${e.missing_parent_ids.length===1?"link":"links"} outside the snapshot`;return a`
      <li class="session-tree-boundary ${n?"loaded-parent":""} ${Pe(s)}">
        <span class="session-tree-boundary-graph" aria-hidden="true">
          <svg viewBox=${`0 0 ${r} 100`} preserveAspectRatio="none" focusable="false">
            ${l.map(p=>a`
              <path class="session-tree-edge boundary" d=${`M ${S(p)} 0 L ${S(p)} 48`}></path>
              <path
                class="session-tree-boundary-dot outline"
                d=${`M ${S(p)} 52 l 0 0.001`}
              ></path>
              <path
                class="session-tree-boundary-dot fill"
                d=${`M ${S(p)} 52 l 0 0.001`}
              ></path>
            `)}
          </svg>
        </span>
        <div class="session-tree-boundary-card" role="note">
          <strong>${d}</strong>
          <span>${_}</span>
          <span title=${n?.node_id??e.missing_parent_ids.join(", ")}>${h}</span>
        </div>
      </li>
    `}renderLoadedNodeContent(e){const s=e.truncation,t=Ps(e.node.reduction_kind),i=s.request_messages.messages_total-s.request_messages.messages_returned,n=s.response_messages.messages_total-s.response_messages.messages_returned,r=i>0||n>0||s.parts_omitted>0||s.content_parts_truncated>0||s.binary_parts_elided>0;return a`
      <div class="session-node-content-actions">
        <span title=${e.node.request_id}>Request ${b(e.node.request_id)}</span>
        <button type="button" class="secondary-button" @click=${()=>this.openRequest(e.node)}>Open request</button>
      </div>
      ${r?a`
            <div class="session-content-boundary" role="status">
              <strong>Bounded content preview</strong>
              <span>
                ${D(s.content_bytes_returned)} of
                ${D(s.content_bytes_total)} inline content returned
                ${i+n>0?` · ${(i+n).toLocaleString()} messages omitted`:""}
                ${s.parts_omitted>0?` · ${s.parts_omitted.toLocaleString()} parts omitted`:""}
                ${s.content_parts_truncated>0?` · ${s.content_parts_truncated.toLocaleString()} parts truncated`:""}
                ${s.binary_parts_elided>0?` · ${s.binary_parts_elided.toLocaleString()} binary parts represented as metadata`:""}
              </span>
            </div>
          `:u}
      <div class="session-conversation-section">
        <header>
          <div>
            <span class="direction-label">${t.direction}</span>
            <h3>${t.title}</h3>
          </div>
          <span>
            ${s.request_messages.messages_returned.toLocaleString()}
            ${s.request_messages.messages_returned===s.request_messages.messages_total?"":`of ${s.request_messages.messages_total.toLocaleString()}`} messages
          </span>
        </header>
        ${this.renderMessages(e.request_messages,t.empty_message)}
      </div>
      <div class="session-conversation-section">
        <header>
          <div>
            <span class="direction-label">Captured</span>
            <h3>Model output</h3>
          </div>
          <span>
            ${s.response_messages.messages_returned.toLocaleString()}
            ${s.response_messages.messages_returned===s.response_messages.messages_total?"":`of ${s.response_messages.messages_total.toLocaleString()}`} messages
          </span>
        </header>
        ${this.renderMessages(e.response_messages,"No semantic output was stored for this node.")}
      </div>
    `}renderNodeContent(e){if(this.selected_node_id!==e.node_id)return u;const s=this.node_detail?.node.node_id===e.node_id?this.node_detail:void 0,t=this.node_state==="loading"?a`<div class="inline-state"><span class="spinner" aria-hidden="true"></span>Loading semantic content…</div>`:this.node_state==="error"?a`
            <div class="inline-error" role="alert">
              <span>${this.node_error_message}</span>
              <button type="button" @click=${this.retryNode}>Retry</button>
            </div>
          `:s?this.renderLoadedNodeContent(s):u;return a`
      <section
        id=${this.nodeDomId("content",e.node_id)}
        class="session-node-content"
        aria-labelledby=${this.nodeDomId("trigger",e.node_id)}
        aria-live="polite"
        aria-busy=${String(this.node_state==="loading")}
      >
        ${t}
      </section>
    `}renderNodeUsage(e){if(this.usage_state==="loading")return a`<span class="session-node-token-usage muted">Token usage loading…</span>`;if(this.usage_state==="error")return a`<span class="session-node-token-usage muted">Token usage unavailable</span>`;if(!e)return a`<span class="session-node-token-usage muted">No token usage</span>`;const s=e.context_tokens===null?"Context tokens unavailable":`${e.context_tokens.toLocaleString()} context tokens`,t=e.input_delta_tokens===null?"Input delta tokens unavailable":`${e.input_delta_tokens.toLocaleString()} uncached input tokens`,i=e.output_tokens===null?"Output tokens unavailable":`${e.output_tokens.toLocaleString()} output tokens`;return a`
      <span class="session-node-token-usage">
        <span class="session-node-token-label">tokens</span>
        <span class="session-node-token-separator" aria-hidden="true">·</span>
        <span title=${s}>${le(e.context_tokens)} context</span>
        <span class="session-node-token-separator" aria-hidden="true">·</span>
        <span title=${t}>
          ${e.input_delta_tokens===null?"—":`+${le(e.input_delta_tokens)}`} input delta
        </span>
        <span class="session-node-token-separator" aria-hidden="true">·</span>
        <span title=${i}>${le(e.output_tokens)} output</span>
      </span>
    `}renderNode(e,s,t,i){const n=e.node,r=n.node_id===this.selected_node_id,c=Cs(n.status),l=!!(i&&n.parent_node_id===i.node_id),d=e.parent_is_missing&&!l,_=["session-node",Pe(s),r?"selected":"",e.is_on_head_path?"head-path":"",d?"boundary-child":"",e.has_topology_warning?"topology-warning":""].filter(Boolean).join(" "),h=n.reduction_kind==="message_tree"?n.input_message_count:n.request_message_count,p=n.reduction_kind==="message_tree"?"input":"input delta",y=n.reduction_kind==="message_tree"?a` (+${n.request_message_count.toLocaleString()} new)`:u,$=n.reduction_kind==="message_tree"?n.output_message_count:n.response_message_count,f=n.reduction_kind==="message_tree"?n.parent_node_id?`Prefix-derived child of ${n.parent_node_id}.`:"Prefix-derived root node.":n.parent_node_id?`Recorded child of ${n.parent_node_id}.`:"Recorded root node.";return a`
      <li class=${_}>
        <span class="session-node-graph" aria-hidden="true">
          ${this.renderNodeGraph(e,s)}
        </span>
        <button
          id=${this.nodeDomId("trigger",n.node_id)}
          type="button"
          class="session-node-trigger"
          data-node-id=${n.node_id}
          aria-expanded=${String(r)}
          aria-controls=${r?this.nodeDomId("content",n.node_id):u}
          aria-current=${n.is_head?"true":u}
          @click=${()=>this.selectNode(n)}
        >
          <span class="session-node-primary">
            <time datetime=${new Date(n.ts).toISOString()}>${j(n.ts,this.timezone)}</time>
            <span class="status ${c.tone}" title=${c.title}>${c.label}</span>
            ${e.child_count>1?a`<span class="branch-badge">${e.child_count.toLocaleString()} branches</span>`:u}
            ${n.is_head?a`<span class="head-badge">Current head</span>`:u}
          </span>
          <span class="session-node-title">
            <strong>${n.model??"Unknown model"}</strong>
            <span>${n.endpoint}</span>
          </span>
          <span class="session-node-context">
            <span>${n.provider_id??"unknown provider"}</span>
            <span aria-hidden="true">·</span>
            <span>${h.toLocaleString()} ${p}${y}</span>
            <span aria-hidden="true">·</span>
            <span>${$.toLocaleString()} output</span>
          </span>
          ${this.renderNodeUsage(t.get(n.request_id))}
          <span class="session-node-id" title=${n.request_id}>
            request ${b(n.request_id)} · ${n.parent_node_id?`parent ${b(n.parent_node_id)}`:"root"}
            ${d?" · outside snapshot":""}
          </span>
          <span class="visually-hidden">
            ${f}
            ${d?" Parent is outside this bounded snapshot.":""}
            ${l?" Parent appears in the loaded session tree.":""}
            ${e.has_topology_warning?" Parent links contain a topology warning.":""}
          </span>
        </button>
        ${r?a`
              <span class="session-node-content-graph" aria-hidden="true">
                ${this.renderNodeGraphContinuation(e,s)}
              </span>
            `:u}
        ${this.renderNodeContent(n)}
      </li>
    `}render(){if(!this.detail)return this.state==="loading"?a`
          <section class="detail-state" aria-live="polite">
            <button type="button" class="mobile-back-button" @click=${this.close}>← Sessions</button>
            <span class="spinner" aria-hidden="true"></span>
            <p>Loading semantic session…</p>
          </section>
        `:this.state==="error"?a`
          <section class="detail-state error-state" role="alert">
            <button type="button" class="mobile-back-button" @click=${this.close}>← Sessions</button>
            <strong>Session could not be loaded</strong>
            <p>${this.error_message}</p>
            <button type="button" class="primary-button" @click=${this.retryDetail}>Retry</button>
          </section>
        `:a`
        <section class="detail-state session-empty-state">
          <span class="session-empty-mark" aria-hidden="true">⌁</span>
          <strong>Choose a session</strong>
          <p>Inspect its semantic nodes and the conversation captured in <code>sessions.db</code>.</p>
        </section>
      `;const{session:e,nodes:s}=this.detail,t=new Map((this.usage?.requests??[]).map(f=>[f.request_id,f])),i=Ue(s),n=Math.max(1,i.max_lane_count),r=Math.max(0,e.request_count-s.length),c=i.missing_parent_ids.length>0,l=!!(this.selected_node_id&&s.some(f=>f.node_id===this.selected_node_id)),d=this.node_detail,_=!l&&d&&d.node.node_id===this.selected_node_id?d.node:void 0,h=_?Ue([_]):void 0,p=h?Math.max(1,h.max_lane_count):1,y=_?.parent_node_id?s.find(f=>f.node_id===_.parent_node_id):void 0,$=e.model??"Unknown model";return a`
      <section class="detail-content session-detail-content">
        <header class="detail-header session-detail-header">
          <button type="button" class="mobile-back-button" @click=${this.close}>← Sessions</button>
          <div class="detail-title">
            <p class="eyebrow">session · ${b(e.session_id)}</p>
            <h2>${$}<span> on ${e.provider_id??"unknown provider"}</span></h2>
            <p class="muted" title=${e.session_id}>${e.session_id||"Missing session identifier"}</p>
          </div>
          <button
            type="button"
            class="icon-button"
            aria-label="Refresh session detail"
            title="Refresh session detail"
            @click=${this.retryDetail}
          >
            ↻
          </button>
        </header>
        ${this.state==="loading"?a`<div class="inline-state"><span class="spinner" aria-hidden="true"></span>Refreshing session…</div>`:u}
        ${this.state==="error"?a`
              <div class="inline-error" role="alert">
                <span>${this.error_message}</span>
                <button type="button" @click=${this.retryDetail}>Retry</button>
              </div>
            `:u}
        <dl class="session-metadata-grid">
          <div><dt>Semantic nodes</dt><dd>${e.request_count.toLocaleString()}</dd></div>
          <div><dt>Duration</dt><dd>${ps(e.first_ts,e.last_ts)}</dd></div>
          <div><dt>First seen</dt><dd>${j(e.first_ts,this.timezone)}</dd></div>
          <div><dt>Last active</dt><dd>${j(e.last_ts,this.timezone)}</dd></div>
          <div><dt>Endpoint</dt><dd title=${e.endpoint??""}>${e.endpoint??"—"}</dd></div>
          <div><dt>Account</dt><dd title=${e.account_id??""}>${e.account_id??"—"}</dd></div>
        </dl>
        ${this.renderUsage()}
        <section class="session-activity">
          <header class="session-section-header">
            <div>
              <p class="eyebrow">Recorded parent graph</p>
              <h3>Session tree</h3>
            </div>
            <span>
              ${s.length.toLocaleString()} loaded · head branch first${this.detail.nodes_truncated?" · bounded":""}
              ${i.max_lane_count>Be?" · compressed lanes":""}
            </span>
          </header>
          ${this.detail.nodes_truncated?a`
                <p class="session-truncation-note">
                  ${r.toLocaleString()} older nodes are omitted.
                  ${c?" Amber graph endpoints continue into the omitted ancestry.":" The graph shows every parent link available in this snapshot."}
                </p>
              `:u}
          ${i.cycle_node_ids.length>0?a`
                <p class="session-topology-warning" role="alert">
                  ${i.cycle_node_ids.length.toLocaleString()} nodes contain cyclic parent links; their graph
                  edges were detached defensively.
                </p>
              `:u}
          ${s.length>0?a`
                <p class="session-tree-direction">
                  <span>Leaves and current-head branch</span>
                  <span aria-hidden="true">↓</span>
                  <span>recorded parents</span>
                </p>
              `:u}
          ${this.selected_node_id?u:a`<p class="session-content-hint">Open a node to load its conversation content from <code>sessions.db</code>.</p>`}
          ${this.selected_node_id&&!l?a`
                <section class="session-linked-node" aria-label="Directly linked session node">
                  <header>
                    <div>
                      <p class="eyebrow">Direct link</p>
                      <h4>Node outside this activity snapshot</h4>
                    </div>
                    <span>${b(this.selected_node_id)}</span>
                  </header>
                  ${h?a`
                        <ol class="session-node-list linked-node-list">
                          ${h.rows.map(f=>this.renderNode(f,p,t,y))}
                          ${this.renderTreeBoundary(h,p,!1,0,y)}
                        </ol>
                      `:this.node_state==="loading"?a`
                          <div class="inline-state" role="status" aria-live="polite">
                            <span class="spinner" aria-hidden="true"></span>Loading linked node…
                          </div>
                        `:this.node_state==="error"?a`
                            <div class="inline-error" role="alert">
                              <span>${this.node_error_message}</span>
                              <button type="button" @click=${this.retryNode}>Retry</button>
                            </div>
                          `:u}
                </section>
              `:u}
          ${s.length>0?a`
                <ol class="session-node-list">
                  ${i.rows.map(f=>this.renderNode(f,n,t))}
                  ${this.renderTreeBoundary(i,n,this.detail.nodes_truncated,r)}
                </ol>
              `:a`<p class="empty">This migrated session has no semantic nodes.</p>`}
        </section>
      </section>
    `}}customElements.define("session-list",Ns);customElements.define("session-detail-view",Ds);const Ne=100;function k(o,e){return o instanceof Error?o.message:e}function Ts(o){return o==="overview"||o==="client"||o==="provider"||o==="raw"}function X(){return{query:"",provider_id:"",status:"",errors_only:!1}}function Os(o){return new Date(o).toISOString().slice(0,10)}class Ms extends w{static properties={active_view:{type:String},info:{attribute:!1},requests:{attribute:!1},request_days:{attribute:!1},selected_day:{type:String},selected_request:{attribute:!1},selected_request_id:{type:String},selected_request_row_id:{type:String},selected_request_detail:{attribute:!1},request_list_state:{type:String},request_list_error:{type:String},request_detail_state:{type:String},request_detail_error:{type:String},next_cursor:{type:String},loading_more:{type:Boolean},load_more_error:{type:String},search_query:{type:String},provider_id:{type:String},status_filter:{type:String},errors_only:{type:Boolean},applied_filters:{attribute:!1},active_detail_tab:{type:String},timezone:{type:String},request_days_loading:{type:Boolean},request_days_error:{type:String},sessions:{attribute:!1},selected_session:{attribute:!1},selected_session_detail:{attribute:!1},selected_session_usage:{attribute:!1},sessions_loading:{type:Boolean},sessions_error:{type:String},session_search_query:{type:String},session_detail_state:{type:String},session_detail_error:{type:String},session_usage_state:{type:String},session_usage_error:{type:String},selected_session_node_id:{type:String},selected_session_node_detail:{attribute:!1},session_node_state:{type:String},session_node_error:{type:String}};request_load_id=0;request_detail_load_id=0;session_detail_load_id=0;session_usage_load_id=0;session_node_load_id=0;session_list_load_id=0;request_days_load_id=0;sessions_loaded=!1;requested_request_id;requested_request_row_id;requested_session_id;requested_session_node_id;request_rows_context;request_controller;request_detail_controller;session_list_controller;session_list_load;session_detail_controller;session_usage_controller;session_node_controller;navigation_workflow_id=0;popstate_handler=()=>{this.restoreFromHistory()};constructor(){super(),this.active_view="requests",this.requests=[],this.request_days=[],this.sessions=[],this.request_list_state="idle",this.request_detail_state="idle",this.search_query="",this.provider_id="",this.status_filter="",this.errors_only=!1,this.applied_filters=X(),this.active_detail_tab="overview",this.timezone="local",this.loading_more=!1,this.request_days_loading=!1,this.sessions_loading=!1,this.session_search_query="",this.session_detail_state="idle",this.session_usage_state="idle",this.session_node_state="idle"}createRenderRoot(){return this}connectedCallback(){super.connectedCallback(),this.restoreUrlState(),window.addEventListener("popstate",this.popstate_handler),this.loadInitialData()}disconnectedCallback(){window.removeEventListener("popstate",this.popstate_handler),this.request_controller?.abort(),this.request_detail_controller?.abort(),this.session_list_controller?.abort(),this.session_detail_controller?.abort(),this.session_usage_controller?.abort(),this.session_node_controller?.abort(),super.disconnectedCallback()}restoreUrlState(){const e=new URLSearchParams(window.location.search);this.active_view=e.get("view")==="sessions"?"sessions":"requests";const s=e.get("day");this.selected_day=s&&/^\d{4}-\d{2}-\d{2}$/.test(s)?s:void 0,this.search_query=e.get("query")??"",this.provider_id=e.get("provider_id")??"";const t=e.get("status")??"";this.status_filter=/^\d{3}$/.test(t)?t:"",this.errors_only=e.get("errors_only")==="true"||e.get("errors_only")==="1",this.applied_filters={query:this.search_query,provider_id:this.provider_id,status:this.status_filter,errors_only:this.errors_only},this.requested_request_id=e.get("request_id")??void 0;const i=e.get("row_id");this.requested_request_row_id=i&&/^-?\d+$/.test(i)?i:void 0;const n=e.get("tab");this.active_detail_tab=Ts(n)?n:"overview",this.requested_session_id=e.has("session_id")?e.get("session_id")??"":void 0,this.requested_session_node_id=e.get("node_id")??void 0,this.timezone=e.get("timezone")==="utc"?"utc":"local"}selectedRequestDay(){return this.selected_request_detail?.day??this.selected_request?.day??this.selected_day}syncUrl(e="replace"){const s=new URLSearchParams;if(this.active_view==="sessions"){s.set("view","sessions");const n=this.selected_session?.session_id??this.requested_session_id;n!==void 0&&s.set("session_id",n),this.selected_session_node_id&&s.set("node_id",this.selected_session_node_id)}else{const n=this.selected_request_id?this.selectedRequestDay():this.selected_day;n&&s.set("day",n),this.applied_filters.query&&s.set("query",this.applied_filters.query),this.applied_filters.provider_id&&s.set("provider_id",this.applied_filters.provider_id),this.applied_filters.status&&s.set("status",this.applied_filters.status),this.applied_filters.errors_only&&s.set("errors_only","true"),this.selected_request_id&&(s.set("request_id",this.selected_request_id),this.selected_request_row_id&&s.set("row_id",this.selected_request_row_id),s.set("tab",this.active_detail_tab))}s.set("timezone",this.timezone);const t=s.toString(),i=`${window.location.pathname}${t?`?${t}`:""}`;`${window.location.pathname}${window.location.search}`!==i&&(e==="push"?window.history.pushState(null,"",i):window.history.replaceState(null,"",i))}async loadInitialData(){const e=++this.navigation_workflow_id;this.loadInfo(),await this.loadUrlState(e)}async restoreFromHistory(){const e=++this.navigation_workflow_id;this.request_controller?.abort(),this.request_detail_controller?.abort(),this.session_detail_controller?.abort(),this.session_node_controller?.abort(),this.resetRequestSelection(),this.resetSessionSelection(),this.restoreUrlState(),this.active_view==="requests"&&(this.requests=[],this.next_cursor=void 0,this.request_rows_context=void 0),await this.loadUrlState(e)}async loadUrlState(e){const s=this.requested_request_id,t=this.requested_request_row_id;if(this.active_view==="sessions"){const n=this.requested_session_id,r=this.requested_session_node_id;if(!await this.ensureSessionsLoaded()||e!==this.navigation_workflow_id||n===void 0)return;await this.loadSession(n,this.sessions.find(l=>l.session_id===n),!1,null,r);return}this.loadRequestDays();let i;if(this.selected_day?i=await this.loadRequests():(i=await this.loadLatestRequests(),i&&this.selected_day&&this.hasAppliedFilters()&&(i=await this.loadRequests())),!(!i||e!==this.navigation_workflow_id)&&s&&this.selected_day){const n=this.requests.find(r=>r.request_id===s&&(!t||r.row_id===t));await this.loadRequestDetail(this.selected_day,s,t??n?.row_id,n,!1,null)}}async loadInfo(){try{this.info=await m("/api/info")}catch{this.info=void 0}}async loadLatestRequests(){this.request_controller?.abort();const e=new AbortController;this.request_controller=e;const s=++this.request_load_id;this.requests=[],this.next_cursor=void 0,this.request_rows_context=void 0,this.request_list_state="loading",this.request_list_error=void 0;try{const t=await m(`/api/requests/latest?limit=${Ne}`,e.signal);return s!==this.request_load_id||this.request_controller!==e?!1:(this.selected_day=t.day??void 0,this.requests=t.requests,this.next_cursor=t.next_cursor??void 0,this.request_rows_context=this.requestContext(this.selected_day,X()),this.request_list_state="ready",this.syncUrl(),!0)}catch(t){return s===this.request_load_id&&!R(t)&&(this.request_list_state="error",this.request_list_error=k(t,"Unable to load recent requests")),!1}finally{this.request_controller===e&&(this.request_controller=void 0)}}requestContext(e=this.selected_day,s=this.applied_filters){return e?JSON.stringify([e,s.query,s.provider_id,s.status,s.errors_only]):void 0}requestParams(e,s,t){const i=new URLSearchParams({day:e,limit:String(Ne)});return s.query&&i.set("query",s.query),s.provider_id&&i.set("provider_id",s.provider_id),s.status&&i.set("status",s.status),s.errors_only&&i.set("errors_only","true"),t&&i.set("cursor",t),i}async loadRequests(e=!1){const s=this.selected_day;if(!s)return this.request_list_state="idle",this.requests=[],this.next_cursor=void 0,this.request_rows_context=void 0,!1;const t={...this.applied_filters},i=this.requestContext(s,t),n=e?this.next_cursor:void 0;if(e&&(!n||this.request_rows_context!==i))return!1;this.request_controller?.abort();const r=new AbortController;this.request_controller=r;const c=++this.request_load_id;e?(this.loading_more=!0,this.load_more_error=void 0):(this.loading_more=!1,this.request_rows_context!==i&&(this.requests=[],this.next_cursor=void 0,this.request_rows_context=void 0),this.request_list_state="loading",this.request_list_error=void 0,this.load_more_error=void 0);try{const l=await m(`/api/requests?${this.requestParams(s,t,n).toString()}`,r.signal);if(c!==this.request_load_id||this.request_controller!==r||this.requestContext()!==i)return!1;if(e){const d=new Set(this.requests.map(_=>M(_)));this.requests=[...this.requests,...l.requests.filter(_=>!d.has(M(_)))]}else this.requests=l.requests;return this.next_cursor=l.next_cursor??void 0,this.request_rows_context=i,this.request_list_state="ready",!0}catch(l){return c!==this.request_load_id||R(l)||(l instanceof Ie&&l.status===503&&this.markRequestDayUnavailable(s),e?this.load_more_error=k(l,"Unable to load more requests"):(this.request_list_state="error",this.request_list_error=k(l,"Unable to load requests"))),!1}finally{c===this.request_load_id&&(this.loading_more=!1),this.request_controller===r&&(this.request_controller=void 0)}}async loadRequestDays(){const e=++this.request_days_load_id;this.request_days_loading=!0,this.request_days_error=void 0;try{const s=await m("/api/request-days");e===this.request_days_load_id&&(this.request_days=s)}catch(s){e===this.request_days_load_id&&(this.request_days_error=k(s,"Unable to load request day states"))}finally{e===this.request_days_load_id&&(this.request_days_loading=!1)}}markRequestDayUnavailable(e){this.request_days.some(s=>s.day===e)?this.request_days=this.request_days.map(s=>s.day===e?{...s,state:"unavailable"}:s):this.request_days=[{day:e,state:"unavailable"},...this.request_days]}resetRequestSelection(){this.request_detail_controller?.abort(),this.request_detail_controller=void 0,this.request_detail_load_id+=1,this.selected_request=void 0,this.selected_request_id=void 0,this.selected_request_row_id=void 0,this.selected_request_detail=void 0,this.request_detail_state="idle",this.request_detail_error=void 0,this.active_detail_tab="overview"}resetSessionSelection(){this.session_detail_controller?.abort(),this.session_usage_controller?.abort(),this.session_node_controller?.abort(),this.session_detail_controller=void 0,this.session_usage_controller=void 0,this.session_node_controller=void 0,this.session_detail_load_id+=1,this.session_usage_load_id+=1,this.session_node_load_id+=1,this.requested_session_id=void 0,this.requested_session_node_id=void 0,this.selected_session=void 0,this.selected_session_detail=void 0,this.selected_session_usage=void 0,this.selected_session_node_id=void 0,this.selected_session_node_detail=void 0,this.session_detail_state="idle",this.session_detail_error=void 0,this.session_usage_state="idle",this.session_usage_error=void 0,this.session_node_state="idle",this.session_node_error=void 0}async closeRequestDetail(){const e=this.selected_request_row_id&&this.selectedRequestDay()?M({day:this.selectedRequestDay(),row_id:this.selected_request_row_id}):void 0;if(++this.navigation_workflow_id,this.resetRequestSelection(),this.syncUrl("push"),!e||!window.matchMedia("(max-width: 680px)").matches)return;await this.updateComplete,[...this.querySelectorAll("request-list [data-request-key]")].find(t=>t.dataset.requestKey===e)?.focus()}async closeSessionDetail(){const e=this.selected_session?.session_id??this.requested_session_id;if(++this.navigation_workflow_id,this.resetSessionSelection(),this.syncUrl("push"),e===void 0||!window.matchMedia("(max-width: 680px)").matches)return;await this.updateComplete,[...this.querySelectorAll("session-list [data-session-id]")].find(t=>t.dataset.sessionId===e)?.focus()}async loadRequestDetail(e,s,t,i,n,r="replace"){this.request_detail_controller?.abort();const c=new AbortController;this.request_detail_controller=c;const l=++this.request_detail_load_id;this.selected_day=e,this.selected_request=i,this.selected_request_id=s,this.selected_request_row_id=t,n||(this.selected_request_detail=void 0),this.request_detail_state="loading",this.request_detail_error=void 0,r&&this.syncUrl(r);try{const d=new URLSearchParams({day:e,request_id:s});t&&d.set("row_id",t);const _=await m(`/api/request?${d.toString()}`,c.signal);if(l===this.request_detail_load_id&&this.request_detail_controller===c){const h=this.selected_request_row_id!==_.row_id;return this.selected_request_detail=_,this.selected_request_row_id=_.row_id,this.request_detail_state="ready",(r||h)&&this.syncUrl("replace"),!0}return!1}catch(d){return l===this.request_detail_load_id&&!R(d)&&(this.request_detail_state="error",this.request_detail_error=k(d,"Unable to load request detail")),!1}finally{this.request_detail_controller===c&&(this.request_detail_controller=void 0)}}async selectRequest(e){++this.navigation_workflow_id;const s=this.selected_request_id===e.request_id&&this.selected_request_detail?.day===e.day&&this.selected_request_detail.row_id===e.row_id,t=this.loadRequestDetail(e.day,e.request_id,e.row_id,e,s,"push");window.matchMedia("(max-width: 680px)").matches&&(await this.updateComplete,this.querySelector("request-detail-view .mobile-back-button")?.focus()),await t&&window.matchMedia("(max-width: 680px)").matches&&(await this.updateComplete,this.querySelector("request-detail-view .mobile-back-button")?.focus())}retryRequestDetail(){const e=this.selected_request_detail?.day??this.selected_request?.day??this.selected_day;e&&this.selected_request_id&&this.loadRequestDetail(e,this.selected_request_id,this.selected_request_row_id,this.selected_request,!!this.selected_request_detail,null)}selectDay(e){e!==this.selected_day&&(++this.navigation_workflow_id,this.selected_day=e,this.resetRequestSelection(),this.syncUrl("push"),this.loadRequests())}pickerDays(){return!this.selected_day||this.request_days.some(e=>e.day===this.selected_day)?this.request_days:[{day:this.selected_day,state:"available"},...this.request_days]}adjacentAvailableDay(e){const s=this.pickerDays().filter(i=>i.state==="available").map(i=>i.day).sort();if(!this.selected_day)return;const t=s.indexOf(this.selected_day);return t<0?void 0:s[t+e]}submitFilters(e){e.preventDefault(),++this.navigation_workflow_id,this.applied_filters={query:this.search_query.trim(),provider_id:this.provider_id.trim(),status:this.status_filter.trim(),errors_only:this.errors_only},this.resetRequestSelection(),this.syncUrl("push"),this.loadRequests()}clearFilters(){this.search_query="",this.provider_id="",this.status_filter="",this.errors_only=!1,this.applied_filters=X(),++this.navigation_workflow_id,this.resetRequestSelection(),this.syncUrl("push"),this.loadRequests()}hasAppliedFilters(){return!!(this.applied_filters.query||this.applied_filters.provider_id||this.applied_filters.status||this.applied_filters.errors_only)}filtersChanged(){return this.search_query.trim()!==this.applied_filters.query||this.provider_id.trim()!==this.applied_filters.provider_id||this.status_filter.trim()!==this.applied_filters.status||this.errors_only!==this.applied_filters.errors_only}providerOptions(){const e=new Set(this.requests.flatMap(s=>s.provider_id?[s.provider_id]:[]));return this.applied_filters.provider_id&&e.add(this.applied_filters.provider_id),[...e].sort()}ensureSessionsLoaded(e=!1){if(this.sessions_loaded&&!e)return Promise.resolve(!0);if(this.session_list_load&&!e)return this.session_list_load;this.session_list_controller?.abort();const s=new AbortController;this.session_list_controller=s;const t=++this.session_list_load_id;this.sessions_loading=!0,this.sessions_error=void 0;const i=this.loadSessions(s,t);return this.session_list_load=i,i}async loadSessions(e,s){try{const t=await m("/api/sessions?limit=100",e.signal);return s!==this.session_list_load_id||this.session_list_controller!==e?!1:(this.sessions=t,this.sessions_loaded=!0,this.selected_session&&(this.selected_session=t.find(i=>i.session_id===this.selected_session?.session_id)??this.selected_session),!0)}catch(t){return s===this.session_list_load_id&&!R(t)&&(this.sessions_error=k(t,"Unable to load sessions")),!1}finally{s===this.session_list_load_id&&this.session_list_controller===e&&(this.session_list_controller=void 0,this.session_list_load=void 0,this.sessions_loading=!1)}}retrySessions(){const e=++this.navigation_workflow_id;this.sessions_loaded=!1,this.retrySessionsAndRestore(e)}async retrySessionsAndRestore(e){if(!await this.ensureSessionsLoaded(!0)||e!==this.navigation_workflow_id||this.active_view!=="sessions")return;const t=this.selected_session?.session_id??this.requested_session_id;if(t===void 0)return;const i=this.selected_session_node_id??this.requested_session_node_id;await this.loadSession(t,this.sessions.find(n=>n.session_id===t),this.selected_session_detail?.session.session_id===t,null,i)}async refreshSessions(){const e=this.navigation_workflow_id,s=this.selected_session?.session_id??this.requested_session_id,t=this.selected_session_node_id,i=await this.ensureSessionsLoaded(!0),n=this.selected_session?.session_id??this.requested_session_id;i&&e===this.navigation_workflow_id&&s!==void 0&&n===s&&this.selected_session_node_id===t&&await this.loadSession(s,this.sessions.find(r=>r.session_id===s),!0,null,t)}filteredSessions(){const e=this.session_search_query.trim().toLocaleLowerCase();return e?this.sessions.filter(s=>[s.session_id,s.model,s.provider_id,s.account_id,s.endpoint,s.status===null?null:String(s.status)].some(t=>t?.toLocaleLowerCase().includes(e))):this.sessions}async loadSessionUsage(e,s){this.session_usage_controller?.abort();const t=new AbortController;this.session_usage_controller=t;const i=++this.session_usage_load_id;s||(this.selected_session_usage=void 0),this.session_usage_state="loading",this.session_usage_error=void 0;try{const n=new URLSearchParams({session_id:e}),r=await m(`/api/session-usage?${n.toString()}`,t.signal);return i===this.session_usage_load_id&&this.session_usage_controller===t?(this.selected_session_usage=r??void 0,this.session_usage_state="ready",!0):!1}catch(n){return i===this.session_usage_load_id&&!R(n)&&(this.session_usage_state="error",this.session_usage_error=k(n,"Unable to load session usage")),!1}finally{this.session_usage_controller===t&&(this.session_usage_controller=void 0)}}async loadSession(e,s,t,i="push",n){this.session_detail_controller?.abort(),this.session_node_controller?.abort();const r=new AbortController;this.session_detail_controller=r;const c=++this.session_detail_load_id,l=++this.session_node_load_id;this.requested_session_id=e,this.requested_session_node_id=n,this.selected_session=s,t||(this.selected_session_detail=void 0,this.selected_session_node_detail=void 0,this.selected_session_node_id=void 0,this.session_node_state="idle",this.session_node_error=void 0),this.loadSessionUsage(e,t),this.session_detail_state="loading",this.session_detail_error=void 0,i&&this.syncUrl(i);try{const d=new URLSearchParams({session_id:e,limit:"500"}),_=await m(`/api/session?${d.toString()}`,r.signal);if(c===this.session_detail_load_id&&this.session_detail_controller===r){if(this.selected_session=_.session,this.selected_session_detail=_,this.sessions=this.sessions.map(h=>h.session_id===_.session.session_id?_.session:h),this.session_detail_state="ready",l!==this.session_node_load_id)return!0;if(n){const h=_.nodes.find(p=>p.node_id===n);this.loadSessionNode(h??n,!1,"replace")}else this.selected_session_node_id=void 0,this.selected_session_node_detail=void 0,this.session_node_state="idle",this.syncUrl("replace");return!0}return!1}catch(d){return c===this.session_detail_load_id&&!R(d)&&(this.session_detail_state="error",this.session_detail_error=k(d,"Unable to load semantic session")),!1}finally{this.session_detail_controller===r&&(this.session_detail_controller=void 0)}}async loadSessionNode(e,s,t="push"){const i=this.selected_session?.session_id??this.requested_session_id;if(i===void 0)return!1;this.session_node_controller?.abort();const n=new AbortController;this.session_node_controller=n;const r=++this.session_node_load_id,c=typeof e=="string"?e:e.node_id;this.requested_session_node_id=c,this.selected_session_node_id=c,s||(this.selected_session_node_detail=void 0),this.session_node_state="loading",this.session_node_error=void 0,t&&this.syncUrl(t);try{const l=new URLSearchParams({session_id:i,node_id:c}),d=await m(`/api/session-node?${l.toString()}`,n.signal);return r===this.session_node_load_id&&this.session_node_controller===n?(this.selected_session_node_detail=d,this.session_node_state="ready",this.syncUrl("replace"),!0):!1}catch(l){return r===this.session_node_load_id&&!R(l)&&(this.session_node_state="error",this.session_node_error=k(l,"Unable to load semantic node content")),!1}finally{this.session_node_controller===n&&(this.session_node_controller=void 0)}}async selectSession(e){const s=++this.navigation_workflow_id;if(!await this.loadSession(e.session_id,e,!1,"push")||s!==this.navigation_workflow_id||this.active_view!=="sessions"||this.selected_session_detail?.session.session_id!==e.session_id||!window.matchMedia("(max-width: 680px)").matches)return;await this.updateComplete;const i=this.querySelector("session-detail-view");await i?.updateComplete,s===this.navigation_workflow_id&&this.active_view==="sessions"&&this.selected_session_detail?.session.session_id===e.session_id&&i?.querySelector(".mobile-back-button")?.focus()}collapseSessionNode(e="push"){this.session_node_controller?.abort(),this.session_node_controller=void 0,++this.session_node_load_id,this.requested_session_node_id=void 0,this.selected_session_node_id=void 0,this.selected_session_node_detail=void 0,this.session_node_state="idle",this.session_node_error=void 0,e&&this.syncUrl(e)}selectSessionNode(e){if(e.node_id===this.selected_session_node_id){this.collapseSessionNode();return}this.loadSessionNode(e,!1,"push")}retrySessionDetail(){const e=this.selected_session?.session_id??this.requested_session_id;e!==void 0&&this.loadSession(e,this.selected_session,!!this.selected_session_detail,null,this.selected_session_node_id??this.requested_session_node_id)}retrySessionUsage(){const e=this.selected_session?.session_id??this.requested_session_id;e!==void 0&&this.loadSessionUsage(e,!!this.selected_session_usage)}retrySessionNode(){const e=this.selected_session_detail?.nodes.find(s=>s.node_id===this.selected_session_node_id);(e??this.selected_session_node_id)&&this.loadSessionNode(e??this.selected_session_node_id,!!this.selected_session_node_detail,null)}async openSession(e){++this.navigation_workflow_id,this.setActiveView("sessions",!1,null),await this.ensureSessionsLoaded();const s=this.sessions.find(t=>t.session_id===e);await this.loadSession(e,s,!1,"push")}async openRequestFromSession(e){++this.navigation_workflow_id,this.setActiveView("requests",!1,null),this.search_query="",this.provider_id="",this.status_filter="",this.errors_only=!1,this.applied_filters=X(),this.selected_day=Os(e.ts),this.resetRequestSelection(),this.loadRequestDays(),this.loadRequests(),!await this.loadRequestDetail(this.selected_day,e.request_id,void 0,void 0,!1,"push")&&this.request_detail_state==="error"&&this.request_detail_error==="request not found"&&(this.request_detail_error="Request history is unavailable; semantic session data is still retained.")}async loadRequestsView(){this.loadRequestDays(),this.selected_day?await this.loadRequests():await this.loadLatestRequests()}setActiveView(e,s=!0,t="push"){t==="push"&&++this.navigation_workflow_id,this.active_view=e,t&&this.syncUrl(t),s&&(e==="sessions"?this.ensureSessionsLoaded():this.request_list_state==="idle"&&this.loadRequestsView())}setTimezone(e){this.timezone=e,this.syncUrl("push")}setDetailTab(e){this.active_detail_tab=e,this.syncUrl("push")}renderDayPicker(){const e=this.pickerDays(),s=this.adjacentAvailableDay(-1),t=this.adjacentAvailableDay(1);return a`
      <div class="day-control">
        <span class="control-label">UTC storage day</span>
        <div class="day-navigation">
          <button
            type="button"
            class="icon-button"
            title="Previous available day"
            aria-label="Previous available day"
            ?disabled=${!s}
            @click=${()=>s&&this.selectDay(s)}
          >
            ←
          </button>
          <select
            aria-label="Request storage day"
            .value=${this.selected_day??""}
            ?disabled=${e.length===0}
            @change=${i=>this.selectDay(i.target.value)}
          >
            ${this.selected_day?u:a`<option value="">No request day</option>`}
            ${e.map(i=>a`
                <option value=${i.day} ?disabled=${i.state!=="available"}>
                  ${i.day}${i.state==="empty"?" · empty":i.state==="unavailable"?" · unavailable":""}
                </option>
              `)}
          </select>
          <button
            type="button"
            class="icon-button"
            title="Next available day"
            aria-label="Next available day"
            ?disabled=${!t}
            @click=${()=>t&&this.selectDay(t)}
          >
            →
          </button>
        </div>
      </div>
    `}renderRequestToolbar(){const e=!!this.selected_day;return a`
      <section class="request-toolbar" aria-label="Request controls">
        <div class="toolbar-primary">
          ${this.renderDayPicker()}
          <button
            type="button"
            class="refresh-button"
            ?disabled=${!e||this.request_list_state==="loading"}
            @click=${()=>{this.loadRequests(),this.loadRequestDays()}}
          >
            <span aria-hidden="true">↻</span> Refresh requests
          </button>
          <div class="timezone-toggle" role="group" aria-label="Timestamp timezone">
            <button
              type="button"
              aria-pressed=${String(this.timezone==="local")}
              @click=${()=>this.setTimezone("local")}
            >
              Local
            </button>
            <button
              type="button"
              aria-pressed=${String(this.timezone==="utc")}
              @click=${()=>this.setTimezone("utc")}
            >
              UTC
            </button>
          </div>
        </div>
        <form class="filter-bar" @submit=${this.submitFilters}>
          <label class="search-field">
            <span class="visually-hidden">Search requests</span>
            <span class="search-icon" aria-hidden="true">⌕</span>
            <input
              type="search"
              .value=${this.search_query}
              ?disabled=${!e}
              placeholder="Search request, session, model…"
              @input=${s=>this.search_query=s.target.value}
            />
          </label>
          <label>
            <span class="visually-hidden">Provider ID</span>
            <input
              list="provider-options"
              .value=${this.provider_id}
              ?disabled=${!e}
              placeholder="Any provider"
              @input=${s=>this.provider_id=s.target.value}
            />
            <datalist id="provider-options">
              ${this.providerOptions().map(s=>a`<option value=${s}></option>`)}
            </datalist>
          </label>
          <label>
            <span class="visually-hidden">Exact response status</span>
            <input
              class="status-filter"
              type="number"
              min="100"
              max="599"
              step="1"
              .value=${this.status_filter}
              ?disabled=${!e}
              placeholder="Any status"
              @input=${s=>this.status_filter=s.target.value}
            />
          </label>
          <label class="errors-filter">
            <input
              type="checkbox"
              .checked=${this.errors_only}
              ?disabled=${!e}
              @change=${s=>this.errors_only=s.target.checked}
            />
            <span>Errors only</span>
          </label>
          <button type="submit" class="primary-button" ?disabled=${!e||!this.filtersChanged()}>Apply</button>
          ${this.hasAppliedFilters()?a`<button type="button" class="text-button" @click=${this.clearFilters}>Clear</button>`:u}
        </form>
        ${this.request_days_error?a`<p class="toolbar-warning" role="status">Day scan: ${this.request_days_error}</p>`:u}
      </section>
    `}renderRequestSidebar(){const e=this.requests.length>0;return a`
      <div class="list-pane" aria-busy=${String(this.request_list_state==="loading")}>
        <header class="list-pane-header">
          <div>
            <strong>Requests</strong>
            <span>${this.requests.length.toLocaleString()} loaded${this.next_cursor?" · more available":""}</span>
          </div>
          ${this.hasAppliedFilters()?a`<span class="filter-indicator">Filtered</span>`:u}
        </header>
        ${this.request_list_state==="loading"?a`
              <div class="inline-state" role="status">
                <span class="spinner" aria-hidden="true"></span>${e?"Refreshing requests…":"Loading requests…"}
              </div>
            `:u}
        ${this.request_list_state==="error"?a`
              <div class="inline-error" role="alert">
                <span>${this.request_list_error}</span>
                <button type="button" @click=${()=>{this.loadRequests()}}>Retry</button>
              </div>
            `:u}
        ${e?a`
              <request-list
                .requests=${this.requests}
                .selected_key=${this.selectedRequestDay()&&this.selected_request_row_id?M({day:this.selectedRequestDay(),row_id:this.selected_request_row_id}):void 0}
                .timezone=${this.timezone}
                @request-select=${s=>{this.selectRequest(P(s))}}
              ></request-list>
            `:this.request_list_state==="ready"?a`<p class="empty">No persisted requests match these filters.</p>`:this.request_list_state==="idle"?a`<p class="empty">Choose an available request day.</p>`:u}
        ${this.load_more_error?a`
              <div class="inline-error load-more-error" role="alert">
                <span>${this.load_more_error}</span>
                <button type="button" @click=${()=>{this.loadRequests(!0)}}>Retry</button>
              </div>
            `:u}
        ${this.next_cursor&&e?a`
              <div class="list-footer">
                <button type="button" class="secondary-button" ?disabled=${this.loading_more} @click=${()=>{this.loadRequests(!0)}}>
                  ${this.loading_more?"Loading…":"Load more"}
                </button>
              </div>
            `:e&&this.request_list_state==="ready"?a`<p class="end-of-list">End of loaded day</p>`:u}
      </div>
    `}renderSessionsSidebar(){const e=this.filteredSessions(),s=this.sessions.length>0;return a`
      <div class="list-pane" aria-busy=${String(this.sessions_loading)}>
        <header class="list-pane-header">
          <div>
            <strong>Recent sessions</strong>
            <span>
              ${this.session_search_query?`${e.length.toLocaleString()} of ${this.sessions.length.toLocaleString()} loaded`:`${this.sessions.length.toLocaleString()} loaded · newest first`}
            </span>
          </div>
          ${this.session_search_query?a`<span class="filter-indicator">Filtered</span>`:u}
        </header>
        ${this.sessions_loading?a`
              <div class="inline-state" role="status">
                <span class="spinner" aria-hidden="true"></span>${s?"Refreshing sessions…":"Loading sessions…"}
              </div>
            `:u}
        ${this.sessions_error?a`
              <div class="inline-error" role="alert">
                <span>${this.sessions_error}</span>
                <button type="button" @click=${this.retrySessions}>Retry</button>
              </div>
            `:u}
        ${e.length>0?a`
              <session-list
                .sessions=${e}
                .selected_session_id=${this.selected_session?.session_id??this.requested_session_id}
                .timezone=${this.timezone}
                @session-select=${t=>{this.selectSession(P(t))}}
              ></session-list>
            `:this.sessions_loaded&&this.session_search_query?a`<p class="empty">No recent sessions match this filter.</p>`:this.sessions_loaded?a`
                  <div class="empty empty-session-list">
                    <strong>No semantic sessions available</strong>
                    <span>The gateway records successful sessions here when session persistence is enabled.</span>
                  </div>
                `:u}
        ${s&&!this.session_search_query?a`<p class="end-of-list">${this.sessions.length===100?"Latest 100 sessions":"End of recent sessions"}</p>`:u}
      </div>
    `}renderSessionDetail(){return a`
      <session-detail-view
        .detail=${this.selected_session_detail}
        .node_detail=${this.selected_session_node_detail}
        .usage=${this.selected_session_usage}
        .state=${this.session_detail_state}
        .error_message=${this.session_detail_error}
        .usage_state=${this.session_usage_state}
        .usage_error_message=${this.session_usage_error}
        .node_state=${this.session_node_state}
        .node_error_message=${this.session_node_error}
        .selected_node_id=${this.selected_session_node_id}
        .timezone=${this.timezone}
        @session-close=${()=>{this.closeSessionDetail()}}
        @session-retry=${this.retrySessionDetail}
        @session-usage-retry=${this.retrySessionUsage}
        @session-node-retry=${this.retrySessionNode}
        @session-node-select=${e=>this.selectSessionNode(P(e))}
        @open-request=${e=>{this.openRequestFromSession(P(e))}}
      ></session-detail-view>
    `}renderSessionToolbar(){return a`
      <section class="session-toolbar">
        <label class="session-search-field">
          <span class="visually-hidden">Filter recent sessions</span>
          <span class="search-icon" aria-hidden="true">⌕</span>
          <input
            type="search"
            .value=${this.session_search_query}
            placeholder="Filter session, model, provider…"
            @input=${e=>this.session_search_query=e.target.value}
          />
        </label>
        <p><span class="source-indicator" aria-hidden="true"></span>Semantic trees and content from <code>sessions.db</code></p>
        <div class="session-toolbar-actions">
          <button
            type="button"
            class="refresh-button"
            ?disabled=${this.sessions_loading}
            @click=${()=>{this.refreshSessions()}}
          >
            <span aria-hidden="true">↻</span> Refresh sessions
          </button>
          <div class="timezone-toggle" role="group" aria-label="Timestamp timezone">
            <button type="button" aria-pressed=${String(this.timezone==="local")} @click=${()=>this.setTimezone("local")}>Local</button>
            <button type="button" aria-pressed=${String(this.timezone==="utc")} @click=${()=>this.setTimezone("utc")}>UTC</button>
          </div>
        </div>
      </section>
    `}render(){const e=this.active_view==="sessions"?this.info?.sessions_db:this.info?.requests_dir,s=this.active_view==="requests"?!!this.selected_request_id:this.requested_session_id!==void 0;return a`
      <header class="app-header">
        <div class="brand">
          <span class="brand-mark" aria-hidden="true">t</span>
          <div><h1>tokn inspect</h1><p>Local · read only</p></div>
        </div>
        <p class="sensitive-notice">History may contain sensitive prompts and responses.</p>
      </header>
      <main class="app-shell">
        <div class="shell-navigation">
          <nav class="view-navigation" aria-label="Inspector views">
            <button
              type="button"
              aria-current=${this.active_view==="requests"?"page":"false"}
              @click=${()=>this.setActiveView("requests")}
            >
              Requests
            </button>
            <button
              type="button"
              aria-current=${this.active_view==="sessions"?"page":"false"}
              @click=${()=>this.setActiveView("sessions")}
            >
              Sessions
            </button>
          </nav>
          <span class="data-path" title=${e??""}>${e??"Loading data source…"}</span>
        </div>
        ${this.active_view==="requests"?this.renderRequestToolbar():this.renderSessionToolbar()}
        <section class="viewer-grid ${this.active_view==="requests"?"request-view":"session-view"} ${s?"has-selection":""}">
          <aside class="sidebar" aria-label=${this.active_view==="requests"?"Request list":"Session list"}>
            ${this.active_view==="requests"?this.renderRequestSidebar():this.renderSessionsSidebar()}
          </aside>
          <article class="detail-pane" aria-label=${this.active_view==="requests"?"Request detail":"Session detail"}>
            ${this.active_view==="requests"?a`
                  <request-detail-view
                    .detail=${this.selected_request_detail}
                    .summary=${this.selected_request}
                    .state=${this.request_detail_state}
                    .error_message=${this.request_detail_error}
                    .active_tab=${this.active_detail_tab}
                    .timezone=${this.timezone}
                    @detail-retry=${this.retryRequestDetail}
                    @detail-close=${()=>{this.closeRequestDetail()}}
                    @detail-tab-change=${t=>this.setDetailTab(P(t))}
                    @open-session=${t=>{this.openSession(P(t))}}
                  ></request-detail-view>
                `:this.renderSessionDetail()}
          </article>
        </section>
      </main>
    `}}customElements.define("inspect-app",Ms);
