(function(){const e=document.createElement("link").relList;if(e&&e.supports&&e.supports("modulepreload"))return;for(const n of document.querySelectorAll('link[rel="modulepreload"]'))s(n);new MutationObserver(n=>{for(const o of n)if(o.type==="childList")for(const r of o.addedNodes)r.tagName==="LINK"&&r.rel==="modulepreload"&&s(r)}).observe(document,{childList:!0,subtree:!0});function t(n){const o={};return n.integrity&&(o.integrity=n.integrity),n.referrerPolicy&&(o.referrerPolicy=n.referrerPolicy),n.crossOrigin==="use-credentials"?o.credentials="include":n.crossOrigin==="anonymous"?o.credentials="omit":o.credentials="same-origin",o}function s(n){if(n.ep)return;n.ep=!0;const o=t(n);fetch(n.href,o)}})();const re=globalThis,be=re.ShadowRoot&&(re.ShadyCSS===void 0||re.ShadyCSS.nativeShadow)&&"adoptedStyleSheets"in Document.prototype&&"replace"in CSSStyleSheet.prototype,Ke=Symbol(),ke=new WeakMap;let ot=class{constructor(e,t,s){if(this._$cssResult$=!0,s!==Ke)throw Error("CSSResult is not constructable. Use `unsafeCSS` or `css` instead.");this.cssText=e,this.t=t}get styleSheet(){let e=this.o;const t=this.t;if(be&&e===void 0){const s=t!==void 0&&t.length===1;s&&(e=ke.get(t)),e===void 0&&((this.o=e=new CSSStyleSheet).replaceSync(this.cssText),s&&ke.set(t,e))}return e}toString(){return this.cssText}};const rt=i=>new ot(typeof i=="string"?i:i+"",void 0,Ke),at=(i,e)=>{if(be)i.adoptedStyleSheets=e.map(t=>t instanceof CSSStyleSheet?t:t.styleSheet);else for(const t of e){const s=document.createElement("style"),n=re.litNonce;n!==void 0&&s.setAttribute("nonce",n),s.textContent=t.cssText,i.appendChild(s)}},Re=be?i=>i:i=>i instanceof CSSStyleSheet?(e=>{let t="";for(const s of e.cssRules)t+=s.cssText;return rt(t)})(i):i;const{is:dt,defineProperty:lt,getOwnPropertyDescriptor:ct,getOwnPropertyNames:ut,getOwnPropertySymbols:_t,getPrototypeOf:ht}=Object,ce=globalThis,Ae=ce.trustedTypes,pt=Ae?Ae.emptyScript:"",yt=ce.reactiveElementPolyfillSupport,Y=(i,e)=>i,ge={toAttribute(i,e){switch(e){case Boolean:i=i?pt:null;break;case Object:case Array:i=i==null?i:JSON.stringify(i)}return i},fromAttribute(i,e){let t=i;switch(e){case Boolean:t=i!==null;break;case Number:t=i===null?null:Number(i);break;case Object:case Array:try{t=JSON.parse(i)}catch{t=null}}return t}},Ze=(i,e)=>!dt(i,e),xe={attribute:!0,type:String,converter:ge,reflect:!1,useDefault:!1,hasChanged:Ze};Symbol.metadata??=Symbol("metadata"),ce.litPropertyMetadata??=new WeakMap;let F=class extends HTMLElement{static addInitializer(e){this._$Ei(),(this.l??=[]).push(e)}static get observedAttributes(){return this.finalize(),this._$Eh&&[...this._$Eh.keys()]}static createProperty(e,t=xe){if(t.state&&(t.attribute=!1),this._$Ei(),this.prototype.hasOwnProperty(e)&&((t=Object.create(t)).wrapped=!0),this.elementProperties.set(e,t),!t.noAccessor){const s=Symbol(),n=this.getPropertyDescriptor(e,s,t);n!==void 0&&lt(this.prototype,e,n)}}static getPropertyDescriptor(e,t,s){const{get:n,set:o}=ct(this.prototype,e)??{get(){return this[t]},set(r){this[t]=r}};return{get:n,set(r){const u=n?.call(this);o?.call(this,r),this.requestUpdate(e,u,s)},configurable:!0,enumerable:!0}}static getPropertyOptions(e){return this.elementProperties.get(e)??xe}static _$Ei(){if(this.hasOwnProperty(Y("elementProperties")))return;const e=ht(this);e.finalize(),e.l!==void 0&&(this.l=[...e.l]),this.elementProperties=new Map(e.elementProperties)}static finalize(){if(this.hasOwnProperty(Y("finalized")))return;if(this.finalized=!0,this._$Ei(),this.hasOwnProperty(Y("properties"))){const t=this.properties,s=[...ut(t),..._t(t)];for(const n of s)this.createProperty(n,t[n])}const e=this[Symbol.metadata];if(e!==null){const t=litPropertyMetadata.get(e);if(t!==void 0)for(const[s,n]of t)this.elementProperties.set(s,n)}this._$Eh=new Map;for(const[t,s]of this.elementProperties){const n=this._$Eu(t,s);n!==void 0&&this._$Eh.set(n,t)}this.elementStyles=this.finalizeStyles(this.styles)}static finalizeStyles(e){const t=[];if(Array.isArray(e)){const s=new Set(e.flat(1/0).reverse());for(const n of s)t.unshift(Re(n))}else e!==void 0&&t.push(Re(e));return t}static _$Eu(e,t){const s=t.attribute;return s===!1?void 0:typeof s=="string"?s:typeof e=="string"?e.toLowerCase():void 0}constructor(){super(),this._$Ep=void 0,this.isUpdatePending=!1,this.hasUpdated=!1,this._$Em=null,this._$Ev()}_$Ev(){this._$ES=new Promise(e=>this.enableUpdating=e),this._$AL=new Map,this._$E_(),this.requestUpdate(),this.constructor.l?.forEach(e=>e(this))}addController(e){(this._$EO??=new Set).add(e),this.renderRoot!==void 0&&this.isConnected&&e.hostConnected?.()}removeController(e){this._$EO?.delete(e)}_$E_(){const e=new Map,t=this.constructor.elementProperties;for(const s of t.keys())this.hasOwnProperty(s)&&(e.set(s,this[s]),delete this[s]);e.size>0&&(this._$Ep=e)}createRenderRoot(){const e=this.shadowRoot??this.attachShadow(this.constructor.shadowRootOptions);return at(e,this.constructor.elementStyles),e}connectedCallback(){this.renderRoot??=this.createRenderRoot(),this.enableUpdating(!0),this._$EO?.forEach(e=>e.hostConnected?.())}enableUpdating(e){}disconnectedCallback(){this._$EO?.forEach(e=>e.hostDisconnected?.())}attributeChangedCallback(e,t,s){this._$AK(e,s)}_$ET(e,t){const s=this.constructor.elementProperties.get(e),n=this.constructor._$Eu(e,s);if(n!==void 0&&s.reflect===!0){const o=(s.converter?.toAttribute!==void 0?s.converter:ge).toAttribute(t,s.type);this._$Em=e,o==null?this.removeAttribute(n):this.setAttribute(n,o),this._$Em=null}}_$AK(e,t){const s=this.constructor,n=s._$Eh.get(e);if(n!==void 0&&this._$Em!==n){const o=s.getPropertyOptions(n),r=typeof o.converter=="function"?{fromAttribute:o.converter}:o.converter?.fromAttribute!==void 0?o.converter:ge;this._$Em=n;const u=r.fromAttribute(t,o.type);this[n]=u??this._$Ej?.get(n)??u,this._$Em=null}}requestUpdate(e,t,s,n=!1,o){if(e!==void 0){const r=this.constructor;if(n===!1&&(o=this[e]),s??=r.getPropertyOptions(e),!((s.hasChanged??Ze)(o,t)||s.useDefault&&s.reflect&&o===this._$Ej?.get(e)&&!this.hasAttribute(r._$Eu(e,s))))return;this.C(e,t,s)}this.isUpdatePending===!1&&(this._$ES=this._$EP())}C(e,t,{useDefault:s,reflect:n,wrapped:o},r){s&&!(this._$Ej??=new Map).has(e)&&(this._$Ej.set(e,r??t??this[e]),o!==!0||r!==void 0)||(this._$AL.has(e)||(this.hasUpdated||s||(t=void 0),this._$AL.set(e,t)),n===!0&&this._$Em!==e&&(this._$Eq??=new Set).add(e))}async _$EP(){this.isUpdatePending=!0;try{await this._$ES}catch(t){Promise.reject(t)}const e=this.scheduleUpdate();return e!=null&&await e,!this.isUpdatePending}scheduleUpdate(){return this.performUpdate()}performUpdate(){if(!this.isUpdatePending)return;if(!this.hasUpdated){if(this.renderRoot??=this.createRenderRoot(),this._$Ep){for(const[n,o]of this._$Ep)this[n]=o;this._$Ep=void 0}const s=this.constructor.elementProperties;if(s.size>0)for(const[n,o]of s){const{wrapped:r}=o,u=this[n];r!==!0||this._$AL.has(n)||u===void 0||this.C(n,void 0,o,u)}}let e=!1;const t=this._$AL;try{e=this.shouldUpdate(t),e?(this.willUpdate(t),this._$EO?.forEach(s=>s.hostUpdate?.()),this.update(t)):this._$EM()}catch(s){throw e=!1,this._$EM(),s}e&&this._$AE(t)}willUpdate(e){}_$AE(e){this._$EO?.forEach(t=>t.hostUpdated?.()),this.hasUpdated||(this.hasUpdated=!0,this.firstUpdated(e)),this.updated(e)}_$EM(){this._$AL=new Map,this.isUpdatePending=!1}get updateComplete(){return this.getUpdateComplete()}getUpdateComplete(){return this._$ES}shouldUpdate(e){return!0}update(e){this._$Eq&&=this._$Eq.forEach(t=>this._$ET(t,this[t])),this._$EM()}updated(e){}firstUpdated(e){}};F.elementStyles=[],F.shadowRootOptions={mode:"open"},F[Y("elementProperties")]=new Map,F[Y("finalized")]=new Map,yt?.({ReactiveElement:F}),(ce.reactiveElementVersions??=[]).push("2.1.2");const we=globalThis,Ce=i=>i,le=we.trustedTypes,Ee=le?le.createPolicy("lit-html",{createHTML:i=>i}):void 0,Ge="$lit$",L=`lit$${Math.random().toFixed(9).slice(2)}$`,Ye="?"+L,ft=`<${Ye}>`,M=document,X=()=>M.createComment(""),ee=i=>i===null||typeof i!="object"&&typeof i!="function",qe=Array.isArray,mt=i=>qe(i)||typeof i?.[Symbol.iterator]=="function",_e=`[ 	
\f\r]`,Z=/<(?:(!--|\/[^a-zA-Z])|(\/?[a-zA-Z][^>\s]*)|(\/?$))/g,Le=/-->/g,Ue=/>/g,P=RegExp(`>|${_e}(?:([^\\s"'>=/]+)(${_e}*=${_e}*(?:[^ 	
\f\r"'\`<>=]|("|')|))|$)`,"g"),Pe=/'/g,Ne=/"/g,Qe=/^(?:script|style|textarea|title)$/i,gt=i=>(e,...t)=>({_$litType$:i,strings:e,values:t}),a=gt(1),J=Symbol.for("lit-noChange"),c=Symbol.for("lit-nothing"),Te=new WeakMap,D=M.createTreeWalker(M,129);function Xe(i,e){if(!qe(i)||!i.hasOwnProperty("raw"))throw Error("invalid template strings array");return Ee!==void 0?Ee.createHTML(e):e}const vt=(i,e)=>{const t=i.length-1,s=[];let n,o=e===2?"<svg>":e===3?"<math>":"",r=Z;for(let u=0;u<t;u++){const l=i[u];let d,_,h=-1,p=0;for(;p<l.length&&(r.lastIndex=p,_=r.exec(l),_!==null);)p=r.lastIndex,r===Z?_[1]==="!--"?r=Le:_[1]!==void 0?r=Ue:_[2]!==void 0?(Qe.test(_[2])&&(n=RegExp("</"+_[2],"g")),r=P):_[3]!==void 0&&(r=P):r===P?_[0]===">"?(r=n??Z,h=-1):_[1]===void 0?h=-2:(h=r.lastIndex-_[2].length,d=_[1],r=_[3]===void 0?P:_[3]==='"'?Ne:Pe):r===Ne||r===Pe?r=P:r===Le||r===Ue?r=Z:(r=P,n=void 0);const f=r===P&&i[u+1].startsWith("/>")?" ":"";o+=r===Z?l+ft:h>=0?(s.push(d),l.slice(0,h)+Ge+l.slice(h)+L+f):l+L+(h===-2?u:f)}return[Xe(i,o+(i[t]||"<?>")+(e===2?"</svg>":e===3?"</math>":"")),s]};class te{constructor({strings:e,_$litType$:t},s){let n;this.parts=[];let o=0,r=0;const u=e.length-1,l=this.parts,[d,_]=vt(e,t);if(this.el=te.createElement(d,s),D.currentNode=this.el.content,t===2||t===3){const h=this.el.content.firstChild;h.replaceWith(...h.childNodes)}for(;(n=D.nextNode())!==null&&l.length<u;){if(n.nodeType===1){if(n.hasAttributes())for(const h of n.getAttributeNames())if(h.endsWith(Ge)){const p=_[r++],f=n.getAttribute(h).split(L),m=/([.?@])?(.*)/.exec(p);l.push({type:1,index:o,name:m[2],strings:f,ctor:m[1]==="."?bt:m[1]==="?"?wt:m[1]==="@"?qt:ue}),n.removeAttribute(h)}else h.startsWith(L)&&(l.push({type:6,index:o}),n.removeAttribute(h));if(Qe.test(n.tagName)){const h=n.textContent.split(L),p=h.length-1;if(p>0){n.textContent=le?le.emptyScript:"";for(let f=0;f<p;f++)n.append(h[f],X()),D.nextNode(),l.push({type:2,index:++o});n.append(h[p],X())}}}else if(n.nodeType===8)if(n.data===Ye)l.push({type:2,index:o});else{let h=-1;for(;(h=n.data.indexOf(L,h+1))!==-1;)l.push({type:7,index:o}),h+=L.length-1}o++}}static createElement(e,t){const s=M.createElement("template");return s.innerHTML=e,s}}function K(i,e,t=i,s){if(e===J)return e;let n=s!==void 0?t._$Co?.[s]:t._$Cl;const o=ee(e)?void 0:e._$litDirective$;return n?.constructor!==o&&(n?._$AO?.(!1),o===void 0?n=void 0:(n=new o(i),n._$AT(i,t,s)),s!==void 0?(t._$Co??=[])[s]=n:t._$Cl=n),n!==void 0&&(e=K(i,n._$AS(i,e.values),n,s)),e}class $t{constructor(e,t){this._$AV=[],this._$AN=void 0,this._$AD=e,this._$AM=t}get parentNode(){return this._$AM.parentNode}get _$AU(){return this._$AM._$AU}u(e){const{el:{content:t},parts:s}=this._$AD,n=(e?.creationScope??M).importNode(t,!0);D.currentNode=n;let o=D.nextNode(),r=0,u=0,l=s[0];for(;l!==void 0;){if(r===l.index){let d;l.type===2?d=new se(o,o.nextSibling,this,e):l.type===1?d=new l.ctor(o,l.name,l.strings,this,e):l.type===6&&(d=new St(o,this,e)),this._$AV.push(d),l=s[++u]}r!==l?.index&&(o=D.nextNode(),r++)}return D.currentNode=M,n}p(e){let t=0;for(const s of this._$AV)s!==void 0&&(s.strings!==void 0?(s._$AI(e,s,t),t+=s.strings.length-2):s._$AI(e[t])),t++}}class se{get _$AU(){return this._$AM?._$AU??this._$Cv}constructor(e,t,s,n){this.type=2,this._$AH=c,this._$AN=void 0,this._$AA=e,this._$AB=t,this._$AM=s,this.options=n,this._$Cv=n?.isConnected??!0}get parentNode(){let e=this._$AA.parentNode;const t=this._$AM;return t!==void 0&&e?.nodeType===11&&(e=t.parentNode),e}get startNode(){return this._$AA}get endNode(){return this._$AB}_$AI(e,t=this){e=K(this,e,t),ee(e)?e===c||e==null||e===""?(this._$AH!==c&&this._$AR(),this._$AH=c):e!==this._$AH&&e!==J&&this._(e):e._$litType$!==void 0?this.$(e):e.nodeType!==void 0?this.T(e):mt(e)?this.k(e):this._(e)}O(e){return this._$AA.parentNode.insertBefore(e,this._$AB)}T(e){this._$AH!==e&&(this._$AR(),this._$AH=this.O(e))}_(e){this._$AH!==c&&ee(this._$AH)?this._$AA.nextSibling.data=e:this.T(M.createTextNode(e)),this._$AH=e}$(e){const{values:t,_$litType$:s}=e,n=typeof s=="number"?this._$AC(e):(s.el===void 0&&(s.el=te.createElement(Xe(s.h,s.h[0]),this.options)),s);if(this._$AH?._$AD===n)this._$AH.p(t);else{const o=new $t(n,this),r=o.u(this.options);o.p(t),this.T(r),this._$AH=o}}_$AC(e){let t=Te.get(e.strings);return t===void 0&&Te.set(e.strings,t=new te(e)),t}k(e){qe(this._$AH)||(this._$AH=[],this._$AR());const t=this._$AH;let s,n=0;for(const o of e)n===t.length?t.push(s=new se(this.O(X()),this.O(X()),this,this.options)):s=t[n],s._$AI(o),n++;n<t.length&&(this._$AR(s&&s._$AB.nextSibling,n),t.length=n)}_$AR(e=this._$AA.nextSibling,t){for(this._$AP?.(!1,!0,t);e!==this._$AB;){const s=Ce(e).nextSibling;Ce(e).remove(),e=s}}setConnected(e){this._$AM===void 0&&(this._$Cv=e,this._$AP?.(e))}}class ue{get tagName(){return this.element.tagName}get _$AU(){return this._$AM._$AU}constructor(e,t,s,n,o){this.type=1,this._$AH=c,this._$AN=void 0,this.element=e,this.name=t,this._$AM=n,this.options=o,s.length>2||s[0]!==""||s[1]!==""?(this._$AH=Array(s.length-1).fill(new String),this.strings=s):this._$AH=c}_$AI(e,t=this,s,n){const o=this.strings;let r=!1;if(o===void 0)e=K(this,e,t,0),r=!ee(e)||e!==this._$AH&&e!==J,r&&(this._$AH=e);else{const u=e;let l,d;for(e=o[0],l=0;l<o.length-1;l++)d=K(this,u[s+l],t,l),d===J&&(d=this._$AH[l]),r||=!ee(d)||d!==this._$AH[l],d===c?e=c:e!==c&&(e+=(d??"")+o[l+1]),this._$AH[l]=d}r&&!n&&this.j(e)}j(e){e===c?this.element.removeAttribute(this.name):this.element.setAttribute(this.name,e??"")}}class bt extends ue{constructor(){super(...arguments),this.type=3}j(e){this.element[this.name]=e===c?void 0:e}}class wt extends ue{constructor(){super(...arguments),this.type=4}j(e){this.element.toggleAttribute(this.name,!!e&&e!==c)}}class qt extends ue{constructor(e,t,s,n,o){super(e,t,s,n,o),this.type=5}_$AI(e,t=this){if((e=K(this,e,t,0)??c)===J)return;const s=this._$AH,n=e===c&&s!==c||e.capture!==s.capture||e.once!==s.once||e.passive!==s.passive,o=e!==c&&(s===c||n);n&&this.element.removeEventListener(this.name,this,s),o&&this.element.addEventListener(this.name,this,e),this._$AH=e}handleEvent(e){typeof this._$AH=="function"?this._$AH.call(this.options?.host??this.element,e):this._$AH.handleEvent(e)}}class St{constructor(e,t,s){this.element=e,this.type=6,this._$AN=void 0,this._$AM=t,this.options=s}get _$AU(){return this._$AM._$AU}_$AI(e){K(this,e)}}const kt=we.litHtmlPolyfillSupport;kt?.(te,se),(we.litHtmlVersions??=[]).push("3.3.3");const Rt=(i,e,t)=>{const s=t?.renderBefore??e;let n=s._$litPart$;if(n===void 0){const o=t?.renderBefore??null;s._$litPart$=n=new se(e.insertBefore(X(),o),o,void 0,t??{})}return n._$AI(i),n};const Se=globalThis;class w extends F{constructor(){super(...arguments),this.renderOptions={host:this},this._$Do=void 0}createRenderRoot(){const e=super.createRenderRoot();return this.renderOptions.renderBefore??=e.firstChild,e}update(e){const t=this.render();this.hasUpdated||(this.renderOptions.isConnected=this.isConnected),super.update(e),this._$Do=Rt(t,this.renderRoot,this.renderOptions)}connectedCallback(){super.connectedCallback(),this._$Do?.setConnected(!0)}disconnectedCallback(){super.disconnectedCallback(),this._$Do?.setConnected(!1)}render(){return J}}w._$litElement$=!0,w.finalized=!0,Se.litElementHydrateSupport?.({LitElement:w});const At=Se.litElementPolyfillSupport;At?.({LitElement:w});(Se.litElementVersions??=[]).push("4.2.2");class et extends Error{status;constructor(e,t){super(t),this.name="HttpError",this.status=e}}async function $(i,e){const t=await fetch(i,{cache:"no-store",signal:e});if(!t.ok){const s=await t.json().catch(()=>({}));throw new et(t.status,s.error??`Request failed (${t.status})`)}return t.json()}function q(i){return i instanceof Error&&i.name==="AbortError"}function W(i,e,t=!1){const s=t?{hour:"2-digit",minute:"2-digit",second:"2-digit"}:{dateStyle:"medium",timeStyle:"medium"};return e==="utc"&&(s.timeZone="UTC"),new Intl.DateTimeFormat(void 0,s).format(new Date(i))}function xt(i,e){const t=new Date(i),s=new Date,n=e==="utc"?t.getUTCFullYear():t.getFullYear(),o=e==="utc"?s.getUTCFullYear():s.getFullYear(),r={month:"short",day:"numeric",hour:"2-digit",minute:"2-digit"};return n!==o&&(r.year="numeric"),e==="utc"&&(r.timeZone="UTC"),new Intl.DateTimeFormat(void 0,r).format(t)}function Ct(i,e){const t=Math.max(0,e-i);if(t<1e3)return`${t.toLocaleString()} ms`;const s=Math.floor(t/1e3);if(s<60)return`${s}s`;const n=Math.floor(s/60);if(n<60)return`${n}m ${s%60}s`;const o=Math.floor(n/60);return o<24?`${o}h ${n%60}m`:`${Math.floor(o/24)}d ${o%24}h`}function V(i){return`${i.day}:${i.row_id}`}function R(i,e=10){return i?i.length>e?`…${i.slice(-e)}`:i:"—"}function Et(i){const e=i.inbound_req_url??i.endpoint;return O(e)}function De(i){const e=i.toLowerCase().replaceAll("_","-");return e==="authorization"||e==="password"||e==="code"||e==="signature"||e==="sig"||e.includes("api-key")||e.includes("access-key")||e.includes("token")||e.includes("secret")||e.includes("credential")}function O(i){if(!i)return"unknown endpoint";try{const e=new URL(i,window.location.origin);for(const t of new Set(e.searchParams.keys()))De(t)&&e.searchParams.set(t,"REDACTED");return`${e.pathname}${e.search}`}catch{return i.replace(/([?&]([^=&]+)=)([^&]*)/g,(e,t,s)=>{let n=s;try{n=decodeURIComponent(s)}catch{}return De(n)?`${t}REDACTED`:e})}}function Lt(i){if(i.request_error)return{label:"ERR",tone:"error",title:i.request_error};const e=i.inbound_resp_status??i.outbound_resp_status??i.status;if(e===null)return{label:"—",tone:"neutral",title:"No response status persisted"};const t=i.inbound_resp_status!==null?"Client response":i.outbound_resp_status!==null?"Provider response":"Request";return e>=400?{label:String(e),tone:"error",title:`${t}: ${e}`}:e>=300?{label:String(e),tone:"warning",title:`${t}: ${e}`}:{label:String(e),tone:"success",title:`${t}: ${e}`}}function Ut(i){const e=i.status;return e===null?{label:"—",tone:"neutral",title:"No status stored for the current session head"}:e>=400?{label:String(e),tone:"error",title:`Current head status: ${e}`}:e>=300?{label:String(e),tone:"warning",title:`Current head status: ${e}`}:{label:String(e),tone:"success",title:`Current head status: ${e}`}}function z(i){return i.detail}function y(i,e){const t=i[e];return typeof t=="string"?t:void 0}function U(i,e){const t=i[e];return typeof t=="number"?t:void 0}const he="••••••••";function pe(i){const e=i.toLowerCase().replaceAll("_","-");return e==="authorization"||e==="proxy-authorization"||e==="cookie"||e==="set-cookie"||e.includes("api-key")||e.includes("token")||e.includes("secret")}function Q(i){if(Array.isArray(i))return i.length===2&&typeof i[0]=="string"&&pe(i[0])?[i[0],he]:i.map(e=>Q(e));if(i!==null&&typeof i=="object")return Object.fromEntries(Object.entries(i).map(([e,t])=>[e,pe(e)?he:Q(t)]));if(typeof i=="string")try{return Q(JSON.parse(i))}catch{return i.replace(/^([^:\r\n]+)(:\s*)(.*)$/gm,(e,t,s)=>pe(t.trim())?`${t}${s}${he}`:e)}return i}function ve(i){return Array.isArray(i)?i.map(e=>ve(e)):i!==null&&typeof i=="object"?Object.fromEntries(Object.entries(i).map(([e,t])=>[e,Pt(e)?Q(t):ve(t)])):i}function Pt(i){const e=i.replace(/([a-z0-9])([A-Z])/g,"$1_$2").toLowerCase().replace(/[-\s]+/g,"_");return e==="headers"||e.endsWith("_headers")}function $e(i){return Array.isArray(i)?i.map(e=>$e(e)):i!==null&&typeof i=="object"?Object.fromEntries(Object.entries(i).map(([e,t])=>[e,e.toLowerCase().endsWith("_url")&&typeof t=="string"?O(t):$e(t)])):i}function Nt(i){if(typeof i=="string")try{return JSON.stringify(JSON.parse(i),null,2)}catch{return i}return JSON.stringify(i,null,2)??String(i)}function Tt(i){if(Array.isArray(i))return`${i.length} item${i.length===1?"":"s"}`;if(i!==null&&typeof i=="object"){const e=Object.keys(i).length;return`${e} field${e===1?"":"s"}`}return typeof i=="string"?`${new Blob([i]).size.toLocaleString()} bytes`:typeof i}class Dt extends w{static properties={label:{type:String},value:{attribute:!1},load_url:{type:String},is_headers:{type:Boolean},redact_record_headers:{type:Boolean},open:{type:Boolean,state:!0},wrap:{type:Boolean,state:!0},revealed:{type:Boolean,state:!0},copy_state:{type:String,state:!0},load_state:{type:String,state:!0},loaded_value:{attribute:!1,state:!0},error_message:{type:String,state:!0}};load_controller;copy_timeout;constructor(){super(),this.label="Payload",this.is_headers=!1,this.redact_record_headers=!1,this.open=!1,this.wrap=!0,this.revealed=!1,this.copy_state="idle",this.load_state="idle"}createRenderRoot(){return this}disconnectedCallback(){this.load_controller?.abort(),this.copy_timeout!==void 0&&window.clearTimeout(this.copy_timeout),super.disconnectedCallback()}willUpdate(e){!e.has("value")&&!e.has("load_url")||(this.load_controller?.abort(),this.load_controller=void 0,this.copy_timeout!==void 0&&(window.clearTimeout(this.copy_timeout),this.copy_timeout=void 0),this.open=!1,this.revealed=!1,this.copy_state="idle",this.load_state="idle",this.loaded_value=void 0,this.error_message=void 0)}effectiveValue(){return this.load_state==="ready"?this.loaded_value:this.value}displayedValue(){const e=this.effectiveValue(),t=this.redact_record_headers?$e(e):e,s=this.revealed?t:this.redact_record_headers?ve(t):this.is_headers?Q(t):t;return Nt(s)}toggleOpen(e){this.open=e.currentTarget.open,this.open&&this.value===void 0&&this.load_url&&this.load_state==="idle"&&this.loadPayload()}async loadPayload(){const e=this.load_url;if(!e)return;this.load_controller?.abort();const t=new AbortController;this.load_controller=t,this.load_state="loading",this.error_message=void 0;try{const s=await $(e,t.signal);if(this.load_controller!==t||this.load_url!==e)return;const n=new URL(e,window.location.origin).searchParams.get("field");if(!n||s.field!==n)throw new Error("Payload response did not match the requested field");this.loaded_value=s.value,this.load_state="ready"}catch(s){if(this.load_controller!==t||q(s))return;this.load_state="error",this.error_message=s instanceof Error?s.message:"Unable to load payload"}finally{this.load_controller===t&&(this.load_controller=void 0)}}async copyValue(){try{await navigator.clipboard.writeText(this.displayedValue()),this.copy_state="copied",this.copy_timeout!==void 0&&window.clearTimeout(this.copy_timeout),this.copy_timeout=window.setTimeout(()=>{this.copy_state="idle",this.copy_timeout=void 0},1500)}catch{this.copy_state="error"}}render(){if(!this.load_url&&(this.value===null||this.value===void 0||this.value===""))return c;const e=this.effectiveValue(),t=this.is_headers||this.redact_record_headers,s=this.load_state==="loading"?"Loading…":this.load_state==="error"?"Load failed":e===null?"No payload":e===void 0?"Load on open":Tt(e);return a`
      <details class="payload-panel" ?open=${this.open} @toggle=${this.toggleOpen}>
        <summary>
          <span>${this.label}</span>
          <span class="payload-summary">${s}</span>
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
                      ${t?a`
                            <button
                              type="button"
                              class=${this.revealed?"danger-button":""}
                              aria-pressed=${String(this.revealed)}
                              @click=${()=>this.revealed=!this.revealed}
                            >
                              ${this.revealed?"Hide sensitive":"Reveal sensitive"}
                            </button>
                          `:c}
                      <span class="payload-security-note">
                        ${t&&!this.revealed?"Sensitive headers redacted":""}
                      </span>
                    </div>
                    <pre class=${this.wrap?"wrap":"nowrap"}><code>${this.displayedValue()}</code></pre>
                  `:c}
      </details>
    `}}customElements.define("payload-panel",Dt);const Oe=new Set(["chat","chat_completions","messages","responses"]),Ot=["/chat/completions","/messages","/responses"];function ae(i){if(i!==null&&typeof i=="object"&&!Array.isArray(i))return i;if(typeof i=="string")try{const e=JSON.parse(i);return e!==null&&typeof e=="object"&&!Array.isArray(e)?e:void 0}catch{return}}function E(i,e){const t=i?.[e];return typeof t=="number"&&Number.isFinite(t)&&t>=0?t:void 0}function Mt(i){if(!(typeof i!="string"||i.length===0))try{return new URL(i,"http://localhost").pathname.toLowerCase().replace(/\/$/,"")}catch{return i.split(/[?#]/,1)[0]?.toLowerCase().replace(/\/$/,"")}}function tt(i){const e=ae(i.usage_json),t=y(e??{},"kind")?.toLowerCase();if(t&&Oe.has(t))return!0;const s=y(i,"endpoint")?.toLowerCase();return s&&Oe.has(s)?!0:y(i,"model")!==void 0||y(i,"provider_id")!==void 0?[i.inbound_req_url,i.outbound_req_url].map(Mt).some(o=>o!==void 0&&Ot.some(r=>o.endsWith(r))):!1}function zt(i){if(!tt(i))return;const e=ae(i.usage_json),t=ae(i.ctx_json),s=ae(i.params_json),n=E(t,"latency_ms"),o=E(t,"latency_header_ms");return{client_method:y(i,"inbound_req_method"),client_url:y(i,"inbound_req_url")??y(i,"endpoint"),provider_method:y(i,"outbound_req_method"),provider_url:y(i,"outbound_req_url"),endpoint:y(i,"endpoint"),provider_id:y(i,"provider_id"),model:y(i,"model"),account_id:y(i,"account_id"),pipeline:y(t??{},"pipeline_id"),mode:y(t??{},"mode"),stream:typeof s?.stream=="boolean"?s.stream:void 0,client_status:U(i,"inbound_resp_status")??U(i,"status"),provider_status:U(i,"outbound_resp_status"),latency_ms:n,first_response_ms:o,streamed_ms:n!==void 0&&o!==void 0?Math.max(0,n-o):void 0,usage:{kind:y(e??{},"kind"),input_tokens:E(e,"input"),output_tokens:E(e,"output"),total_tokens:E(e,"total"),cache_read_tokens:E(e,"cache_read"),cache_write_tokens:E(e,"cache_write"),reasoning_tokens:E(e,"reasoning")}}}function Bt(i){if(!(i.input_tokens===void 0||i.input_tokens===0||i.cache_read_tokens===void 0))return Math.min(100,i.cache_read_tokens/i.input_tokens*100)}function N(i){return i&&i.length>0?i:"—"}function B(i){return i===void 0?"—":i.toLocaleString()}function G(i){if(i===void 0)return"—";if(i<1e3)return`${Math.round(i).toLocaleString()} ms`;const e=i/1e3;return`${e>=10?e.toFixed(1):e.toFixed(2)} s`}function jt(i){return i<1e3?`${i} B`:`${(i/1e3).toFixed(1)} KB`}function Me(i){return i===void 0?"neutral":i>=400?"error":i>=300?"warning":"success"}function ze(i){return i===void 0?"—":String(i)}class It extends w{static properties={request:{attribute:!1},day:{type:String},request_id:{type:String},row_id:{type:String},timezone:{type:String},content_state:{type:String,state:!0},content_summary:{attribute:!1,state:!0},content_error:{type:String,state:!0}};content_controller;constructor(){super(),this.day="",this.request_id="",this.row_id="",this.content_state="idle"}createRenderRoot(){return this}disconnectedCallback(){this.content_controller?.abort(),super.disconnectedCallback()}updated(e){(e.has("day")||e.has("request_id")||e.has("row_id"))&&this.loadContentSummary()}async loadContentSummary(){if(!this.day||!this.request_id||!this.row_id)return;const e=`${this.day}:${this.request_id}:${this.row_id}`;this.content_controller?.abort();const t=new AbortController;this.content_controller=t,this.content_state="loading",this.content_error=void 0;try{const s=new URLSearchParams({day:this.day,request_id:this.request_id,row_id:this.row_id}),n=await $(`/api/request-llm-summary?${s}`,t.signal);if(this.content_controller!==t||e!==`${this.day}:${this.request_id}:${this.row_id}`)return;this.content_summary=n,this.content_state="ready"}catch(s){if(this.content_controller!==t||q(s))return;this.content_state="error",this.content_error=s instanceof Error?s.message:"Unable to load messages and tools"}finally{this.content_controller===t&&(this.content_controller=void 0)}}renderContentSummary(){if(this.content_state==="idle"||this.content_state==="loading")return a`
        <section class="llm-content-state" aria-live="polite">
          <span class="spinner" aria-hidden="true"></span>
          <span>Loading messages and tools…</span>
        </section>
      `;if(this.content_state==="error")return a`
        <section class="llm-content-state error-state" role="alert">
          <div><strong>Messages and tools could not be loaded</strong><span>${this.content_error}</span></div>
          <button type="button" @click=${()=>{this.loadContentSummary()}}>Retry</button>
        </section>
      `;const e=this.content_summary;return e?a`
      ${e.warning?a`<p class="llm-content-warning">${e.warning}</p>`:c}
      <div class="llm-content-grid">
        <section class="llm-content-panel" aria-labelledby="llm-messages-heading">
          <header>
            <div><p class="eyebrow">Conversation</p><h3 id="llm-messages-heading">Messages</h3></div>
            <span>${e.messages.length} of ${e.messages_total} shown</span>
          </header>
          <div class="llm-message-list">
            ${e.messages.length===0?a`<p class="llm-content-empty">No conversational messages recorded.</p>`:e.messages.map(t=>a`
                  <article class="llm-message-preview">
                    <header>
                      <strong>${t.role}</strong>
                      <span>${t.phase}</span>
                    </header>
                    <p>${t.text??"Non-text message"}</p>
                  </article>
                `)}
          </div>
        </section>

        <section class="llm-content-panel" aria-labelledby="llm-tools-heading">
          <header>
            <div><p class="eyebrow">Tool activity</p><h3 id="llm-tools-heading">Tools</h3></div>
            <span>${e.tool_calls_total} calls · ${e.tool_results_total} results</span>
          </header>
          ${e.tool_definitions.length>0?a`
                <div class="llm-tool-definitions">
                  ${e.tool_definitions.map(t=>a`
                    <span title=${t.kind}><strong>${t.name}</strong><small>${t.kind}</small></span>
                  `)}
                  ${e.tool_definitions_total>e.tool_definitions.length?a`<span>+${e.tool_definitions_total-e.tool_definitions.length} more</span>`:c}
                </div>
              `:c}
          <div class="llm-tool-list">
            ${e.tool_calls.length===0?a`<p class="llm-content-empty">No tool calls recorded.</p>`:e.tool_calls.map(t=>a`
                  <article class="llm-tool-call">
                    <div><strong>${t.name}</strong><span>${t.kind} · ${t.phase}</span></div>
                    <div><span>${jt(t.argument_bytes)}</span>${t.status?a`<span>${t.status}</span>`:c}</div>
                  </article>
                `)}
          </div>
        </section>
      </div>
    `:c}render(){const e=zt(this.request);if(!e)return c;const t=U(this.request,"ts"),s=Bt(e.usage),n=e.latency_ms??0,o=n>0&&e.first_response_ms!==void 0?Math.min(100,e.first_response_ms/n*100):0,r=[e.pipeline,e.mode].filter(Boolean).join(" · ");return a`
      <section class="llm-overview" aria-label="LLM request overview">
        <section class="llm-route-flow" aria-label="Model request route">
          <div class="llm-route-step">
            <span class="eyebrow">Client</span>
            <strong>${N(e.client_method)} ${O(e.client_url)}</strong>
            <small>Response <span class="status-text ${Me(e.client_status)}">${ze(e.client_status)}</span></small>
          </div>
          <span class="llm-route-arrow" aria-hidden="true">→</span>
          <div class="llm-route-step llm-route-model">
            <span class="eyebrow">${N(e.provider_id)}</span>
            <strong>${N(e.model)}</strong>
            <small>${N(e.endpoint)}${e.stream===void 0?"":e.stream?" · streaming":" · buffered"}</small>
          </div>
          <span class="llm-route-arrow" aria-hidden="true">→</span>
          <div class="llm-route-step">
            <span class="eyebrow">Provider</span>
            <strong>${N(e.provider_method)} ${O(e.provider_url)}</strong>
            <small>Response <span class="status-text ${Me(e.provider_status)}">${ze(e.provider_status)}</span></small>
          </div>
        </section>

        <div class="llm-metrics-grid">
          <section class="llm-metric-panel llm-token-panel" aria-labelledby="llm-token-heading">
            <header>
              <div>
                <p class="eyebrow">${e.usage.kind?`${e.usage.kind} usage`:"Usage"}</p>
                <h3 id="llm-token-heading">Token usage</h3>
              </div>
              ${s===void 0?c:a`<span>${s.toFixed(1)}% of input cached</span>`}
            </header>
            <dl class="llm-token-grid">
              <div class="llm-primary-metric"><dt>Total</dt><dd>${B(e.usage.total_tokens)}</dd></div>
              <div><dt>Input</dt><dd>${B(e.usage.input_tokens)}</dd></div>
              <div><dt>Output</dt><dd>${B(e.usage.output_tokens)}</dd></div>
              <div><dt>Cache read</dt><dd>${B(e.usage.cache_read_tokens)}</dd></div>
              ${e.usage.cache_write_tokens===void 0?c:a`<div><dt>Cache write</dt><dd>${B(e.usage.cache_write_tokens)}</dd></div>`}
              <div><dt>Reasoning</dt><dd>${B(e.usage.reasoning_tokens)}</dd></div>
            </dl>
          </section>

          <section class="llm-metric-panel llm-timing-panel" aria-labelledby="llm-timing-heading">
            <header>
              <div>
                <p class="eyebrow">Performance</p>
                <h3 id="llm-timing-heading">Response timing</h3>
              </div>
              <span>${e.stream?"Streamed":e.stream===!1?"Buffered":"Mode unknown"}</span>
            </header>
            ${e.latency_ms!==void 0&&e.first_response_ms!==void 0?a`
                  <div class="llm-timing-bar" title="First response ${G(e.first_response_ms)} of ${G(e.latency_ms)} total">
                    <span style=${`width: ${o}%`}></span>
                  </div>
                `:c}
            <dl class="llm-timing-grid">
              <div><dt>First response</dt><dd>${G(e.first_response_ms)}</dd></div>
              ${e.stream&&e.streamed_ms!==void 0?a`<div><dt>Streaming</dt><dd>${G(e.streamed_ms)}</dd></div>`:c}
              <div class="llm-primary-metric"><dt>Total</dt><dd>${G(e.latency_ms)}</dd></div>
            </dl>
          </section>
        </div>

        ${this.renderContentSummary()}

        <dl class="metadata-grid llm-metadata-grid">
          <div><dt>Timestamp</dt><dd>${t===void 0?"—":W(t,this.timezone)}</dd></div>
          <div><dt>Storage day</dt><dd>${this.day}</dd></div>
          <div><dt>Account</dt><dd title=${N(e.account_id)}>${N(e.account_id)}</dd></div>
          <div><dt>Pipeline</dt><dd title=${r||"—"}>${r||"—"}</dd></div>
        </dl>
      </section>
    `}}customElements.define("llm-request-overview",It);const Be="/backend-api/codex/alpha/search";function C(i){return i!==null&&typeof i=="object"&&!Array.isArray(i)?i:void 0}function v(i){return typeof i=="string"&&i.length>0?i:void 0}function st(i){return Array.isArray(i)?i.filter(e=>typeof e=="string"):[]}function ye(i){return typeof i=="number"&&Number.isFinite(i)?i:void 0}function Ht(i,e){const t=C(e);switch(i){case"search_query":{const s=v(t?.q);return s?{kind:i,value:s,domains:st(t?.domains),recency_days:ye(t?.recency)}:void 0}case"open":{const s=v(t?.ref_id);return s?{kind:i,value:s,line_number:ye(t?.lineno)}:void 0}case"click":{const s=v(t?.ref_id),n=ye(t?.id);return s&&n!==void 0?{kind:i,value:s,link_id:n}:void 0}case"find":{const s=v(t?.ref_id),n=v(t?.pattern);return s&&n?{kind:i,value:s,pattern:n}:void 0}default:return}}function Ft(i){const e=C(i);return e?Object.entries(e).flatMap(([t,s])=>Array.isArray(s)?s.flatMap(n=>{const o=Ht(t,n);return o?[o]:[]}):[]):[]}function Vt(i){if(i.length===0)return"No operations";if(new Set(i.map(s=>s.kind)).size!==1)return`${i.length} operations`;const t={search_query:["query","queries"],open:["page open","page opens"],click:["link click","link clicks"],find:["find","finds"]}[i[0].kind];return`${i.length} ${t[i.length===1?0:1]}`}function Wt(i){const e=C(i);if(!e)return;const t={type:v(e.type),domain:v(e.domain),ref_id:v(e.ref_id),snippet:v(e.snippet),title:v(e.title),url:v(e.url)};return Object.values(t).some(s=>s!==void 0)?t:void 0}function Jt(i){if(Array.isArray(i))for(const e of i){const s=C(e)?.content;if(Array.isArray(s))for(const n of s){const o=C(n),r=v(o?.text)??v(o?.input_text);if(r)return r}}}function Kt(i){const e=i.replace(/\s/g,"");if(!e||!/^[A-Za-z0-9_\-+/]*={0,2}$/.test(e))return;const t=e.replace(/=+$/,"").length;if(t%4!==1)return Math.floor(t*3/4)}function Zt(i,e){const t=C(i),s=C(e),n=C(t?.commands),o=C(t?.settings),r=Array.isArray(s?.results)?s.results:[],u=v(s?.encrypted_output);return{operations:Ft(n),response_length:v(n?.response_length),allowed_callers:st(o?.allowed_callers),external_web_access:typeof o?.external_web_access=="boolean"?o.external_web_access:void 0,prompt:Jt(t?.input),output:v(s?.output),results:r.flatMap(l=>{const d=Wt(l);return d?[d]:[]}),encrypted_output_bytes:u?Kt(u):void 0}}function Gt(i){if(typeof i!="string")return!1;try{return new URL(i,"http://localhost").pathname===Be}catch{return i.split("?",1)[0]===Be}}function je(i){if(i)try{const e=new URL(i);return e.protocol==="http:"||e.protocol==="https:"?e.href:void 0}catch{return}}function Yt(i){return i<1e3?`${i} B`:i<1e6?`${(i/1e3).toFixed(1)} KB`:`${(i/1e6).toFixed(1)} MB`}function Qt(i){return{search_query:"Query",open:"Open",click:"Click",find:"Find"}[i.kind]}function Xt(i){switch(i.kind){case"search_query":{const e=[];return i.domains.length>0&&e.push(`Domains: ${i.domains.join(", ")}`),i.recency_days!==void 0&&e.push(`Last ${i.recency_days} days`),e.join(" · ")||void 0}case"open":return i.line_number===void 0?void 0:`Starting at line ${i.line_number}`;case"click":return`Link ${i.link_id}`;case"find":return`Pattern: ${i.pattern}`}}class es extends w{static properties={request_url:{type:String},response_url:{type:String},load_state:{type:String,state:!0},request_payload:{attribute:!1,state:!0},response_payload:{attribute:!1,state:!0},error_message:{type:String,state:!0}};load_controller;constructor(){super(),this.request_url="",this.response_url="",this.load_state="idle"}createRenderRoot(){return this}disconnectedCallback(){this.load_controller?.abort(),super.disconnectedCallback()}updated(e){(e.has("request_url")||e.has("response_url"))&&this.load()}async load(){if(!this.request_url||!this.response_url)return;const e=this.request_url,t=this.response_url;this.load_controller?.abort();const s=new AbortController;this.load_controller=s,this.load_state="loading",this.error_message=void 0;try{const[n,o]=await Promise.all([$(e,s.signal),$(t,s.signal)]);if(this.load_controller!==s||this.request_url!==e||this.response_url!==t)return;if(n.field!=="inbound_req_body"||o.field!=="inbound_resp_body")throw new Error("Search payload response did not match the requested fields");this.request_payload=n.value,this.response_payload=o.value,this.load_state="ready"}catch(n){if(this.load_controller!==s||q(n))return;this.load_state="error",this.error_message=n instanceof Error?n.message:"Unable to load web search"}finally{this.load_controller===s&&(this.load_controller=void 0)}}render(){if(this.load_state==="loading"||this.load_state==="idle")return a`
        <section class="web-search-inspection web-search-state" aria-label="Web search" aria-live="polite">
          <span class="spinner" aria-hidden="true"></span>
          <span>Loading web search…</span>
        </section>
      `;if(this.load_state==="error")return a`
        <section class="web-search-inspection web-search-state error-state" aria-label="Web search" role="alert">
          <div><strong>Web search could not be loaded</strong><span>${this.error_message}</span></div>
          <button type="button" @click=${()=>{this.load()}}>Retry</button>
        </section>
      `;const e=Zt(this.request_payload,this.response_payload);return a`
      <section class="web-search-inspection" aria-label="Web search">
        <header class="web-search-heading">
          <div>
            <p class="eyebrow">Codex web search</p>
            <h3>${Vt(e.operations)}</h3>
          </div>
          <div class="web-search-metrics">
            <span><strong>${e.results.length}</strong> results</span>
            ${e.response_length?a`<span><strong>${e.response_length}</strong> response</span>`:c}
            ${e.encrypted_output_bytes!==void 0?a`<span title="Decoded encrypted payload size"><strong>${Yt(e.encrypted_output_bytes)}</strong> encrypted</span>`:c}
          </div>
        </header>

        <div class="web-search-operations">
          ${e.operations.length===0?a`<p class="web-search-empty">No supported web operation was persisted.</p>`:e.operations.map((t,s)=>{const n=Xt(t),o=t.kind==="open"?je(t.value):void 0;return a`
                  <article>
                    <span class="web-search-operation-index">${s+1}</span>
                    <div>
                      <span class="web-search-operation-kind">${Qt(t)}</span>
                      ${o?a`<a href=${o} target="_blank" rel="noopener noreferrer"><code>${t.value}</code></a>`:a`<code>${t.value}</code>`}
                      ${n?a`<p>${n}</p>`:c}
                    </div>
                  </article>
                `})}
        </div>

        <dl class="web-search-settings">
          <div><dt>Caller</dt><dd>${e.allowed_callers.join(", ")||"—"}</dd></div>
          <div><dt>External web access</dt><dd>${e.external_web_access===void 0?"—":String(e.external_web_access)}</dd></div>
        </dl>

        <div class="web-search-results">
          <h4>Results</h4>
          ${e.results.length===0?a`<p class="web-search-empty">No structured results were returned.</p>`:e.results.map((t,s)=>{const n=je(t.url);return a`
                  <article class="web-search-result">
                    <span class="web-search-result-index">${s+1}</span>
                    <div>
                      <div class="web-search-result-title">
                        ${n?a`<a href=${n} target="_blank" rel="noopener noreferrer">${t.title??t.url}</a>`:a`<strong>${t.title??t.url??"Untitled result"}</strong>`}
                        <span>${t.domain??""}</span>
                      </div>
                      ${t.snippet?a`<p>${t.snippet}</p>`:c}
                      ${t.ref_id?a`<code>${t.ref_id}</code>`:c}
                    </div>
                  </article>
                `})}
        </div>

        <div class="payload-stack web-search-payloads">
          ${e.output?a`<payload-panel label="Synthesized search output" .value=${e.output}></payload-panel>`:c}
          ${e.prompt?a`<payload-panel label="Prompt context sent to search" .value=${e.prompt}></payload-panel>`:c}
        </div>
      </section>
    `}}customElements.define("web-search-detail",es);const T=[{id:"overview",label:"Overview"},{id:"client",label:"Client"},{id:"provider",label:"Provider"},{id:"raw",label:"Raw"}];function j(i){return i==null||i===""?"—":typeof i=="boolean"?i?"Yes":"No":String(i)}function ts(i){if(i!==null&&typeof i=="object"&&!Array.isArray(i))return i;if(typeof i=="string")try{const e=JSON.parse(i);return e!==null&&typeof e=="object"&&!Array.isArray(e)?e:void 0}catch{return}}function Ie(i,e,t){return ts(i[e])?.[t]??i[t]}function S(i,e,t,s){return`/api/request-payload?${new URLSearchParams({day:i,request_id:e,row_id:t,field:s}).toString()}`}function He(i){return i===void 0?"neutral":i>=400?"error":i>=300?"warning":"success"}class ss extends w{static properties={detail:{attribute:!1},summary:{attribute:!1},state:{type:String},error_message:{type:String},active_tab:{type:String},timezone:{type:String}};createRenderRoot(){return this}openSession(e){this.dispatchEvent(new CustomEvent("open-session",{detail:e,bubbles:!0,composed:!0}))}retry(){this.dispatchEvent(new CustomEvent("detail-retry",{bubbles:!0,composed:!0}))}close(){this.dispatchEvent(new CustomEvent("detail-close",{bubbles:!0,composed:!0}))}selectTab(e){this.dispatchEvent(new CustomEvent("detail-tab-change",{detail:e,bubbles:!0,composed:!0}))}tabKeydown(e){const t=T.findIndex(r=>r.id===this.active_tab);let s;if(e.key==="ArrowRight"?s=(t+1)%T.length:e.key==="ArrowLeft"?s=(t-1+T.length)%T.length:e.key==="Home"?s=0:e.key==="End"&&(s=T.length-1),s===void 0)return;e.preventDefault();const n=T[s];this.selectTab(n.id),this.querySelectorAll("[role=tab]")[s]?.focus()}renderOverview(e){if(tt(e)&&this.detail)return a`
        <llm-request-overview
          .request=${e}
          .day=${this.detail.day}
          .request_id=${y(e,"request_id")??this.summary?.request_id??""}
          .row_id=${this.detail.row_id}
          .timezone=${this.timezone}
        ></llm-request-overview>
      `;const t=U(e,"ts"),s=Ie(e,"ctx_json","latency_ms"),n=Ie(e,"params_json","stream"),o=[["Timestamp",t===void 0?void 0:W(t,this.timezone)],["Storage day",this.detail?.day],["Endpoint",e.endpoint],["Model",e.model],["Provider",e.provider_id],["Account",e.account_id],["Latency",typeof s=="number"?`${s} ms`:s],["Streaming",n]],r=U(e,"inbound_resp_status"),u=U(e,"outbound_resp_status"),l=U(e,"status"),d=y(e,"request_id")??this.summary?.request_id,_=this.detail?.row_id,h=y(e,"inbound_req_url")??y(e,"endpoint"),p=this.detail&&d&&_&&Gt(h)?a`
          <web-search-detail
            .request_url=${S(this.detail.day,d,_,"inbound_req_body")}
            .response_url=${S(this.detail.day,d,_,"inbound_resp_body")}
          ></web-search-detail>
        `:c;return a`
      <section class="flow-grid" aria-label="Request flow">
        <div>
          <span>Client request</span>
          <strong>${y(e,"inbound_req_method")??"—"}</strong>
        </div>
        <span class="flow-arrow" aria-hidden="true">→</span>
        <div>
          <span>Provider response</span>
          <strong class="status-text ${He(u)}">${j(u)}</strong>
        </div>
        <span class="flow-arrow" aria-hidden="true">→</span>
        <div>
          <span>Client response</span>
          <strong class="status-text ${He(r??l)}">
            ${j(r??l)}
          </strong>
        </div>
      </section>
      <dl class="metadata-grid">
        ${o.map(([f,m])=>a`
            <div>
              <dt>${f}</dt>
              <dd title=${j(m)}>${j(m)}</dd>
            </div>
          `)}
      </dl>
      ${p}
      <div class="payload-stack">
        <payload-panel label="Usage" .value=${e.usage_json}></payload-panel>
      </div>
    `}renderRaw(e){return a`
      <p class="raw-note">Network headers and bodies remain lazy and are not included in this overview record.</p>
      <div class="payload-stack">
        <payload-panel label="Request parameters" .value=${e.params_json}></payload-panel>
        <payload-panel label="Request context" .value=${e.ctx_json}></payload-panel>
        <payload-panel
          label="Persisted overview record"
          .value=${e}
          .redact_record_headers=${!0}
        ></payload-panel>
      </div>
    `}renderClient(e,t,s,n){return a`
      <section class="payload-group">
        <div class="payload-group-heading">
          <div><span class="direction-label">Incoming</span><h3>Client request</h3></div>
          <span>${y(e,"inbound_req_method")??"—"} ${O(y(e,"inbound_req_url"))}</span>
        </div>
        <payload-panel
          label="Request headers"
          .value=${e.inbound_req_headers}
          .load_url=${S(t,s,n,"inbound_req_headers")}
          .is_headers=${!0}
        ></payload-panel>
        <payload-panel
          label="Request body"
          .value=${e.inbound_req_body}
          .load_url=${S(t,s,n,"inbound_req_body")}
        ></payload-panel>
      </section>
      <section class="payload-group">
        <div class="payload-group-heading">
          <div><span class="direction-label">Outgoing</span><h3>Client response</h3></div>
          <span>Status ${j(e.inbound_resp_status??e.status)}</span>
        </div>
        <payload-panel
          label="Response headers"
          .value=${e.inbound_resp_headers}
          .load_url=${S(t,s,n,"inbound_resp_headers")}
          .is_headers=${!0}
        ></payload-panel>
        <payload-panel
          label="Response body"
          .value=${e.inbound_resp_body}
          .load_url=${S(t,s,n,"inbound_resp_body")}
        ></payload-panel>
      </section>
    `}renderProvider(e,t,s,n){return a`
      <section class="payload-group">
        <div class="payload-group-heading">
          <div><span class="direction-label">Outgoing</span><h3>Provider request</h3></div>
          <span>${y(e,"outbound_req_method")??"—"} ${O(y(e,"outbound_req_url"))}</span>
        </div>
        <payload-panel
          label="Request headers"
          .value=${e.outbound_req_headers}
          .load_url=${S(t,s,n,"outbound_req_headers")}
          .is_headers=${!0}
        ></payload-panel>
        <payload-panel
          label="Request body"
          .value=${e.outbound_req_body}
          .load_url=${S(t,s,n,"outbound_req_body")}
        ></payload-panel>
      </section>
      <section class="payload-group">
        <div class="payload-group-heading">
          <div><span class="direction-label">Incoming</span><h3>Provider response</h3></div>
          <span>Status ${j(e.outbound_resp_status)}</span>
        </div>
        <payload-panel
          label="Response headers"
          .value=${e.outbound_resp_headers}
          .load_url=${S(t,s,n,"outbound_resp_headers")}
          .is_headers=${!0}
        ></payload-panel>
        <payload-panel
          label="Response body"
          .value=${e.outbound_resp_body}
          .load_url=${S(t,s,n,"outbound_resp_body")}
        ></payload-panel>
      </section>
    `}renderTab(e,t,s,n){switch(this.active_tab){case"client":return this.renderClient(e,t,s,n);case"provider":return this.renderProvider(e,t,s,n);case"raw":return this.renderRaw(e);default:return this.renderOverview(e)}}render(){if(!this.detail)return this.state==="loading"?a`
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
        `:a`<section class="detail-state"><p>Select a request to inspect its route, payloads, and responses.</p></section>`;const e=this.detail.request,t=y(e,"request_id")??this.summary?.request_id??"unknown id",s=y(e,"session_id")??this.summary?.session_id??void 0,n=y(e,"inbound_req_method")??this.summary?.inbound_req_method??"REQUEST",o=O(y(e,"inbound_req_url")??this.summary?.inbound_req_url??y(e,"endpoint"));return a`
      <section class="detail-content">
        <header class="detail-header">
          <button type="button" class="mobile-back-button" @click=${this.close}>← Requests</button>
          <div class="detail-title">
            <p class="eyebrow">request · ${R(t)}</p>
            <h2><span>${n}</span> ${o}</h2>
            <p class="muted" title=${t}>${t}</p>
          </div>
          <div class="detail-actions">
            ${s?a`<button type="button" class="secondary-button" @click=${()=>this.openSession(s)}>Open session</button>`:c}
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
        ${this.state==="loading"?a`<div class="inline-state" role="status"><span class="spinner" aria-hidden="true"></span>Refreshing detail…</div>`:c}
        ${this.state==="error"?a`
              <div class="inline-error" role="alert">
                <span>${this.error_message}</span>
                <button type="button" @click=${this.retry}>Retry</button>
              </div>
            `:c}
        ${e.request_error?a`<div class="request-error" role="alert">${String(e.request_error)}</div>`:c}
        <div class="detail-tabs" role="tablist" aria-label="Request detail sections" @keydown=${this.tabKeydown}>
          ${T.map(r=>a`
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
          ${this.renderTab(e,this.detail.day,t,this.detail.row_id)}
        </section>
      </section>
    `}}customElements.define("request-detail-view",ss);class is extends w{static properties={requests:{attribute:!1},selected_key:{type:String},timezone:{type:String}};createRenderRoot(){return this}selectRequest(e){this.dispatchEvent(new CustomEvent("request-select",{detail:e,bubbles:!0,composed:!0}))}render(){const e=this.requests??[];return e.length===0?a`<p class="empty">No persisted requests match these filters.</p>`:a`
      <ul class="request-list" aria-label="Requests">
        ${e.map(t=>{const s=Lt(t),n=this.selected_key===V(t),o=t.inbound_req_method??"REQUEST",r=Et(t);return a`
            <li>
              <button
                type="button"
                class="request-row ${n?"selected":""}"
                data-request-key=${V(t)}
                aria-current=${n?"true":"false"}
                @click=${()=>this.selectRequest(t)}
              >
                <span class="request-row-time">${W(t.ts,this.timezone,!0)}</span>
                <span class="status ${s.tone}" title=${s.title}>${s.label}</span>
                <span class="request-row-main">
                  <span class="request-route"><strong>${o}</strong><span>${r}</span></span>
                  <span class="request-context">
                    <span>${t.model??"unknown model"}</span>
                    <span aria-hidden="true">·</span>
                    <span>${t.provider_id??"unknown provider"}</span>
                  </span>
                  <span class="request-identifiers">
                    <span title=${t.request_id}>req ${R(t.request_id)}</span>
                    ${t.session_id?a`<span title=${t.session_id}>session ${R(t.session_id)}</span>`:a`<span>no session</span>`}
                  </span>
                </span>
              </button>
            </li>
          `})}
      </ul>
    `}}customElements.define("request-list",is);function ns(i,e){const t=new Set,s=new Set;for(const n of i){if(s.has(n.node_id))continue;const o=[],r=new Map;let u=n;for(;u&&!s.has(u.node_id);){const l=r.get(u.node_id);if(l!==void 0){for(const d of o.slice(l))t.add(d);break}r.set(u.node_id,o.length),o.push(u.node_id),u=u.parent_node_id?e.get(u.parent_node_id):void 0}for(const l of o)s.add(l)}return t}function os(i,e,t){const s=Number(t.has(e.node_id))-Number(t.has(i.node_id));return s!==0?s:i.ts!==e.ts?e.ts-i.ts:i.node_id.localeCompare(e.node_id)}function rs(i,e,t){const s=[...i].filter(r=>r.is_head).sort((r,u)=>u.ts-r.ts||r.node_id.localeCompare(u.node_id))[0],n=new Set;let o=s;for(;o;){if(n.has(o.node_id)){t.add(o.node_id);break}n.add(o.node_id),o=o.parent_node_id?e.get(o.parent_node_id):void 0}return n}function Fe(i,e,t,s,n){const o=[{node:i,next_child:0}];for(;o.length>0;){const r=o[o.length-1],u=t.get(r.node.node_id);if(u==="done"){o.pop();continue}u===void 0&&t.set(r.node.node_id,"visiting");const l=e.get(r.node.node_id)??[];if(r.next_child<l.length){const d=l[r.next_child];r.next_child+=1;const _=t.get(d.node_id);_===void 0?o.push({node:d,next_child:0}):_==="visiting"&&(s.add(r.node.node_id),s.add(d.node_id));continue}t.set(r.node.node_id,"done"),n.push(r.node),o.pop()}}function as(i,e,t,s,n){const o=(d,_)=>os(d,_,s);for(const d of t.values())d.sort(o);const r=i.filter(d=>d.parent_node_id===null||!e.has(d.parent_node_id)||n.has(d.node_id)).sort(o),u=new Map,l=[];for(const d of r)Fe(d,t,u,n,l);for(const d of[...i].sort(o))u.has(d.node_id)||(n.add(d.node_id),Fe(d,t,u,n,l));return l}function ds(i,e,t,s,n){const o=[],r=[],u=new Set;let l=0;for(const d of i){let _=r.indexOf(d.node_id);const h=_===-1;h&&(_=r.length,r.push(d.node_id));const p=[...r],f=[];let m;const g=d.parent_node_id,A=g&&n.has(d.node_id)&&n.has(g)?null:g;if(A&&!u.has(A)){const b=r.findIndex((ne,nt)=>nt!==_&&ne===A);b===-1?(r[_]=A,m=_):(r.splice(_,1),m=b-+(_<b))}else A&&u.has(A)&&(n.add(d.node_id),n.add(A)),r.splice(_,1);const ie=[...r];for(let b=0;b<p.length;b+=1){if(b===_)continue;const ne=ie.indexOf(p[b]);ne!==-1&&f.push({from_lane:b,to_lane:ne,kind:"continuation",active:t.has(p[b])})}m!==void 0&&f.push({from_lane:_,to_lane:m,kind:"parent",active:t.has(d.node_id)}),l=Math.max(l,p.length,ie.length),o.push({node:d,top_lanes:p,bottom_lanes:ie,node_lane:_,starts_here:h,connections:f,bottom_lane_is_active:ie.map(b=>t.has(b)),child_count:e.get(d.node_id)?.length??0,parent_is_missing:!!(A&&s.has(A)),is_on_head_path:t.has(d.node_id),has_topology_warning:n.has(d.node_id)}),u.add(d.node_id)}return{rows:o,max_lane_count:l,remaining_lanes:[...r]}}function Ve(i){const e=new Map;for(const d of i)e.has(d.node_id)||e.set(d.node_id,d);const t=[...e.values()],s=new Map(t.map(d=>[d.node_id,[]])),n=new Set,o=ns(t,e);for(const d of t){const _=d.parent_node_id;_&&(e.has(_)&&!(o.has(d.node_id)&&o.has(_))?s.get(_)?.push(d):e.has(_)||n.add(_))}const r=rs(t,e,o),u=as(t,e,s,r,o),l=ds(u,s,r,n,o);for(const d of l.rows)d.has_topology_warning=o.has(d.node.node_id);return{...l,missing_parent_ids:[...n].sort(),remaining_lanes:l.remaining_lanes.filter(d=>n.has(d)),cycle_node_ids:[...o].sort()}}const it=6,de=16,fe=25;function ls(i){return i===null?{label:"—",tone:"neutral",title:"No response status stored"}:i>=400?{label:String(i),tone:"error",title:`Response status: ${i}`}:i>=300?{label:String(i),tone:"warning",title:`Response status: ${i}`}:{label:String(i),tone:"success",title:`Response status: ${i}`}}function cs(i){switch(i.toLowerCase()){case"assistant":return"assistant";case"system":case"developer":return"system";case"tool":case"function":return"tool";case"compaction":return"compaction";default:return"user"}}function us(i){try{return JSON.stringify(i,null,2)??String(i)}catch{return String(i)}}function I(i){if(i<1024)return`${i.toLocaleString()} B`;const e=["KiB","MiB","GiB"];let t=i/1024,s=e[0];for(const n of e.slice(1)){if(t<1024)break;t/=1024,s=n}return`${t>=10?t.toFixed(0):t.toFixed(1)} ${s}`}function H(i){return i===null?"—":i.toLocaleString()}function me(i){return i===null?"—":new Intl.NumberFormat(void 0,{notation:"compact",maximumFractionDigits:i>=1e4?1:0}).format(i)}function _s(i){switch(i){case"message_tree":return{direction:"New",title:"Input delta",empty_message:"No new semantic input was stored for this observation."};case"suffix_append":return{direction:"Appended",title:"Input delta",empty_message:"No new semantic input was stored for this node."};case"root_snapshot":return{direction:"Initial",title:"Input snapshot",empty_message:"No semantic input was stored for this root snapshot."};case"conflict_snapshot":return{direction:"Replaced",title:"Replacement snapshot",empty_message:"No semantic input was stored for this replacement snapshot."};default:return{direction:"Stored",title:"Node input",empty_message:"No semantic input was stored for this node."}}}function x(i){return(i+.5)*de}function We(i){return`session-tree-lanes-${Math.min(i,it)}`}class hs extends w{static properties={sessions:{attribute:!1},selected_session_id:{type:String},timezone:{type:String}};createRenderRoot(){return this}selectSession(e){this.dispatchEvent(new CustomEvent("session-select",{detail:e,bubbles:!0,composed:!0}))}render(){const e=this.sessions??[];return a`
      <ul class="session-list" aria-label="Sessions">
        ${e.map(t=>{const s=this.selected_session_id===t.session_id,n=Ut(t);return a`
            <li>
              <button
                type="button"
                class="session-row ${s?"selected":""}"
                data-session-id=${t.session_id}
                aria-current=${s?"true":"false"}
                @click=${()=>this.selectSession(t)}
              >
                <time datetime=${new Date(t.last_ts).toISOString()}>
                  ${xt(t.last_ts,this.timezone)}
                </time>
                <span class="status ${n.tone}" title=${n.title}>${n.label}</span>
                <span class="session-row-main">
                  <span class="session-row-title">
                    <strong>${t.model??"Unknown model"}</strong>
                    <span>${t.endpoint??"unknown endpoint"}</span>
                  </span>
                  <span class="session-row-context">
                    <span>${t.provider_id??"unknown provider"}</span>
                    <span aria-hidden="true">·</span>
                    <span>${t.request_count.toLocaleString()} ${t.request_count===1?"node":"nodes"}</span>
                  </span>
                  <span class="session-row-id" title=${t.session_id}>
                    session ${R(t.session_id)}
                  </span>
                </span>
                <span class="session-row-chevron" aria-hidden="true">›</span>
              </button>
            </li>
          `})}
      </ul>
    `}}class ps extends w{static properties={detail:{attribute:!1},node_detail:{attribute:!1},state:{type:String},error_message:{type:String},node_state:{type:String},node_error_message:{type:String},selected_node_id:{type:String},usage:{attribute:!1},usage_state:{type:String},usage_error_message:{type:String},timezone:{type:String}};createRenderRoot(){return this}close(){this.dispatchEvent(new CustomEvent("session-close",{bubbles:!0,composed:!0}))}retryDetail(){this.dispatchEvent(new CustomEvent("session-retry",{bubbles:!0,composed:!0}))}retryNode(){this.dispatchEvent(new CustomEvent("session-node-retry",{bubbles:!0,composed:!0}))}retryUsage(){this.dispatchEvent(new CustomEvent("session-usage-retry",{bubbles:!0,composed:!0}))}selectNode(e){this.dispatchEvent(new CustomEvent("session-node-select",{detail:e,bubbles:!0,composed:!0}))}openRequest(e){this.dispatchEvent(new CustomEvent("open-request",{detail:e,bubbles:!0,composed:!0}))}renderPart(e){switch(e.content.encoding){case"text":{const t=e.content.value||a`<span class="faint">Empty text part</span>`,s=e.content.truncated?a`<p class="session-part-note">Preview truncated · ${I(e.byte_length)} stored</p>`:c;return a`<div class="session-part-text">${t}${s}</div>`}case"json":return a`
          <details class="session-structured-part">
            <summary>${e.part_type.replaceAll("_"," ")}</summary>
            <pre>${us(e.content.value)}</pre>
          </details>
        `;case"encrypted":return a`
          <details class="session-structured-part">
            <summary>${e.part_type.replaceAll("_"," ")} · encrypted</summary>
            <p>
              ${I(e.content.byte_length)} encrypted payload stored. Plaintext is unavailable and the
              encrypted content is not returned to the viewer.
            </p>
          </details>
        `;case"binary":return a`
          <details class="session-structured-part">
            <summary>${e.part_type.replaceAll("_"," ")} · binary</summary>
            <p>${I(e.content.byte_length)} stored. Binary bytes are not returned to the viewer.</p>
          </details>
        `;case"omitted":return a`
          <details class="session-structured-part">
            <summary>${e.part_type.replaceAll("_"," ")} · omitted</summary>
            <p>
              ${I(e.byte_length)} ${e.content.original_encoding} content omitted after reaching the
              ${e.content.reason==="part_limit"?"per-part byte preview":"node content-size"} limit.
            </p>
          </details>
        `}}renderMessages(e,t){return e.length===0?a`<p class="session-message-empty">${t}</p>`:a`
      <div class="session-message-stack">
        ${e.map(s=>a`
          <article class="session-message ${cs(s.role)}">
            <header>
              <span>${s.role}</span>
              <span>
                ${s.parts.length.toLocaleString()}${s.parts.length===s.parts_total?"":` of ${s.parts_total.toLocaleString()}`} parts
                ${s.status===null?c:a` · status ${s.status}`}
              </span>
            </header>
            <div class="session-message-parts">
              ${s.parts.length>0?s.parts.map(n=>this.renderPart(n)):s.parts_total>0?a`
                      <p class="session-message-empty">
                        ${s.parts_total.toLocaleString()} stored parts were omitted from this bounded preview.
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
          <div><dt>Input</dt><dd>${H(e.input_tokens)}</dd></div>
          <div><dt>Output</dt><dd>${H(e.output_tokens)}</dd></div>
          <div><dt>Total</dt><dd>${H(e.total_tokens)}</dd></div>
          <div><dt>Cache read</dt><dd>${H(e.cache_read_tokens)}</dd></div>
          <div><dt>Cache write</dt><dd>${H(e.cache_write_tokens)}</dd></div>
          <div><dt>Reasoning</dt><dd>${H(e.reasoning_tokens)}</dd></div>
        </dl>
      </section>
    `}nodeDomId(e,t){return`session-node-${e}-${encodeURIComponent(t)}`}renderNodeGraph(e,t){const s=t*de,n=x(e.node_lane),o=`M ${n} ${fe} l 0 0.001`,r=e.connections.map(l=>{const d=x(l.from_lane),_=x(l.to_lane),h=l.kind==="parent"?fe:0;return a`
        <path
          class="session-tree-edge ${l.kind} ${l.active?"active":""}"
          d=${`M ${d} ${h} L ${_} 100`}
        ></path>
      `}),u=["session-tree-dot",e.node.is_head?"head":"",e.child_count>1?"branch":"",e.has_topology_warning?"warning":""].filter(Boolean).join(" ");return a`
      <svg
        viewBox=${`0 0 ${s} 100`}
        preserveAspectRatio="none"
        focusable="false"
        aria-hidden="true"
      >
        ${e.starts_here?c:a`
              <path
                class="session-tree-edge incoming ${e.is_on_head_path?"active":""}"
                d=${`M ${n} 0 L ${n} ${fe}`}
              ></path>
            `}
        ${r}
        <path class="${u} outline" d=${o}></path>
        <path class="${u} fill" d=${o}></path>
      </svg>
    `}renderNodeGraphContinuation(e,t){const s=t*de;return a`
      <svg
        viewBox=${`0 0 ${s} 100`}
        preserveAspectRatio="none"
        focusable="false"
        aria-hidden="true"
      >
        ${e.bottom_lanes.map((n,o)=>a`
          <path
            class="session-tree-edge continuation ${e.bottom_lane_is_active[o]?"active":""}"
            d=${`M ${x(o)} 0 L ${x(o)} 100`}
          ></path>
        `)}
      </svg>
    `}renderTreeBoundary(e,t,s,n,o){if(e.missing_parent_ids.length===0)return c;const r=t*de,u=e.remaining_lanes.length>0?e.remaining_lanes.map((p,f)=>f):e.missing_parent_ids.map((p,f)=>f),l=[...new Set(u)],d=o?"Connects to loaded tree":s?"Earlier ancestry omitted":"Parent nodes unavailable",_=o?`Parent ${R(o.node_id)} appears in the session tree below.`:s?`${n.toLocaleString()} ${n===1?"node falls":"nodes fall"} outside this bounded tree snapshot.`:"The stored parent links point outside the returned session tree.",h=o?"Parent link resolved in the loaded snapshot":`${e.missing_parent_ids.length.toLocaleString()} parent ${e.missing_parent_ids.length===1?"link":"links"} outside the snapshot`;return a`
      <li class="session-tree-boundary ${o?"loaded-parent":""} ${We(t)}">
        <span class="session-tree-boundary-graph" aria-hidden="true">
          <svg viewBox=${`0 0 ${r} 100`} preserveAspectRatio="none" focusable="false">
            ${l.map(p=>a`
              <path class="session-tree-edge boundary" d=${`M ${x(p)} 0 L ${x(p)} 48`}></path>
              <path
                class="session-tree-boundary-dot outline"
                d=${`M ${x(p)} 52 l 0 0.001`}
              ></path>
              <path
                class="session-tree-boundary-dot fill"
                d=${`M ${x(p)} 52 l 0 0.001`}
              ></path>
            `)}
          </svg>
        </span>
        <div class="session-tree-boundary-card" role="note">
          <strong>${d}</strong>
          <span>${_}</span>
          <span title=${o?.node_id??e.missing_parent_ids.join(", ")}>${h}</span>
        </div>
      </li>
    `}renderLoadedNodeContent(e){const t=e.truncation,s=_s(e.node.reduction_kind),n=t.request_messages.messages_total-t.request_messages.messages_returned,o=t.response_messages.messages_total-t.response_messages.messages_returned,r=n>0||o>0||t.parts_omitted>0||t.content_parts_truncated>0||t.binary_parts_elided>0;return a`
      <div class="session-node-content-actions">
        <span title=${e.node.request_id}>Request ${R(e.node.request_id)}</span>
        <button type="button" class="secondary-button" @click=${()=>this.openRequest(e.node)}>Open request</button>
      </div>
      ${r?a`
            <div class="session-content-boundary" role="status">
              <strong>Bounded content preview</strong>
              <span>
                ${I(t.content_bytes_returned)} of
                ${I(t.content_bytes_total)} inline content returned
                ${n+o>0?` · ${(n+o).toLocaleString()} messages omitted`:""}
                ${t.parts_omitted>0?` · ${t.parts_omitted.toLocaleString()} parts omitted`:""}
                ${t.content_parts_truncated>0?` · ${t.content_parts_truncated.toLocaleString()} parts truncated`:""}
                ${t.binary_parts_elided>0?` · ${t.binary_parts_elided.toLocaleString()} binary parts represented as metadata`:""}
              </span>
            </div>
          `:c}
      <div class="session-conversation-section">
        <header>
          <div>
            <span class="direction-label">${s.direction}</span>
            <h3>${s.title}</h3>
          </div>
          <span>
            ${t.request_messages.messages_returned.toLocaleString()}
            ${t.request_messages.messages_returned===t.request_messages.messages_total?"":`of ${t.request_messages.messages_total.toLocaleString()}`} messages
          </span>
        </header>
        ${this.renderMessages(e.request_messages,s.empty_message)}
      </div>
      <div class="session-conversation-section">
        <header>
          <div>
            <span class="direction-label">Captured</span>
            <h3>Model output</h3>
          </div>
          <span>
            ${t.response_messages.messages_returned.toLocaleString()}
            ${t.response_messages.messages_returned===t.response_messages.messages_total?"":`of ${t.response_messages.messages_total.toLocaleString()}`} messages
          </span>
        </header>
        ${this.renderMessages(e.response_messages,"No semantic output was stored for this node.")}
      </div>
    `}renderNodeContent(e){if(this.selected_node_id!==e.node_id)return c;const t=this.node_detail?.node.node_id===e.node_id?this.node_detail:void 0,s=this.node_state==="loading"?a`<div class="inline-state"><span class="spinner" aria-hidden="true"></span>Loading semantic content…</div>`:this.node_state==="error"?a`
            <div class="inline-error" role="alert">
              <span>${this.node_error_message}</span>
              <button type="button" @click=${this.retryNode}>Retry</button>
            </div>
          `:t?this.renderLoadedNodeContent(t):c;return a`
      <section
        id=${this.nodeDomId("content",e.node_id)}
        class="session-node-content"
        aria-labelledby=${this.nodeDomId("trigger",e.node_id)}
        aria-live="polite"
        aria-busy=${String(this.node_state==="loading")}
      >
        ${s}
      </section>
    `}renderNodeUsage(e){if(this.usage_state==="loading")return a`<span class="session-node-token-usage muted">Token usage loading…</span>`;if(this.usage_state==="error")return a`<span class="session-node-token-usage muted">Token usage unavailable</span>`;if(!e)return a`<span class="session-node-token-usage muted">No token usage</span>`;const t=e.context_tokens===null?"Context tokens unavailable":`${e.context_tokens.toLocaleString()} context tokens`,s=e.input_delta_tokens===null?"Input delta tokens unavailable":`${e.input_delta_tokens.toLocaleString()} uncached input tokens`,n=e.output_tokens===null?"Output tokens unavailable":`${e.output_tokens.toLocaleString()} output tokens`;return a`
      <span class="session-node-token-usage">
        <span class="session-node-token-label">tokens</span>
        <span class="session-node-token-separator" aria-hidden="true">·</span>
        <span title=${t}>${me(e.context_tokens)} context</span>
        <span class="session-node-token-separator" aria-hidden="true">·</span>
        <span title=${s}>
          ${e.input_delta_tokens===null?"—":`+${me(e.input_delta_tokens)}`} input delta
        </span>
        <span class="session-node-token-separator" aria-hidden="true">·</span>
        <span title=${n}>${me(e.output_tokens)} output</span>
      </span>
    `}renderNode(e,t,s,n){const o=e.node,r=o.node_id===this.selected_node_id,u=ls(o.status),l=!!(n&&o.parent_node_id===n.node_id),d=e.parent_is_missing&&!l,_=["session-node",We(t),r?"selected":"",e.is_on_head_path?"head-path":"",d?"boundary-child":"",e.has_topology_warning?"topology-warning":""].filter(Boolean).join(" "),h=o.reduction_kind==="message_tree"?o.input_message_count:o.request_message_count,p=o.reduction_kind==="message_tree"?"input":"input delta",f=o.reduction_kind==="message_tree"?a` (+${o.request_message_count.toLocaleString()} new)`:c,m=o.reduction_kind==="message_tree"?o.output_message_count:o.response_message_count,g=o.reduction_kind==="message_tree"?o.parent_node_id?`Prefix-derived child of ${o.parent_node_id}.`:"Prefix-derived root node.":o.parent_node_id?`Recorded child of ${o.parent_node_id}.`:"Recorded root node.";return a`
      <li class=${_}>
        <span class="session-node-graph" aria-hidden="true">
          ${this.renderNodeGraph(e,t)}
        </span>
        <button
          id=${this.nodeDomId("trigger",o.node_id)}
          type="button"
          class="session-node-trigger"
          data-node-id=${o.node_id}
          aria-expanded=${String(r)}
          aria-controls=${r?this.nodeDomId("content",o.node_id):c}
          aria-current=${o.is_head?"true":c}
          @click=${()=>this.selectNode(o)}
        >
          <span class="session-node-primary">
            <time datetime=${new Date(o.ts).toISOString()}>${W(o.ts,this.timezone)}</time>
            <span class="status ${u.tone}" title=${u.title}>${u.label}</span>
            ${e.child_count>1?a`<span class="branch-badge">${e.child_count.toLocaleString()} branches</span>`:c}
            ${o.is_head?a`<span class="head-badge">Current head</span>`:c}
          </span>
          <span class="session-node-title">
            <strong>${o.model??"Unknown model"}</strong>
            <span>${o.endpoint}</span>
          </span>
          <span class="session-node-context">
            <span>${o.provider_id??"unknown provider"}</span>
            <span aria-hidden="true">·</span>
            <span>${h.toLocaleString()} ${p}${f}</span>
            <span aria-hidden="true">·</span>
            <span>${m.toLocaleString()} output</span>
          </span>
          ${this.renderNodeUsage(s.get(o.request_id))}
          <span class="session-node-id" title=${o.request_id}>
            request ${R(o.request_id)} · ${o.parent_node_id?`parent ${R(o.parent_node_id)}`:"root"}
            ${d?" · outside snapshot":""}
          </span>
          <span class="visually-hidden">
            ${g}
            ${d?" Parent is outside this bounded snapshot.":""}
            ${l?" Parent appears in the loaded session tree.":""}
            ${e.has_topology_warning?" Parent links contain a topology warning.":""}
          </span>
        </button>
        ${r?a`
              <span class="session-node-content-graph" aria-hidden="true">
                ${this.renderNodeGraphContinuation(e,t)}
              </span>
            `:c}
        ${this.renderNodeContent(o)}
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
      `;const{session:e,nodes:t}=this.detail,s=new Map((this.usage?.requests??[]).map(g=>[g.request_id,g])),n=Ve(t),o=Math.max(1,n.max_lane_count),r=Math.max(0,e.request_count-t.length),u=n.missing_parent_ids.length>0,l=!!(this.selected_node_id&&t.some(g=>g.node_id===this.selected_node_id)),d=this.node_detail,_=!l&&d&&d.node.node_id===this.selected_node_id?d.node:void 0,h=_?Ve([_]):void 0,p=h?Math.max(1,h.max_lane_count):1,f=_?.parent_node_id?t.find(g=>g.node_id===_.parent_node_id):void 0,m=e.model??"Unknown model";return a`
      <section class="detail-content session-detail-content">
        <header class="detail-header session-detail-header">
          <button type="button" class="mobile-back-button" @click=${this.close}>← Sessions</button>
          <div class="detail-title">
            <p class="eyebrow">session · ${R(e.session_id)}</p>
            <h2>${m}<span> on ${e.provider_id??"unknown provider"}</span></h2>
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
        ${this.state==="loading"?a`<div class="inline-state"><span class="spinner" aria-hidden="true"></span>Refreshing session…</div>`:c}
        ${this.state==="error"?a`
              <div class="inline-error" role="alert">
                <span>${this.error_message}</span>
                <button type="button" @click=${this.retryDetail}>Retry</button>
              </div>
            `:c}
        <dl class="session-metadata-grid">
          <div><dt>Semantic nodes</dt><dd>${e.request_count.toLocaleString()}</dd></div>
          <div><dt>Duration</dt><dd>${Ct(e.first_ts,e.last_ts)}</dd></div>
          <div><dt>First seen</dt><dd>${W(e.first_ts,this.timezone)}</dd></div>
          <div><dt>Last active</dt><dd>${W(e.last_ts,this.timezone)}</dd></div>
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
              ${t.length.toLocaleString()} loaded · head branch first${this.detail.nodes_truncated?" · bounded":""}
              ${n.max_lane_count>it?" · compressed lanes":""}
            </span>
          </header>
          ${this.detail.nodes_truncated?a`
                <p class="session-truncation-note">
                  ${r.toLocaleString()} older nodes are omitted.
                  ${u?" Amber graph endpoints continue into the omitted ancestry.":" The graph shows every parent link available in this snapshot."}
                </p>
              `:c}
          ${n.cycle_node_ids.length>0?a`
                <p class="session-topology-warning" role="alert">
                  ${n.cycle_node_ids.length.toLocaleString()} nodes contain cyclic parent links; their graph
                  edges were detached defensively.
                </p>
              `:c}
          ${t.length>0?a`
                <p class="session-tree-direction">
                  <span>Leaves and current-head branch</span>
                  <span aria-hidden="true">↓</span>
                  <span>recorded parents</span>
                </p>
              `:c}
          ${this.selected_node_id?c:a`<p class="session-content-hint">Open a node to load its conversation content from <code>sessions.db</code>.</p>`}
          ${this.selected_node_id&&!l?a`
                <section class="session-linked-node" aria-label="Directly linked session node">
                  <header>
                    <div>
                      <p class="eyebrow">Direct link</p>
                      <h4>Node outside this activity snapshot</h4>
                    </div>
                    <span>${R(this.selected_node_id)}</span>
                  </header>
                  ${h?a`
                        <ol class="session-node-list linked-node-list">
                          ${h.rows.map(g=>this.renderNode(g,p,s,f))}
                          ${this.renderTreeBoundary(h,p,!1,0,f)}
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
                          `:c}
                </section>
              `:c}
          ${t.length>0?a`
                <ol class="session-node-list">
                  ${n.rows.map(g=>this.renderNode(g,o,s))}
                  ${this.renderTreeBoundary(n,o,this.detail.nodes_truncated,r)}
                </ol>
              `:a`<p class="empty">This migrated session has no semantic nodes.</p>`}
        </section>
      </section>
    `}}customElements.define("session-list",hs);customElements.define("session-detail-view",ps);const Je=100;function k(i,e){return i instanceof Error?i.message:e}function ys(i){return i==="overview"||i==="client"||i==="provider"||i==="raw"}function oe(){return{query:"",provider_id:"",url_path:"",status:"",errors_only:!1}}function fs(i){return new Date(i).toISOString().slice(0,10)}class ms extends w{static properties={active_view:{type:String},info:{attribute:!1},requests:{attribute:!1},request_days:{attribute:!1},selected_day:{type:String},selected_request:{attribute:!1},selected_request_id:{type:String},selected_request_row_id:{type:String},selected_request_detail:{attribute:!1},request_list_state:{type:String},request_list_error:{type:String},request_detail_state:{type:String},request_detail_error:{type:String},next_cursor:{type:String},loading_more:{type:Boolean},load_more_error:{type:String},search_query:{type:String},provider_id:{type:String},url_path:{type:String},request_url_paths:{attribute:!1},request_url_paths_loading:{type:Boolean},request_url_paths_error:{type:String},status_filter:{type:String},errors_only:{type:Boolean},applied_filters:{attribute:!1},active_detail_tab:{type:String},timezone:{type:String},request_days_loading:{type:Boolean},request_days_error:{type:String},sessions:{attribute:!1},selected_session:{attribute:!1},selected_session_detail:{attribute:!1},selected_session_usage:{attribute:!1},sessions_loading:{type:Boolean},sessions_error:{type:String},session_search_query:{type:String},session_detail_state:{type:String},session_detail_error:{type:String},session_usage_state:{type:String},session_usage_error:{type:String},selected_session_node_id:{type:String},selected_session_node_detail:{attribute:!1},session_node_state:{type:String},session_node_error:{type:String}};request_load_id=0;request_detail_load_id=0;session_detail_load_id=0;session_usage_load_id=0;session_node_load_id=0;session_list_load_id=0;request_days_load_id=0;request_url_paths_load_id=0;sessions_loaded=!1;requested_request_id;requested_request_row_id;requested_session_id;requested_session_node_id;request_rows_context;request_controller;request_url_paths_controller;request_detail_controller;session_list_controller;session_list_load;session_detail_controller;session_usage_controller;session_node_controller;navigation_workflow_id=0;popstate_handler=()=>{this.restoreFromHistory()};constructor(){super(),this.active_view="requests",this.requests=[],this.request_days=[],this.sessions=[],this.request_list_state="idle",this.request_detail_state="idle",this.search_query="",this.provider_id="",this.url_path="",this.request_url_paths=[],this.request_url_paths_loading=!1,this.status_filter="",this.errors_only=!1,this.applied_filters=oe(),this.active_detail_tab="overview",this.timezone="local",this.loading_more=!1,this.request_days_loading=!1,this.sessions_loading=!1,this.session_search_query="",this.session_detail_state="idle",this.session_usage_state="idle",this.session_node_state="idle"}createRenderRoot(){return this}connectedCallback(){super.connectedCallback(),this.restoreUrlState(),window.addEventListener("popstate",this.popstate_handler),this.loadInitialData()}disconnectedCallback(){window.removeEventListener("popstate",this.popstate_handler),this.request_controller?.abort(),this.request_url_paths_controller?.abort(),this.request_detail_controller?.abort(),this.session_list_controller?.abort(),this.session_detail_controller?.abort(),this.session_usage_controller?.abort(),this.session_node_controller?.abort(),super.disconnectedCallback()}restoreUrlState(){const e=new URLSearchParams(window.location.search);this.active_view=e.get("view")==="sessions"?"sessions":"requests";const t=e.get("day");this.selected_day=t&&/^\d{4}-\d{2}-\d{2}$/.test(t)?t:void 0,this.search_query=e.get("query")??"",this.provider_id=e.get("provider_id")??"",this.url_path=e.get("url_path")??"";const s=e.get("status")??"";this.status_filter=/^\d{3}$/.test(s)?s:"",this.errors_only=e.get("errors_only")==="true"||e.get("errors_only")==="1",this.applied_filters={query:this.search_query,provider_id:this.provider_id,url_path:this.url_path,status:this.status_filter,errors_only:this.errors_only},this.requested_request_id=e.get("request_id")??void 0;const n=e.get("row_id");this.requested_request_row_id=n&&/^-?\d+$/.test(n)?n:void 0;const o=e.get("tab");this.active_detail_tab=ys(o)?o:"overview",this.requested_session_id=e.has("session_id")?e.get("session_id")??"":void 0,this.requested_session_node_id=e.get("node_id")??void 0,this.timezone=e.get("timezone")==="utc"?"utc":"local"}selectedRequestDay(){return this.selected_request_detail?.day??this.selected_request?.day??this.selected_day}syncUrl(e="replace"){const t=new URLSearchParams;if(this.active_view==="sessions"){t.set("view","sessions");const o=this.selected_session?.session_id??this.requested_session_id;o!==void 0&&t.set("session_id",o),this.selected_session_node_id&&t.set("node_id",this.selected_session_node_id)}else{const o=this.selected_request_id?this.selectedRequestDay():this.selected_day;o&&t.set("day",o),this.applied_filters.query&&t.set("query",this.applied_filters.query),this.applied_filters.provider_id&&t.set("provider_id",this.applied_filters.provider_id),this.applied_filters.url_path&&t.set("url_path",this.applied_filters.url_path),this.applied_filters.status&&t.set("status",this.applied_filters.status),this.applied_filters.errors_only&&t.set("errors_only","true"),this.selected_request_id&&(t.set("request_id",this.selected_request_id),this.selected_request_row_id&&t.set("row_id",this.selected_request_row_id),t.set("tab",this.active_detail_tab))}t.set("timezone",this.timezone);const s=t.toString(),n=`${window.location.pathname}${s?`?${s}`:""}`;`${window.location.pathname}${window.location.search}`!==n&&(e==="push"?window.history.pushState(null,"",n):window.history.replaceState(null,"",n))}async loadInitialData(){const e=++this.navigation_workflow_id;this.loadInfo(),await this.loadUrlState(e)}async restoreFromHistory(){const e=++this.navigation_workflow_id;this.request_controller?.abort(),this.request_detail_controller?.abort(),this.session_detail_controller?.abort(),this.session_node_controller?.abort(),this.resetRequestSelection(),this.resetSessionSelection(),this.restoreUrlState(),this.active_view==="requests"&&(this.requests=[],this.next_cursor=void 0,this.request_rows_context=void 0),await this.loadUrlState(e)}async loadUrlState(e){const t=this.requested_request_id,s=this.requested_request_row_id;if(this.active_view==="sessions"){const o=this.requested_session_id,r=this.requested_session_node_id;if(!await this.ensureSessionsLoaded()||e!==this.navigation_workflow_id||o===void 0)return;await this.loadSession(o,this.sessions.find(l=>l.session_id===o),!1,null,r);return}this.loadRequestDays();let n;if(this.selected_day?(this.loadRequestUrlPaths(this.selected_day),n=await this.loadRequests()):(n=await this.loadLatestRequests(),n&&this.selected_day&&this.loadRequestUrlPaths(this.selected_day),n&&this.selected_day&&this.hasAppliedFilters()&&(n=await this.loadRequests())),!(!n||e!==this.navigation_workflow_id)&&t&&this.selected_day){const o=this.requests.find(r=>r.request_id===t&&(!s||r.row_id===s));await this.loadRequestDetail(this.selected_day,t,s??o?.row_id,o,!1,null)}}async loadInfo(){try{this.info=await $("/api/info")}catch{this.info=void 0}}async loadLatestRequests(){this.request_controller?.abort();const e=new AbortController;this.request_controller=e;const t=++this.request_load_id;this.requests=[],this.next_cursor=void 0,this.request_rows_context=void 0,this.request_list_state="loading",this.request_list_error=void 0;try{const s=await $(`/api/requests/latest?limit=${Je}`,e.signal);return t!==this.request_load_id||this.request_controller!==e?!1:(this.selected_day=s.day??void 0,this.requests=s.requests,this.next_cursor=s.next_cursor??void 0,this.request_rows_context=this.requestContext(this.selected_day,oe()),this.request_list_state="ready",this.syncUrl(),!0)}catch(s){return t===this.request_load_id&&!q(s)&&(this.request_list_state="error",this.request_list_error=k(s,"Unable to load recent requests")),!1}finally{this.request_controller===e&&(this.request_controller=void 0)}}requestContext(e=this.selected_day,t=this.applied_filters){return e?JSON.stringify([e,t.query,t.provider_id,t.url_path,t.status,t.errors_only]):void 0}requestParams(e,t,s){const n=new URLSearchParams({day:e,limit:String(Je)});return t.query&&n.set("query",t.query),t.provider_id&&n.set("provider_id",t.provider_id),t.url_path&&n.set("url_path",t.url_path),t.status&&n.set("status",t.status),t.errors_only&&n.set("errors_only","true"),s&&n.set("cursor",s),n}async loadRequests(e=!1){const t=this.selected_day;if(!t)return this.request_list_state="idle",this.requests=[],this.next_cursor=void 0,this.request_rows_context=void 0,!1;const s={...this.applied_filters},n=this.requestContext(t,s),o=e?this.next_cursor:void 0;if(e&&(!o||this.request_rows_context!==n))return!1;this.request_controller?.abort();const r=new AbortController;this.request_controller=r;const u=++this.request_load_id;e?(this.loading_more=!0,this.load_more_error=void 0):(this.loading_more=!1,this.request_rows_context!==n&&(this.requests=[],this.next_cursor=void 0,this.request_rows_context=void 0),this.request_list_state="loading",this.request_list_error=void 0,this.load_more_error=void 0);try{const l=await $(`/api/requests?${this.requestParams(t,s,o).toString()}`,r.signal);if(u!==this.request_load_id||this.request_controller!==r||this.requestContext()!==n)return!1;if(e){const d=new Set(this.requests.map(_=>V(_)));this.requests=[...this.requests,...l.requests.filter(_=>!d.has(V(_)))]}else this.requests=l.requests;return this.next_cursor=l.next_cursor??void 0,this.request_rows_context=n,this.request_list_state="ready",!0}catch(l){return u!==this.request_load_id||q(l)||(l instanceof et&&l.status===503&&this.markRequestDayUnavailable(t),e?this.load_more_error=k(l,"Unable to load more requests"):(this.request_list_state="error",this.request_list_error=k(l,"Unable to load requests"))),!1}finally{u===this.request_load_id&&(this.loading_more=!1),this.request_controller===r&&(this.request_controller=void 0)}}async loadRequestDays(){const e=++this.request_days_load_id;this.request_days_loading=!0,this.request_days_error=void 0;try{const t=await $("/api/request-days");e===this.request_days_load_id&&(this.request_days=t)}catch(t){e===this.request_days_load_id&&(this.request_days_error=k(t,"Unable to load request day states"))}finally{e===this.request_days_load_id&&(this.request_days_loading=!1)}}async loadRequestUrlPaths(e){this.request_url_paths_controller?.abort();const t=new AbortController;this.request_url_paths_controller=t;const s=++this.request_url_paths_load_id;this.request_url_paths_loading=!0,this.request_url_paths_error=void 0;try{const n=new URLSearchParams({day:e}),o=await $(`/api/request-url-paths?${n.toString()}`,t.signal);s===this.request_url_paths_load_id&&this.selected_day===e&&(this.request_url_paths=o)}catch(n){s===this.request_url_paths_load_id&&!q(n)&&(this.request_url_paths=[],this.request_url_paths_error=k(n,"Unable to load URL paths"))}finally{s===this.request_url_paths_load_id&&(this.request_url_paths_loading=!1),this.request_url_paths_controller===t&&(this.request_url_paths_controller=void 0)}}markRequestDayUnavailable(e){this.request_days.some(t=>t.day===e)?this.request_days=this.request_days.map(t=>t.day===e?{...t,state:"unavailable"}:t):this.request_days=[{day:e,state:"unavailable"},...this.request_days]}resetRequestSelection(){this.request_detail_controller?.abort(),this.request_detail_controller=void 0,this.request_detail_load_id+=1,this.selected_request=void 0,this.selected_request_id=void 0,this.selected_request_row_id=void 0,this.selected_request_detail=void 0,this.request_detail_state="idle",this.request_detail_error=void 0,this.active_detail_tab="overview"}resetSessionSelection(){this.session_detail_controller?.abort(),this.session_usage_controller?.abort(),this.session_node_controller?.abort(),this.session_detail_controller=void 0,this.session_usage_controller=void 0,this.session_node_controller=void 0,this.session_detail_load_id+=1,this.session_usage_load_id+=1,this.session_node_load_id+=1,this.requested_session_id=void 0,this.requested_session_node_id=void 0,this.selected_session=void 0,this.selected_session_detail=void 0,this.selected_session_usage=void 0,this.selected_session_node_id=void 0,this.selected_session_node_detail=void 0,this.session_detail_state="idle",this.session_detail_error=void 0,this.session_usage_state="idle",this.session_usage_error=void 0,this.session_node_state="idle",this.session_node_error=void 0}async closeRequestDetail(){const e=this.selected_request_row_id&&this.selectedRequestDay()?V({day:this.selectedRequestDay(),row_id:this.selected_request_row_id}):void 0;if(++this.navigation_workflow_id,this.resetRequestSelection(),this.syncUrl("push"),!e||!window.matchMedia("(max-width: 680px)").matches)return;await this.updateComplete,[...this.querySelectorAll("request-list [data-request-key]")].find(s=>s.dataset.requestKey===e)?.focus()}async closeSessionDetail(){const e=this.selected_session?.session_id??this.requested_session_id;if(++this.navigation_workflow_id,this.resetSessionSelection(),this.syncUrl("push"),e===void 0||!window.matchMedia("(max-width: 680px)").matches)return;await this.updateComplete,[...this.querySelectorAll("session-list [data-session-id]")].find(s=>s.dataset.sessionId===e)?.focus()}async loadRequestDetail(e,t,s,n,o,r="replace"){this.request_detail_controller?.abort();const u=new AbortController;this.request_detail_controller=u;const l=++this.request_detail_load_id;this.selected_day=e,this.selected_request=n,this.selected_request_id=t,this.selected_request_row_id=s,o||(this.selected_request_detail=void 0),this.request_detail_state="loading",this.request_detail_error=void 0,r&&this.syncUrl(r);try{const d=new URLSearchParams({day:e,request_id:t});s&&d.set("row_id",s);const _=await $(`/api/request?${d.toString()}`,u.signal);if(l===this.request_detail_load_id&&this.request_detail_controller===u){const h=this.selected_request_row_id!==_.row_id;return this.selected_request_detail=_,this.selected_request_row_id=_.row_id,this.request_detail_state="ready",(r||h)&&this.syncUrl("replace"),!0}return!1}catch(d){return l===this.request_detail_load_id&&!q(d)&&(this.request_detail_state="error",this.request_detail_error=k(d,"Unable to load request detail")),!1}finally{this.request_detail_controller===u&&(this.request_detail_controller=void 0)}}async selectRequest(e){++this.navigation_workflow_id;const t=this.selected_request_id===e.request_id&&this.selected_request_detail?.day===e.day&&this.selected_request_detail.row_id===e.row_id,s=this.loadRequestDetail(e.day,e.request_id,e.row_id,e,t,"push");window.matchMedia("(max-width: 680px)").matches&&(await this.updateComplete,this.querySelector("request-detail-view .mobile-back-button")?.focus()),await s&&window.matchMedia("(max-width: 680px)").matches&&(await this.updateComplete,this.querySelector("request-detail-view .mobile-back-button")?.focus())}retryRequestDetail(){const e=this.selected_request_detail?.day??this.selected_request?.day??this.selected_day;e&&this.selected_request_id&&this.loadRequestDetail(e,this.selected_request_id,this.selected_request_row_id,this.selected_request,!!this.selected_request_detail,null)}selectDay(e){e!==this.selected_day&&(++this.navigation_workflow_id,this.selected_day=e,this.request_url_paths=[],this.resetRequestSelection(),this.syncUrl("push"),this.loadRequestUrlPaths(e),this.loadRequests())}pickerDays(){return!this.selected_day||this.request_days.some(e=>e.day===this.selected_day)?this.request_days:[{day:this.selected_day,state:"available"},...this.request_days]}adjacentAvailableDay(e){const t=this.pickerDays().filter(n=>n.state==="available").map(n=>n.day).sort();if(!this.selected_day)return;const s=t.indexOf(this.selected_day);return s<0?void 0:t[s+e]}submitFilters(e){e.preventDefault(),++this.navigation_workflow_id,this.applied_filters={query:this.search_query.trim(),provider_id:this.provider_id.trim(),url_path:this.url_path,status:this.status_filter.trim(),errors_only:this.errors_only},this.resetRequestSelection(),this.syncUrl("push"),this.loadRequests()}clearFilters(){this.search_query="",this.provider_id="",this.url_path="",this.status_filter="",this.errors_only=!1,this.applied_filters=oe(),++this.navigation_workflow_id,this.resetRequestSelection(),this.syncUrl("push"),this.loadRequests()}hasAppliedFilters(){return!!(this.applied_filters.query||this.applied_filters.provider_id||this.applied_filters.url_path||this.applied_filters.status||this.applied_filters.errors_only)}filtersChanged(){return this.search_query.trim()!==this.applied_filters.query||this.provider_id.trim()!==this.applied_filters.provider_id||this.url_path!==this.applied_filters.url_path||this.status_filter.trim()!==this.applied_filters.status||this.errors_only!==this.applied_filters.errors_only}providerOptions(){const e=new Set(this.requests.flatMap(t=>t.provider_id?[t.provider_id]:[]));return this.applied_filters.provider_id&&e.add(this.applied_filters.provider_id),[...e].sort()}urlPathOptions(){return!this.url_path||this.request_url_paths.some(e=>e.url_path===this.url_path)?this.request_url_paths:[{url_path:this.url_path,request_count:0},...this.request_url_paths]}ensureSessionsLoaded(e=!1){if(this.sessions_loaded&&!e)return Promise.resolve(!0);if(this.session_list_load&&!e)return this.session_list_load;this.session_list_controller?.abort();const t=new AbortController;this.session_list_controller=t;const s=++this.session_list_load_id;this.sessions_loading=!0,this.sessions_error=void 0;const n=this.loadSessions(t,s);return this.session_list_load=n,n}async loadSessions(e,t){try{const s=await $("/api/sessions?limit=100",e.signal);return t!==this.session_list_load_id||this.session_list_controller!==e?!1:(this.sessions=s,this.sessions_loaded=!0,this.selected_session&&(this.selected_session=s.find(n=>n.session_id===this.selected_session?.session_id)??this.selected_session),!0)}catch(s){return t===this.session_list_load_id&&!q(s)&&(this.sessions_error=k(s,"Unable to load sessions")),!1}finally{t===this.session_list_load_id&&this.session_list_controller===e&&(this.session_list_controller=void 0,this.session_list_load=void 0,this.sessions_loading=!1)}}retrySessions(){const e=++this.navigation_workflow_id;this.sessions_loaded=!1,this.retrySessionsAndRestore(e)}async retrySessionsAndRestore(e){if(!await this.ensureSessionsLoaded(!0)||e!==this.navigation_workflow_id||this.active_view!=="sessions")return;const s=this.selected_session?.session_id??this.requested_session_id;if(s===void 0)return;const n=this.selected_session_node_id??this.requested_session_node_id;await this.loadSession(s,this.sessions.find(o=>o.session_id===s),this.selected_session_detail?.session.session_id===s,null,n)}async refreshSessions(){const e=this.navigation_workflow_id,t=this.selected_session?.session_id??this.requested_session_id,s=this.selected_session_node_id,n=await this.ensureSessionsLoaded(!0),o=this.selected_session?.session_id??this.requested_session_id;n&&e===this.navigation_workflow_id&&t!==void 0&&o===t&&this.selected_session_node_id===s&&await this.loadSession(t,this.sessions.find(r=>r.session_id===t),!0,null,s)}filteredSessions(){const e=this.session_search_query.trim().toLocaleLowerCase();return e?this.sessions.filter(t=>[t.session_id,t.model,t.provider_id,t.account_id,t.endpoint,t.status===null?null:String(t.status)].some(s=>s?.toLocaleLowerCase().includes(e))):this.sessions}async loadSessionUsage(e,t){this.session_usage_controller?.abort();const s=new AbortController;this.session_usage_controller=s;const n=++this.session_usage_load_id;t||(this.selected_session_usage=void 0),this.session_usage_state="loading",this.session_usage_error=void 0;try{const o=new URLSearchParams({session_id:e}),r=await $(`/api/session-usage?${o.toString()}`,s.signal);return n===this.session_usage_load_id&&this.session_usage_controller===s?(this.selected_session_usage=r??void 0,this.session_usage_state="ready",!0):!1}catch(o){return n===this.session_usage_load_id&&!q(o)&&(this.session_usage_state="error",this.session_usage_error=k(o,"Unable to load session usage")),!1}finally{this.session_usage_controller===s&&(this.session_usage_controller=void 0)}}async loadSession(e,t,s,n="push",o){this.session_detail_controller?.abort(),this.session_node_controller?.abort();const r=new AbortController;this.session_detail_controller=r;const u=++this.session_detail_load_id,l=++this.session_node_load_id;this.requested_session_id=e,this.requested_session_node_id=o,this.selected_session=t,s||(this.selected_session_detail=void 0,this.selected_session_node_detail=void 0,this.selected_session_node_id=void 0,this.session_node_state="idle",this.session_node_error=void 0),this.loadSessionUsage(e,s),this.session_detail_state="loading",this.session_detail_error=void 0,n&&this.syncUrl(n);try{const d=new URLSearchParams({session_id:e,limit:"500"}),_=await $(`/api/session?${d.toString()}`,r.signal);if(u===this.session_detail_load_id&&this.session_detail_controller===r){if(this.selected_session=_.session,this.selected_session_detail=_,this.sessions=this.sessions.map(h=>h.session_id===_.session.session_id?_.session:h),this.session_detail_state="ready",l!==this.session_node_load_id)return!0;if(o){const h=_.nodes.find(p=>p.node_id===o);this.loadSessionNode(h??o,!1,"replace")}else this.selected_session_node_id=void 0,this.selected_session_node_detail=void 0,this.session_node_state="idle",this.syncUrl("replace");return!0}return!1}catch(d){return u===this.session_detail_load_id&&!q(d)&&(this.session_detail_state="error",this.session_detail_error=k(d,"Unable to load semantic session")),!1}finally{this.session_detail_controller===r&&(this.session_detail_controller=void 0)}}async loadSessionNode(e,t,s="push"){const n=this.selected_session?.session_id??this.requested_session_id;if(n===void 0)return!1;this.session_node_controller?.abort();const o=new AbortController;this.session_node_controller=o;const r=++this.session_node_load_id,u=typeof e=="string"?e:e.node_id;this.requested_session_node_id=u,this.selected_session_node_id=u,t||(this.selected_session_node_detail=void 0),this.session_node_state="loading",this.session_node_error=void 0,s&&this.syncUrl(s);try{const l=new URLSearchParams({session_id:n,node_id:u}),d=await $(`/api/session-node?${l.toString()}`,o.signal);return r===this.session_node_load_id&&this.session_node_controller===o?(this.selected_session_node_detail=d,this.session_node_state="ready",this.syncUrl("replace"),!0):!1}catch(l){return r===this.session_node_load_id&&!q(l)&&(this.session_node_state="error",this.session_node_error=k(l,"Unable to load semantic node content")),!1}finally{this.session_node_controller===o&&(this.session_node_controller=void 0)}}async selectSession(e){const t=++this.navigation_workflow_id;if(!await this.loadSession(e.session_id,e,!1,"push")||t!==this.navigation_workflow_id||this.active_view!=="sessions"||this.selected_session_detail?.session.session_id!==e.session_id||!window.matchMedia("(max-width: 680px)").matches)return;await this.updateComplete;const n=this.querySelector("session-detail-view");await n?.updateComplete,t===this.navigation_workflow_id&&this.active_view==="sessions"&&this.selected_session_detail?.session.session_id===e.session_id&&n?.querySelector(".mobile-back-button")?.focus()}collapseSessionNode(e="push"){this.session_node_controller?.abort(),this.session_node_controller=void 0,++this.session_node_load_id,this.requested_session_node_id=void 0,this.selected_session_node_id=void 0,this.selected_session_node_detail=void 0,this.session_node_state="idle",this.session_node_error=void 0,e&&this.syncUrl(e)}selectSessionNode(e){if(e.node_id===this.selected_session_node_id){this.collapseSessionNode();return}this.loadSessionNode(e,!1,"push")}retrySessionDetail(){const e=this.selected_session?.session_id??this.requested_session_id;e!==void 0&&this.loadSession(e,this.selected_session,!!this.selected_session_detail,null,this.selected_session_node_id??this.requested_session_node_id)}retrySessionUsage(){const e=this.selected_session?.session_id??this.requested_session_id;e!==void 0&&this.loadSessionUsage(e,!!this.selected_session_usage)}retrySessionNode(){const e=this.selected_session_detail?.nodes.find(t=>t.node_id===this.selected_session_node_id);(e??this.selected_session_node_id)&&this.loadSessionNode(e??this.selected_session_node_id,!!this.selected_session_node_detail,null)}async openSession(e){++this.navigation_workflow_id,this.setActiveView("sessions",!1,null),await this.ensureSessionsLoaded();const t=this.sessions.find(s=>s.session_id===e);await this.loadSession(e,t,!1,"push")}async openRequestFromSession(e){++this.navigation_workflow_id,this.setActiveView("requests",!1,null),this.search_query="",this.provider_id="",this.url_path="",this.status_filter="",this.errors_only=!1,this.applied_filters=oe(),this.selected_day=fs(e.ts),this.resetRequestSelection(),this.loadRequestDays(),this.loadRequestUrlPaths(this.selected_day),this.loadRequests(),!await this.loadRequestDetail(this.selected_day,e.request_id,void 0,void 0,!1,"push")&&this.request_detail_state==="error"&&this.request_detail_error==="request not found"&&(this.request_detail_error="Request history is unavailable; semantic session data is still retained.")}async loadRequestsView(){this.loadRequestDays(),this.selected_day?(this.loadRequestUrlPaths(this.selected_day),await this.loadRequests()):await this.loadLatestRequests()}setActiveView(e,t=!0,s="push"){s==="push"&&++this.navigation_workflow_id,this.active_view=e,s&&this.syncUrl(s),t&&(e==="sessions"?this.ensureSessionsLoaded():this.request_list_state==="idle"&&this.loadRequestsView())}setTimezone(e){this.timezone=e,this.syncUrl("push")}setDetailTab(e){this.active_detail_tab=e,this.syncUrl("push")}renderDayPicker(){const e=this.pickerDays(),t=this.adjacentAvailableDay(-1),s=this.adjacentAvailableDay(1);return a`
      <div class="day-control">
        <span class="control-label">UTC storage day</span>
        <div class="day-navigation">
          <button
            type="button"
            class="icon-button"
            title="Previous available day"
            aria-label="Previous available day"
            ?disabled=${!t}
            @click=${()=>t&&this.selectDay(t)}
          >
            ←
          </button>
          <select
            aria-label="Request storage day"
            .value=${this.selected_day??""}
            ?disabled=${e.length===0}
            @change=${n=>this.selectDay(n.target.value)}
          >
            ${this.selected_day?c:a`<option value="">No request day</option>`}
            ${e.map(n=>a`
                <option value=${n.day} ?disabled=${n.state!=="available"}>
                  ${n.day}${n.state==="empty"?" · empty":n.state==="unavailable"?" · unavailable":""}
                </option>
              `)}
          </select>
          <button
            type="button"
            class="icon-button"
            title="Next available day"
            aria-label="Next available day"
            ?disabled=${!s}
            @click=${()=>s&&this.selectDay(s)}
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
            @click=${()=>{this.loadRequests(),this.loadRequestDays(),this.selected_day&&this.loadRequestUrlPaths(this.selected_day)}}
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
              @input=${t=>this.search_query=t.target.value}
            />
          </label>
          <label>
            <span class="visually-hidden">Provider ID</span>
            <input
              list="provider-options"
              .value=${this.provider_id}
              ?disabled=${!e}
              placeholder="Any provider"
              @input=${t=>this.provider_id=t.target.value}
            />
            <datalist id="provider-options">
              ${this.providerOptions().map(t=>a`<option value=${t}></option>`)}
            </datalist>
          </label>
          <label>
            <span class="visually-hidden">URL path</span>
            <select
              class="url-path-filter"
              .value=${this.url_path}
              ?disabled=${!e||this.request_url_paths_loading}
              @change=${t=>this.url_path=t.target.value}
            >
              <option value="">${this.request_url_paths_loading?"Loading URL paths…":"Any URL path"}</option>
              ${this.urlPathOptions().map(t=>a`
                  <option value=${t.url_path}>
                    ${t.url_path}${t.request_count?` · ${t.request_count.toLocaleString()}`:""}
                  </option>
                `)}
            </select>
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
              @input=${t=>this.status_filter=t.target.value}
            />
          </label>
          <label class="errors-filter">
            <input
              type="checkbox"
              .checked=${this.errors_only}
              ?disabled=${!e}
              @change=${t=>this.errors_only=t.target.checked}
            />
            <span>Errors only</span>
          </label>
          <button type="submit" class="primary-button" ?disabled=${!e||!this.filtersChanged()}>Apply</button>
          ${this.hasAppliedFilters()?a`<button type="button" class="text-button" @click=${this.clearFilters}>Clear</button>`:c}
        </form>
        ${this.request_days_error?a`<p class="toolbar-warning" role="status">Day scan: ${this.request_days_error}</p>`:c}
        ${this.request_url_paths_error?a`<p class="toolbar-warning" role="status">URL paths: ${this.request_url_paths_error}</p>`:c}
      </section>
    `}renderRequestSidebar(){const e=this.requests.length>0;return a`
      <div class="list-pane" aria-busy=${String(this.request_list_state==="loading")}>
        <header class="list-pane-header">
          <div>
            <strong>Requests</strong>
            <span>${this.requests.length.toLocaleString()} loaded${this.next_cursor?" · more available":""}</span>
          </div>
          ${this.hasAppliedFilters()?a`<span class="filter-indicator">Filtered</span>`:c}
        </header>
        ${this.request_list_state==="loading"?a`
              <div class="inline-state" role="status">
                <span class="spinner" aria-hidden="true"></span>${e?"Refreshing requests…":"Loading requests…"}
              </div>
            `:c}
        ${this.request_list_state==="error"?a`
              <div class="inline-error" role="alert">
                <span>${this.request_list_error}</span>
                <button type="button" @click=${()=>{this.loadRequests()}}>Retry</button>
              </div>
            `:c}
        ${e?a`
              <request-list
                .requests=${this.requests}
                .selected_key=${this.selectedRequestDay()&&this.selected_request_row_id?V({day:this.selectedRequestDay(),row_id:this.selected_request_row_id}):void 0}
                .timezone=${this.timezone}
                @request-select=${t=>{this.selectRequest(z(t))}}
              ></request-list>
            `:this.request_list_state==="ready"?a`<p class="empty">No persisted requests match these filters.</p>`:this.request_list_state==="idle"?a`<p class="empty">Choose an available request day.</p>`:c}
        ${this.load_more_error?a`
              <div class="inline-error load-more-error" role="alert">
                <span>${this.load_more_error}</span>
                <button type="button" @click=${()=>{this.loadRequests(!0)}}>Retry</button>
              </div>
            `:c}
        ${this.next_cursor&&e?a`
              <div class="list-footer">
                <button type="button" class="secondary-button" ?disabled=${this.loading_more} @click=${()=>{this.loadRequests(!0)}}>
                  ${this.loading_more?"Loading…":"Load more"}
                </button>
              </div>
            `:e&&this.request_list_state==="ready"?a`<p class="end-of-list">End of loaded day</p>`:c}
      </div>
    `}renderSessionsSidebar(){const e=this.filteredSessions(),t=this.sessions.length>0;return a`
      <div class="list-pane" aria-busy=${String(this.sessions_loading)}>
        <header class="list-pane-header">
          <div>
            <strong>Recent sessions</strong>
            <span>
              ${this.session_search_query?`${e.length.toLocaleString()} of ${this.sessions.length.toLocaleString()} loaded`:`${this.sessions.length.toLocaleString()} loaded · newest first`}
            </span>
          </div>
          ${this.session_search_query?a`<span class="filter-indicator">Filtered</span>`:c}
        </header>
        ${this.sessions_loading?a`
              <div class="inline-state" role="status">
                <span class="spinner" aria-hidden="true"></span>${t?"Refreshing sessions…":"Loading sessions…"}
              </div>
            `:c}
        ${this.sessions_error?a`
              <div class="inline-error" role="alert">
                <span>${this.sessions_error}</span>
                <button type="button" @click=${this.retrySessions}>Retry</button>
              </div>
            `:c}
        ${e.length>0?a`
              <session-list
                .sessions=${e}
                .selected_session_id=${this.selected_session?.session_id??this.requested_session_id}
                .timezone=${this.timezone}
                @session-select=${s=>{this.selectSession(z(s))}}
              ></session-list>
            `:this.sessions_loaded&&this.session_search_query?a`<p class="empty">No recent sessions match this filter.</p>`:this.sessions_loaded?a`
                  <div class="empty empty-session-list">
                    <strong>No semantic sessions available</strong>
                    <span>The gateway records successful sessions here when session persistence is enabled.</span>
                  </div>
                `:c}
        ${t&&!this.session_search_query?a`<p class="end-of-list">${this.sessions.length===100?"Latest 100 sessions":"End of recent sessions"}</p>`:c}
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
        @session-node-select=${e=>this.selectSessionNode(z(e))}
        @open-request=${e=>{this.openRequestFromSession(z(e))}}
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
    `}render(){const e=this.active_view==="sessions"?this.info?.sessions_db:this.info?.requests_dir,t=this.active_view==="requests"?!!this.selected_request_id:this.requested_session_id!==void 0;return a`
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
        <section class="viewer-grid ${this.active_view==="requests"?"request-view":"session-view"} ${t?"has-selection":""}">
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
                    @detail-tab-change=${s=>this.setDetailTab(z(s))}
                    @open-session=${s=>{this.openSession(z(s))}}
                  ></request-detail-view>
                `:this.renderSessionDetail()}
          </article>
        </section>
      </main>
    `}}customElements.define("inspect-app",ms);
