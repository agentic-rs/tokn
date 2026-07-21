(function(){const e=document.createElement("link").relList;if(e&&e.supports&&e.supports("modulepreload"))return;for(const o of document.querySelectorAll('link[rel="modulepreload"]'))s(o);new MutationObserver(o=>{for(const n of o)if(n.type==="childList")for(const r of n.addedNodes)r.tagName==="LINK"&&r.rel==="modulepreload"&&s(r)}).observe(document,{childList:!0,subtree:!0});function t(o){const n={};return o.integrity&&(n.integrity=o.integrity),o.referrerPolicy&&(n.referrerPolicy=o.referrerPolicy),o.crossOrigin==="use-credentials"?n.credentials="include":o.crossOrigin==="anonymous"?n.credentials="omit":n.credentials="same-origin",n}function s(o){if(o.ep)return;o.ep=!0;const n=t(o);fetch(o.href,n)}})();const se=globalThis,ye=se.ShadowRoot&&(se.ShadyCSS===void 0||se.ShadyCSS.nativeShadow)&&"adoptedStyleSheets"in Document.prototype&&"replace"in CSSStyleSheet.prototype,Me=Symbol(),ve=new WeakMap;let Ke=class{constructor(e,t,s){if(this._$cssResult$=!0,s!==Me)throw Error("CSSResult is not constructable. Use `unsafeCSS` or `css` instead.");this.cssText=e,this.t=t}get styleSheet(){let e=this.o;const t=this.t;if(ye&&e===void 0){const s=t!==void 0&&t.length===1;s&&(e=ve.get(t)),e===void 0&&((this.o=e=new CSSStyleSheet).replaceSync(this.cssText),s&&ve.set(t,e))}return e}toString(){return this.cssText}};const Ze=i=>new Ke(typeof i=="string"?i:i+"",void 0,Me),Ge=(i,e)=>{if(ye)i.adoptedStyleSheets=e.map(t=>t instanceof CSSStyleSheet?t:t.styleSheet);else for(const t of e){const s=document.createElement("style"),o=se.litNonce;o!==void 0&&s.setAttribute("nonce",o),s.textContent=t.cssText,i.appendChild(s)}},me=ye?i=>i:i=>i instanceof CSSStyleSheet?(e=>{let t="";for(const s of e.cssRules)t+=s.cssText;return Ze(t)})(i):i;const{is:Ye,defineProperty:Qe,getOwnPropertyDescriptor:Xe,getOwnPropertyNames:et,getOwnPropertySymbols:tt,getPrototypeOf:st}=Object,ne=globalThis,be=ne.trustedTypes,it=be?be.emptyScript:"",ot=ne.reactiveElementPolyfillSupport,F=(i,e)=>i,_e={toAttribute(i,e){switch(e){case Boolean:i=i?it:null;break;case Object:case Array:i=i==null?i:JSON.stringify(i)}return i},fromAttribute(i,e){let t=i;switch(e){case Boolean:t=i!==null;break;case Number:t=i===null?null:Number(i);break;case Object:case Array:try{t=JSON.parse(i)}catch{t=null}}return t}},ze=(i,e)=>!Ye(i,e),we={attribute:!0,type:String,converter:_e,reflect:!1,useDefault:!1,hasChanged:ze};Symbol.metadata??=Symbol("metadata"),ne.litPropertyMetadata??=new WeakMap;let z=class extends HTMLElement{static addInitializer(e){this._$Ei(),(this.l??=[]).push(e)}static get observedAttributes(){return this.finalize(),this._$Eh&&[...this._$Eh.keys()]}static createProperty(e,t=we){if(t.state&&(t.attribute=!1),this._$Ei(),this.prototype.hasOwnProperty(e)&&((t=Object.create(t)).wrapped=!0),this.elementProperties.set(e,t),!t.noAccessor){const s=Symbol(),o=this.getPropertyDescriptor(e,s,t);o!==void 0&&Qe(this.prototype,e,o)}}static getPropertyDescriptor(e,t,s){const{get:o,set:n}=Xe(this.prototype,e)??{get(){return this[t]},set(r){this[t]=r}};return{get:o,set(r){const c=o?.call(this);n?.call(this,r),this.requestUpdate(e,c,s)},configurable:!0,enumerable:!0}}static getPropertyOptions(e){return this.elementProperties.get(e)??we}static _$Ei(){if(this.hasOwnProperty(F("elementProperties")))return;const e=st(this);e.finalize(),e.l!==void 0&&(this.l=[...e.l]),this.elementProperties=new Map(e.elementProperties)}static finalize(){if(this.hasOwnProperty(F("finalized")))return;if(this.finalized=!0,this._$Ei(),this.hasOwnProperty(F("properties"))){const t=this.properties,s=[...et(t),...tt(t)];for(const o of s)this.createProperty(o,t[o])}const e=this[Symbol.metadata];if(e!==null){const t=litPropertyMetadata.get(e);if(t!==void 0)for(const[s,o]of t)this.elementProperties.set(s,o)}this._$Eh=new Map;for(const[t,s]of this.elementProperties){const o=this._$Eu(t,s);o!==void 0&&this._$Eh.set(o,t)}this.elementStyles=this.finalizeStyles(this.styles)}static finalizeStyles(e){const t=[];if(Array.isArray(e)){const s=new Set(e.flat(1/0).reverse());for(const o of s)t.unshift(me(o))}else e!==void 0&&t.push(me(e));return t}static _$Eu(e,t){const s=t.attribute;return s===!1?void 0:typeof s=="string"?s:typeof e=="string"?e.toLowerCase():void 0}constructor(){super(),this._$Ep=void 0,this.isUpdatePending=!1,this.hasUpdated=!1,this._$Em=null,this._$Ev()}_$Ev(){this._$ES=new Promise(e=>this.enableUpdating=e),this._$AL=new Map,this._$E_(),this.requestUpdate(),this.constructor.l?.forEach(e=>e(this))}addController(e){(this._$EO??=new Set).add(e),this.renderRoot!==void 0&&this.isConnected&&e.hostConnected?.()}removeController(e){this._$EO?.delete(e)}_$E_(){const e=new Map,t=this.constructor.elementProperties;for(const s of t.keys())this.hasOwnProperty(s)&&(e.set(s,this[s]),delete this[s]);e.size>0&&(this._$Ep=e)}createRenderRoot(){const e=this.shadowRoot??this.attachShadow(this.constructor.shadowRootOptions);return Ge(e,this.constructor.elementStyles),e}connectedCallback(){this.renderRoot??=this.createRenderRoot(),this.enableUpdating(!0),this._$EO?.forEach(e=>e.hostConnected?.())}enableUpdating(e){}disconnectedCallback(){this._$EO?.forEach(e=>e.hostDisconnected?.())}attributeChangedCallback(e,t,s){this._$AK(e,s)}_$ET(e,t){const s=this.constructor.elementProperties.get(e),o=this.constructor._$Eu(e,s);if(o!==void 0&&s.reflect===!0){const n=(s.converter?.toAttribute!==void 0?s.converter:_e).toAttribute(t,s.type);this._$Em=e,n==null?this.removeAttribute(o):this.setAttribute(o,n),this._$Em=null}}_$AK(e,t){const s=this.constructor,o=s._$Eh.get(e);if(o!==void 0&&this._$Em!==o){const n=s.getPropertyOptions(o),r=typeof n.converter=="function"?{fromAttribute:n.converter}:n.converter?.fromAttribute!==void 0?n.converter:_e;this._$Em=o;const c=r.fromAttribute(t,n.type);this[o]=c??this._$Ej?.get(o)??c,this._$Em=null}}requestUpdate(e,t,s,o=!1,n){if(e!==void 0){const r=this.constructor;if(o===!1&&(n=this[e]),s??=r.getPropertyOptions(e),!((s.hasChanged??ze)(n,t)||s.useDefault&&s.reflect&&n===this._$Ej?.get(e)&&!this.hasAttribute(r._$Eu(e,s))))return;this.C(e,t,s)}this.isUpdatePending===!1&&(this._$ES=this._$EP())}C(e,t,{useDefault:s,reflect:o,wrapped:n},r){s&&!(this._$Ej??=new Map).has(e)&&(this._$Ej.set(e,r??t??this[e]),n!==!0||r!==void 0)||(this._$AL.has(e)||(this.hasUpdated||s||(t=void 0),this._$AL.set(e,t)),o===!0&&this._$Em!==e&&(this._$Eq??=new Set).add(e))}async _$EP(){this.isUpdatePending=!0;try{await this._$ES}catch(t){Promise.reject(t)}const e=this.scheduleUpdate();return e!=null&&await e,!this.isUpdatePending}scheduleUpdate(){return this.performUpdate()}performUpdate(){if(!this.isUpdatePending)return;if(!this.hasUpdated){if(this.renderRoot??=this.createRenderRoot(),this._$Ep){for(const[o,n]of this._$Ep)this[o]=n;this._$Ep=void 0}const s=this.constructor.elementProperties;if(s.size>0)for(const[o,n]of s){const{wrapped:r}=n,c=this[o];r!==!0||this._$AL.has(o)||c===void 0||this.C(o,void 0,n,c)}}let e=!1;const t=this._$AL;try{e=this.shouldUpdate(t),e?(this.willUpdate(t),this._$EO?.forEach(s=>s.hostUpdate?.()),this.update(t)):this._$EM()}catch(s){throw e=!1,this._$EM(),s}e&&this._$AE(t)}willUpdate(e){}_$AE(e){this._$EO?.forEach(t=>t.hostUpdated?.()),this.hasUpdated||(this.hasUpdated=!0,this.firstUpdated(e)),this.updated(e)}_$EM(){this._$AL=new Map,this.isUpdatePending=!1}get updateComplete(){return this.getUpdateComplete()}getUpdateComplete(){return this._$ES}shouldUpdate(e){return!0}update(e){this._$Eq&&=this._$Eq.forEach(t=>this._$ET(t,this[t])),this._$EM()}updated(e){}firstUpdated(e){}};z.elementStyles=[],z.shadowRootOptions={mode:"open"},z[F("elementProperties")]=new Map,z[F("finalized")]=new Map,ot?.({ReactiveElement:z}),(ne.reactiveElementVersions??=[]).push("2.1.2");const fe=globalThis,qe=i=>i,oe=fe.trustedTypes,Se=oe?oe.createPolicy("lit-html",{createHTML:i=>i}):void 0,Be="$lit$",E=`lit$${Math.random().toFixed(9).slice(2)}$`,He="?"+E,nt=`<${He}>`,P=document,K=()=>P.createComment(""),Z=i=>i===null||typeof i!="object"&&typeof i!="function",ge=Array.isArray,rt=i=>ge(i)||typeof i?.[Symbol.iterator]=="function",ae=`[ 	
\f\r]`,j=/<(?:(!--|\/[^a-zA-Z])|(\/?[a-zA-Z][^>\s]*)|(\/?$))/g,Ae=/-->/g,ke=/>/g,L=RegExp(`>|${ae}(?:([^\\s"'>=/]+)(${ae}*=${ae}*(?:[^ 	
\f\r"'\`<>=]|("|')|))|$)`,"g"),Re=/'/g,xe=/"/g,Ie=/^(?:script|style|textarea|title)$/i,at=i=>(e,...t)=>({_$litType$:i,strings:e,values:t}),a=at(1),H=Symbol.for("lit-noChange"),_=Symbol.for("lit-nothing"),Ee=new WeakMap,N=P.createTreeWalker(P,129);function je(i,e){if(!ge(i)||!i.hasOwnProperty("raw"))throw Error("invalid template strings array");return Se!==void 0?Se.createHTML(e):e}const dt=(i,e)=>{const t=i.length-1,s=[];let o,n=e===2?"<svg>":e===3?"<math>":"",r=j;for(let c=0;c<t;c++){const l=i[c];let d,u,h=-1,p=0;for(;p<l.length&&(r.lastIndex=p,u=r.exec(l),u!==null);)p=r.lastIndex,r===j?u[1]==="!--"?r=Ae:u[1]!==void 0?r=ke:u[2]!==void 0?(Ie.test(u[2])&&(o=RegExp("</"+u[2],"g")),r=L):u[3]!==void 0&&(r=L):r===L?u[0]===">"?(r=o??j,h=-1):u[1]===void 0?h=-2:(h=r.lastIndex-u[2].length,d=u[1],r=u[3]===void 0?L:u[3]==='"'?xe:Re):r===xe||r===Re?r=L:r===Ae||r===ke?r=j:(r=L,o=void 0);const y=r===L&&i[c+1].startsWith("/>")?" ":"";n+=r===j?l+nt:h>=0?(s.push(d),l.slice(0,h)+Be+l.slice(h)+E+y):l+E+(h===-2?c:y)}return[je(i,n+(i[t]||"<?>")+(e===2?"</svg>":e===3?"</math>":"")),s]};class G{constructor({strings:e,_$litType$:t},s){let o;this.parts=[];let n=0,r=0;const c=e.length-1,l=this.parts,[d,u]=dt(e,t);if(this.el=G.createElement(d,s),N.currentNode=this.el.content,t===2||t===3){const h=this.el.content.firstChild;h.replaceWith(...h.childNodes)}for(;(o=N.nextNode())!==null&&l.length<c;){if(o.nodeType===1){if(o.hasAttributes())for(const h of o.getAttributeNames())if(h.endsWith(Be)){const p=u[r++],y=o.getAttribute(h).split(E),f=/([.?@])?(.*)/.exec(p);l.push({type:1,index:n,name:f[2],strings:y,ctor:f[1]==="."?ct:f[1]==="?"?ut:f[1]==="@"?_t:re}),o.removeAttribute(h)}else h.startsWith(E)&&(l.push({type:6,index:n}),o.removeAttribute(h));if(Ie.test(o.tagName)){const h=o.textContent.split(E),p=h.length-1;if(p>0){o.textContent=oe?oe.emptyScript:"";for(let y=0;y<p;y++)o.append(h[y],K()),N.nextNode(),l.push({type:2,index:++n});o.append(h[p],K())}}}else if(o.nodeType===8)if(o.data===He)l.push({type:2,index:n});else{let h=-1;for(;(h=o.data.indexOf(E,h+1))!==-1;)l.push({type:7,index:n}),h+=E.length-1}n++}}static createElement(e,t){const s=P.createElement("template");return s.innerHTML=e,s}}function I(i,e,t=i,s){if(e===H)return e;let o=s!==void 0?t._$Co?.[s]:t._$Cl;const n=Z(e)?void 0:e._$litDirective$;return o?.constructor!==n&&(o?._$AO?.(!1),n===void 0?o=void 0:(o=new n(i),o._$AT(i,t,s)),s!==void 0?(t._$Co??=[])[s]=o:t._$Cl=o),o!==void 0&&(e=I(i,o._$AS(i,e.values),o,s)),e}class lt{constructor(e,t){this._$AV=[],this._$AN=void 0,this._$AD=e,this._$AM=t}get parentNode(){return this._$AM.parentNode}get _$AU(){return this._$AM._$AU}u(e){const{el:{content:t},parts:s}=this._$AD,o=(e?.creationScope??P).importNode(t,!0);N.currentNode=o;let n=N.nextNode(),r=0,c=0,l=s[0];for(;l!==void 0;){if(r===l.index){let d;l.type===2?d=new Y(n,n.nextSibling,this,e):l.type===1?d=new l.ctor(n,l.name,l.strings,this,e):l.type===6&&(d=new ht(n,this,e)),this._$AV.push(d),l=s[++c]}r!==l?.index&&(n=N.nextNode(),r++)}return N.currentNode=P,o}p(e){let t=0;for(const s of this._$AV)s!==void 0&&(s.strings!==void 0?(s._$AI(e,s,t),t+=s.strings.length-2):s._$AI(e[t])),t++}}class Y{get _$AU(){return this._$AM?._$AU??this._$Cv}constructor(e,t,s,o){this.type=2,this._$AH=_,this._$AN=void 0,this._$AA=e,this._$AB=t,this._$AM=s,this.options=o,this._$Cv=o?.isConnected??!0}get parentNode(){let e=this._$AA.parentNode;const t=this._$AM;return t!==void 0&&e?.nodeType===11&&(e=t.parentNode),e}get startNode(){return this._$AA}get endNode(){return this._$AB}_$AI(e,t=this){e=I(this,e,t),Z(e)?e===_||e==null||e===""?(this._$AH!==_&&this._$AR(),this._$AH=_):e!==this._$AH&&e!==H&&this._(e):e._$litType$!==void 0?this.$(e):e.nodeType!==void 0?this.T(e):rt(e)?this.k(e):this._(e)}O(e){return this._$AA.parentNode.insertBefore(e,this._$AB)}T(e){this._$AH!==e&&(this._$AR(),this._$AH=this.O(e))}_(e){this._$AH!==_&&Z(this._$AH)?this._$AA.nextSibling.data=e:this.T(P.createTextNode(e)),this._$AH=e}$(e){const{values:t,_$litType$:s}=e,o=typeof s=="number"?this._$AC(e):(s.el===void 0&&(s.el=G.createElement(je(s.h,s.h[0]),this.options)),s);if(this._$AH?._$AD===o)this._$AH.p(t);else{const n=new lt(o,this),r=n.u(this.options);n.p(t),this.T(r),this._$AH=n}}_$AC(e){let t=Ee.get(e.strings);return t===void 0&&Ee.set(e.strings,t=new G(e)),t}k(e){ge(this._$AH)||(this._$AH=[],this._$AR());const t=this._$AH;let s,o=0;for(const n of e)o===t.length?t.push(s=new Y(this.O(K()),this.O(K()),this,this.options)):s=t[o],s._$AI(n),o++;o<t.length&&(this._$AR(s&&s._$AB.nextSibling,o),t.length=o)}_$AR(e=this._$AA.nextSibling,t){for(this._$AP?.(!1,!0,t);e!==this._$AB;){const s=qe(e).nextSibling;qe(e).remove(),e=s}}setConnected(e){this._$AM===void 0&&(this._$Cv=e,this._$AP?.(e))}}class re{get tagName(){return this.element.tagName}get _$AU(){return this._$AM._$AU}constructor(e,t,s,o,n){this.type=1,this._$AH=_,this._$AN=void 0,this.element=e,this.name=t,this._$AM=o,this.options=n,s.length>2||s[0]!==""||s[1]!==""?(this._$AH=Array(s.length-1).fill(new String),this.strings=s):this._$AH=_}_$AI(e,t=this,s,o){const n=this.strings;let r=!1;if(n===void 0)e=I(this,e,t,0),r=!Z(e)||e!==this._$AH&&e!==H,r&&(this._$AH=e);else{const c=e;let l,d;for(e=n[0],l=0;l<n.length-1;l++)d=I(this,c[s+l],t,l),d===H&&(d=this._$AH[l]),r||=!Z(d)||d!==this._$AH[l],d===_?e=_:e!==_&&(e+=(d??"")+n[l+1]),this._$AH[l]=d}r&&!o&&this.j(e)}j(e){e===_?this.element.removeAttribute(this.name):this.element.setAttribute(this.name,e??"")}}class ct extends re{constructor(){super(...arguments),this.type=3}j(e){this.element[this.name]=e===_?void 0:e}}class ut extends re{constructor(){super(...arguments),this.type=4}j(e){this.element.toggleAttribute(this.name,!!e&&e!==_)}}class _t extends re{constructor(e,t,s,o,n){super(e,t,s,o,n),this.type=5}_$AI(e,t=this){if((e=I(this,e,t,0)??_)===H)return;const s=this._$AH,o=e===_&&s!==_||e.capture!==s.capture||e.once!==s.once||e.passive!==s.passive,n=e!==_&&(s===_||o);o&&this.element.removeEventListener(this.name,this,s),n&&this.element.addEventListener(this.name,this,e),this._$AH=e}handleEvent(e){typeof this._$AH=="function"?this._$AH.call(this.options?.host??this.element,e):this._$AH.handleEvent(e)}}class ht{constructor(e,t,s){this.element=e,this.type=6,this._$AN=void 0,this._$AM=t,this.options=s}get _$AU(){return this._$AM._$AU}_$AI(e){I(this,e)}}const pt=fe.litHtmlPolyfillSupport;pt?.(G,Y),(fe.litHtmlVersions??=[]).push("3.3.3");const yt=(i,e,t)=>{const s=t?.renderBefore??e;let o=s._$litPart$;if(o===void 0){const n=t?.renderBefore??null;s._$litPart$=o=new Y(e.insertBefore(K(),n),n,void 0,t??{})}return o._$AI(i),o};const $e=globalThis;class w extends z{constructor(){super(...arguments),this.renderOptions={host:this},this._$Do=void 0}createRenderRoot(){const e=super.createRenderRoot();return this.renderOptions.renderBefore??=e.firstChild,e}update(e){const t=this.render();this.hasUpdated||(this.renderOptions.isConnected=this.isConnected),super.update(e),this._$Do=yt(t,this.renderRoot,this.renderOptions)}connectedCallback(){super.connectedCallback(),this._$Do?.setConnected(!0)}disconnectedCallback(){super.disconnectedCallback(),this._$Do?.setConnected(!1)}render(){return H}}w._$litElement$=!0,w.finalized=!0,$e.litElementHydrateSupport?.({LitElement:w});const ft=$e.litElementPolyfillSupport;ft?.({LitElement:w});($e.litElementVersions??=[]).push("4.2.2");class Fe extends Error{status;constructor(e,t){super(t),this.name="HttpError",this.status=e}}async function m(i,e){const t=await fetch(i,{cache:"no-store",signal:e});if(!t.ok){const s=await t.json().catch(()=>({}));throw new Fe(t.status,s.error??`Request failed (${t.status})`)}return t.json()}function x(i){return i instanceof Error&&i.name==="AbortError"}function V(i,e,t=!1){const s=t?{hour:"2-digit",minute:"2-digit",second:"2-digit"}:{dateStyle:"medium",timeStyle:"medium"};return e==="utc"&&(s.timeZone="UTC"),new Intl.DateTimeFormat(void 0,s).format(new Date(i))}function gt(i,e){const t=new Date(i),s=new Date,o=e==="utc"?t.getUTCFullYear():t.getFullYear(),n=e==="utc"?s.getUTCFullYear():s.getFullYear(),r={month:"short",day:"numeric",hour:"2-digit",minute:"2-digit"};return o!==n&&(r.year="numeric"),e==="utc"&&(r.timeZone="UTC"),new Intl.DateTimeFormat(void 0,r).format(t)}function $t(i,e){const t=Math.max(0,e-i);if(t<1e3)return`${t.toLocaleString()} ms`;const s=Math.floor(t/1e3);if(s<60)return`${s}s`;const o=Math.floor(s/60);if(o<60)return`${o}m ${s%60}s`;const n=Math.floor(o/60);return n<24?`${n}h ${o%60}m`:`${Math.floor(n/24)}d ${n%24}h`}function B(i){return`${i.day}:${i.row_id}`}function S(i,e=10){return i?i.length>e?`…${i.slice(-e)}`:i:"—"}function vt(i){const e=i.inbound_req_url??i.endpoint;return W(e)}function Ce(i){const e=i.toLowerCase().replaceAll("_","-");return e==="authorization"||e==="password"||e==="code"||e==="signature"||e==="sig"||e.includes("api-key")||e.includes("access-key")||e.includes("token")||e.includes("secret")||e.includes("credential")}function W(i){if(!i)return"unknown endpoint";try{const e=new URL(i,window.location.origin);for(const t of new Set(e.searchParams.keys()))Ce(t)&&e.searchParams.set(t,"REDACTED");return`${e.pathname}${e.search}`}catch{return i.replace(/([?&]([^=&]+)=)([^&]*)/g,(e,t,s)=>{let o=s;try{o=decodeURIComponent(s)}catch{}return Ce(o)?`${t}REDACTED`:e})}}function mt(i){if(i.request_error)return{label:"ERR",tone:"error",title:i.request_error};const e=i.inbound_resp_status??i.outbound_resp_status??i.status;if(e===null)return{label:"—",tone:"neutral",title:"No response status persisted"};const t=i.inbound_resp_status!==null?"Client response":i.outbound_resp_status!==null?"Provider response":"Request";return e>=400?{label:String(e),tone:"error",title:`${t}: ${e}`}:e>=300?{label:String(e),tone:"warning",title:`${t}: ${e}`}:{label:String(e),tone:"success",title:`${t}: ${e}`}}function bt(i){const e=i.status;return e===null?{label:"—",tone:"neutral",title:"No status stored for the current session head"}:e>=400?{label:String(e),tone:"error",title:`Current head status: ${e}`}:e>=300?{label:String(e),tone:"warning",title:`Current head status: ${e}`}:{label:String(e),tone:"success",title:`Current head status: ${e}`}}function D(i){return i.detail}function $(i,e){const t=i[e];return typeof t=="string"?t:void 0}function ee(i,e){const t=i[e];return typeof t=="number"?t:void 0}const de="••••••••";function le(i){const e=i.toLowerCase().replaceAll("_","-");return e==="authorization"||e==="proxy-authorization"||e==="cookie"||e==="set-cookie"||e.includes("api-key")||e.includes("token")||e.includes("secret")}function J(i){if(Array.isArray(i))return i.length===2&&typeof i[0]=="string"&&le(i[0])?[i[0],de]:i.map(e=>J(e));if(i!==null&&typeof i=="object")return Object.fromEntries(Object.entries(i).map(([e,t])=>[e,le(e)?de:J(t)]));if(typeof i=="string")try{return J(JSON.parse(i))}catch{return i.replace(/^([^:\r\n]+)(:\s*)(.*)$/gm,(e,t,s)=>le(t.trim())?`${t}${s}${de}`:e)}return i}function he(i){return Array.isArray(i)?i.map(e=>he(e)):i!==null&&typeof i=="object"?Object.fromEntries(Object.entries(i).map(([e,t])=>[e,wt(e)?J(t):he(t)])):i}function wt(i){const e=i.replace(/([a-z0-9])([A-Z])/g,"$1_$2").toLowerCase().replace(/[-\s]+/g,"_");return e==="headers"||e.endsWith("_headers")}function pe(i){return Array.isArray(i)?i.map(e=>pe(e)):i!==null&&typeof i=="object"?Object.fromEntries(Object.entries(i).map(([e,t])=>[e,e.toLowerCase().endsWith("_url")&&typeof t=="string"?W(t):pe(t)])):i}function qt(i){if(typeof i=="string")try{return JSON.stringify(JSON.parse(i),null,2)}catch{return i}return JSON.stringify(i,null,2)??String(i)}function St(i){if(Array.isArray(i))return`${i.length} item${i.length===1?"":"s"}`;if(i!==null&&typeof i=="object"){const e=Object.keys(i).length;return`${e} field${e===1?"":"s"}`}return typeof i=="string"?`${new Blob([i]).size.toLocaleString()} bytes`:typeof i}class At extends w{static properties={label:{type:String},value:{attribute:!1},load_url:{type:String},is_headers:{type:Boolean},redact_record_headers:{type:Boolean},open:{type:Boolean,state:!0},wrap:{type:Boolean,state:!0},revealed:{type:Boolean,state:!0},copy_state:{type:String,state:!0},load_state:{type:String,state:!0},loaded_value:{attribute:!1,state:!0},error_message:{type:String,state:!0}};load_controller;copy_timeout;constructor(){super(),this.label="Payload",this.is_headers=!1,this.redact_record_headers=!1,this.open=!1,this.wrap=!0,this.revealed=!1,this.copy_state="idle",this.load_state="idle"}createRenderRoot(){return this}disconnectedCallback(){this.load_controller?.abort(),this.copy_timeout!==void 0&&window.clearTimeout(this.copy_timeout),super.disconnectedCallback()}willUpdate(e){!e.has("value")&&!e.has("load_url")||(this.load_controller?.abort(),this.load_controller=void 0,this.copy_timeout!==void 0&&(window.clearTimeout(this.copy_timeout),this.copy_timeout=void 0),this.open=!1,this.revealed=!1,this.copy_state="idle",this.load_state="idle",this.loaded_value=void 0,this.error_message=void 0)}effectiveValue(){return this.load_state==="ready"?this.loaded_value:this.value}displayedValue(){const e=this.effectiveValue(),t=this.redact_record_headers?pe(e):e,s=this.revealed?t:this.redact_record_headers?he(t):this.is_headers?J(t):t;return qt(s)}toggleOpen(e){this.open=e.currentTarget.open,this.open&&this.value===void 0&&this.load_url&&this.load_state==="idle"&&this.loadPayload()}async loadPayload(){const e=this.load_url;if(!e)return;this.load_controller?.abort();const t=new AbortController;this.load_controller=t,this.load_state="loading",this.error_message=void 0;try{const s=await m(e,t.signal);if(this.load_controller!==t||this.load_url!==e)return;const o=new URL(e,window.location.origin).searchParams.get("field");if(!o||s.field!==o)throw new Error("Payload response did not match the requested field");this.loaded_value=s.value,this.load_state="ready"}catch(s){if(this.load_controller!==t||x(s))return;this.load_state="error",this.error_message=s instanceof Error?s.message:"Unable to load payload"}finally{this.load_controller===t&&(this.load_controller=void 0)}}async copyValue(){try{await navigator.clipboard.writeText(this.displayedValue()),this.copy_state="copied",this.copy_timeout!==void 0&&window.clearTimeout(this.copy_timeout),this.copy_timeout=window.setTimeout(()=>{this.copy_state="idle",this.copy_timeout=void 0},1500)}catch{this.copy_state="error"}}render(){if(!this.load_url&&(this.value===null||this.value===void 0||this.value===""))return _;const e=this.effectiveValue(),t=this.is_headers||this.redact_record_headers,s=this.load_state==="loading"?"Loading…":this.load_state==="error"?"Load failed":e===null?"No payload":e===void 0?"Load on open":St(e);return a`
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
                          `:_}
                      <span class="payload-security-note">
                        ${t&&!this.revealed?"Sensitive headers redacted":""}
                      </span>
                    </div>
                    <pre class=${this.wrap?"wrap":"nowrap"}><code>${this.displayedValue()}</code></pre>
                  `:_}
      </details>
    `}}customElements.define("payload-panel",At);const Le="/backend-api/codex/alpha/search";function C(i){return i!==null&&typeof i=="object"&&!Array.isArray(i)?i:void 0}function b(i){return typeof i=="string"&&i.length>0?i:void 0}function Ve(i){return Array.isArray(i)?i.filter(e=>typeof e=="string"):[]}function kt(i){return typeof i=="number"&&Number.isFinite(i)?i:void 0}function Rt(i){const e=C(i),t=b(e?.q);if(t)return{query:t,domains:Ve(e?.domains),recency_days:kt(e?.recency)}}function xt(i){const e=C(i);if(!e)return;const t={type:b(e.type),domain:b(e.domain),ref_id:b(e.ref_id),snippet:b(e.snippet),title:b(e.title),url:b(e.url)};return Object.values(t).some(s=>s!==void 0)?t:void 0}function Et(i){if(Array.isArray(i))for(const e of i){const s=C(e)?.content;if(Array.isArray(s))for(const o of s){const n=C(o),r=b(n?.text)??b(n?.input_text);if(r)return r}}}function Ct(i){const e=i.replace(/\s/g,"");if(!e||!/^[A-Za-z0-9_\-+/]*={0,2}$/.test(e))return;const t=e.replace(/=+$/,"").length;if(t%4!==1)return Math.floor(t*3/4)}function Lt(i,e){const t=C(i),s=C(e),o=C(t?.commands),n=C(t?.settings),r=Array.isArray(o?.search_query)?o.search_query:[],c=Array.isArray(s?.results)?s.results:[],l=b(s?.encrypted_output);return{queries:r.flatMap(d=>{const u=Rt(d);return u?[u]:[]}),response_length:b(o?.response_length),allowed_callers:Ve(n?.allowed_callers),external_web_access:typeof n?.external_web_access=="boolean"?n.external_web_access:void 0,prompt:Et(t?.input),output:b(s?.output),results:c.flatMap(d=>{const u=xt(d);return u?[u]:[]}),encrypted_output_bytes:l?Ct(l):void 0}}function Ut(i){if(typeof i!="string")return!1;try{return new URL(i,"http://localhost").pathname===Le}catch{return i.split("?",1)[0]===Le}}function Nt(i){if(i)try{const e=new URL(i);return e.protocol==="http:"||e.protocol==="https:"?e.href:void 0}catch{return}}function Pt(i){return i<1e3?`${i} B`:i<1e6?`${(i/1e3).toFixed(1)} KB`:`${(i/1e6).toFixed(1)} MB`}class Dt extends w{static properties={request_url:{type:String},response_url:{type:String},load_state:{type:String,state:!0},request_payload:{attribute:!1,state:!0},response_payload:{attribute:!1,state:!0},error_message:{type:String,state:!0}};load_controller;constructor(){super(),this.request_url="",this.response_url="",this.load_state="idle"}createRenderRoot(){return this}disconnectedCallback(){this.load_controller?.abort(),super.disconnectedCallback()}updated(e){(e.has("request_url")||e.has("response_url"))&&this.load()}async load(){if(!this.request_url||!this.response_url)return;const e=this.request_url,t=this.response_url;this.load_controller?.abort();const s=new AbortController;this.load_controller=s,this.load_state="loading",this.error_message=void 0;try{const[o,n]=await Promise.all([m(e,s.signal),m(t,s.signal)]);if(this.load_controller!==s||this.request_url!==e||this.response_url!==t)return;if(o.field!=="inbound_req_body"||n.field!=="inbound_resp_body")throw new Error("Search payload response did not match the requested fields");this.request_payload=o.value,this.response_payload=n.value,this.load_state="ready"}catch(o){if(this.load_controller!==s||x(o))return;this.load_state="error",this.error_message=o instanceof Error?o.message:"Unable to load web search"}finally{this.load_controller===s&&(this.load_controller=void 0)}}render(){if(this.load_state==="loading"||this.load_state==="idle")return a`
        <section class="web-search-inspection web-search-state" aria-label="Web search" aria-live="polite">
          <span class="spinner" aria-hidden="true"></span>
          <span>Loading web search…</span>
        </section>
      `;if(this.load_state==="error")return a`
        <section class="web-search-inspection web-search-state error-state" aria-label="Web search" role="alert">
          <div><strong>Web search could not be loaded</strong><span>${this.error_message}</span></div>
          <button type="button" @click=${()=>{this.load()}}>Retry</button>
        </section>
      `;const e=Lt(this.request_payload,this.response_payload);return a`
      <section class="web-search-inspection" aria-label="Web search">
        <header class="web-search-heading">
          <div>
            <p class="eyebrow">Codex web search</p>
            <h3>${e.queries.length} ${e.queries.length===1?"query":"queries"}</h3>
          </div>
          <div class="web-search-metrics">
            <span><strong>${e.results.length}</strong> results</span>
            ${e.response_length?a`<span><strong>${e.response_length}</strong> response</span>`:_}
            ${e.encrypted_output_bytes!==void 0?a`<span title="Decoded encrypted payload size"><strong>${Pt(e.encrypted_output_bytes)}</strong> encrypted</span>`:_}
          </div>
        </header>

        <div class="web-search-queries">
          ${e.queries.length===0?a`<p class="web-search-empty">No valid search query was persisted.</p>`:e.queries.map((t,s)=>a`
                <article>
                  <span class="web-search-query-index">${s+1}</span>
                  <div>
                    <code>${t.query}</code>
                    ${t.domains.length>0||t.recency_days!==void 0?a`
                          <p>
                            ${t.domains.length>0?`Domains: ${t.domains.join(", ")}`:""}
                            ${t.domains.length>0&&t.recency_days!==void 0?" · ":""}
                            ${t.recency_days!==void 0?`Last ${t.recency_days} days`:""}
                          </p>
                        `:_}
                  </div>
                </article>
              `)}
        </div>

        <dl class="web-search-settings">
          <div><dt>Caller</dt><dd>${e.allowed_callers.join(", ")||"—"}</dd></div>
          <div><dt>External web access</dt><dd>${e.external_web_access===void 0?"—":String(e.external_web_access)}</dd></div>
        </dl>

        <div class="web-search-results">
          <h4>Results</h4>
          ${e.results.length===0?a`<p class="web-search-empty">No structured results were returned.</p>`:e.results.map((t,s)=>{const o=Nt(t.url);return a`
                  <article class="web-search-result">
                    <span class="web-search-result-index">${s+1}</span>
                    <div>
                      <div class="web-search-result-title">
                        ${o?a`<a href=${o} target="_blank" rel="noopener noreferrer">${t.title??t.url}</a>`:a`<strong>${t.title??t.url??"Untitled result"}</strong>`}
                        <span>${t.domain??""}</span>
                      </div>
                      ${t.snippet?a`<p>${t.snippet}</p>`:_}
                      ${t.ref_id?a`<code>${t.ref_id}</code>`:_}
                    </div>
                  </article>
                `})}
        </div>

        <div class="payload-stack web-search-payloads">
          ${e.output?a`<payload-panel label="Synthesized search output" .value=${e.output}></payload-panel>`:_}
          ${e.prompt?a`<payload-panel label="Prompt context sent to search" .value=${e.prompt}></payload-panel>`:_}
        </div>
      </section>
    `}}customElements.define("web-search-detail",Dt);const U=[{id:"overview",label:"Overview"},{id:"client",label:"Client"},{id:"provider",label:"Provider"},{id:"raw",label:"Raw"}];function T(i){return i==null||i===""?"—":typeof i=="boolean"?i?"Yes":"No":String(i)}function Tt(i){if(i!==null&&typeof i=="object"&&!Array.isArray(i))return i;if(typeof i=="string")try{const e=JSON.parse(i);return e!==null&&typeof e=="object"&&!Array.isArray(e)?e:void 0}catch{return}}function Ue(i,e,t){return Tt(i[e])?.[t]??i[t]}function q(i,e,t,s){return`/api/request-payload?${new URLSearchParams({day:i,request_id:e,row_id:t,field:s}).toString()}`}function Ne(i){return i===void 0?"neutral":i>=400?"error":i>=300?"warning":"success"}class Ot extends w{static properties={detail:{attribute:!1},summary:{attribute:!1},state:{type:String},error_message:{type:String},active_tab:{type:String},timezone:{type:String}};createRenderRoot(){return this}openSession(e){this.dispatchEvent(new CustomEvent("open-session",{detail:e,bubbles:!0,composed:!0}))}retry(){this.dispatchEvent(new CustomEvent("detail-retry",{bubbles:!0,composed:!0}))}close(){this.dispatchEvent(new CustomEvent("detail-close",{bubbles:!0,composed:!0}))}selectTab(e){this.dispatchEvent(new CustomEvent("detail-tab-change",{detail:e,bubbles:!0,composed:!0}))}tabKeydown(e){const t=U.findIndex(r=>r.id===this.active_tab);let s;if(e.key==="ArrowRight"?s=(t+1)%U.length:e.key==="ArrowLeft"?s=(t-1+U.length)%U.length:e.key==="Home"?s=0:e.key==="End"&&(s=U.length-1),s===void 0)return;e.preventDefault();const o=U[s];this.selectTab(o.id),this.querySelectorAll("[role=tab]")[s]?.focus()}renderOverview(e){const t=ee(e,"ts"),s=Ue(e,"ctx_json","latency_ms"),o=Ue(e,"params_json","stream"),n=[["Timestamp",t===void 0?void 0:V(t,this.timezone)],["Storage day",this.detail?.day],["Endpoint",e.endpoint],["Model",e.model],["Provider",e.provider_id],["Account",e.account_id],["Latency",typeof s=="number"?`${s} ms`:s],["Streaming",o]],r=ee(e,"inbound_resp_status"),c=ee(e,"outbound_resp_status"),l=ee(e,"status"),d=$(e,"request_id")??this.summary?.request_id,u=this.detail?.row_id,h=$(e,"inbound_req_url")??$(e,"endpoint"),p=this.detail&&d&&u&&Ut(h)?a`
          <web-search-detail
            .request_url=${q(this.detail.day,d,u,"inbound_req_body")}
            .response_url=${q(this.detail.day,d,u,"inbound_resp_body")}
          ></web-search-detail>
        `:_;return a`
      <section class="flow-grid" aria-label="Request flow">
        <div>
          <span>Client request</span>
          <strong>${$(e,"inbound_req_method")??"—"}</strong>
        </div>
        <span class="flow-arrow" aria-hidden="true">→</span>
        <div>
          <span>Provider response</span>
          <strong class="status-text ${Ne(c)}">${T(c)}</strong>
        </div>
        <span class="flow-arrow" aria-hidden="true">→</span>
        <div>
          <span>Client response</span>
          <strong class="status-text ${Ne(r??l)}">
            ${T(r??l)}
          </strong>
        </div>
      </section>
      <dl class="metadata-grid">
        ${n.map(([y,f])=>a`
            <div>
              <dt>${y}</dt>
              <dd title=${T(f)}>${T(f)}</dd>
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
    `}renderClient(e,t,s,o){return a`
      <section class="payload-group">
        <div class="payload-group-heading">
          <div><span class="direction-label">Incoming</span><h3>Client request</h3></div>
          <span>${$(e,"inbound_req_method")??"—"} ${W($(e,"inbound_req_url"))}</span>
        </div>
        <payload-panel
          label="Request headers"
          .value=${e.inbound_req_headers}
          .load_url=${q(t,s,o,"inbound_req_headers")}
          .is_headers=${!0}
        ></payload-panel>
        <payload-panel
          label="Request body"
          .value=${e.inbound_req_body}
          .load_url=${q(t,s,o,"inbound_req_body")}
        ></payload-panel>
      </section>
      <section class="payload-group">
        <div class="payload-group-heading">
          <div><span class="direction-label">Outgoing</span><h3>Client response</h3></div>
          <span>Status ${T(e.inbound_resp_status??e.status)}</span>
        </div>
        <payload-panel
          label="Response headers"
          .value=${e.inbound_resp_headers}
          .load_url=${q(t,s,o,"inbound_resp_headers")}
          .is_headers=${!0}
        ></payload-panel>
        <payload-panel
          label="Response body"
          .value=${e.inbound_resp_body}
          .load_url=${q(t,s,o,"inbound_resp_body")}
        ></payload-panel>
      </section>
    `}renderProvider(e,t,s,o){return a`
      <section class="payload-group">
        <div class="payload-group-heading">
          <div><span class="direction-label">Outgoing</span><h3>Provider request</h3></div>
          <span>${$(e,"outbound_req_method")??"—"} ${W($(e,"outbound_req_url"))}</span>
        </div>
        <payload-panel
          label="Request headers"
          .value=${e.outbound_req_headers}
          .load_url=${q(t,s,o,"outbound_req_headers")}
          .is_headers=${!0}
        ></payload-panel>
        <payload-panel
          label="Request body"
          .value=${e.outbound_req_body}
          .load_url=${q(t,s,o,"outbound_req_body")}
        ></payload-panel>
      </section>
      <section class="payload-group">
        <div class="payload-group-heading">
          <div><span class="direction-label">Incoming</span><h3>Provider response</h3></div>
          <span>Status ${T(e.outbound_resp_status)}</span>
        </div>
        <payload-panel
          label="Response headers"
          .value=${e.outbound_resp_headers}
          .load_url=${q(t,s,o,"outbound_resp_headers")}
          .is_headers=${!0}
        ></payload-panel>
        <payload-panel
          label="Response body"
          .value=${e.outbound_resp_body}
          .load_url=${q(t,s,o,"outbound_resp_body")}
        ></payload-panel>
      </section>
    `}renderTab(e,t,s,o){switch(this.active_tab){case"client":return this.renderClient(e,t,s,o);case"provider":return this.renderProvider(e,t,s,o);case"raw":return this.renderRaw(e);default:return this.renderOverview(e)}}render(){if(!this.detail)return this.state==="loading"?a`
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
        `:a`<section class="detail-state"><p>Select a request to inspect its route, payloads, and responses.</p></section>`;const e=this.detail.request,t=$(e,"request_id")??this.summary?.request_id??"unknown id",s=$(e,"session_id")??this.summary?.session_id??void 0,o=$(e,"inbound_req_method")??this.summary?.inbound_req_method??"REQUEST",n=W($(e,"inbound_req_url")??this.summary?.inbound_req_url??$(e,"endpoint"));return a`
      <section class="detail-content">
        <header class="detail-header">
          <button type="button" class="mobile-back-button" @click=${this.close}>← Requests</button>
          <div class="detail-title">
            <p class="eyebrow">request · ${S(t)}</p>
            <h2><span>${o}</span> ${n}</h2>
            <p class="muted" title=${t}>${t}</p>
          </div>
          <div class="detail-actions">
            ${s?a`<button type="button" class="secondary-button" @click=${()=>this.openSession(s)}>Open session</button>`:_}
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
        ${this.state==="loading"?a`<div class="inline-state" role="status"><span class="spinner" aria-hidden="true"></span>Refreshing detail…</div>`:_}
        ${this.state==="error"?a`
              <div class="inline-error" role="alert">
                <span>${this.error_message}</span>
                <button type="button" @click=${this.retry}>Retry</button>
              </div>
            `:_}
        ${e.request_error?a`<div class="request-error" role="alert">${String(e.request_error)}</div>`:_}
        <div class="detail-tabs" role="tablist" aria-label="Request detail sections" @keydown=${this.tabKeydown}>
          ${U.map(r=>a`
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
    `}}customElements.define("request-detail-view",Ot);class Mt extends w{static properties={requests:{attribute:!1},selected_key:{type:String},timezone:{type:String}};createRenderRoot(){return this}selectRequest(e){this.dispatchEvent(new CustomEvent("request-select",{detail:e,bubbles:!0,composed:!0}))}render(){const e=this.requests??[];return e.length===0?a`<p class="empty">No persisted requests match these filters.</p>`:a`
      <ul class="request-list" aria-label="Requests">
        ${e.map(t=>{const s=mt(t),o=this.selected_key===B(t),n=t.inbound_req_method??"REQUEST",r=vt(t);return a`
            <li>
              <button
                type="button"
                class="request-row ${o?"selected":""}"
                data-request-key=${B(t)}
                aria-current=${o?"true":"false"}
                @click=${()=>this.selectRequest(t)}
              >
                <span class="request-row-time">${V(t.ts,this.timezone,!0)}</span>
                <span class="status ${s.tone}" title=${s.title}>${s.label}</span>
                <span class="request-row-main">
                  <span class="request-route"><strong>${n}</strong><span>${r}</span></span>
                  <span class="request-context">
                    <span>${t.model??"unknown model"}</span>
                    <span aria-hidden="true">·</span>
                    <span>${t.provider_id??"unknown provider"}</span>
                  </span>
                  <span class="request-identifiers">
                    <span title=${t.request_id}>req ${S(t.request_id)}</span>
                    ${t.session_id?a`<span title=${t.session_id}>session ${S(t.session_id)}</span>`:a`<span>no session</span>`}
                  </span>
                </span>
              </button>
            </li>
          `})}
      </ul>
    `}}customElements.define("request-list",Mt);function zt(i,e){const t=new Set,s=new Set;for(const o of i){if(s.has(o.node_id))continue;const n=[],r=new Map;let c=o;for(;c&&!s.has(c.node_id);){const l=r.get(c.node_id);if(l!==void 0){for(const d of n.slice(l))t.add(d);break}r.set(c.node_id,n.length),n.push(c.node_id),c=c.parent_node_id?e.get(c.parent_node_id):void 0}for(const l of n)s.add(l)}return t}function Bt(i,e,t){const s=Number(t.has(e.node_id))-Number(t.has(i.node_id));return s!==0?s:i.ts!==e.ts?e.ts-i.ts:i.node_id.localeCompare(e.node_id)}function Ht(i,e,t){const s=[...i].filter(r=>r.is_head).sort((r,c)=>c.ts-r.ts||r.node_id.localeCompare(c.node_id))[0],o=new Set;let n=s;for(;n;){if(o.has(n.node_id)){t.add(n.node_id);break}o.add(n.node_id),n=n.parent_node_id?e.get(n.parent_node_id):void 0}return o}function Pe(i,e,t,s,o){const n=[{node:i,next_child:0}];for(;n.length>0;){const r=n[n.length-1],c=t.get(r.node.node_id);if(c==="done"){n.pop();continue}c===void 0&&t.set(r.node.node_id,"visiting");const l=e.get(r.node.node_id)??[];if(r.next_child<l.length){const d=l[r.next_child];r.next_child+=1;const u=t.get(d.node_id);u===void 0?n.push({node:d,next_child:0}):u==="visiting"&&(s.add(r.node.node_id),s.add(d.node_id));continue}t.set(r.node.node_id,"done"),o.push(r.node),n.pop()}}function It(i,e,t,s,o){const n=(d,u)=>Bt(d,u,s);for(const d of t.values())d.sort(n);const r=i.filter(d=>d.parent_node_id===null||!e.has(d.parent_node_id)||o.has(d.node_id)).sort(n),c=new Map,l=[];for(const d of r)Pe(d,t,c,o,l);for(const d of[...i].sort(n))c.has(d.node_id)||(o.add(d.node_id),Pe(d,t,c,o,l));return l}function jt(i,e,t,s,o){const n=[],r=[],c=new Set;let l=0;for(const d of i){let u=r.indexOf(d.node_id);const h=u===-1;h&&(u=r.length,r.push(d.node_id));const p=[...r],y=[];let f;const g=d.parent_node_id,A=g&&o.has(d.node_id)&&o.has(g)?null:g;if(A&&!c.has(A)){const v=r.findIndex((X,Je)=>Je!==u&&X===A);v===-1?(r[u]=A,f=u):(r.splice(u,1),f=v-+(u<v))}else A&&c.has(A)&&(o.add(d.node_id),o.add(A)),r.splice(u,1);const Q=[...r];for(let v=0;v<p.length;v+=1){if(v===u)continue;const X=Q.indexOf(p[v]);X!==-1&&y.push({from_lane:v,to_lane:X,kind:"continuation",active:t.has(p[v])})}f!==void 0&&y.push({from_lane:u,to_lane:f,kind:"parent",active:t.has(d.node_id)}),l=Math.max(l,p.length,Q.length),n.push({node:d,top_lanes:p,bottom_lanes:Q,node_lane:u,starts_here:h,connections:y,bottom_lane_is_active:Q.map(v=>t.has(v)),child_count:e.get(d.node_id)?.length??0,parent_is_missing:!!(A&&s.has(A)),is_on_head_path:t.has(d.node_id),has_topology_warning:o.has(d.node_id)}),c.add(d.node_id)}return{rows:n,max_lane_count:l,remaining_lanes:[...r]}}function De(i){const e=new Map;for(const d of i)e.has(d.node_id)||e.set(d.node_id,d);const t=[...e.values()],s=new Map(t.map(d=>[d.node_id,[]])),o=new Set,n=zt(t,e);for(const d of t){const u=d.parent_node_id;u&&(e.has(u)&&!(n.has(d.node_id)&&n.has(u))?s.get(u)?.push(d):e.has(u)||o.add(u))}const r=Ht(t,e,n),c=It(t,e,s,r,n),l=jt(c,s,r,o,n);for(const d of l.rows)d.has_topology_warning=n.has(d.node.node_id);return{...l,missing_parent_ids:[...o].sort(),remaining_lanes:l.remaining_lanes.filter(d=>o.has(d)),cycle_node_ids:[...n].sort()}}const We=6,ie=16,ce=25;function Ft(i){return i===null?{label:"—",tone:"neutral",title:"No response status stored"}:i>=400?{label:String(i),tone:"error",title:`Response status: ${i}`}:i>=300?{label:String(i),tone:"warning",title:`Response status: ${i}`}:{label:String(i),tone:"success",title:`Response status: ${i}`}}function Vt(i){switch(i.toLowerCase()){case"assistant":return"assistant";case"system":case"developer":return"system";case"tool":case"function":return"tool";case"compaction":return"compaction";default:return"user"}}function Wt(i){try{return JSON.stringify(i,null,2)??String(i)}catch{return String(i)}}function O(i){if(i<1024)return`${i.toLocaleString()} B`;const e=["KiB","MiB","GiB"];let t=i/1024,s=e[0];for(const o of e.slice(1)){if(t<1024)break;t/=1024,s=o}return`${t>=10?t.toFixed(0):t.toFixed(1)} ${s}`}function M(i){return i===null?"—":i.toLocaleString()}function ue(i){return i===null?"—":new Intl.NumberFormat(void 0,{notation:"compact",maximumFractionDigits:i>=1e4?1:0}).format(i)}function Jt(i){switch(i){case"message_tree":return{direction:"New",title:"Input delta",empty_message:"No new semantic input was stored for this observation."};case"suffix_append":return{direction:"Appended",title:"Input delta",empty_message:"No new semantic input was stored for this node."};case"root_snapshot":return{direction:"Initial",title:"Input snapshot",empty_message:"No semantic input was stored for this root snapshot."};case"conflict_snapshot":return{direction:"Replaced",title:"Replacement snapshot",empty_message:"No semantic input was stored for this replacement snapshot."};default:return{direction:"Stored",title:"Node input",empty_message:"No semantic input was stored for this node."}}}function k(i){return(i+.5)*ie}function Te(i){return`session-tree-lanes-${Math.min(i,We)}`}class Kt extends w{static properties={sessions:{attribute:!1},selected_session_id:{type:String},timezone:{type:String}};createRenderRoot(){return this}selectSession(e){this.dispatchEvent(new CustomEvent("session-select",{detail:e,bubbles:!0,composed:!0}))}render(){const e=this.sessions??[];return a`
      <ul class="session-list" aria-label="Sessions">
        ${e.map(t=>{const s=this.selected_session_id===t.session_id,o=bt(t);return a`
            <li>
              <button
                type="button"
                class="session-row ${s?"selected":""}"
                data-session-id=${t.session_id}
                aria-current=${s?"true":"false"}
                @click=${()=>this.selectSession(t)}
              >
                <time datetime=${new Date(t.last_ts).toISOString()}>
                  ${gt(t.last_ts,this.timezone)}
                </time>
                <span class="status ${o.tone}" title=${o.title}>${o.label}</span>
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
                    session ${S(t.session_id)}
                  </span>
                </span>
                <span class="session-row-chevron" aria-hidden="true">›</span>
              </button>
            </li>
          `})}
      </ul>
    `}}class Zt extends w{static properties={detail:{attribute:!1},node_detail:{attribute:!1},state:{type:String},error_message:{type:String},node_state:{type:String},node_error_message:{type:String},selected_node_id:{type:String},usage:{attribute:!1},usage_state:{type:String},usage_error_message:{type:String},timezone:{type:String}};createRenderRoot(){return this}close(){this.dispatchEvent(new CustomEvent("session-close",{bubbles:!0,composed:!0}))}retryDetail(){this.dispatchEvent(new CustomEvent("session-retry",{bubbles:!0,composed:!0}))}retryNode(){this.dispatchEvent(new CustomEvent("session-node-retry",{bubbles:!0,composed:!0}))}retryUsage(){this.dispatchEvent(new CustomEvent("session-usage-retry",{bubbles:!0,composed:!0}))}selectNode(e){this.dispatchEvent(new CustomEvent("session-node-select",{detail:e,bubbles:!0,composed:!0}))}openRequest(e){this.dispatchEvent(new CustomEvent("open-request",{detail:e,bubbles:!0,composed:!0}))}renderPart(e){switch(e.content.encoding){case"text":{const t=e.content.value||a`<span class="faint">Empty text part</span>`,s=e.content.truncated?a`<p class="session-part-note">Preview truncated · ${O(e.byte_length)} stored</p>`:_;return a`<div class="session-part-text">${t}${s}</div>`}case"json":return a`
          <details class="session-structured-part">
            <summary>${e.part_type.replaceAll("_"," ")}</summary>
            <pre>${Wt(e.content.value)}</pre>
          </details>
        `;case"encrypted":return a`
          <details class="session-structured-part">
            <summary>${e.part_type.replaceAll("_"," ")} · encrypted</summary>
            <p>
              ${O(e.content.byte_length)} encrypted payload stored. Plaintext is unavailable and the
              encrypted content is not returned to the viewer.
            </p>
          </details>
        `;case"binary":return a`
          <details class="session-structured-part">
            <summary>${e.part_type.replaceAll("_"," ")} · binary</summary>
            <p>${O(e.content.byte_length)} stored. Binary bytes are not returned to the viewer.</p>
          </details>
        `;case"omitted":return a`
          <details class="session-structured-part">
            <summary>${e.part_type.replaceAll("_"," ")} · omitted</summary>
            <p>
              ${O(e.byte_length)} ${e.content.original_encoding} content omitted after reaching the
              ${e.content.reason==="part_limit"?"per-part byte preview":"node content-size"} limit.
            </p>
          </details>
        `}}renderMessages(e,t){return e.length===0?a`<p class="session-message-empty">${t}</p>`:a`
      <div class="session-message-stack">
        ${e.map(s=>a`
          <article class="session-message ${Vt(s.role)}">
            <header>
              <span>${s.role}</span>
              <span>
                ${s.parts.length.toLocaleString()}${s.parts.length===s.parts_total?"":` of ${s.parts_total.toLocaleString()}`} parts
                ${s.status===null?_:a` · status ${s.status}`}
              </span>
            </header>
            <div class="session-message-parts">
              ${s.parts.length>0?s.parts.map(o=>this.renderPart(o)):s.parts_total>0?a`
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
          <div><dt>Input</dt><dd>${M(e.input_tokens)}</dd></div>
          <div><dt>Output</dt><dd>${M(e.output_tokens)}</dd></div>
          <div><dt>Total</dt><dd>${M(e.total_tokens)}</dd></div>
          <div><dt>Cache read</dt><dd>${M(e.cache_read_tokens)}</dd></div>
          <div><dt>Cache write</dt><dd>${M(e.cache_write_tokens)}</dd></div>
          <div><dt>Reasoning</dt><dd>${M(e.reasoning_tokens)}</dd></div>
        </dl>
      </section>
    `}nodeDomId(e,t){return`session-node-${e}-${encodeURIComponent(t)}`}renderNodeGraph(e,t){const s=t*ie,o=k(e.node_lane),n=`M ${o} ${ce} l 0 0.001`,r=e.connections.map(l=>{const d=k(l.from_lane),u=k(l.to_lane),h=l.kind==="parent"?ce:0;return a`
        <path
          class="session-tree-edge ${l.kind} ${l.active?"active":""}"
          d=${`M ${d} ${h} L ${u} 100`}
        ></path>
      `}),c=["session-tree-dot",e.node.is_head?"head":"",e.child_count>1?"branch":"",e.has_topology_warning?"warning":""].filter(Boolean).join(" ");return a`
      <svg
        viewBox=${`0 0 ${s} 100`}
        preserveAspectRatio="none"
        focusable="false"
        aria-hidden="true"
      >
        ${e.starts_here?_:a`
              <path
                class="session-tree-edge incoming ${e.is_on_head_path?"active":""}"
                d=${`M ${o} 0 L ${o} ${ce}`}
              ></path>
            `}
        ${r}
        <path class="${c} outline" d=${n}></path>
        <path class="${c} fill" d=${n}></path>
      </svg>
    `}renderNodeGraphContinuation(e,t){const s=t*ie;return a`
      <svg
        viewBox=${`0 0 ${s} 100`}
        preserveAspectRatio="none"
        focusable="false"
        aria-hidden="true"
      >
        ${e.bottom_lanes.map((o,n)=>a`
          <path
            class="session-tree-edge continuation ${e.bottom_lane_is_active[n]?"active":""}"
            d=${`M ${k(n)} 0 L ${k(n)} 100`}
          ></path>
        `)}
      </svg>
    `}renderTreeBoundary(e,t,s,o,n){if(e.missing_parent_ids.length===0)return _;const r=t*ie,c=e.remaining_lanes.length>0?e.remaining_lanes.map((p,y)=>y):e.missing_parent_ids.map((p,y)=>y),l=[...new Set(c)],d=n?"Connects to loaded tree":s?"Earlier ancestry omitted":"Parent nodes unavailable",u=n?`Parent ${S(n.node_id)} appears in the session tree below.`:s?`${o.toLocaleString()} ${o===1?"node falls":"nodes fall"} outside this bounded tree snapshot.`:"The stored parent links point outside the returned session tree.",h=n?"Parent link resolved in the loaded snapshot":`${e.missing_parent_ids.length.toLocaleString()} parent ${e.missing_parent_ids.length===1?"link":"links"} outside the snapshot`;return a`
      <li class="session-tree-boundary ${n?"loaded-parent":""} ${Te(t)}">
        <span class="session-tree-boundary-graph" aria-hidden="true">
          <svg viewBox=${`0 0 ${r} 100`} preserveAspectRatio="none" focusable="false">
            ${l.map(p=>a`
              <path class="session-tree-edge boundary" d=${`M ${k(p)} 0 L ${k(p)} 48`}></path>
              <path
                class="session-tree-boundary-dot outline"
                d=${`M ${k(p)} 52 l 0 0.001`}
              ></path>
              <path
                class="session-tree-boundary-dot fill"
                d=${`M ${k(p)} 52 l 0 0.001`}
              ></path>
            `)}
          </svg>
        </span>
        <div class="session-tree-boundary-card" role="note">
          <strong>${d}</strong>
          <span>${u}</span>
          <span title=${n?.node_id??e.missing_parent_ids.join(", ")}>${h}</span>
        </div>
      </li>
    `}renderLoadedNodeContent(e){const t=e.truncation,s=Jt(e.node.reduction_kind),o=t.request_messages.messages_total-t.request_messages.messages_returned,n=t.response_messages.messages_total-t.response_messages.messages_returned,r=o>0||n>0||t.parts_omitted>0||t.content_parts_truncated>0||t.binary_parts_elided>0;return a`
      <div class="session-node-content-actions">
        <span title=${e.node.request_id}>Request ${S(e.node.request_id)}</span>
        <button type="button" class="secondary-button" @click=${()=>this.openRequest(e.node)}>Open request</button>
      </div>
      ${r?a`
            <div class="session-content-boundary" role="status">
              <strong>Bounded content preview</strong>
              <span>
                ${O(t.content_bytes_returned)} of
                ${O(t.content_bytes_total)} inline content returned
                ${o+n>0?` · ${(o+n).toLocaleString()} messages omitted`:""}
                ${t.parts_omitted>0?` · ${t.parts_omitted.toLocaleString()} parts omitted`:""}
                ${t.content_parts_truncated>0?` · ${t.content_parts_truncated.toLocaleString()} parts truncated`:""}
                ${t.binary_parts_elided>0?` · ${t.binary_parts_elided.toLocaleString()} binary parts represented as metadata`:""}
              </span>
            </div>
          `:_}
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
    `}renderNodeContent(e){if(this.selected_node_id!==e.node_id)return _;const t=this.node_detail?.node.node_id===e.node_id?this.node_detail:void 0,s=this.node_state==="loading"?a`<div class="inline-state"><span class="spinner" aria-hidden="true"></span>Loading semantic content…</div>`:this.node_state==="error"?a`
            <div class="inline-error" role="alert">
              <span>${this.node_error_message}</span>
              <button type="button" @click=${this.retryNode}>Retry</button>
            </div>
          `:t?this.renderLoadedNodeContent(t):_;return a`
      <section
        id=${this.nodeDomId("content",e.node_id)}
        class="session-node-content"
        aria-labelledby=${this.nodeDomId("trigger",e.node_id)}
        aria-live="polite"
        aria-busy=${String(this.node_state==="loading")}
      >
        ${s}
      </section>
    `}renderNodeUsage(e){if(this.usage_state==="loading")return a`<span class="session-node-token-usage muted">Token usage loading…</span>`;if(this.usage_state==="error")return a`<span class="session-node-token-usage muted">Token usage unavailable</span>`;if(!e)return a`<span class="session-node-token-usage muted">No token usage</span>`;const t=e.context_tokens===null?"Context tokens unavailable":`${e.context_tokens.toLocaleString()} context tokens`,s=e.input_delta_tokens===null?"Input delta tokens unavailable":`${e.input_delta_tokens.toLocaleString()} uncached input tokens`,o=e.output_tokens===null?"Output tokens unavailable":`${e.output_tokens.toLocaleString()} output tokens`;return a`
      <span class="session-node-token-usage">
        <span class="session-node-token-label">tokens</span>
        <span class="session-node-token-separator" aria-hidden="true">·</span>
        <span title=${t}>${ue(e.context_tokens)} context</span>
        <span class="session-node-token-separator" aria-hidden="true">·</span>
        <span title=${s}>
          ${e.input_delta_tokens===null?"—":`+${ue(e.input_delta_tokens)}`} input delta
        </span>
        <span class="session-node-token-separator" aria-hidden="true">·</span>
        <span title=${o}>${ue(e.output_tokens)} output</span>
      </span>
    `}renderNode(e,t,s,o){const n=e.node,r=n.node_id===this.selected_node_id,c=Ft(n.status),l=!!(o&&n.parent_node_id===o.node_id),d=e.parent_is_missing&&!l,u=["session-node",Te(t),r?"selected":"",e.is_on_head_path?"head-path":"",d?"boundary-child":"",e.has_topology_warning?"topology-warning":""].filter(Boolean).join(" "),h=n.reduction_kind==="message_tree"?n.input_message_count:n.request_message_count,p=n.reduction_kind==="message_tree"?"input":"input delta",y=n.reduction_kind==="message_tree"?a` (+${n.request_message_count.toLocaleString()} new)`:_,f=n.reduction_kind==="message_tree"?n.output_message_count:n.response_message_count,g=n.reduction_kind==="message_tree"?n.parent_node_id?`Prefix-derived child of ${n.parent_node_id}.`:"Prefix-derived root node.":n.parent_node_id?`Recorded child of ${n.parent_node_id}.`:"Recorded root node.";return a`
      <li class=${u}>
        <span class="session-node-graph" aria-hidden="true">
          ${this.renderNodeGraph(e,t)}
        </span>
        <button
          id=${this.nodeDomId("trigger",n.node_id)}
          type="button"
          class="session-node-trigger"
          data-node-id=${n.node_id}
          aria-expanded=${String(r)}
          aria-controls=${r?this.nodeDomId("content",n.node_id):_}
          aria-current=${n.is_head?"true":_}
          @click=${()=>this.selectNode(n)}
        >
          <span class="session-node-primary">
            <time datetime=${new Date(n.ts).toISOString()}>${V(n.ts,this.timezone)}</time>
            <span class="status ${c.tone}" title=${c.title}>${c.label}</span>
            ${e.child_count>1?a`<span class="branch-badge">${e.child_count.toLocaleString()} branches</span>`:_}
            ${n.is_head?a`<span class="head-badge">Current head</span>`:_}
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
            <span>${f.toLocaleString()} output</span>
          </span>
          ${this.renderNodeUsage(s.get(n.request_id))}
          <span class="session-node-id" title=${n.request_id}>
            request ${S(n.request_id)} · ${n.parent_node_id?`parent ${S(n.parent_node_id)}`:"root"}
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
            `:_}
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
      `;const{session:e,nodes:t}=this.detail,s=new Map((this.usage?.requests??[]).map(g=>[g.request_id,g])),o=De(t),n=Math.max(1,o.max_lane_count),r=Math.max(0,e.request_count-t.length),c=o.missing_parent_ids.length>0,l=!!(this.selected_node_id&&t.some(g=>g.node_id===this.selected_node_id)),d=this.node_detail,u=!l&&d&&d.node.node_id===this.selected_node_id?d.node:void 0,h=u?De([u]):void 0,p=h?Math.max(1,h.max_lane_count):1,y=u?.parent_node_id?t.find(g=>g.node_id===u.parent_node_id):void 0,f=e.model??"Unknown model";return a`
      <section class="detail-content session-detail-content">
        <header class="detail-header session-detail-header">
          <button type="button" class="mobile-back-button" @click=${this.close}>← Sessions</button>
          <div class="detail-title">
            <p class="eyebrow">session · ${S(e.session_id)}</p>
            <h2>${f}<span> on ${e.provider_id??"unknown provider"}</span></h2>
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
        ${this.state==="loading"?a`<div class="inline-state"><span class="spinner" aria-hidden="true"></span>Refreshing session…</div>`:_}
        ${this.state==="error"?a`
              <div class="inline-error" role="alert">
                <span>${this.error_message}</span>
                <button type="button" @click=${this.retryDetail}>Retry</button>
              </div>
            `:_}
        <dl class="session-metadata-grid">
          <div><dt>Semantic nodes</dt><dd>${e.request_count.toLocaleString()}</dd></div>
          <div><dt>Duration</dt><dd>${$t(e.first_ts,e.last_ts)}</dd></div>
          <div><dt>First seen</dt><dd>${V(e.first_ts,this.timezone)}</dd></div>
          <div><dt>Last active</dt><dd>${V(e.last_ts,this.timezone)}</dd></div>
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
              ${o.max_lane_count>We?" · compressed lanes":""}
            </span>
          </header>
          ${this.detail.nodes_truncated?a`
                <p class="session-truncation-note">
                  ${r.toLocaleString()} older nodes are omitted.
                  ${c?" Amber graph endpoints continue into the omitted ancestry.":" The graph shows every parent link available in this snapshot."}
                </p>
              `:_}
          ${o.cycle_node_ids.length>0?a`
                <p class="session-topology-warning" role="alert">
                  ${o.cycle_node_ids.length.toLocaleString()} nodes contain cyclic parent links; their graph
                  edges were detached defensively.
                </p>
              `:_}
          ${t.length>0?a`
                <p class="session-tree-direction">
                  <span>Leaves and current-head branch</span>
                  <span aria-hidden="true">↓</span>
                  <span>recorded parents</span>
                </p>
              `:_}
          ${this.selected_node_id?_:a`<p class="session-content-hint">Open a node to load its conversation content from <code>sessions.db</code>.</p>`}
          ${this.selected_node_id&&!l?a`
                <section class="session-linked-node" aria-label="Directly linked session node">
                  <header>
                    <div>
                      <p class="eyebrow">Direct link</p>
                      <h4>Node outside this activity snapshot</h4>
                    </div>
                    <span>${S(this.selected_node_id)}</span>
                  </header>
                  ${h?a`
                        <ol class="session-node-list linked-node-list">
                          ${h.rows.map(g=>this.renderNode(g,p,s,y))}
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
                          `:_}
                </section>
              `:_}
          ${t.length>0?a`
                <ol class="session-node-list">
                  ${o.rows.map(g=>this.renderNode(g,n,s))}
                  ${this.renderTreeBoundary(o,n,this.detail.nodes_truncated,r)}
                </ol>
              `:a`<p class="empty">This migrated session has no semantic nodes.</p>`}
        </section>
      </section>
    `}}customElements.define("session-list",Kt);customElements.define("session-detail-view",Zt);const Oe=100;function R(i,e){return i instanceof Error?i.message:e}function Gt(i){return i==="overview"||i==="client"||i==="provider"||i==="raw"}function te(){return{query:"",provider_id:"",status:"",errors_only:!1}}function Yt(i){return new Date(i).toISOString().slice(0,10)}class Qt extends w{static properties={active_view:{type:String},info:{attribute:!1},requests:{attribute:!1},request_days:{attribute:!1},selected_day:{type:String},selected_request:{attribute:!1},selected_request_id:{type:String},selected_request_row_id:{type:String},selected_request_detail:{attribute:!1},request_list_state:{type:String},request_list_error:{type:String},request_detail_state:{type:String},request_detail_error:{type:String},next_cursor:{type:String},loading_more:{type:Boolean},load_more_error:{type:String},search_query:{type:String},provider_id:{type:String},status_filter:{type:String},errors_only:{type:Boolean},applied_filters:{attribute:!1},active_detail_tab:{type:String},timezone:{type:String},request_days_loading:{type:Boolean},request_days_error:{type:String},sessions:{attribute:!1},selected_session:{attribute:!1},selected_session_detail:{attribute:!1},selected_session_usage:{attribute:!1},sessions_loading:{type:Boolean},sessions_error:{type:String},session_search_query:{type:String},session_detail_state:{type:String},session_detail_error:{type:String},session_usage_state:{type:String},session_usage_error:{type:String},selected_session_node_id:{type:String},selected_session_node_detail:{attribute:!1},session_node_state:{type:String},session_node_error:{type:String}};request_load_id=0;request_detail_load_id=0;session_detail_load_id=0;session_usage_load_id=0;session_node_load_id=0;session_list_load_id=0;request_days_load_id=0;sessions_loaded=!1;requested_request_id;requested_request_row_id;requested_session_id;requested_session_node_id;request_rows_context;request_controller;request_detail_controller;session_list_controller;session_list_load;session_detail_controller;session_usage_controller;session_node_controller;navigation_workflow_id=0;popstate_handler=()=>{this.restoreFromHistory()};constructor(){super(),this.active_view="requests",this.requests=[],this.request_days=[],this.sessions=[],this.request_list_state="idle",this.request_detail_state="idle",this.search_query="",this.provider_id="",this.status_filter="",this.errors_only=!1,this.applied_filters=te(),this.active_detail_tab="overview",this.timezone="local",this.loading_more=!1,this.request_days_loading=!1,this.sessions_loading=!1,this.session_search_query="",this.session_detail_state="idle",this.session_usage_state="idle",this.session_node_state="idle"}createRenderRoot(){return this}connectedCallback(){super.connectedCallback(),this.restoreUrlState(),window.addEventListener("popstate",this.popstate_handler),this.loadInitialData()}disconnectedCallback(){window.removeEventListener("popstate",this.popstate_handler),this.request_controller?.abort(),this.request_detail_controller?.abort(),this.session_list_controller?.abort(),this.session_detail_controller?.abort(),this.session_usage_controller?.abort(),this.session_node_controller?.abort(),super.disconnectedCallback()}restoreUrlState(){const e=new URLSearchParams(window.location.search);this.active_view=e.get("view")==="sessions"?"sessions":"requests";const t=e.get("day");this.selected_day=t&&/^\d{4}-\d{2}-\d{2}$/.test(t)?t:void 0,this.search_query=e.get("query")??"",this.provider_id=e.get("provider_id")??"";const s=e.get("status")??"";this.status_filter=/^\d{3}$/.test(s)?s:"",this.errors_only=e.get("errors_only")==="true"||e.get("errors_only")==="1",this.applied_filters={query:this.search_query,provider_id:this.provider_id,status:this.status_filter,errors_only:this.errors_only},this.requested_request_id=e.get("request_id")??void 0;const o=e.get("row_id");this.requested_request_row_id=o&&/^-?\d+$/.test(o)?o:void 0;const n=e.get("tab");this.active_detail_tab=Gt(n)?n:"overview",this.requested_session_id=e.has("session_id")?e.get("session_id")??"":void 0,this.requested_session_node_id=e.get("node_id")??void 0,this.timezone=e.get("timezone")==="utc"?"utc":"local"}selectedRequestDay(){return this.selected_request_detail?.day??this.selected_request?.day??this.selected_day}syncUrl(e="replace"){const t=new URLSearchParams;if(this.active_view==="sessions"){t.set("view","sessions");const n=this.selected_session?.session_id??this.requested_session_id;n!==void 0&&t.set("session_id",n),this.selected_session_node_id&&t.set("node_id",this.selected_session_node_id)}else{const n=this.selected_request_id?this.selectedRequestDay():this.selected_day;n&&t.set("day",n),this.applied_filters.query&&t.set("query",this.applied_filters.query),this.applied_filters.provider_id&&t.set("provider_id",this.applied_filters.provider_id),this.applied_filters.status&&t.set("status",this.applied_filters.status),this.applied_filters.errors_only&&t.set("errors_only","true"),this.selected_request_id&&(t.set("request_id",this.selected_request_id),this.selected_request_row_id&&t.set("row_id",this.selected_request_row_id),t.set("tab",this.active_detail_tab))}t.set("timezone",this.timezone);const s=t.toString(),o=`${window.location.pathname}${s?`?${s}`:""}`;`${window.location.pathname}${window.location.search}`!==o&&(e==="push"?window.history.pushState(null,"",o):window.history.replaceState(null,"",o))}async loadInitialData(){const e=++this.navigation_workflow_id;this.loadInfo(),await this.loadUrlState(e)}async restoreFromHistory(){const e=++this.navigation_workflow_id;this.request_controller?.abort(),this.request_detail_controller?.abort(),this.session_detail_controller?.abort(),this.session_node_controller?.abort(),this.resetRequestSelection(),this.resetSessionSelection(),this.restoreUrlState(),this.active_view==="requests"&&(this.requests=[],this.next_cursor=void 0,this.request_rows_context=void 0),await this.loadUrlState(e)}async loadUrlState(e){const t=this.requested_request_id,s=this.requested_request_row_id;if(this.active_view==="sessions"){const n=this.requested_session_id,r=this.requested_session_node_id;if(!await this.ensureSessionsLoaded()||e!==this.navigation_workflow_id||n===void 0)return;await this.loadSession(n,this.sessions.find(l=>l.session_id===n),!1,null,r);return}this.loadRequestDays();let o;if(this.selected_day?o=await this.loadRequests():(o=await this.loadLatestRequests(),o&&this.selected_day&&this.hasAppliedFilters()&&(o=await this.loadRequests())),!(!o||e!==this.navigation_workflow_id)&&t&&this.selected_day){const n=this.requests.find(r=>r.request_id===t&&(!s||r.row_id===s));await this.loadRequestDetail(this.selected_day,t,s??n?.row_id,n,!1,null)}}async loadInfo(){try{this.info=await m("/api/info")}catch{this.info=void 0}}async loadLatestRequests(){this.request_controller?.abort();const e=new AbortController;this.request_controller=e;const t=++this.request_load_id;this.requests=[],this.next_cursor=void 0,this.request_rows_context=void 0,this.request_list_state="loading",this.request_list_error=void 0;try{const s=await m(`/api/requests/latest?limit=${Oe}`,e.signal);return t!==this.request_load_id||this.request_controller!==e?!1:(this.selected_day=s.day??void 0,this.requests=s.requests,this.next_cursor=s.next_cursor??void 0,this.request_rows_context=this.requestContext(this.selected_day,te()),this.request_list_state="ready",this.syncUrl(),!0)}catch(s){return t===this.request_load_id&&!x(s)&&(this.request_list_state="error",this.request_list_error=R(s,"Unable to load recent requests")),!1}finally{this.request_controller===e&&(this.request_controller=void 0)}}requestContext(e=this.selected_day,t=this.applied_filters){return e?JSON.stringify([e,t.query,t.provider_id,t.status,t.errors_only]):void 0}requestParams(e,t,s){const o=new URLSearchParams({day:e,limit:String(Oe)});return t.query&&o.set("query",t.query),t.provider_id&&o.set("provider_id",t.provider_id),t.status&&o.set("status",t.status),t.errors_only&&o.set("errors_only","true"),s&&o.set("cursor",s),o}async loadRequests(e=!1){const t=this.selected_day;if(!t)return this.request_list_state="idle",this.requests=[],this.next_cursor=void 0,this.request_rows_context=void 0,!1;const s={...this.applied_filters},o=this.requestContext(t,s),n=e?this.next_cursor:void 0;if(e&&(!n||this.request_rows_context!==o))return!1;this.request_controller?.abort();const r=new AbortController;this.request_controller=r;const c=++this.request_load_id;e?(this.loading_more=!0,this.load_more_error=void 0):(this.loading_more=!1,this.request_rows_context!==o&&(this.requests=[],this.next_cursor=void 0,this.request_rows_context=void 0),this.request_list_state="loading",this.request_list_error=void 0,this.load_more_error=void 0);try{const l=await m(`/api/requests?${this.requestParams(t,s,n).toString()}`,r.signal);if(c!==this.request_load_id||this.request_controller!==r||this.requestContext()!==o)return!1;if(e){const d=new Set(this.requests.map(u=>B(u)));this.requests=[...this.requests,...l.requests.filter(u=>!d.has(B(u)))]}else this.requests=l.requests;return this.next_cursor=l.next_cursor??void 0,this.request_rows_context=o,this.request_list_state="ready",!0}catch(l){return c!==this.request_load_id||x(l)||(l instanceof Fe&&l.status===503&&this.markRequestDayUnavailable(t),e?this.load_more_error=R(l,"Unable to load more requests"):(this.request_list_state="error",this.request_list_error=R(l,"Unable to load requests"))),!1}finally{c===this.request_load_id&&(this.loading_more=!1),this.request_controller===r&&(this.request_controller=void 0)}}async loadRequestDays(){const e=++this.request_days_load_id;this.request_days_loading=!0,this.request_days_error=void 0;try{const t=await m("/api/request-days");e===this.request_days_load_id&&(this.request_days=t)}catch(t){e===this.request_days_load_id&&(this.request_days_error=R(t,"Unable to load request day states"))}finally{e===this.request_days_load_id&&(this.request_days_loading=!1)}}markRequestDayUnavailable(e){this.request_days.some(t=>t.day===e)?this.request_days=this.request_days.map(t=>t.day===e?{...t,state:"unavailable"}:t):this.request_days=[{day:e,state:"unavailable"},...this.request_days]}resetRequestSelection(){this.request_detail_controller?.abort(),this.request_detail_controller=void 0,this.request_detail_load_id+=1,this.selected_request=void 0,this.selected_request_id=void 0,this.selected_request_row_id=void 0,this.selected_request_detail=void 0,this.request_detail_state="idle",this.request_detail_error=void 0,this.active_detail_tab="overview"}resetSessionSelection(){this.session_detail_controller?.abort(),this.session_usage_controller?.abort(),this.session_node_controller?.abort(),this.session_detail_controller=void 0,this.session_usage_controller=void 0,this.session_node_controller=void 0,this.session_detail_load_id+=1,this.session_usage_load_id+=1,this.session_node_load_id+=1,this.requested_session_id=void 0,this.requested_session_node_id=void 0,this.selected_session=void 0,this.selected_session_detail=void 0,this.selected_session_usage=void 0,this.selected_session_node_id=void 0,this.selected_session_node_detail=void 0,this.session_detail_state="idle",this.session_detail_error=void 0,this.session_usage_state="idle",this.session_usage_error=void 0,this.session_node_state="idle",this.session_node_error=void 0}async closeRequestDetail(){const e=this.selected_request_row_id&&this.selectedRequestDay()?B({day:this.selectedRequestDay(),row_id:this.selected_request_row_id}):void 0;if(++this.navigation_workflow_id,this.resetRequestSelection(),this.syncUrl("push"),!e||!window.matchMedia("(max-width: 680px)").matches)return;await this.updateComplete,[...this.querySelectorAll("request-list [data-request-key]")].find(s=>s.dataset.requestKey===e)?.focus()}async closeSessionDetail(){const e=this.selected_session?.session_id??this.requested_session_id;if(++this.navigation_workflow_id,this.resetSessionSelection(),this.syncUrl("push"),e===void 0||!window.matchMedia("(max-width: 680px)").matches)return;await this.updateComplete,[...this.querySelectorAll("session-list [data-session-id]")].find(s=>s.dataset.sessionId===e)?.focus()}async loadRequestDetail(e,t,s,o,n,r="replace"){this.request_detail_controller?.abort();const c=new AbortController;this.request_detail_controller=c;const l=++this.request_detail_load_id;this.selected_day=e,this.selected_request=o,this.selected_request_id=t,this.selected_request_row_id=s,n||(this.selected_request_detail=void 0),this.request_detail_state="loading",this.request_detail_error=void 0,r&&this.syncUrl(r);try{const d=new URLSearchParams({day:e,request_id:t});s&&d.set("row_id",s);const u=await m(`/api/request?${d.toString()}`,c.signal);if(l===this.request_detail_load_id&&this.request_detail_controller===c){const h=this.selected_request_row_id!==u.row_id;return this.selected_request_detail=u,this.selected_request_row_id=u.row_id,this.request_detail_state="ready",(r||h)&&this.syncUrl("replace"),!0}return!1}catch(d){return l===this.request_detail_load_id&&!x(d)&&(this.request_detail_state="error",this.request_detail_error=R(d,"Unable to load request detail")),!1}finally{this.request_detail_controller===c&&(this.request_detail_controller=void 0)}}async selectRequest(e){++this.navigation_workflow_id;const t=this.selected_request_id===e.request_id&&this.selected_request_detail?.day===e.day&&this.selected_request_detail.row_id===e.row_id,s=this.loadRequestDetail(e.day,e.request_id,e.row_id,e,t,"push");window.matchMedia("(max-width: 680px)").matches&&(await this.updateComplete,this.querySelector("request-detail-view .mobile-back-button")?.focus()),await s&&window.matchMedia("(max-width: 680px)").matches&&(await this.updateComplete,this.querySelector("request-detail-view .mobile-back-button")?.focus())}retryRequestDetail(){const e=this.selected_request_detail?.day??this.selected_request?.day??this.selected_day;e&&this.selected_request_id&&this.loadRequestDetail(e,this.selected_request_id,this.selected_request_row_id,this.selected_request,!!this.selected_request_detail,null)}selectDay(e){e!==this.selected_day&&(++this.navigation_workflow_id,this.selected_day=e,this.resetRequestSelection(),this.syncUrl("push"),this.loadRequests())}pickerDays(){return!this.selected_day||this.request_days.some(e=>e.day===this.selected_day)?this.request_days:[{day:this.selected_day,state:"available"},...this.request_days]}adjacentAvailableDay(e){const t=this.pickerDays().filter(o=>o.state==="available").map(o=>o.day).sort();if(!this.selected_day)return;const s=t.indexOf(this.selected_day);return s<0?void 0:t[s+e]}submitFilters(e){e.preventDefault(),++this.navigation_workflow_id,this.applied_filters={query:this.search_query.trim(),provider_id:this.provider_id.trim(),status:this.status_filter.trim(),errors_only:this.errors_only},this.resetRequestSelection(),this.syncUrl("push"),this.loadRequests()}clearFilters(){this.search_query="",this.provider_id="",this.status_filter="",this.errors_only=!1,this.applied_filters=te(),++this.navigation_workflow_id,this.resetRequestSelection(),this.syncUrl("push"),this.loadRequests()}hasAppliedFilters(){return!!(this.applied_filters.query||this.applied_filters.provider_id||this.applied_filters.status||this.applied_filters.errors_only)}filtersChanged(){return this.search_query.trim()!==this.applied_filters.query||this.provider_id.trim()!==this.applied_filters.provider_id||this.status_filter.trim()!==this.applied_filters.status||this.errors_only!==this.applied_filters.errors_only}providerOptions(){const e=new Set(this.requests.flatMap(t=>t.provider_id?[t.provider_id]:[]));return this.applied_filters.provider_id&&e.add(this.applied_filters.provider_id),[...e].sort()}ensureSessionsLoaded(e=!1){if(this.sessions_loaded&&!e)return Promise.resolve(!0);if(this.session_list_load&&!e)return this.session_list_load;this.session_list_controller?.abort();const t=new AbortController;this.session_list_controller=t;const s=++this.session_list_load_id;this.sessions_loading=!0,this.sessions_error=void 0;const o=this.loadSessions(t,s);return this.session_list_load=o,o}async loadSessions(e,t){try{const s=await m("/api/sessions?limit=100",e.signal);return t!==this.session_list_load_id||this.session_list_controller!==e?!1:(this.sessions=s,this.sessions_loaded=!0,this.selected_session&&(this.selected_session=s.find(o=>o.session_id===this.selected_session?.session_id)??this.selected_session),!0)}catch(s){return t===this.session_list_load_id&&!x(s)&&(this.sessions_error=R(s,"Unable to load sessions")),!1}finally{t===this.session_list_load_id&&this.session_list_controller===e&&(this.session_list_controller=void 0,this.session_list_load=void 0,this.sessions_loading=!1)}}retrySessions(){const e=++this.navigation_workflow_id;this.sessions_loaded=!1,this.retrySessionsAndRestore(e)}async retrySessionsAndRestore(e){if(!await this.ensureSessionsLoaded(!0)||e!==this.navigation_workflow_id||this.active_view!=="sessions")return;const s=this.selected_session?.session_id??this.requested_session_id;if(s===void 0)return;const o=this.selected_session_node_id??this.requested_session_node_id;await this.loadSession(s,this.sessions.find(n=>n.session_id===s),this.selected_session_detail?.session.session_id===s,null,o)}async refreshSessions(){const e=this.navigation_workflow_id,t=this.selected_session?.session_id??this.requested_session_id,s=this.selected_session_node_id,o=await this.ensureSessionsLoaded(!0),n=this.selected_session?.session_id??this.requested_session_id;o&&e===this.navigation_workflow_id&&t!==void 0&&n===t&&this.selected_session_node_id===s&&await this.loadSession(t,this.sessions.find(r=>r.session_id===t),!0,null,s)}filteredSessions(){const e=this.session_search_query.trim().toLocaleLowerCase();return e?this.sessions.filter(t=>[t.session_id,t.model,t.provider_id,t.account_id,t.endpoint,t.status===null?null:String(t.status)].some(s=>s?.toLocaleLowerCase().includes(e))):this.sessions}async loadSessionUsage(e,t){this.session_usage_controller?.abort();const s=new AbortController;this.session_usage_controller=s;const o=++this.session_usage_load_id;t||(this.selected_session_usage=void 0),this.session_usage_state="loading",this.session_usage_error=void 0;try{const n=new URLSearchParams({session_id:e}),r=await m(`/api/session-usage?${n.toString()}`,s.signal);return o===this.session_usage_load_id&&this.session_usage_controller===s?(this.selected_session_usage=r??void 0,this.session_usage_state="ready",!0):!1}catch(n){return o===this.session_usage_load_id&&!x(n)&&(this.session_usage_state="error",this.session_usage_error=R(n,"Unable to load session usage")),!1}finally{this.session_usage_controller===s&&(this.session_usage_controller=void 0)}}async loadSession(e,t,s,o="push",n){this.session_detail_controller?.abort(),this.session_node_controller?.abort();const r=new AbortController;this.session_detail_controller=r;const c=++this.session_detail_load_id,l=++this.session_node_load_id;this.requested_session_id=e,this.requested_session_node_id=n,this.selected_session=t,s||(this.selected_session_detail=void 0,this.selected_session_node_detail=void 0,this.selected_session_node_id=void 0,this.session_node_state="idle",this.session_node_error=void 0),this.loadSessionUsage(e,s),this.session_detail_state="loading",this.session_detail_error=void 0,o&&this.syncUrl(o);try{const d=new URLSearchParams({session_id:e,limit:"500"}),u=await m(`/api/session?${d.toString()}`,r.signal);if(c===this.session_detail_load_id&&this.session_detail_controller===r){if(this.selected_session=u.session,this.selected_session_detail=u,this.sessions=this.sessions.map(h=>h.session_id===u.session.session_id?u.session:h),this.session_detail_state="ready",l!==this.session_node_load_id)return!0;if(n){const h=u.nodes.find(p=>p.node_id===n);this.loadSessionNode(h??n,!1,"replace")}else this.selected_session_node_id=void 0,this.selected_session_node_detail=void 0,this.session_node_state="idle",this.syncUrl("replace");return!0}return!1}catch(d){return c===this.session_detail_load_id&&!x(d)&&(this.session_detail_state="error",this.session_detail_error=R(d,"Unable to load semantic session")),!1}finally{this.session_detail_controller===r&&(this.session_detail_controller=void 0)}}async loadSessionNode(e,t,s="push"){const o=this.selected_session?.session_id??this.requested_session_id;if(o===void 0)return!1;this.session_node_controller?.abort();const n=new AbortController;this.session_node_controller=n;const r=++this.session_node_load_id,c=typeof e=="string"?e:e.node_id;this.requested_session_node_id=c,this.selected_session_node_id=c,t||(this.selected_session_node_detail=void 0),this.session_node_state="loading",this.session_node_error=void 0,s&&this.syncUrl(s);try{const l=new URLSearchParams({session_id:o,node_id:c}),d=await m(`/api/session-node?${l.toString()}`,n.signal);return r===this.session_node_load_id&&this.session_node_controller===n?(this.selected_session_node_detail=d,this.session_node_state="ready",this.syncUrl("replace"),!0):!1}catch(l){return r===this.session_node_load_id&&!x(l)&&(this.session_node_state="error",this.session_node_error=R(l,"Unable to load semantic node content")),!1}finally{this.session_node_controller===n&&(this.session_node_controller=void 0)}}async selectSession(e){const t=++this.navigation_workflow_id;if(!await this.loadSession(e.session_id,e,!1,"push")||t!==this.navigation_workflow_id||this.active_view!=="sessions"||this.selected_session_detail?.session.session_id!==e.session_id||!window.matchMedia("(max-width: 680px)").matches)return;await this.updateComplete;const o=this.querySelector("session-detail-view");await o?.updateComplete,t===this.navigation_workflow_id&&this.active_view==="sessions"&&this.selected_session_detail?.session.session_id===e.session_id&&o?.querySelector(".mobile-back-button")?.focus()}collapseSessionNode(e="push"){this.session_node_controller?.abort(),this.session_node_controller=void 0,++this.session_node_load_id,this.requested_session_node_id=void 0,this.selected_session_node_id=void 0,this.selected_session_node_detail=void 0,this.session_node_state="idle",this.session_node_error=void 0,e&&this.syncUrl(e)}selectSessionNode(e){if(e.node_id===this.selected_session_node_id){this.collapseSessionNode();return}this.loadSessionNode(e,!1,"push")}retrySessionDetail(){const e=this.selected_session?.session_id??this.requested_session_id;e!==void 0&&this.loadSession(e,this.selected_session,!!this.selected_session_detail,null,this.selected_session_node_id??this.requested_session_node_id)}retrySessionUsage(){const e=this.selected_session?.session_id??this.requested_session_id;e!==void 0&&this.loadSessionUsage(e,!!this.selected_session_usage)}retrySessionNode(){const e=this.selected_session_detail?.nodes.find(t=>t.node_id===this.selected_session_node_id);(e??this.selected_session_node_id)&&this.loadSessionNode(e??this.selected_session_node_id,!!this.selected_session_node_detail,null)}async openSession(e){++this.navigation_workflow_id,this.setActiveView("sessions",!1,null),await this.ensureSessionsLoaded();const t=this.sessions.find(s=>s.session_id===e);await this.loadSession(e,t,!1,"push")}async openRequestFromSession(e){++this.navigation_workflow_id,this.setActiveView("requests",!1,null),this.search_query="",this.provider_id="",this.status_filter="",this.errors_only=!1,this.applied_filters=te(),this.selected_day=Yt(e.ts),this.resetRequestSelection(),this.loadRequestDays(),this.loadRequests(),!await this.loadRequestDetail(this.selected_day,e.request_id,void 0,void 0,!1,"push")&&this.request_detail_state==="error"&&this.request_detail_error==="request not found"&&(this.request_detail_error="Request history is unavailable; semantic session data is still retained.")}async loadRequestsView(){this.loadRequestDays(),this.selected_day?await this.loadRequests():await this.loadLatestRequests()}setActiveView(e,t=!0,s="push"){s==="push"&&++this.navigation_workflow_id,this.active_view=e,s&&this.syncUrl(s),t&&(e==="sessions"?this.ensureSessionsLoaded():this.request_list_state==="idle"&&this.loadRequestsView())}setTimezone(e){this.timezone=e,this.syncUrl("push")}setDetailTab(e){this.active_detail_tab=e,this.syncUrl("push")}renderDayPicker(){const e=this.pickerDays(),t=this.adjacentAvailableDay(-1),s=this.adjacentAvailableDay(1);return a`
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
            @change=${o=>this.selectDay(o.target.value)}
          >
            ${this.selected_day?_:a`<option value="">No request day</option>`}
            ${e.map(o=>a`
                <option value=${o.day} ?disabled=${o.state!=="available"}>
                  ${o.day}${o.state==="empty"?" · empty":o.state==="unavailable"?" · unavailable":""}
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
          ${this.hasAppliedFilters()?a`<button type="button" class="text-button" @click=${this.clearFilters}>Clear</button>`:_}
        </form>
        ${this.request_days_error?a`<p class="toolbar-warning" role="status">Day scan: ${this.request_days_error}</p>`:_}
      </section>
    `}renderRequestSidebar(){const e=this.requests.length>0;return a`
      <div class="list-pane" aria-busy=${String(this.request_list_state==="loading")}>
        <header class="list-pane-header">
          <div>
            <strong>Requests</strong>
            <span>${this.requests.length.toLocaleString()} loaded${this.next_cursor?" · more available":""}</span>
          </div>
          ${this.hasAppliedFilters()?a`<span class="filter-indicator">Filtered</span>`:_}
        </header>
        ${this.request_list_state==="loading"?a`
              <div class="inline-state" role="status">
                <span class="spinner" aria-hidden="true"></span>${e?"Refreshing requests…":"Loading requests…"}
              </div>
            `:_}
        ${this.request_list_state==="error"?a`
              <div class="inline-error" role="alert">
                <span>${this.request_list_error}</span>
                <button type="button" @click=${()=>{this.loadRequests()}}>Retry</button>
              </div>
            `:_}
        ${e?a`
              <request-list
                .requests=${this.requests}
                .selected_key=${this.selectedRequestDay()&&this.selected_request_row_id?B({day:this.selectedRequestDay(),row_id:this.selected_request_row_id}):void 0}
                .timezone=${this.timezone}
                @request-select=${t=>{this.selectRequest(D(t))}}
              ></request-list>
            `:this.request_list_state==="ready"?a`<p class="empty">No persisted requests match these filters.</p>`:this.request_list_state==="idle"?a`<p class="empty">Choose an available request day.</p>`:_}
        ${this.load_more_error?a`
              <div class="inline-error load-more-error" role="alert">
                <span>${this.load_more_error}</span>
                <button type="button" @click=${()=>{this.loadRequests(!0)}}>Retry</button>
              </div>
            `:_}
        ${this.next_cursor&&e?a`
              <div class="list-footer">
                <button type="button" class="secondary-button" ?disabled=${this.loading_more} @click=${()=>{this.loadRequests(!0)}}>
                  ${this.loading_more?"Loading…":"Load more"}
                </button>
              </div>
            `:e&&this.request_list_state==="ready"?a`<p class="end-of-list">End of loaded day</p>`:_}
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
          ${this.session_search_query?a`<span class="filter-indicator">Filtered</span>`:_}
        </header>
        ${this.sessions_loading?a`
              <div class="inline-state" role="status">
                <span class="spinner" aria-hidden="true"></span>${t?"Refreshing sessions…":"Loading sessions…"}
              </div>
            `:_}
        ${this.sessions_error?a`
              <div class="inline-error" role="alert">
                <span>${this.sessions_error}</span>
                <button type="button" @click=${this.retrySessions}>Retry</button>
              </div>
            `:_}
        ${e.length>0?a`
              <session-list
                .sessions=${e}
                .selected_session_id=${this.selected_session?.session_id??this.requested_session_id}
                .timezone=${this.timezone}
                @session-select=${s=>{this.selectSession(D(s))}}
              ></session-list>
            `:this.sessions_loaded&&this.session_search_query?a`<p class="empty">No recent sessions match this filter.</p>`:this.sessions_loaded?a`
                  <div class="empty empty-session-list">
                    <strong>No semantic sessions available</strong>
                    <span>The gateway records successful sessions here when session persistence is enabled.</span>
                  </div>
                `:_}
        ${t&&!this.session_search_query?a`<p class="end-of-list">${this.sessions.length===100?"Latest 100 sessions":"End of recent sessions"}</p>`:_}
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
        @session-node-select=${e=>this.selectSessionNode(D(e))}
        @open-request=${e=>{this.openRequestFromSession(D(e))}}
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
                    @detail-tab-change=${s=>this.setDetailTab(D(s))}
                    @open-session=${s=>{this.openSession(D(s))}}
                  ></request-detail-view>
                `:this.renderSessionDetail()}
          </article>
        </section>
      </main>
    `}}customElements.define("inspect-app",Qt);
