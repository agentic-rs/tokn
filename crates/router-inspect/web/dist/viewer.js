(function(){const e=document.createElement("link").relList;if(e&&e.supports&&e.supports("modulepreload"))return;for(const i of document.querySelectorAll('link[rel="modulepreload"]'))s(i);new MutationObserver(i=>{for(const r of i)if(r.type==="childList")for(const n of r.addedNodes)n.tagName==="LINK"&&n.rel==="modulepreload"&&s(n)}).observe(document,{childList:!0,subtree:!0});function t(i){const r={};return i.integrity&&(r.integrity=i.integrity),i.referrerPolicy&&(r.referrerPolicy=i.referrerPolicy),i.crossOrigin==="use-credentials"?r.credentials="include":i.crossOrigin==="anonymous"?r.credentials="omit":r.credentials="same-origin",r}function s(i){if(i.ep)return;i.ep=!0;const r=t(i);fetch(i.href,r)}})();const J=globalThis,ie=J.ShadowRoot&&(J.ShadyCSS===void 0||J.ShadyCSS.nativeShadow)&&"adoptedStyleSheets"in Document.prototype&&"replace"in CSSStyleSheet.prototype,we=Symbol(),ae=new WeakMap;let Ce=class{constructor(e,t,s){if(this._$cssResult$=!0,s!==we)throw Error("CSSResult is not constructable. Use `unsafeCSS` or `css` instead.");this.cssText=e,this.t=t}get styleSheet(){let e=this.o;const t=this.t;if(ie&&e===void 0){const s=t!==void 0&&t.length===1;s&&(e=ae.get(t)),e===void 0&&((this.o=e=new CSSStyleSheet).replaceSync(this.cssText),s&&ae.set(t,e))}return e}toString(){return this.cssText}};const xe=o=>new Ce(typeof o=="string"?o:o+"",void 0,we),Ue=(o,e)=>{if(ie)o.adoptedStyleSheets=e.map(t=>t instanceof CSSStyleSheet?t:t.styleSheet);else for(const t of e){const s=document.createElement("style"),i=J.litNonce;i!==void 0&&s.setAttribute("nonce",i),s.textContent=t.cssText,o.appendChild(s)}},de=ie?o=>o:o=>o instanceof CSSStyleSheet?(e=>{let t="";for(const s of e.cssRules)t+=s.cssText;return xe(t)})(o):o;const{is:Le,defineProperty:Pe,getOwnPropertyDescriptor:De,getOwnPropertyNames:Ne,getOwnPropertySymbols:Te,getPrototypeOf:Oe}=Object,K=globalThis,le=K.trustedTypes,ze=le?le.emptyScript:"",Me=K.reactiveElementPolyfillSupport,T=(o,e)=>o,ee={toAttribute(o,e){switch(e){case Boolean:o=o?ze:null;break;case Object:case Array:o=o==null?o:JSON.stringify(o)}return o},fromAttribute(o,e){let t=o;switch(e){case Boolean:t=o!==null;break;case Number:t=o===null?null:Number(o);break;case Object:case Array:try{t=JSON.parse(o)}catch{t=null}}return t}},qe=(o,e)=>!Le(o,e),ce={attribute:!0,type:String,converter:ee,reflect:!1,useDefault:!1,hasChanged:qe};Symbol.metadata??=Symbol("metadata"),K.litPropertyMetadata??=new WeakMap;let C=class extends HTMLElement{static addInitializer(e){this._$Ei(),(this.l??=[]).push(e)}static get observedAttributes(){return this.finalize(),this._$Eh&&[...this._$Eh.keys()]}static createProperty(e,t=ce){if(t.state&&(t.attribute=!1),this._$Ei(),this.prototype.hasOwnProperty(e)&&((t=Object.create(t)).wrapped=!0),this.elementProperties.set(e,t),!t.noAccessor){const s=Symbol(),i=this.getPropertyDescriptor(e,s,t);i!==void 0&&Pe(this.prototype,e,i)}}static getPropertyDescriptor(e,t,s){const{get:i,set:r}=De(this.prototype,e)??{get(){return this[t]},set(n){this[t]=n}};return{get:i,set(n){const c=i?.call(this);r?.call(this,n),this.requestUpdate(e,c,s)},configurable:!0,enumerable:!0}}static getPropertyOptions(e){return this.elementProperties.get(e)??ce}static _$Ei(){if(this.hasOwnProperty(T("elementProperties")))return;const e=Oe(this);e.finalize(),e.l!==void 0&&(this.l=[...e.l]),this.elementProperties=new Map(e.elementProperties)}static finalize(){if(this.hasOwnProperty(T("finalized")))return;if(this.finalized=!0,this._$Ei(),this.hasOwnProperty(T("properties"))){const t=this.properties,s=[...Ne(t),...Te(t)];for(const i of s)this.createProperty(i,t[i])}const e=this[Symbol.metadata];if(e!==null){const t=litPropertyMetadata.get(e);if(t!==void 0)for(const[s,i]of t)this.elementProperties.set(s,i)}this._$Eh=new Map;for(const[t,s]of this.elementProperties){const i=this._$Eu(t,s);i!==void 0&&this._$Eh.set(i,t)}this.elementStyles=this.finalizeStyles(this.styles)}static finalizeStyles(e){const t=[];if(Array.isArray(e)){const s=new Set(e.flat(1/0).reverse());for(const i of s)t.unshift(de(i))}else e!==void 0&&t.push(de(e));return t}static _$Eu(e,t){const s=t.attribute;return s===!1?void 0:typeof s=="string"?s:typeof e=="string"?e.toLowerCase():void 0}constructor(){super(),this._$Ep=void 0,this.isUpdatePending=!1,this.hasUpdated=!1,this._$Em=null,this._$Ev()}_$Ev(){this._$ES=new Promise(e=>this.enableUpdating=e),this._$AL=new Map,this._$E_(),this.requestUpdate(),this.constructor.l?.forEach(e=>e(this))}addController(e){(this._$EO??=new Set).add(e),this.renderRoot!==void 0&&this.isConnected&&e.hostConnected?.()}removeController(e){this._$EO?.delete(e)}_$E_(){const e=new Map,t=this.constructor.elementProperties;for(const s of t.keys())this.hasOwnProperty(s)&&(e.set(s,this[s]),delete this[s]);e.size>0&&(this._$Ep=e)}createRenderRoot(){const e=this.shadowRoot??this.attachShadow(this.constructor.shadowRootOptions);return Ue(e,this.constructor.elementStyles),e}connectedCallback(){this.renderRoot??=this.createRenderRoot(),this.enableUpdating(!0),this._$EO?.forEach(e=>e.hostConnected?.())}enableUpdating(e){}disconnectedCallback(){this._$EO?.forEach(e=>e.hostDisconnected?.())}attributeChangedCallback(e,t,s){this._$AK(e,s)}_$ET(e,t){const s=this.constructor.elementProperties.get(e),i=this.constructor._$Eu(e,s);if(i!==void 0&&s.reflect===!0){const r=(s.converter?.toAttribute!==void 0?s.converter:ee).toAttribute(t,s.type);this._$Em=e,r==null?this.removeAttribute(i):this.setAttribute(i,r),this._$Em=null}}_$AK(e,t){const s=this.constructor,i=s._$Eh.get(e);if(i!==void 0&&this._$Em!==i){const r=s.getPropertyOptions(i),n=typeof r.converter=="function"?{fromAttribute:r.converter}:r.converter?.fromAttribute!==void 0?r.converter:ee;this._$Em=i;const c=n.fromAttribute(t,r.type);this[i]=c??this._$Ej?.get(i)??c,this._$Em=null}}requestUpdate(e,t,s,i=!1,r){if(e!==void 0){const n=this.constructor;if(i===!1&&(r=this[e]),s??=n.getPropertyOptions(e),!((s.hasChanged??qe)(r,t)||s.useDefault&&s.reflect&&r===this._$Ej?.get(e)&&!this.hasAttribute(n._$Eu(e,s))))return;this.C(e,t,s)}this.isUpdatePending===!1&&(this._$ES=this._$EP())}C(e,t,{useDefault:s,reflect:i,wrapped:r},n){s&&!(this._$Ej??=new Map).has(e)&&(this._$Ej.set(e,n??t??this[e]),r!==!0||n!==void 0)||(this._$AL.has(e)||(this.hasUpdated||s||(t=void 0),this._$AL.set(e,t)),i===!0&&this._$Em!==e&&(this._$Eq??=new Set).add(e))}async _$EP(){this.isUpdatePending=!0;try{await this._$ES}catch(t){Promise.reject(t)}const e=this.scheduleUpdate();return e!=null&&await e,!this.isUpdatePending}scheduleUpdate(){return this.performUpdate()}performUpdate(){if(!this.isUpdatePending)return;if(!this.hasUpdated){if(this.renderRoot??=this.createRenderRoot(),this._$Ep){for(const[i,r]of this._$Ep)this[i]=r;this._$Ep=void 0}const s=this.constructor.elementProperties;if(s.size>0)for(const[i,r]of s){const{wrapped:n}=r,c=this[i];n!==!0||this._$AL.has(i)||c===void 0||this.C(i,void 0,r,c)}}let e=!1;const t=this._$AL;try{e=this.shouldUpdate(t),e?(this.willUpdate(t),this._$EO?.forEach(s=>s.hostUpdate?.()),this.update(t)):this._$EM()}catch(s){throw e=!1,this._$EM(),s}e&&this._$AE(t)}willUpdate(e){}_$AE(e){this._$EO?.forEach(t=>t.hostUpdated?.()),this.hasUpdated||(this.hasUpdated=!0,this.firstUpdated(e)),this.updated(e)}_$EM(){this._$AL=new Map,this.isUpdatePending=!1}get updateComplete(){return this.getUpdateComplete()}getUpdateComplete(){return this._$ES}shouldUpdate(e){return!0}update(e){this._$Eq&&=this._$Eq.forEach(t=>this._$ET(t,this[t])),this._$EM()}updated(e){}firstUpdated(e){}};C.elementStyles=[],C.shadowRootOptions={mode:"open"},C[T("elementProperties")]=new Map,C[T("finalized")]=new Map,Me?.({ReactiveElement:C}),(K.reactiveElementVersions??=[]).push("2.1.2");const oe=globalThis,he=o=>o,W=oe.trustedTypes,ue=W?W.createPolicy("lit-html",{createHTML:o=>o}):void 0,Se="$lit$",w=`lit$${Math.random().toFixed(9).slice(2)}$`,Ae="?"+w,He=`<${Ae}>`,k=document,H=()=>k.createComment(""),I=o=>o===null||typeof o!="object"&&typeof o!="function",re=Array.isArray,Ie=o=>re(o)||typeof o?.[Symbol.iterator]=="function",Y=`[ 	
\f\r]`,P=/<(?:(!--|\/[^a-zA-Z])|(\/?[a-zA-Z][^>\s]*)|(\/?$))/g,_e=/-->/g,pe=/>/g,q=RegExp(`>|${Y}(?:([^\\s"'>=/]+)(${Y}*=${Y}*(?:[^ 	
\f\r"'\`<>=]|("|')|))|$)`,"g"),ye=/'/g,fe=/"/g,Re=/^(?:script|style|textarea|title)$/i,je=o=>(e,...t)=>({_$litType$:o,strings:e,values:t}),a=je(1),U=Symbol.for("lit-noChange"),l=Symbol.for("lit-nothing"),$e=new WeakMap,R=k.createTreeWalker(k,129);function ke(o,e){if(!re(o)||!o.hasOwnProperty("raw"))throw Error("invalid template strings array");return ue!==void 0?ue.createHTML(e):e}const Be=(o,e)=>{const t=o.length-1,s=[];let i,r=e===2?"<svg>":e===3?"<math>":"",n=P;for(let c=0;c<t;c++){const d=o[c];let h,u,_=-1,p=0;for(;p<d.length&&(n.lastIndex=p,u=n.exec(d),u!==null);)p=n.lastIndex,n===P?u[1]==="!--"?n=_e:u[1]!==void 0?n=pe:u[2]!==void 0?(Re.test(u[2])&&(i=RegExp("</"+u[2],"g")),n=q):u[3]!==void 0&&(n=q):n===q?u[0]===">"?(n=i??P,_=-1):u[1]===void 0?_=-2:(_=n.lastIndex-u[2].length,h=u[1],n=u[3]===void 0?q:u[3]==='"'?fe:ye):n===fe||n===ye?n=q:n===_e||n===pe?n=P:(n=q,i=void 0);const m=n===q&&o[c+1].startsWith("/>")?" ":"";r+=n===P?d+He:_>=0?(s.push(h),d.slice(0,_)+Se+d.slice(_)+w+m):d+w+(_===-2?c:m)}return[ke(o,r+(o[t]||"<?>")+(e===2?"</svg>":e===3?"</math>":"")),s]};class j{constructor({strings:e,_$litType$:t},s){let i;this.parts=[];let r=0,n=0;const c=e.length-1,d=this.parts,[h,u]=Be(e,t);if(this.el=j.createElement(h,s),R.currentNode=this.el.content,t===2||t===3){const _=this.el.content.firstChild;_.replaceWith(..._.childNodes)}for(;(i=R.nextNode())!==null&&d.length<c;){if(i.nodeType===1){if(i.hasAttributes())for(const _ of i.getAttributeNames())if(_.endsWith(Se)){const p=u[n++],m=i.getAttribute(_).split(w),F=/([.?@])?(.*)/.exec(p);d.push({type:1,index:r,name:F[2],strings:m,ctor:F[1]==="."?Ve:F[1]==="?"?Je:F[1]==="@"?We:Z}),i.removeAttribute(_)}else _.startsWith(w)&&(d.push({type:6,index:r}),i.removeAttribute(_));if(Re.test(i.tagName)){const _=i.textContent.split(w),p=_.length-1;if(p>0){i.textContent=W?W.emptyScript:"";for(let m=0;m<p;m++)i.append(_[m],H()),R.nextNode(),d.push({type:2,index:++r});i.append(_[p],H())}}}else if(i.nodeType===8)if(i.data===Ae)d.push({type:2,index:r});else{let _=-1;for(;(_=i.data.indexOf(w,_+1))!==-1;)d.push({type:7,index:r}),_+=w.length-1}r++}}static createElement(e,t){const s=k.createElement("template");return s.innerHTML=e,s}}function L(o,e,t=o,s){if(e===U)return e;let i=s!==void 0?t._$Co?.[s]:t._$Cl;const r=I(e)?void 0:e._$litDirective$;return i?.constructor!==r&&(i?._$AO?.(!1),r===void 0?i=void 0:(i=new r(o),i._$AT(o,t,s)),s!==void 0?(t._$Co??=[])[s]=i:t._$Cl=i),i!==void 0&&(e=L(o,i._$AS(o,e.values),i,s)),e}class Fe{constructor(e,t){this._$AV=[],this._$AN=void 0,this._$AD=e,this._$AM=t}get parentNode(){return this._$AM.parentNode}get _$AU(){return this._$AM._$AU}u(e){const{el:{content:t},parts:s}=this._$AD,i=(e?.creationScope??k).importNode(t,!0);R.currentNode=i;let r=R.nextNode(),n=0,c=0,d=s[0];for(;d!==void 0;){if(n===d.index){let h;d.type===2?h=new B(r,r.nextSibling,this,e):d.type===1?h=new d.ctor(r,d.name,d.strings,this,e):d.type===6&&(h=new Ke(r,this,e)),this._$AV.push(h),d=s[++c]}n!==d?.index&&(r=R.nextNode(),n++)}return R.currentNode=k,i}p(e){let t=0;for(const s of this._$AV)s!==void 0&&(s.strings!==void 0?(s._$AI(e,s,t),t+=s.strings.length-2):s._$AI(e[t])),t++}}class B{get _$AU(){return this._$AM?._$AU??this._$Cv}constructor(e,t,s,i){this.type=2,this._$AH=l,this._$AN=void 0,this._$AA=e,this._$AB=t,this._$AM=s,this.options=i,this._$Cv=i?.isConnected??!0}get parentNode(){let e=this._$AA.parentNode;const t=this._$AM;return t!==void 0&&e?.nodeType===11&&(e=t.parentNode),e}get startNode(){return this._$AA}get endNode(){return this._$AB}_$AI(e,t=this){e=L(this,e,t),I(e)?e===l||e==null||e===""?(this._$AH!==l&&this._$AR(),this._$AH=l):e!==this._$AH&&e!==U&&this._(e):e._$litType$!==void 0?this.$(e):e.nodeType!==void 0?this.T(e):Ie(e)?this.k(e):this._(e)}O(e){return this._$AA.parentNode.insertBefore(e,this._$AB)}T(e){this._$AH!==e&&(this._$AR(),this._$AH=this.O(e))}_(e){this._$AH!==l&&I(this._$AH)?this._$AA.nextSibling.data=e:this.T(k.createTextNode(e)),this._$AH=e}$(e){const{values:t,_$litType$:s}=e,i=typeof s=="number"?this._$AC(e):(s.el===void 0&&(s.el=j.createElement(ke(s.h,s.h[0]),this.options)),s);if(this._$AH?._$AD===i)this._$AH.p(t);else{const r=new Fe(i,this),n=r.u(this.options);r.p(t),this.T(n),this._$AH=r}}_$AC(e){let t=$e.get(e.strings);return t===void 0&&$e.set(e.strings,t=new j(e)),t}k(e){re(this._$AH)||(this._$AH=[],this._$AR());const t=this._$AH;let s,i=0;for(const r of e)i===t.length?t.push(s=new B(this.O(H()),this.O(H()),this,this.options)):s=t[i],s._$AI(r),i++;i<t.length&&(this._$AR(s&&s._$AB.nextSibling,i),t.length=i)}_$AR(e=this._$AA.nextSibling,t){for(this._$AP?.(!1,!0,t);e!==this._$AB;){const s=he(e).nextSibling;he(e).remove(),e=s}}setConnected(e){this._$AM===void 0&&(this._$Cv=e,this._$AP?.(e))}}class Z{get tagName(){return this.element.tagName}get _$AU(){return this._$AM._$AU}constructor(e,t,s,i,r){this.type=1,this._$AH=l,this._$AN=void 0,this.element=e,this.name=t,this._$AM=i,this.options=r,s.length>2||s[0]!==""||s[1]!==""?(this._$AH=Array(s.length-1).fill(new String),this.strings=s):this._$AH=l}_$AI(e,t=this,s,i){const r=this.strings;let n=!1;if(r===void 0)e=L(this,e,t,0),n=!I(e)||e!==this._$AH&&e!==U,n&&(this._$AH=e);else{const c=e;let d,h;for(e=r[0],d=0;d<r.length-1;d++)h=L(this,c[s+d],t,d),h===U&&(h=this._$AH[d]),n||=!I(h)||h!==this._$AH[d],h===l?e=l:e!==l&&(e+=(h??"")+r[d+1]),this._$AH[d]=h}n&&!i&&this.j(e)}j(e){e===l?this.element.removeAttribute(this.name):this.element.setAttribute(this.name,e??"")}}class Ve extends Z{constructor(){super(...arguments),this.type=3}j(e){this.element[this.name]=e===l?void 0:e}}class Je extends Z{constructor(){super(...arguments),this.type=4}j(e){this.element.toggleAttribute(this.name,!!e&&e!==l)}}class We extends Z{constructor(e,t,s,i,r){super(e,t,s,i,r),this.type=5}_$AI(e,t=this){if((e=L(this,e,t,0)??l)===U)return;const s=this._$AH,i=e===l&&s!==l||e.capture!==s.capture||e.once!==s.once||e.passive!==s.passive,r=e!==l&&(s===l||i);i&&this.element.removeEventListener(this.name,this,s),r&&this.element.addEventListener(this.name,this,e),this._$AH=e}handleEvent(e){typeof this._$AH=="function"?this._$AH.call(this.options?.host??this.element,e):this._$AH.handleEvent(e)}}class Ke{constructor(e,t,s){this.element=e,this.type=6,this._$AN=void 0,this._$AM=t,this.options=s}get _$AU(){return this._$AM._$AU}_$AI(e){L(this,e)}}const Ze=oe.litHtmlPolyfillSupport;Ze?.(j,B),(oe.litHtmlVersions??=[]).push("3.3.3");const Ye=(o,e,t)=>{const s=t?.renderBefore??e;let i=s._$litPart$;if(i===void 0){const r=t?.renderBefore??null;s._$litPart$=i=new B(e.insertBefore(H(),r),r,void 0,t??{})}return i._$AI(o),i};const ne=globalThis;class f extends C{constructor(){super(...arguments),this.renderOptions={host:this},this._$Do=void 0}createRenderRoot(){const e=super.createRenderRoot();return this.renderOptions.renderBefore??=e.firstChild,e}update(e){const t=this.render();this.hasUpdated||(this.renderOptions.isConnected=this.isConnected),super.update(e),this._$Do=Ye(t,this.renderRoot,this.renderOptions)}connectedCallback(){super.connectedCallback(),this._$Do?.setConnected(!0)}disconnectedCallback(){super.disconnectedCallback(),this._$Do?.setConnected(!1)}render(){return U}}f._$litElement$=!0,f.finalized=!0,ne.litElementHydrateSupport?.({LitElement:f});const Ge=ne.litElementPolyfillSupport;Ge?.({LitElement:f});(ne.litElementVersions??=[]).push("4.2.2");class Ee extends Error{status;constructor(e,t){super(t),this.name="HttpError",this.status=e}}async function $(o,e){const t=await fetch(o,{cache:"no-store",signal:e});if(!t.ok){const s=await t.json().catch(()=>({}));throw new Ee(t.status,s.error??`Request failed (${t.status})`)}return t.json()}function A(o){return o instanceof Error&&o.name==="AbortError"}function O(o,e,t=!1){const s=t?{hour:"2-digit",minute:"2-digit",second:"2-digit"}:{dateStyle:"medium",timeStyle:"medium"};return e==="utc"&&(s.timeZone="UTC"),new Intl.DateTimeFormat(void 0,s).format(new Date(o))}function Qe(o,e){const t=new Date(o),s=new Date,i=e==="utc"?t.getUTCFullYear():t.getFullYear(),r=e==="utc"?s.getUTCFullYear():s.getFullYear(),n={month:"short",day:"numeric",hour:"2-digit",minute:"2-digit"};return i!==r&&(n.year="numeric"),e==="utc"&&(n.timeZone="UTC"),new Intl.DateTimeFormat(void 0,n).format(t)}function Xe(o,e){const t=Math.max(0,e-o);if(t<1e3)return`${t.toLocaleString()} ms`;const s=Math.floor(t/1e3);if(s<60)return`${s}s`;const i=Math.floor(s/60);if(i<60)return`${i}m ${s%60}s`;const r=Math.floor(i/60);return r<24?`${r}h ${i%60}m`:`${Math.floor(r/24)}d ${r%24}h`}function x(o){return`${o.day}:${o.row_id}`}function v(o,e=10){return o?o.length>e?`…${o.slice(-e)}`:o:"—"}function et(o){const e=o.inbound_req_url??o.endpoint;return z(e)}function ve(o){const e=o.toLowerCase().replaceAll("_","-");return e==="authorization"||e==="password"||e==="code"||e==="signature"||e==="sig"||e.includes("api-key")||e.includes("access-key")||e.includes("token")||e.includes("secret")||e.includes("credential")}function z(o){if(!o)return"unknown endpoint";try{const e=new URL(o,window.location.origin);for(const t of new Set(e.searchParams.keys()))ve(t)&&e.searchParams.set(t,"REDACTED");return`${e.pathname}${e.search}`}catch{return o.replace(/([?&]([^=&]+)=)([^&]*)/g,(e,t,s)=>{let i=s;try{i=decodeURIComponent(s)}catch{}return ve(i)?`${t}REDACTED`:e})}}function tt(o){if(o.request_error)return{label:"ERR",tone:"error",title:o.request_error};const e=o.inbound_resp_status??o.outbound_resp_status??o.status;if(e===null)return{label:"—",tone:"neutral",title:"No response status persisted"};const t=o.inbound_resp_status!==null?"Client response":o.outbound_resp_status!==null?"Provider response":"Request";return e>=400?{label:String(e),tone:"error",title:`${t}: ${e}`}:e>=300?{label:String(e),tone:"warning",title:`${t}: ${e}`}:{label:String(e),tone:"success",title:`${t}: ${e}`}}function st(o){const e=o.status;return e===null?{label:"—",tone:"neutral",title:"No status stored for the current session head"}:e>=400?{label:String(e),tone:"error",title:`Current head status: ${e}`}:e>=300?{label:String(e),tone:"warning",title:`Current head status: ${e}`}:{label:String(e),tone:"success",title:`Current head status: ${e}`}}function D(o){return o.detail}function y(o,e){const t=o[e];return typeof t=="string"?t:void 0}function V(o,e){const t=o[e];return typeof t=="number"?t:void 0}const G="••••••••";function Q(o){const e=o.toLowerCase().replaceAll("_","-");return e==="authorization"||e==="proxy-authorization"||e==="cookie"||e==="set-cookie"||e.includes("api-key")||e.includes("token")||e.includes("secret")}function M(o){if(Array.isArray(o))return o.length===2&&typeof o[0]=="string"&&Q(o[0])?[o[0],G]:o.map(e=>M(e));if(o!==null&&typeof o=="object")return Object.fromEntries(Object.entries(o).map(([e,t])=>[e,Q(e)?G:M(t)]));if(typeof o=="string")try{return M(JSON.parse(o))}catch{return o.replace(/^([^:\r\n]+)(:\s*)(.*)$/gm,(e,t,s)=>Q(t.trim())?`${t}${s}${G}`:e)}return o}function te(o){return Array.isArray(o)?o.map(e=>te(e)):o!==null&&typeof o=="object"?Object.fromEntries(Object.entries(o).map(([e,t])=>[e,it(e)?M(t):te(t)])):o}function it(o){const e=o.replace(/([a-z0-9])([A-Z])/g,"$1_$2").toLowerCase().replace(/[-\s]+/g,"_");return e==="headers"||e.endsWith("_headers")}function se(o){return Array.isArray(o)?o.map(e=>se(e)):o!==null&&typeof o=="object"?Object.fromEntries(Object.entries(o).map(([e,t])=>[e,e.toLowerCase().endsWith("_url")&&typeof t=="string"?z(t):se(t)])):o}function ot(o){if(typeof o=="string")try{return JSON.stringify(JSON.parse(o),null,2)}catch{return o}return JSON.stringify(o,null,2)??String(o)}function rt(o){if(Array.isArray(o))return`${o.length} item${o.length===1?"":"s"}`;if(o!==null&&typeof o=="object"){const e=Object.keys(o).length;return`${e} field${e===1?"":"s"}`}return typeof o=="string"?`${new Blob([o]).size.toLocaleString()} bytes`:typeof o}class nt extends f{static properties={label:{type:String},value:{attribute:!1},load_url:{type:String},is_headers:{type:Boolean},redact_record_headers:{type:Boolean},open:{type:Boolean,state:!0},wrap:{type:Boolean,state:!0},revealed:{type:Boolean,state:!0},copy_state:{type:String,state:!0},load_state:{type:String,state:!0},loaded_value:{attribute:!1,state:!0},error_message:{type:String,state:!0}};load_controller;copy_timeout;constructor(){super(),this.label="Payload",this.is_headers=!1,this.redact_record_headers=!1,this.open=!1,this.wrap=!0,this.revealed=!1,this.copy_state="idle",this.load_state="idle"}createRenderRoot(){return this}disconnectedCallback(){this.load_controller?.abort(),this.copy_timeout!==void 0&&window.clearTimeout(this.copy_timeout),super.disconnectedCallback()}willUpdate(e){!e.has("value")&&!e.has("load_url")||(this.load_controller?.abort(),this.load_controller=void 0,this.copy_timeout!==void 0&&(window.clearTimeout(this.copy_timeout),this.copy_timeout=void 0),this.open=!1,this.revealed=!1,this.copy_state="idle",this.load_state="idle",this.loaded_value=void 0,this.error_message=void 0)}effectiveValue(){return this.load_state==="ready"?this.loaded_value:this.value}displayedValue(){const e=this.effectiveValue(),t=this.redact_record_headers?se(e):e,s=this.revealed?t:this.redact_record_headers?te(t):this.is_headers?M(t):t;return ot(s)}toggleOpen(e){this.open=e.currentTarget.open,this.open&&this.value===void 0&&this.load_url&&this.load_state==="idle"&&this.loadPayload()}async loadPayload(){const e=this.load_url;if(!e)return;this.load_controller?.abort();const t=new AbortController;this.load_controller=t,this.load_state="loading",this.error_message=void 0;try{const s=await $(e,t.signal);if(this.load_controller!==t||this.load_url!==e)return;const i=new URL(e,window.location.origin).searchParams.get("field");if(!i||s.field!==i)throw new Error("Payload response did not match the requested field");this.loaded_value=s.value,this.load_state="ready"}catch(s){if(this.load_controller!==t||A(s))return;this.load_state="error",this.error_message=s instanceof Error?s.message:"Unable to load payload"}finally{this.load_controller===t&&(this.load_controller=void 0)}}async copyValue(){try{await navigator.clipboard.writeText(this.displayedValue()),this.copy_state="copied",this.copy_timeout!==void 0&&window.clearTimeout(this.copy_timeout),this.copy_timeout=window.setTimeout(()=>{this.copy_state="idle",this.copy_timeout=void 0},1500)}catch{this.copy_state="error"}}render(){if(!this.load_url&&(this.value===null||this.value===void 0||this.value===""))return l;const e=this.effectiveValue(),t=this.is_headers||this.redact_record_headers,s=this.load_state==="loading"?"Loading…":this.load_state==="error"?"Load failed":e===null?"No payload":e===void 0?"Load on open":rt(e);return a`
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
                          `:l}
                      <span class="payload-security-note">
                        ${t&&!this.revealed?"Sensitive headers redacted":""}
                      </span>
                    </div>
                    <pre class=${this.wrap?"wrap":"nowrap"}><code>${this.displayedValue()}</code></pre>
                  `:l}
      </details>
    `}}customElements.define("payload-panel",nt);const S=[{id:"overview",label:"Overview"},{id:"client",label:"Client"},{id:"provider",label:"Provider"},{id:"raw",label:"Raw"}];function E(o){return o==null||o===""?"—":typeof o=="boolean"?o?"Yes":"No":String(o)}function at(o){if(o!==null&&typeof o=="object"&&!Array.isArray(o))return o;if(typeof o=="string")try{const e=JSON.parse(o);return e!==null&&typeof e=="object"&&!Array.isArray(e)?e:void 0}catch{return}}function me(o,e,t){return at(o[e])?.[t]??o[t]}function b(o,e,t,s){return`/api/request-payload?${new URLSearchParams({day:o,request_id:e,row_id:t,field:s}).toString()}`}function be(o){return o===void 0?"neutral":o>=400?"error":o>=300?"warning":"success"}class dt extends f{static properties={detail:{attribute:!1},summary:{attribute:!1},state:{type:String},error_message:{type:String},active_tab:{type:String},timezone:{type:String}};createRenderRoot(){return this}openSession(e){this.dispatchEvent(new CustomEvent("open-session",{detail:e,bubbles:!0,composed:!0}))}retry(){this.dispatchEvent(new CustomEvent("detail-retry",{bubbles:!0,composed:!0}))}close(){this.dispatchEvent(new CustomEvent("detail-close",{bubbles:!0,composed:!0}))}selectTab(e){this.dispatchEvent(new CustomEvent("detail-tab-change",{detail:e,bubbles:!0,composed:!0}))}tabKeydown(e){const t=S.findIndex(n=>n.id===this.active_tab);let s;if(e.key==="ArrowRight"?s=(t+1)%S.length:e.key==="ArrowLeft"?s=(t-1+S.length)%S.length:e.key==="Home"?s=0:e.key==="End"&&(s=S.length-1),s===void 0)return;e.preventDefault();const i=S[s];this.selectTab(i.id),this.querySelectorAll("[role=tab]")[s]?.focus()}renderOverview(e){const t=V(e,"ts"),s=me(e,"ctx_json","latency_ms"),i=me(e,"params_json","stream"),r=[["Timestamp",t===void 0?void 0:O(t,this.timezone)],["Storage day",this.detail?.day],["Endpoint",e.endpoint],["Model",e.model],["Provider",e.provider_id],["Account",e.account_id],["Latency",typeof s=="number"?`${s} ms`:s],["Streaming",i]],n=V(e,"inbound_resp_status"),c=V(e,"outbound_resp_status"),d=V(e,"status");return a`
      <section class="flow-grid" aria-label="Request flow">
        <div>
          <span>Client request</span>
          <strong>${y(e,"inbound_req_method")??"—"}</strong>
        </div>
        <span class="flow-arrow" aria-hidden="true">→</span>
        <div>
          <span>Provider response</span>
          <strong class="status-text ${be(c)}">${E(c)}</strong>
        </div>
        <span class="flow-arrow" aria-hidden="true">→</span>
        <div>
          <span>Client response</span>
          <strong class="status-text ${be(n??d)}">
            ${E(n??d)}
          </strong>
        </div>
      </section>
      <dl class="metadata-grid">
        ${r.map(([h,u])=>a`
            <div>
              <dt>${h}</dt>
              <dd title=${E(u)}>${E(u)}</dd>
            </div>
          `)}
      </dl>
      <div class="payload-stack">
        <payload-panel label="Request parameters" .value=${e.params_json}></payload-panel>
        <payload-panel label="Usage" .value=${e.usage_json}></payload-panel>
        <payload-panel label="Request context" .value=${e.ctx_json}></payload-panel>
      </div>
    `}renderClient(e,t,s,i){return a`
      <section class="payload-group">
        <div class="payload-group-heading">
          <div><span class="direction-label">Incoming</span><h3>Client request</h3></div>
          <span>${y(e,"inbound_req_method")??"—"} ${z(y(e,"inbound_req_url"))}</span>
        </div>
        <payload-panel
          label="Request headers"
          .value=${e.inbound_req_headers}
          .load_url=${b(t,s,i,"inbound_req_headers")}
          .is_headers=${!0}
        ></payload-panel>
        <payload-panel
          label="Request body"
          .value=${e.inbound_req_body}
          .load_url=${b(t,s,i,"inbound_req_body")}
        ></payload-panel>
      </section>
      <section class="payload-group">
        <div class="payload-group-heading">
          <div><span class="direction-label">Outgoing</span><h3>Client response</h3></div>
          <span>Status ${E(e.inbound_resp_status??e.status)}</span>
        </div>
        <payload-panel
          label="Response headers"
          .value=${e.inbound_resp_headers}
          .load_url=${b(t,s,i,"inbound_resp_headers")}
          .is_headers=${!0}
        ></payload-panel>
        <payload-panel
          label="Response body"
          .value=${e.inbound_resp_body}
          .load_url=${b(t,s,i,"inbound_resp_body")}
        ></payload-panel>
      </section>
    `}renderProvider(e,t,s,i){return a`
      <section class="payload-group">
        <div class="payload-group-heading">
          <div><span class="direction-label">Outgoing</span><h3>Provider request</h3></div>
          <span>${y(e,"outbound_req_method")??"—"} ${z(y(e,"outbound_req_url"))}</span>
        </div>
        <payload-panel
          label="Request headers"
          .value=${e.outbound_req_headers}
          .load_url=${b(t,s,i,"outbound_req_headers")}
          .is_headers=${!0}
        ></payload-panel>
        <payload-panel
          label="Request body"
          .value=${e.outbound_req_body}
          .load_url=${b(t,s,i,"outbound_req_body")}
        ></payload-panel>
      </section>
      <section class="payload-group">
        <div class="payload-group-heading">
          <div><span class="direction-label">Incoming</span><h3>Provider response</h3></div>
          <span>Status ${E(e.outbound_resp_status)}</span>
        </div>
        <payload-panel
          label="Response headers"
          .value=${e.outbound_resp_headers}
          .load_url=${b(t,s,i,"outbound_resp_headers")}
          .is_headers=${!0}
        ></payload-panel>
        <payload-panel
          label="Response body"
          .value=${e.outbound_resp_body}
          .load_url=${b(t,s,i,"outbound_resp_body")}
        ></payload-panel>
      </section>
    `}renderTab(e,t,s,i){switch(this.active_tab){case"client":return this.renderClient(e,t,s,i);case"provider":return this.renderProvider(e,t,s,i);case"raw":return a`
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
        `:a`<section class="detail-state"><p>Select a request to inspect its route, payloads, and responses.</p></section>`;const e=this.detail.request,t=y(e,"request_id")??this.summary?.request_id??"unknown id",s=y(e,"session_id")??this.summary?.session_id??void 0,i=y(e,"inbound_req_method")??this.summary?.inbound_req_method??"REQUEST",r=z(y(e,"inbound_req_url")??this.summary?.inbound_req_url??y(e,"endpoint"));return a`
      <section class="detail-content">
        <header class="detail-header">
          <button type="button" class="mobile-back-button" @click=${this.close}>← Requests</button>
          <div class="detail-title">
            <p class="eyebrow">request · ${v(t)}</p>
            <h2><span>${i}</span> ${r}</h2>
            <p class="muted" title=${t}>${t}</p>
          </div>
          <div class="detail-actions">
            ${s?a`<button type="button" class="secondary-button" @click=${()=>this.openSession(s)}>Open session</button>`:l}
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
        ${this.state==="loading"?a`<div class="inline-state" role="status"><span class="spinner" aria-hidden="true"></span>Refreshing detail…</div>`:l}
        ${this.state==="error"?a`
              <div class="inline-error" role="alert">
                <span>${this.error_message}</span>
                <button type="button" @click=${this.retry}>Retry</button>
              </div>
            `:l}
        ${e.request_error?a`<div class="request-error" role="alert">${String(e.request_error)}</div>`:l}
        <div class="detail-tabs" role="tablist" aria-label="Request detail sections" @keydown=${this.tabKeydown}>
          ${S.map(n=>a`
              <button
                id="request-tab-${n.id}"
                type="button"
                role="tab"
                aria-selected=${String(this.active_tab===n.id)}
                aria-controls="request-panel-${n.id}"
                tabindex=${this.active_tab===n.id?"0":"-1"}
                @click=${()=>this.selectTab(n.id)}
              >
                ${n.label}
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
    `}}customElements.define("request-detail-view",dt);class lt extends f{static properties={requests:{attribute:!1},selected_key:{type:String},timezone:{type:String}};createRenderRoot(){return this}selectRequest(e){this.dispatchEvent(new CustomEvent("request-select",{detail:e,bubbles:!0,composed:!0}))}render(){const e=this.requests??[];return e.length===0?a`<p class="empty">No persisted requests match these filters.</p>`:a`
      <ul class="request-list" aria-label="Requests">
        ${e.map(t=>{const s=tt(t),i=this.selected_key===x(t),r=t.inbound_req_method??"REQUEST",n=et(t);return a`
            <li>
              <button
                type="button"
                class="request-row ${i?"selected":""}"
                data-request-key=${x(t)}
                aria-current=${i?"true":"false"}
                @click=${()=>this.selectRequest(t)}
              >
                <span class="request-row-time">${O(t.ts,this.timezone,!0)}</span>
                <span class="status ${s.tone}" title=${s.title}>${s.label}</span>
                <span class="request-row-main">
                  <span class="request-route"><strong>${r}</strong><span>${n}</span></span>
                  <span class="request-context">
                    <span>${t.model??"unknown model"}</span>
                    <span aria-hidden="true">·</span>
                    <span>${t.provider_id??"unknown provider"}</span>
                  </span>
                  <span class="request-identifiers">
                    <span title=${t.request_id}>req ${v(t.request_id)}</span>
                    ${t.session_id?a`<span title=${t.session_id}>session ${v(t.session_id)}</span>`:a`<span>no session</span>`}
                  </span>
                </span>
              </button>
            </li>
          `})}
      </ul>
    `}}customElements.define("request-list",lt);function ct(o){return o===null?{label:"—",tone:"neutral",title:"No response status stored"}:o>=400?{label:String(o),tone:"error",title:`Response status: ${o}`}:o>=300?{label:String(o),tone:"warning",title:`Response status: ${o}`}:{label:String(o),tone:"success",title:`Response status: ${o}`}}function ht(o){switch(o.toLowerCase()){case"assistant":return"assistant";case"system":case"developer":return"system";case"tool":case"function":return"tool";default:return"user"}}function ut(o){try{return JSON.stringify(o,null,2)??String(o)}catch{return String(o)}}function N(o){if(o<1024)return`${o.toLocaleString()} B`;const e=["KiB","MiB","GiB"];let t=o/1024,s=e[0];for(const i of e.slice(1)){if(t<1024)break;t/=1024,s=i}return`${t>=10?t.toFixed(0):t.toFixed(1)} ${s}`}function _t(o){switch(o){case"message_tree":return{direction:"Complete",title:"Input prefix",empty_message:"No semantic input was stored for this observation."};case"suffix_append":return{direction:"Appended",title:"Input delta",empty_message:"No new semantic input was stored for this node."};case"root_snapshot":return{direction:"Initial",title:"Input snapshot",empty_message:"No semantic input was stored for this root snapshot."};case"conflict_snapshot":return{direction:"Replaced",title:"Replacement snapshot",empty_message:"No semantic input was stored for this replacement snapshot."};default:return{direction:"Stored",title:"Node input",empty_message:"No semantic input was stored for this node."}}}class pt extends f{static properties={sessions:{attribute:!1},selected_session_id:{type:String},timezone:{type:String}};createRenderRoot(){return this}selectSession(e){this.dispatchEvent(new CustomEvent("session-select",{detail:e,bubbles:!0,composed:!0}))}render(){const e=this.sessions??[];return a`
      <ul class="session-list" aria-label="Sessions">
        ${e.map(t=>{const s=this.selected_session_id===t.session_id,i=st(t);return a`
            <li>
              <button
                type="button"
                class="session-row ${s?"selected":""}"
                data-session-id=${t.session_id}
                aria-current=${s?"true":"false"}
                @click=${()=>this.selectSession(t)}
              >
                <time datetime=${new Date(t.last_ts).toISOString()}>
                  ${Qe(t.last_ts,this.timezone)}
                </time>
                <span class="status ${i.tone}" title=${i.title}>${i.label}</span>
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
                    session ${v(t.session_id)}
                  </span>
                </span>
                <span class="session-row-chevron" aria-hidden="true">›</span>
              </button>
            </li>
          `})}
      </ul>
    `}}class yt extends f{static properties={detail:{attribute:!1},node_detail:{attribute:!1},state:{type:String},error_message:{type:String},node_state:{type:String},node_error_message:{type:String},selected_node_id:{type:String},timezone:{type:String}};createRenderRoot(){return this}close(){this.dispatchEvent(new CustomEvent("session-close",{bubbles:!0,composed:!0}))}retryDetail(){this.dispatchEvent(new CustomEvent("session-retry",{bubbles:!0,composed:!0}))}retryNode(){this.dispatchEvent(new CustomEvent("session-node-retry",{bubbles:!0,composed:!0}))}selectNode(e){this.dispatchEvent(new CustomEvent("session-node-select",{detail:e,bubbles:!0,composed:!0}))}renderPart(e){switch(e.content.encoding){case"text":{const t=e.content.value||a`<span class="faint">Empty text part</span>`,s=e.content.truncated?a`<p class="session-part-note">Preview truncated · ${N(e.byte_length)} stored</p>`:l;return a`<div class="session-part-text">${t}${s}</div>`}case"json":return a`
          <details class="session-structured-part">
            <summary>${e.part_type.replaceAll("_"," ")}</summary>
            <pre>${ut(e.content.value)}</pre>
          </details>
        `;case"binary":return a`
          <details class="session-structured-part">
            <summary>${e.part_type.replaceAll("_"," ")} · binary</summary>
            <p>${N(e.content.byte_length)} stored. Binary bytes are not returned to the viewer.</p>
          </details>
        `;case"omitted":return a`
          <details class="session-structured-part">
            <summary>${e.part_type.replaceAll("_"," ")} · omitted</summary>
            <p>
              ${N(e.byte_length)} ${e.content.original_encoding} content omitted after reaching the
              ${e.content.reason==="part_limit"?"per-part byte preview":"node content-size"} limit.
            </p>
          </details>
        `}}renderMessages(e,t){return e.length===0?a`<p class="session-message-empty">${t}</p>`:a`
      <div class="session-message-stack">
        ${e.map(s=>a`
          <article class="session-message ${ht(s.role)}">
            <header>
              <span>${s.role}</span>
              <span>
                ${s.parts.length.toLocaleString()}${s.parts.length===s.parts_total?"":` of ${s.parts_total.toLocaleString()}`} parts
                ${s.status===null?l:a` · status ${s.status}`}
              </span>
            </header>
            <div class="session-message-parts">
              ${s.parts.length>0?s.parts.map(i=>this.renderPart(i)):s.parts_total>0?a`
                      <p class="session-message-empty">
                        ${s.parts_total.toLocaleString()} stored parts were omitted from this bounded preview.
                      </p>
                    `:a`<p class="session-message-empty">No stored parts in this message.</p>`}
            </div>
          </article>
        `)}
      </div>
    `}nodeDomId(e,t){return`session-node-${e}-${encodeURIComponent(t)}`}renderLoadedNodeContent(e){const t=e.truncation,s=_t(e.node.reduction_kind),i=t.request_messages.messages_total-t.request_messages.messages_returned,r=t.response_messages.messages_total-t.response_messages.messages_returned,n=i>0||r>0||t.parts_omitted>0||t.content_parts_truncated>0||t.binary_parts_elided>0;return a`
      ${n?a`
            <div class="session-content-boundary" role="status">
              <strong>Bounded content preview</strong>
              <span>
                ${N(t.content_bytes_returned)} of
                ${N(t.content_bytes_total)} inline content returned
                ${i+r>0?` · ${(i+r).toLocaleString()} messages omitted`:""}
                ${t.parts_omitted>0?` · ${t.parts_omitted.toLocaleString()} parts omitted`:""}
                ${t.content_parts_truncated>0?` · ${t.content_parts_truncated.toLocaleString()} parts truncated`:""}
                ${t.binary_parts_elided>0?` · ${t.binary_parts_elided.toLocaleString()} binary parts represented as metadata`:""}
              </span>
            </div>
          `:l}
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
    `}renderNodeContent(e){if(this.selected_node_id!==e.node_id)return l;const t=this.node_detail?.node.node_id===e.node_id?this.node_detail:void 0,s=this.node_state==="loading"?a`<div class="inline-state"><span class="spinner" aria-hidden="true"></span>Loading semantic content…</div>`:this.node_state==="error"?a`
            <div class="inline-error" role="alert">
              <span>${this.node_error_message}</span>
              <button type="button" @click=${this.retryNode}>Retry</button>
            </div>
          `:t?this.renderLoadedNodeContent(t):l;return a`
      <section
        id=${this.nodeDomId("content",e.node_id)}
        class="session-node-content"
        aria-labelledby=${this.nodeDomId("trigger",e.node_id)}
        aria-live="polite"
        aria-busy=${String(this.node_state==="loading")}
      >
        ${s}
      </section>
    `}renderNode(e){const t=e.node_id===this.selected_node_id,s=ct(e.status),i=e.reduction_kind==="message_tree"?e.input_message_count:e.request_message_count,r=e.reduction_kind==="message_tree"?"input":"input delta",n=e.reduction_kind==="message_tree"?e.output_message_count:e.response_message_count,c=e.reduction_kind==="message_tree"?e.message_id?`message ${v(e.message_id)}`:"message unavailable":e.parent_node_id?`parent ${v(e.parent_node_id)}`:"root";return a`
      <li class="session-node ${t?"selected":""}">
        <span class="session-node-rail" aria-hidden="true"><span></span></span>
        <button
          id=${this.nodeDomId("trigger",e.node_id)}
          type="button"
          class="session-node-trigger"
          data-node-id=${e.node_id}
          aria-expanded=${String(t)}
          aria-controls=${t?this.nodeDomId("content",e.node_id):l}
          @click=${()=>this.selectNode(e)}
        >
          <span class="session-node-primary">
            <time datetime=${new Date(e.ts).toISOString()}>${O(e.ts,this.timezone)}</time>
            <span class="status ${s.tone}" title=${s.title}>${s.label}</span>
            ${e.is_head?a`<span class="head-badge">Current head</span>`:l}
          </span>
          <span class="session-node-title">
            <strong>${e.model??"Unknown model"}</strong>
            <span>${e.endpoint}</span>
          </span>
          <span class="session-node-context">
            <span>${e.provider_id??"unknown provider"}</span>
            <span aria-hidden="true">·</span>
            <span>${i.toLocaleString()} ${r}</span>
            <span aria-hidden="true">·</span>
            <span>${n.toLocaleString()} output</span>
          </span>
          <span class="session-node-id" title=${e.request_id}>
            request ${v(e.request_id)} · ${c}
          </span>
        </button>
        ${this.renderNodeContent(e)}
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
      `;const{session:e,nodes:t}=this.detail,s=[...t].reverse(),i=!!(this.selected_node_id&&t.some(d=>d.node_id===this.selected_node_id)),r=this.node_detail,n=!i&&r&&r.node.node_id===this.selected_node_id?r.node:void 0,c=e.model??"Unknown model";return a`
      <section class="detail-content session-detail-content">
        <header class="detail-header session-detail-header">
          <button type="button" class="mobile-back-button" @click=${this.close}>← Sessions</button>
          <div class="detail-title">
            <p class="eyebrow">session · ${v(e.session_id)}</p>
            <h2>${c}<span> on ${e.provider_id??"unknown provider"}</span></h2>
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
        ${this.state==="loading"?a`<div class="inline-state"><span class="spinner" aria-hidden="true"></span>Refreshing session…</div>`:l}
        ${this.state==="error"?a`
              <div class="inline-error" role="alert">
                <span>${this.error_message}</span>
                <button type="button" @click=${this.retryDetail}>Retry</button>
              </div>
            `:l}
        <dl class="session-metadata-grid">
          <div><dt>Semantic nodes</dt><dd>${e.request_count.toLocaleString()}</dd></div>
          <div><dt>Duration</dt><dd>${Xe(e.first_ts,e.last_ts)}</dd></div>
          <div><dt>First seen</dt><dd>${O(e.first_ts,this.timezone)}</dd></div>
          <div><dt>Last active</dt><dd>${O(e.last_ts,this.timezone)}</dd></div>
          <div><dt>Endpoint</dt><dd title=${e.endpoint??""}>${e.endpoint??"—"}</dd></div>
          <div><dt>Account</dt><dd title=${e.account_id??""}>${e.account_id??"—"}</dd></div>
        </dl>
        <section class="session-activity">
          <header class="session-section-header">
            <div>
              <p class="eyebrow">Recent semantic nodes</p>
              <h3>Session activity</h3>
            </div>
            <span>${t.length.toLocaleString()} loaded · latest first${this.detail.nodes_truncated?" · bounded":""}</span>
          </header>
          ${this.detail.nodes_truncated?a`<p class="session-truncation-note">Older nodes are omitted from this bounded viewer response.</p>`:l}
          ${this.selected_node_id?l:a`<p class="session-content-hint">Open a node to load its conversation content from <code>sessions.db</code>.</p>`}
          ${this.selected_node_id&&!i?a`
                <section class="session-linked-node" aria-label="Directly linked session node">
                  <header>
                    <div>
                      <p class="eyebrow">Direct link</p>
                      <h4>Node outside this activity snapshot</h4>
                    </div>
                    <span>${v(this.selected_node_id)}</span>
                  </header>
                  ${n?a`<ol class="session-node-list linked-node-list">${this.renderNode(n)}</ol>`:this.node_state==="loading"?a`
                          <div class="inline-state" role="status" aria-live="polite">
                            <span class="spinner" aria-hidden="true"></span>Loading linked node…
                          </div>
                        `:this.node_state==="error"?a`
                            <div class="inline-error" role="alert">
                              <span>${this.node_error_message}</span>
                              <button type="button" @click=${this.retryNode}>Retry</button>
                            </div>
                          `:l}
                </section>
              `:l}
          ${t.length>0?a`<ol class="session-node-list">${s.map(d=>this.renderNode(d))}</ol>`:a`<p class="empty">This migrated session has no semantic nodes.</p>`}
        </section>
      </section>
    `}}customElements.define("session-list",pt);customElements.define("session-detail-view",yt);const ge=100;function g(o,e){return o instanceof Error?o.message:e}function ft(o){return o==="overview"||o==="client"||o==="provider"||o==="raw"}function X(){return{query:"",provider_id:"",status:"",errors_only:!1}}class $t extends f{static properties={active_view:{type:String},info:{attribute:!1},requests:{attribute:!1},request_days:{attribute:!1},selected_day:{type:String},selected_request:{attribute:!1},selected_request_id:{type:String},selected_request_row_id:{type:String},selected_request_detail:{attribute:!1},request_list_state:{type:String},request_list_error:{type:String},request_detail_state:{type:String},request_detail_error:{type:String},next_cursor:{type:String},loading_more:{type:Boolean},load_more_error:{type:String},search_query:{type:String},provider_id:{type:String},status_filter:{type:String},errors_only:{type:Boolean},applied_filters:{attribute:!1},active_detail_tab:{type:String},timezone:{type:String},request_days_loading:{type:Boolean},request_days_error:{type:String},sessions:{attribute:!1},selected_session:{attribute:!1},selected_session_detail:{attribute:!1},sessions_loading:{type:Boolean},sessions_error:{type:String},session_search_query:{type:String},session_detail_state:{type:String},session_detail_error:{type:String},selected_session_node_id:{type:String},selected_session_node_detail:{attribute:!1},session_node_state:{type:String},session_node_error:{type:String}};request_load_id=0;request_detail_load_id=0;session_detail_load_id=0;session_node_load_id=0;session_list_load_id=0;request_days_load_id=0;sessions_loaded=!1;requested_request_id;requested_request_row_id;requested_session_id;requested_session_node_id;request_rows_context;request_controller;request_detail_controller;session_list_controller;session_list_load;session_detail_controller;session_node_controller;navigation_workflow_id=0;popstate_handler=()=>{this.restoreFromHistory()};constructor(){super(),this.active_view="requests",this.requests=[],this.request_days=[],this.sessions=[],this.request_list_state="idle",this.request_detail_state="idle",this.search_query="",this.provider_id="",this.status_filter="",this.errors_only=!1,this.applied_filters=X(),this.active_detail_tab="overview",this.timezone="local",this.loading_more=!1,this.request_days_loading=!1,this.sessions_loading=!1,this.session_search_query="",this.session_detail_state="idle",this.session_node_state="idle"}createRenderRoot(){return this}connectedCallback(){super.connectedCallback(),this.restoreUrlState(),window.addEventListener("popstate",this.popstate_handler),this.loadInitialData()}disconnectedCallback(){window.removeEventListener("popstate",this.popstate_handler),this.request_controller?.abort(),this.request_detail_controller?.abort(),this.session_list_controller?.abort(),this.session_detail_controller?.abort(),this.session_node_controller?.abort(),super.disconnectedCallback()}restoreUrlState(){const e=new URLSearchParams(window.location.search);this.active_view=e.get("view")==="sessions"?"sessions":"requests";const t=e.get("day");this.selected_day=t&&/^\d{4}-\d{2}-\d{2}$/.test(t)?t:void 0,this.search_query=e.get("query")??"",this.provider_id=e.get("provider_id")??"";const s=e.get("status")??"";this.status_filter=/^\d{3}$/.test(s)?s:"",this.errors_only=e.get("errors_only")==="true"||e.get("errors_only")==="1",this.applied_filters={query:this.search_query,provider_id:this.provider_id,status:this.status_filter,errors_only:this.errors_only},this.requested_request_id=e.get("request_id")??void 0;const i=e.get("row_id");this.requested_request_row_id=i&&/^-?\d+$/.test(i)?i:void 0;const r=e.get("tab");this.active_detail_tab=ft(r)?r:"overview",this.requested_session_id=e.has("session_id")?e.get("session_id")??"":void 0,this.requested_session_node_id=e.get("node_id")??void 0,this.timezone=e.get("timezone")==="utc"?"utc":"local"}selectedRequestDay(){return this.selected_request_detail?.day??this.selected_request?.day??this.selected_day}syncUrl(e="replace"){const t=new URLSearchParams;if(this.active_view==="sessions"){t.set("view","sessions");const r=this.selected_session?.session_id??this.requested_session_id;r!==void 0&&t.set("session_id",r),this.selected_session_node_id&&t.set("node_id",this.selected_session_node_id)}else{const r=this.selected_request_id?this.selectedRequestDay():this.selected_day;r&&t.set("day",r),this.applied_filters.query&&t.set("query",this.applied_filters.query),this.applied_filters.provider_id&&t.set("provider_id",this.applied_filters.provider_id),this.applied_filters.status&&t.set("status",this.applied_filters.status),this.applied_filters.errors_only&&t.set("errors_only","true"),this.selected_request_id&&(t.set("request_id",this.selected_request_id),this.selected_request_row_id&&t.set("row_id",this.selected_request_row_id),t.set("tab",this.active_detail_tab))}t.set("timezone",this.timezone);const s=t.toString(),i=`${window.location.pathname}${s?`?${s}`:""}`;`${window.location.pathname}${window.location.search}`!==i&&(e==="push"?window.history.pushState(null,"",i):window.history.replaceState(null,"",i))}async loadInitialData(){const e=++this.navigation_workflow_id;this.loadInfo(),await this.loadUrlState(e)}async restoreFromHistory(){const e=++this.navigation_workflow_id;this.request_controller?.abort(),this.request_detail_controller?.abort(),this.session_detail_controller?.abort(),this.session_node_controller?.abort(),this.resetRequestSelection(),this.resetSessionSelection(),this.restoreUrlState(),this.active_view==="requests"&&(this.requests=[],this.next_cursor=void 0,this.request_rows_context=void 0),await this.loadUrlState(e)}async loadUrlState(e){const t=this.requested_request_id,s=this.requested_request_row_id;if(this.active_view==="sessions"){const r=this.requested_session_id,n=this.requested_session_node_id;if(!await this.ensureSessionsLoaded()||e!==this.navigation_workflow_id||r===void 0)return;await this.loadSession(r,this.sessions.find(d=>d.session_id===r),!1,null,n);return}this.loadRequestDays();let i;if(this.selected_day?i=await this.loadRequests():(i=await this.loadLatestRequests(),i&&this.selected_day&&this.hasAppliedFilters()&&(i=await this.loadRequests())),!(!i||e!==this.navigation_workflow_id)&&t&&this.selected_day){const r=this.requests.find(n=>n.request_id===t&&(!s||n.row_id===s));await this.loadRequestDetail(this.selected_day,t,s??r?.row_id,r,!1,null)}}async loadInfo(){try{this.info=await $("/api/info")}catch{this.info=void 0}}async loadLatestRequests(){this.request_controller?.abort();const e=new AbortController;this.request_controller=e;const t=++this.request_load_id;this.requests=[],this.next_cursor=void 0,this.request_rows_context=void 0,this.request_list_state="loading",this.request_list_error=void 0;try{const s=await $(`/api/requests/latest?limit=${ge}`,e.signal);return t!==this.request_load_id||this.request_controller!==e?!1:(this.selected_day=s.day??void 0,this.requests=s.requests,this.next_cursor=s.next_cursor??void 0,this.request_rows_context=this.requestContext(this.selected_day,X()),this.request_list_state="ready",this.syncUrl(),!0)}catch(s){return t===this.request_load_id&&!A(s)&&(this.request_list_state="error",this.request_list_error=g(s,"Unable to load recent requests")),!1}finally{this.request_controller===e&&(this.request_controller=void 0)}}requestContext(e=this.selected_day,t=this.applied_filters){return e?JSON.stringify([e,t.query,t.provider_id,t.status,t.errors_only]):void 0}requestParams(e,t,s){const i=new URLSearchParams({day:e,limit:String(ge)});return t.query&&i.set("query",t.query),t.provider_id&&i.set("provider_id",t.provider_id),t.status&&i.set("status",t.status),t.errors_only&&i.set("errors_only","true"),s&&i.set("cursor",s),i}async loadRequests(e=!1){const t=this.selected_day;if(!t)return this.request_list_state="idle",this.requests=[],this.next_cursor=void 0,this.request_rows_context=void 0,!1;const s={...this.applied_filters},i=this.requestContext(t,s),r=e?this.next_cursor:void 0;if(e&&(!r||this.request_rows_context!==i))return!1;this.request_controller?.abort();const n=new AbortController;this.request_controller=n;const c=++this.request_load_id;e?(this.loading_more=!0,this.load_more_error=void 0):(this.loading_more=!1,this.request_rows_context!==i&&(this.requests=[],this.next_cursor=void 0,this.request_rows_context=void 0),this.request_list_state="loading",this.request_list_error=void 0,this.load_more_error=void 0);try{const d=await $(`/api/requests?${this.requestParams(t,s,r).toString()}`,n.signal);if(c!==this.request_load_id||this.request_controller!==n||this.requestContext()!==i)return!1;if(e){const h=new Set(this.requests.map(u=>x(u)));this.requests=[...this.requests,...d.requests.filter(u=>!h.has(x(u)))]}else this.requests=d.requests;return this.next_cursor=d.next_cursor??void 0,this.request_rows_context=i,this.request_list_state="ready",!0}catch(d){return c!==this.request_load_id||A(d)||(d instanceof Ee&&d.status===503&&this.markRequestDayUnavailable(t),e?this.load_more_error=g(d,"Unable to load more requests"):(this.request_list_state="error",this.request_list_error=g(d,"Unable to load requests"))),!1}finally{c===this.request_load_id&&(this.loading_more=!1),this.request_controller===n&&(this.request_controller=void 0)}}async loadRequestDays(){const e=++this.request_days_load_id;this.request_days_loading=!0,this.request_days_error=void 0;try{const t=await $("/api/request-days");e===this.request_days_load_id&&(this.request_days=t)}catch(t){e===this.request_days_load_id&&(this.request_days_error=g(t,"Unable to load request day states"))}finally{e===this.request_days_load_id&&(this.request_days_loading=!1)}}markRequestDayUnavailable(e){this.request_days.some(t=>t.day===e)?this.request_days=this.request_days.map(t=>t.day===e?{...t,state:"unavailable"}:t):this.request_days=[{day:e,state:"unavailable"},...this.request_days]}resetRequestSelection(){this.request_detail_controller?.abort(),this.request_detail_controller=void 0,this.request_detail_load_id+=1,this.selected_request=void 0,this.selected_request_id=void 0,this.selected_request_row_id=void 0,this.selected_request_detail=void 0,this.request_detail_state="idle",this.request_detail_error=void 0,this.active_detail_tab="overview"}resetSessionSelection(){this.session_detail_controller?.abort(),this.session_node_controller?.abort(),this.session_detail_controller=void 0,this.session_node_controller=void 0,this.session_detail_load_id+=1,this.session_node_load_id+=1,this.requested_session_id=void 0,this.requested_session_node_id=void 0,this.selected_session=void 0,this.selected_session_detail=void 0,this.selected_session_node_id=void 0,this.selected_session_node_detail=void 0,this.session_detail_state="idle",this.session_detail_error=void 0,this.session_node_state="idle",this.session_node_error=void 0}async closeRequestDetail(){const e=this.selected_request_row_id&&this.selectedRequestDay()?x({day:this.selectedRequestDay(),row_id:this.selected_request_row_id}):void 0;if(++this.navigation_workflow_id,this.resetRequestSelection(),this.syncUrl("push"),!e||!window.matchMedia("(max-width: 680px)").matches)return;await this.updateComplete,[...this.querySelectorAll("request-list [data-request-key]")].find(s=>s.dataset.requestKey===e)?.focus()}async closeSessionDetail(){const e=this.selected_session?.session_id??this.requested_session_id;if(++this.navigation_workflow_id,this.resetSessionSelection(),this.syncUrl("push"),e===void 0||!window.matchMedia("(max-width: 680px)").matches)return;await this.updateComplete,[...this.querySelectorAll("session-list [data-session-id]")].find(s=>s.dataset.sessionId===e)?.focus()}async loadRequestDetail(e,t,s,i,r,n="replace"){this.request_detail_controller?.abort();const c=new AbortController;this.request_detail_controller=c;const d=++this.request_detail_load_id;this.selected_day=e,this.selected_request=i,this.selected_request_id=t,this.selected_request_row_id=s,r||(this.selected_request_detail=void 0),this.request_detail_state="loading",this.request_detail_error=void 0,n&&this.syncUrl(n);try{const h=new URLSearchParams({day:e,request_id:t});s&&h.set("row_id",s);const u=await $(`/api/request?${h.toString()}`,c.signal);if(d===this.request_detail_load_id&&this.request_detail_controller===c){const _=this.selected_request_row_id!==u.row_id;return this.selected_request_detail=u,this.selected_request_row_id=u.row_id,this.request_detail_state="ready",(n||_)&&this.syncUrl("replace"),!0}return!1}catch(h){return d===this.request_detail_load_id&&!A(h)&&(this.request_detail_state="error",this.request_detail_error=g(h,"Unable to load request detail")),!1}finally{this.request_detail_controller===c&&(this.request_detail_controller=void 0)}}async selectRequest(e){++this.navigation_workflow_id;const t=this.selected_request_id===e.request_id&&this.selected_request_detail?.day===e.day&&this.selected_request_detail.row_id===e.row_id,s=this.loadRequestDetail(e.day,e.request_id,e.row_id,e,t,"push");window.matchMedia("(max-width: 680px)").matches&&(await this.updateComplete,this.querySelector("request-detail-view .mobile-back-button")?.focus()),await s&&window.matchMedia("(max-width: 680px)").matches&&(await this.updateComplete,this.querySelector("request-detail-view .mobile-back-button")?.focus())}retryRequestDetail(){const e=this.selected_request_detail?.day??this.selected_request?.day??this.selected_day;e&&this.selected_request_id&&this.loadRequestDetail(e,this.selected_request_id,this.selected_request_row_id,this.selected_request,!!this.selected_request_detail,null)}selectDay(e){e!==this.selected_day&&(++this.navigation_workflow_id,this.selected_day=e,this.resetRequestSelection(),this.syncUrl("push"),this.loadRequests())}pickerDays(){return!this.selected_day||this.request_days.some(e=>e.day===this.selected_day)?this.request_days:[{day:this.selected_day,state:"available"},...this.request_days]}adjacentAvailableDay(e){const t=this.pickerDays().filter(i=>i.state==="available").map(i=>i.day).sort();if(!this.selected_day)return;const s=t.indexOf(this.selected_day);return s<0?void 0:t[s+e]}submitFilters(e){e.preventDefault(),++this.navigation_workflow_id,this.applied_filters={query:this.search_query.trim(),provider_id:this.provider_id.trim(),status:this.status_filter.trim(),errors_only:this.errors_only},this.resetRequestSelection(),this.syncUrl("push"),this.loadRequests()}clearFilters(){this.search_query="",this.provider_id="",this.status_filter="",this.errors_only=!1,this.applied_filters=X(),++this.navigation_workflow_id,this.resetRequestSelection(),this.syncUrl("push"),this.loadRequests()}hasAppliedFilters(){return!!(this.applied_filters.query||this.applied_filters.provider_id||this.applied_filters.status||this.applied_filters.errors_only)}filtersChanged(){return this.search_query.trim()!==this.applied_filters.query||this.provider_id.trim()!==this.applied_filters.provider_id||this.status_filter.trim()!==this.applied_filters.status||this.errors_only!==this.applied_filters.errors_only}providerOptions(){const e=new Set(this.requests.flatMap(t=>t.provider_id?[t.provider_id]:[]));return this.applied_filters.provider_id&&e.add(this.applied_filters.provider_id),[...e].sort()}ensureSessionsLoaded(e=!1){if(this.sessions_loaded&&!e)return Promise.resolve(!0);if(this.session_list_load&&!e)return this.session_list_load;this.session_list_controller?.abort();const t=new AbortController;this.session_list_controller=t;const s=++this.session_list_load_id;this.sessions_loading=!0,this.sessions_error=void 0;const i=this.loadSessions(t,s);return this.session_list_load=i,i}async loadSessions(e,t){try{const s=await $("/api/sessions?limit=100",e.signal);return t!==this.session_list_load_id||this.session_list_controller!==e?!1:(this.sessions=s,this.sessions_loaded=!0,this.selected_session&&(this.selected_session=s.find(i=>i.session_id===this.selected_session?.session_id)??this.selected_session),!0)}catch(s){return t===this.session_list_load_id&&!A(s)&&(this.sessions_error=g(s,"Unable to load sessions")),!1}finally{t===this.session_list_load_id&&this.session_list_controller===e&&(this.session_list_controller=void 0,this.session_list_load=void 0,this.sessions_loading=!1)}}retrySessions(){const e=++this.navigation_workflow_id;this.sessions_loaded=!1,this.retrySessionsAndRestore(e)}async retrySessionsAndRestore(e){if(!await this.ensureSessionsLoaded(!0)||e!==this.navigation_workflow_id||this.active_view!=="sessions")return;const s=this.selected_session?.session_id??this.requested_session_id;if(s===void 0)return;const i=this.selected_session_node_id??this.requested_session_node_id;await this.loadSession(s,this.sessions.find(r=>r.session_id===s),this.selected_session_detail?.session.session_id===s,null,i)}async refreshSessions(){const e=this.navigation_workflow_id,t=this.selected_session?.session_id??this.requested_session_id,s=this.selected_session_node_id,i=await this.ensureSessionsLoaded(!0),r=this.selected_session?.session_id??this.requested_session_id;i&&e===this.navigation_workflow_id&&t!==void 0&&r===t&&this.selected_session_node_id===s&&await this.loadSession(t,this.sessions.find(n=>n.session_id===t),!0,null,s)}filteredSessions(){const e=this.session_search_query.trim().toLocaleLowerCase();return e?this.sessions.filter(t=>[t.session_id,t.model,t.provider_id,t.account_id,t.endpoint,t.status===null?null:String(t.status)].some(s=>s?.toLocaleLowerCase().includes(e))):this.sessions}async loadSession(e,t,s,i="push",r){this.session_detail_controller?.abort(),this.session_node_controller?.abort();const n=new AbortController;this.session_detail_controller=n;const c=++this.session_detail_load_id,d=++this.session_node_load_id;this.requested_session_id=e,this.requested_session_node_id=r,this.selected_session=t,s||(this.selected_session_detail=void 0,this.selected_session_node_detail=void 0,this.selected_session_node_id=void 0,this.session_node_state="idle",this.session_node_error=void 0),this.session_detail_state="loading",this.session_detail_error=void 0,i&&this.syncUrl(i);try{const h=new URLSearchParams({session_id:e,limit:"500"}),u=await $(`/api/session?${h.toString()}`,n.signal);if(c===this.session_detail_load_id&&this.session_detail_controller===n){if(this.selected_session=u.session,this.selected_session_detail=u,this.sessions=this.sessions.map(_=>_.session_id===u.session.session_id?u.session:_),this.session_detail_state="ready",d!==this.session_node_load_id)return!0;if(r){const _=u.nodes.find(p=>p.node_id===r);this.loadSessionNode(_??r,!1,"replace")}else this.selected_session_node_id=void 0,this.selected_session_node_detail=void 0,this.session_node_state="idle",this.syncUrl("replace");return!0}return!1}catch(h){return c===this.session_detail_load_id&&!A(h)&&(this.session_detail_state="error",this.session_detail_error=g(h,"Unable to load semantic session")),!1}finally{this.session_detail_controller===n&&(this.session_detail_controller=void 0)}}async loadSessionNode(e,t,s="push"){const i=this.selected_session?.session_id??this.requested_session_id;if(i===void 0)return!1;this.session_node_controller?.abort();const r=new AbortController;this.session_node_controller=r;const n=++this.session_node_load_id,c=typeof e=="string"?e:e.node_id;this.requested_session_node_id=c,this.selected_session_node_id=c,t||(this.selected_session_node_detail=void 0),this.session_node_state="loading",this.session_node_error=void 0,s&&this.syncUrl(s);try{const d=new URLSearchParams({session_id:i,node_id:c}),h=await $(`/api/session-node?${d.toString()}`,r.signal);return n===this.session_node_load_id&&this.session_node_controller===r?(this.selected_session_node_detail=h,this.session_node_state="ready",this.syncUrl("replace"),!0):!1}catch(d){return n===this.session_node_load_id&&!A(d)&&(this.session_node_state="error",this.session_node_error=g(d,"Unable to load semantic node content")),!1}finally{this.session_node_controller===r&&(this.session_node_controller=void 0)}}async selectSession(e){const t=++this.navigation_workflow_id;if(!await this.loadSession(e.session_id,e,!1,"push")||t!==this.navigation_workflow_id||this.active_view!=="sessions"||this.selected_session_detail?.session.session_id!==e.session_id||!window.matchMedia("(max-width: 680px)").matches)return;await this.updateComplete;const i=this.querySelector("session-detail-view");await i?.updateComplete,t===this.navigation_workflow_id&&this.active_view==="sessions"&&this.selected_session_detail?.session.session_id===e.session_id&&i?.querySelector(".mobile-back-button")?.focus()}selectSessionNode(e){e.node_id===this.selected_session_node_id&&this.session_node_state==="ready"||this.loadSessionNode(e,!1,"push")}retrySessionDetail(){const e=this.selected_session?.session_id??this.requested_session_id;e!==void 0&&this.loadSession(e,this.selected_session,!!this.selected_session_detail,null,this.selected_session_node_id??this.requested_session_node_id)}retrySessionNode(){const e=this.selected_session_detail?.nodes.find(t=>t.node_id===this.selected_session_node_id);(e??this.selected_session_node_id)&&this.loadSessionNode(e??this.selected_session_node_id,!!this.selected_session_node_detail,null)}async openSession(e){++this.navigation_workflow_id,this.setActiveView("sessions",!1,null),await this.ensureSessionsLoaded();const t=this.sessions.find(s=>s.session_id===e);await this.loadSession(e,t,!1,"push")}async loadRequestsView(){this.loadRequestDays(),this.selected_day?await this.loadRequests():await this.loadLatestRequests()}setActiveView(e,t=!0,s="push"){s==="push"&&++this.navigation_workflow_id,this.active_view=e,s&&this.syncUrl(s),t&&(e==="sessions"?this.ensureSessionsLoaded():this.request_list_state==="idle"&&this.loadRequestsView())}setTimezone(e){this.timezone=e,this.syncUrl("push")}setDetailTab(e){this.active_detail_tab=e,this.syncUrl("push")}renderDayPicker(){const e=this.pickerDays(),t=this.adjacentAvailableDay(-1),s=this.adjacentAvailableDay(1);return a`
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
            @change=${i=>this.selectDay(i.target.value)}
          >
            ${this.selected_day?l:a`<option value="">No request day</option>`}
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
          ${this.hasAppliedFilters()?a`<button type="button" class="text-button" @click=${this.clearFilters}>Clear</button>`:l}
        </form>
        ${this.request_days_error?a`<p class="toolbar-warning" role="status">Day scan: ${this.request_days_error}</p>`:l}
      </section>
    `}renderRequestSidebar(){const e=this.requests.length>0;return a`
      <div class="list-pane" aria-busy=${String(this.request_list_state==="loading")}>
        <header class="list-pane-header">
          <div>
            <strong>Requests</strong>
            <span>${this.requests.length.toLocaleString()} loaded${this.next_cursor?" · more available":""}</span>
          </div>
          ${this.hasAppliedFilters()?a`<span class="filter-indicator">Filtered</span>`:l}
        </header>
        ${this.request_list_state==="loading"?a`
              <div class="inline-state" role="status">
                <span class="spinner" aria-hidden="true"></span>${e?"Refreshing requests…":"Loading requests…"}
              </div>
            `:l}
        ${this.request_list_state==="error"?a`
              <div class="inline-error" role="alert">
                <span>${this.request_list_error}</span>
                <button type="button" @click=${()=>{this.loadRequests()}}>Retry</button>
              </div>
            `:l}
        ${e?a`
              <request-list
                .requests=${this.requests}
                .selected_key=${this.selectedRequestDay()&&this.selected_request_row_id?x({day:this.selectedRequestDay(),row_id:this.selected_request_row_id}):void 0}
                .timezone=${this.timezone}
                @request-select=${t=>{this.selectRequest(D(t))}}
              ></request-list>
            `:this.request_list_state==="ready"?a`<p class="empty">No persisted requests match these filters.</p>`:this.request_list_state==="idle"?a`<p class="empty">Choose an available request day.</p>`:l}
        ${this.load_more_error?a`
              <div class="inline-error load-more-error" role="alert">
                <span>${this.load_more_error}</span>
                <button type="button" @click=${()=>{this.loadRequests(!0)}}>Retry</button>
              </div>
            `:l}
        ${this.next_cursor&&e?a`
              <div class="list-footer">
                <button type="button" class="secondary-button" ?disabled=${this.loading_more} @click=${()=>{this.loadRequests(!0)}}>
                  ${this.loading_more?"Loading…":"Load more"}
                </button>
              </div>
            `:e&&this.request_list_state==="ready"?a`<p class="end-of-list">End of loaded day</p>`:l}
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
          ${this.session_search_query?a`<span class="filter-indicator">Filtered</span>`:l}
        </header>
        ${this.sessions_loading?a`
              <div class="inline-state" role="status">
                <span class="spinner" aria-hidden="true"></span>${t?"Refreshing sessions…":"Loading sessions…"}
              </div>
            `:l}
        ${this.sessions_error?a`
              <div class="inline-error" role="alert">
                <span>${this.sessions_error}</span>
                <button type="button" @click=${this.retrySessions}>Retry</button>
              </div>
            `:l}
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
                `:l}
        ${t&&!this.session_search_query?a`<p class="end-of-list">${this.sessions.length===100?"Latest 100 sessions":"End of recent sessions"}</p>`:l}
      </div>
    `}renderSessionDetail(){return a`
      <session-detail-view
        .detail=${this.selected_session_detail}
        .node_detail=${this.selected_session_node_detail}
        .state=${this.session_detail_state}
        .error_message=${this.session_detail_error}
        .node_state=${this.session_node_state}
        .node_error_message=${this.session_node_error}
        .selected_node_id=${this.selected_session_node_id}
        .timezone=${this.timezone}
        @session-close=${()=>{this.closeSessionDetail()}}
        @session-retry=${this.retrySessionDetail}
        @session-node-retry=${this.retrySessionNode}
        @session-node-select=${e=>this.selectSessionNode(D(e))}
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
    `}}customElements.define("inspect-app",$t);
