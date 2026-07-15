(function(){const e=document.createElement("link").relList;if(e&&e.supports&&e.supports("modulepreload"))return;for(const i of document.querySelectorAll('link[rel="modulepreload"]'))s(i);new MutationObserver(i=>{for(const o of i)if(o.type==="childList")for(const a of o.addedNodes)a.tagName==="LINK"&&a.rel==="modulepreload"&&s(a)}).observe(document,{childList:!0,subtree:!0});function t(i){const o={};return i.integrity&&(o.integrity=i.integrity),i.referrerPolicy&&(o.referrerPolicy=i.referrerPolicy),i.crossOrigin==="use-credentials"?o.credentials="include":i.crossOrigin==="anonymous"?o.credentials="omit":o.credentials="same-origin",o}function s(i){if(i.ep)return;i.ep=!0;const o=t(i);fetch(i.href,o)}})();const F=globalThis,se=F.ShadowRoot&&(F.ShadyCSS===void 0||F.ShadyCSS.nativeShadow)&&"adoptedStyleSheets"in Document.prototype&&"replace"in CSSStyleSheet.prototype,ge=Symbol(),ae=new WeakMap;let Ce=class{constructor(e,t,s){if(this._$cssResult$=!0,s!==ge)throw Error("CSSResult is not constructable. Use `unsafeCSS` or `css` instead.");this.cssText=e,this.t=t}get styleSheet(){let e=this.o;const t=this.t;if(se&&e===void 0){const s=t!==void 0&&t.length===1;s&&(e=ae.get(t)),e===void 0&&((this.o=e=new CSSStyleSheet).replaceSync(this.cssText),s&&ae.set(t,e))}return e}toString(){return this.cssText}};const xe=r=>new Ce(typeof r=="string"?r:r+"",void 0,ge),Pe=(r,e)=>{if(se)r.adoptedStyleSheets=e.map(t=>t instanceof CSSStyleSheet?t:t.styleSheet);else for(const t of e){const s=document.createElement("style"),i=F.litNonce;i!==void 0&&s.setAttribute("nonce",i),s.textContent=t.cssText,r.appendChild(s)}},ne=se?r=>r:r=>r instanceof CSSStyleSheet?(e=>{let t="";for(const s of e.cssRules)t+=s.cssText;return xe(t)})(r):r;const{is:Ue,defineProperty:Te,getOwnPropertyDescriptor:De,getOwnPropertyNames:Oe,getOwnPropertySymbols:Le,getPrototypeOf:ze}=Object,K=globalThis,le=K.trustedTypes,Ne=le?le.emptyScript:"",He=K.reactiveElementPolyfillSupport,D=(r,e)=>r,X={toAttribute(r,e){switch(e){case Boolean:r=r?Ne:null;break;case Object:case Array:r=r==null?r:JSON.stringify(r)}return r},fromAttribute(r,e){let t=r;switch(e){case Boolean:t=r!==null;break;case Number:t=r===null?null:Number(r);break;case Object:case Array:try{t=JSON.parse(r)}catch{t=null}}return t}},qe=(r,e)=>!Ue(r,e),de={attribute:!0,type:String,converter:X,reflect:!1,useDefault:!1,hasChanged:qe};Symbol.metadata??=Symbol("metadata"),K.litPropertyMetadata??=new WeakMap;let k=class extends HTMLElement{static addInitializer(e){this._$Ei(),(this.l??=[]).push(e)}static get observedAttributes(){return this.finalize(),this._$Eh&&[...this._$Eh.keys()]}static createProperty(e,t=de){if(t.state&&(t.attribute=!1),this._$Ei(),this.prototype.hasOwnProperty(e)&&((t=Object.create(t)).wrapped=!0),this.elementProperties.set(e,t),!t.noAccessor){const s=Symbol(),i=this.getPropertyDescriptor(e,s,t);i!==void 0&&Te(this.prototype,e,i)}}static getPropertyDescriptor(e,t,s){const{get:i,set:o}=De(this.prototype,e)??{get(){return this[t]},set(a){this[t]=a}};return{get:i,set(a){const c=i?.call(this);o?.call(this,a),this.requestUpdate(e,c,s)},configurable:!0,enumerable:!0}}static getPropertyOptions(e){return this.elementProperties.get(e)??de}static _$Ei(){if(this.hasOwnProperty(D("elementProperties")))return;const e=ze(this);e.finalize(),e.l!==void 0&&(this.l=[...e.l]),this.elementProperties=new Map(e.elementProperties)}static finalize(){if(this.hasOwnProperty(D("finalized")))return;if(this.finalized=!0,this._$Ei(),this.hasOwnProperty(D("properties"))){const t=this.properties,s=[...Oe(t),...Le(t)];for(const i of s)this.createProperty(i,t[i])}const e=this[Symbol.metadata];if(e!==null){const t=litPropertyMetadata.get(e);if(t!==void 0)for(const[s,i]of t)this.elementProperties.set(s,i)}this._$Eh=new Map;for(const[t,s]of this.elementProperties){const i=this._$Eu(t,s);i!==void 0&&this._$Eh.set(i,t)}this.elementStyles=this.finalizeStyles(this.styles)}static finalizeStyles(e){const t=[];if(Array.isArray(e)){const s=new Set(e.flat(1/0).reverse());for(const i of s)t.unshift(ne(i))}else e!==void 0&&t.push(ne(e));return t}static _$Eu(e,t){const s=t.attribute;return s===!1?void 0:typeof s=="string"?s:typeof e=="string"?e.toLowerCase():void 0}constructor(){super(),this._$Ep=void 0,this.isUpdatePending=!1,this.hasUpdated=!1,this._$Em=null,this._$Ev()}_$Ev(){this._$ES=new Promise(e=>this.enableUpdating=e),this._$AL=new Map,this._$E_(),this.requestUpdate(),this.constructor.l?.forEach(e=>e(this))}addController(e){(this._$EO??=new Set).add(e),this.renderRoot!==void 0&&this.isConnected&&e.hostConnected?.()}removeController(e){this._$EO?.delete(e)}_$E_(){const e=new Map,t=this.constructor.elementProperties;for(const s of t.keys())this.hasOwnProperty(s)&&(e.set(s,this[s]),delete this[s]);e.size>0&&(this._$Ep=e)}createRenderRoot(){const e=this.shadowRoot??this.attachShadow(this.constructor.shadowRootOptions);return Pe(e,this.constructor.elementStyles),e}connectedCallback(){this.renderRoot??=this.createRenderRoot(),this.enableUpdating(!0),this._$EO?.forEach(e=>e.hostConnected?.())}enableUpdating(e){}disconnectedCallback(){this._$EO?.forEach(e=>e.hostDisconnected?.())}attributeChangedCallback(e,t,s){this._$AK(e,s)}_$ET(e,t){const s=this.constructor.elementProperties.get(e),i=this.constructor._$Eu(e,s);if(i!==void 0&&s.reflect===!0){const o=(s.converter?.toAttribute!==void 0?s.converter:X).toAttribute(t,s.type);this._$Em=e,o==null?this.removeAttribute(i):this.setAttribute(i,o),this._$Em=null}}_$AK(e,t){const s=this.constructor,i=s._$Eh.get(e);if(i!==void 0&&this._$Em!==i){const o=s.getPropertyOptions(i),a=typeof o.converter=="function"?{fromAttribute:o.converter}:o.converter?.fromAttribute!==void 0?o.converter:X;this._$Em=i;const c=a.fromAttribute(t,o.type);this[i]=c??this._$Ej?.get(i)??c,this._$Em=null}}requestUpdate(e,t,s,i=!1,o){if(e!==void 0){const a=this.constructor;if(i===!1&&(o=this[e]),s??=a.getPropertyOptions(e),!((s.hasChanged??qe)(o,t)||s.useDefault&&s.reflect&&o===this._$Ej?.get(e)&&!this.hasAttribute(a._$Eu(e,s))))return;this.C(e,t,s)}this.isUpdatePending===!1&&(this._$ES=this._$EP())}C(e,t,{useDefault:s,reflect:i,wrapped:o},a){s&&!(this._$Ej??=new Map).has(e)&&(this._$Ej.set(e,a??t??this[e]),o!==!0||a!==void 0)||(this._$AL.has(e)||(this.hasUpdated||s||(t=void 0),this._$AL.set(e,t)),i===!0&&this._$Em!==e&&(this._$Eq??=new Set).add(e))}async _$EP(){this.isUpdatePending=!0;try{await this._$ES}catch(t){Promise.reject(t)}const e=this.scheduleUpdate();return e!=null&&await e,!this.isUpdatePending}scheduleUpdate(){return this.performUpdate()}performUpdate(){if(!this.isUpdatePending)return;if(!this.hasUpdated){if(this.renderRoot??=this.createRenderRoot(),this._$Ep){for(const[i,o]of this._$Ep)this[i]=o;this._$Ep=void 0}const s=this.constructor.elementProperties;if(s.size>0)for(const[i,o]of s){const{wrapped:a}=o,c=this[i];a!==!0||this._$AL.has(i)||c===void 0||this.C(i,void 0,o,c)}}let e=!1;const t=this._$AL;try{e=this.shouldUpdate(t),e?(this.willUpdate(t),this._$EO?.forEach(s=>s.hostUpdate?.()),this.update(t)):this._$EM()}catch(s){throw e=!1,this._$EM(),s}e&&this._$AE(t)}willUpdate(e){}_$AE(e){this._$EO?.forEach(t=>t.hostUpdated?.()),this.hasUpdated||(this.hasUpdated=!0,this.firstUpdated(e)),this.updated(e)}_$EM(){this._$AL=new Map,this.isUpdatePending=!1}get updateComplete(){return this.getUpdateComplete()}getUpdateComplete(){return this._$ES}shouldUpdate(e){return!0}update(e){this._$Eq&&=this._$Eq.forEach(t=>this._$ET(t,this[t])),this._$EM()}updated(e){}firstUpdated(e){}};k.elementStyles=[],k.shadowRootOptions={mode:"open"},k[D("elementProperties")]=new Map,k[D("finalized")]=new Map,He?.({ReactiveElement:k}),(K.reactiveElementVersions??=[]).push("2.1.2");const ie=globalThis,ce=r=>r,J=ie.trustedTypes,he=J?J.createPolicy("lit-html",{createHTML:r=>r}):void 0,we="$lit$",m=`lit$${Math.random().toFixed(9).slice(2)}$`,Se="?"+m,Me=`<${Se}>`,A=document,z=()=>A.createComment(""),N=r=>r===null||typeof r!="object"&&typeof r!="function",re=Array.isArray,je=r=>re(r)||typeof r?.[Symbol.iterator]=="function",Q=`[ 	
\f\r]`,T=/<(?:(!--|\/[^a-zA-Z])|(\/?[a-zA-Z][^>\s]*)|(\/?$))/g,ue=/-->/g,pe=/>/g,g=RegExp(`>|${Q}(?:([^\\s"'>=/]+)(${Q}*=${Q}*(?:[^ 	
\f\r"'\`<>=]|("|')|))|$)`,"g"),_e=/'/g,ye=/"/g,Ae=/^(?:script|style|textarea|title)$/i,Be=r=>(e,...t)=>({_$litType$:r,strings:e,values:t}),n=Be(1),P=Symbol.for("lit-noChange"),d=Symbol.for("lit-nothing"),fe=new WeakMap,S=A.createTreeWalker(A,129);function Re(r,e){if(!re(r)||!r.hasOwnProperty("raw"))throw Error("invalid template strings array");return he!==void 0?he.createHTML(e):e}const Ie=(r,e)=>{const t=r.length-1,s=[];let i,o=e===2?"<svg>":e===3?"<math>":"",a=T;for(let c=0;c<t;c++){const l=r[c];let h,u,p=-1,f=0;for(;f<l.length&&(a.lastIndex=f,u=a.exec(l),u!==null);)f=a.lastIndex,a===T?u[1]==="!--"?a=ue:u[1]!==void 0?a=pe:u[2]!==void 0?(Ae.test(u[2])&&(i=RegExp("</"+u[2],"g")),a=g):u[3]!==void 0&&(a=g):a===g?u[0]===">"?(a=i??T,p=-1):u[1]===void 0?p=-2:(p=a.lastIndex-u[2].length,h=u[1],a=u[3]===void 0?g:u[3]==='"'?ye:_e):a===ye||a===_e?a=g:a===ue||a===pe?a=T:(a=g,i=void 0);const $=a===g&&r[c+1].startsWith("/>")?" ":"";o+=a===T?l+Me:p>=0?(s.push(h),l.slice(0,p)+we+l.slice(p)+m+$):l+m+(p===-2?c:$)}return[Re(r,o+(r[t]||"<?>")+(e===2?"</svg>":e===3?"</math>":"")),s]};class H{constructor({strings:e,_$litType$:t},s){let i;this.parts=[];let o=0,a=0;const c=e.length-1,l=this.parts,[h,u]=Ie(e,t);if(this.el=H.createElement(h,s),S.currentNode=this.el.content,t===2||t===3){const p=this.el.content.firstChild;p.replaceWith(...p.childNodes)}for(;(i=S.nextNode())!==null&&l.length<c;){if(i.nodeType===1){if(i.hasAttributes())for(const p of i.getAttributeNames())if(p.endsWith(we)){const f=u[a++],$=i.getAttribute(p).split(m),B=/([.?@])?(.*)/.exec(f);l.push({type:1,index:o,name:B[2],strings:$,ctor:B[1]==="."?Fe:B[1]==="?"?We:B[1]==="@"?Je:Z}),i.removeAttribute(p)}else p.startsWith(m)&&(l.push({type:6,index:o}),i.removeAttribute(p));if(Ae.test(i.tagName)){const p=i.textContent.split(m),f=p.length-1;if(f>0){i.textContent=J?J.emptyScript:"";for(let $=0;$<f;$++)i.append(p[$],z()),S.nextNode(),l.push({type:2,index:++o});i.append(p[f],z())}}}else if(i.nodeType===8)if(i.data===Se)l.push({type:2,index:o});else{let p=-1;for(;(p=i.data.indexOf(m,p+1))!==-1;)l.push({type:7,index:o}),p+=m.length-1}o++}}static createElement(e,t){const s=A.createElement("template");return s.innerHTML=e,s}}function U(r,e,t=r,s){if(e===P)return e;let i=s!==void 0?t._$Co?.[s]:t._$Cl;const o=N(e)?void 0:e._$litDirective$;return i?.constructor!==o&&(i?._$AO?.(!1),o===void 0?i=void 0:(i=new o(r),i._$AT(r,t,s)),s!==void 0?(t._$Co??=[])[s]=i:t._$Cl=i),i!==void 0&&(e=U(r,i._$AS(r,e.values),i,s)),e}class Ve{constructor(e,t){this._$AV=[],this._$AN=void 0,this._$AD=e,this._$AM=t}get parentNode(){return this._$AM.parentNode}get _$AU(){return this._$AM._$AU}u(e){const{el:{content:t},parts:s}=this._$AD,i=(e?.creationScope??A).importNode(t,!0);S.currentNode=i;let o=S.nextNode(),a=0,c=0,l=s[0];for(;l!==void 0;){if(a===l.index){let h;l.type===2?h=new j(o,o.nextSibling,this,e):l.type===1?h=new l.ctor(o,l.name,l.strings,this,e):l.type===6&&(h=new Ke(o,this,e)),this._$AV.push(h),l=s[++c]}a!==l?.index&&(o=S.nextNode(),a++)}return S.currentNode=A,i}p(e){let t=0;for(const s of this._$AV)s!==void 0&&(s.strings!==void 0?(s._$AI(e,s,t),t+=s.strings.length-2):s._$AI(e[t])),t++}}class j{get _$AU(){return this._$AM?._$AU??this._$Cv}constructor(e,t,s,i){this.type=2,this._$AH=d,this._$AN=void 0,this._$AA=e,this._$AB=t,this._$AM=s,this.options=i,this._$Cv=i?.isConnected??!0}get parentNode(){let e=this._$AA.parentNode;const t=this._$AM;return t!==void 0&&e?.nodeType===11&&(e=t.parentNode),e}get startNode(){return this._$AA}get endNode(){return this._$AB}_$AI(e,t=this){e=U(this,e,t),N(e)?e===d||e==null||e===""?(this._$AH!==d&&this._$AR(),this._$AH=d):e!==this._$AH&&e!==P&&this._(e):e._$litType$!==void 0?this.$(e):e.nodeType!==void 0?this.T(e):je(e)?this.k(e):this._(e)}O(e){return this._$AA.parentNode.insertBefore(e,this._$AB)}T(e){this._$AH!==e&&(this._$AR(),this._$AH=this.O(e))}_(e){this._$AH!==d&&N(this._$AH)?this._$AA.nextSibling.data=e:this.T(A.createTextNode(e)),this._$AH=e}$(e){const{values:t,_$litType$:s}=e,i=typeof s=="number"?this._$AC(e):(s.el===void 0&&(s.el=H.createElement(Re(s.h,s.h[0]),this.options)),s);if(this._$AH?._$AD===i)this._$AH.p(t);else{const o=new Ve(i,this),a=o.u(this.options);o.p(t),this.T(a),this._$AH=o}}_$AC(e){let t=fe.get(e.strings);return t===void 0&&fe.set(e.strings,t=new H(e)),t}k(e){re(this._$AH)||(this._$AH=[],this._$AR());const t=this._$AH;let s,i=0;for(const o of e)i===t.length?t.push(s=new j(this.O(z()),this.O(z()),this,this.options)):s=t[i],s._$AI(o),i++;i<t.length&&(this._$AR(s&&s._$AB.nextSibling,i),t.length=i)}_$AR(e=this._$AA.nextSibling,t){for(this._$AP?.(!1,!0,t);e!==this._$AB;){const s=ce(e).nextSibling;ce(e).remove(),e=s}}setConnected(e){this._$AM===void 0&&(this._$Cv=e,this._$AP?.(e))}}class Z{get tagName(){return this.element.tagName}get _$AU(){return this._$AM._$AU}constructor(e,t,s,i,o){this.type=1,this._$AH=d,this._$AN=void 0,this.element=e,this.name=t,this._$AM=i,this.options=o,s.length>2||s[0]!==""||s[1]!==""?(this._$AH=Array(s.length-1).fill(new String),this.strings=s):this._$AH=d}_$AI(e,t=this,s,i){const o=this.strings;let a=!1;if(o===void 0)e=U(this,e,t,0),a=!N(e)||e!==this._$AH&&e!==P,a&&(this._$AH=e);else{const c=e;let l,h;for(e=o[0],l=0;l<o.length-1;l++)h=U(this,c[s+l],t,l),h===P&&(h=this._$AH[l]),a||=!N(h)||h!==this._$AH[l],h===d?e=d:e!==d&&(e+=(h??"")+o[l+1]),this._$AH[l]=h}a&&!i&&this.j(e)}j(e){e===d?this.element.removeAttribute(this.name):this.element.setAttribute(this.name,e??"")}}class Fe extends Z{constructor(){super(...arguments),this.type=3}j(e){this.element[this.name]=e===d?void 0:e}}class We extends Z{constructor(){super(...arguments),this.type=4}j(e){this.element.toggleAttribute(this.name,!!e&&e!==d)}}class Je extends Z{constructor(e,t,s,i,o){super(e,t,s,i,o),this.type=5}_$AI(e,t=this){if((e=U(this,e,t,0)??d)===P)return;const s=this._$AH,i=e===d&&s!==d||e.capture!==s.capture||e.once!==s.once||e.passive!==s.passive,o=e!==d&&(s===d||i);i&&this.element.removeEventListener(this.name,this,s),o&&this.element.addEventListener(this.name,this,e),this._$AH=e}handleEvent(e){typeof this._$AH=="function"?this._$AH.call(this.options?.host??this.element,e):this._$AH.handleEvent(e)}}class Ke{constructor(e,t,s){this.element=e,this.type=6,this._$AN=void 0,this._$AM=t,this.options=s}get _$AU(){return this._$AM._$AU}_$AI(e){U(this,e)}}const Ze=ie.litHtmlPolyfillSupport;Ze?.(H,j),(ie.litHtmlVersions??=[]).push("3.3.3");const Qe=(r,e,t)=>{const s=t?.renderBefore??e;let i=s._$litPart$;if(i===void 0){const o=t?.renderBefore??null;s._$litPart$=i=new j(e.insertBefore(z(),o),o,void 0,t??{})}return i._$AI(r),i};const oe=globalThis;class y extends k{constructor(){super(...arguments),this.renderOptions={host:this},this._$Do=void 0}createRenderRoot(){const e=super.createRenderRoot();return this.renderOptions.renderBefore??=e.firstChild,e}update(e){const t=this.render();this.hasUpdated||(this.renderOptions.isConnected=this.isConnected),super.update(e),this._$Do=Qe(t,this.renderRoot,this.renderOptions)}connectedCallback(){super.connectedCallback(),this._$Do?.setConnected(!0)}disconnectedCallback(){super.disconnectedCallback(),this._$Do?.setConnected(!1)}render(){return P}}y._$litElement$=!0,y.finalized=!0,oe.litElementHydrateSupport?.({LitElement:y});const Ge=oe.litElementPolyfillSupport;Ge?.({LitElement:y});(oe.litElementVersions??=[]).push("4.2.2");class Ee extends Error{status;constructor(e,t){super(t),this.name="HttpError",this.status=e}}async function b(r,e){const t=await fetch(r,{cache:"no-store",signal:e});if(!t.ok){const s=await t.json().catch(()=>({}));throw new Ee(t.status,s.error??`Request failed (${t.status})`)}return t.json()}function W(r){return r instanceof Error&&r.name==="AbortError"}function x(r,e,t=!1){const s=t?{hour:"2-digit",minute:"2-digit",second:"2-digit"}:{dateStyle:"medium",timeStyle:"medium"};return e==="utc"&&(s.timeZone="UTC"),new Intl.DateTimeFormat(void 0,s).format(new Date(r))}function C(r){return`${r.day}:${r.row_id}`}function M(r,e=10){return r?r.length>e?`…${r.slice(-e)}`:r:"—"}function Ye(r){const e=r.inbound_req_url??r.endpoint;return O(e)}function $e(r){const e=r.toLowerCase().replaceAll("_","-");return e==="authorization"||e==="password"||e==="code"||e==="signature"||e==="sig"||e.includes("api-key")||e.includes("access-key")||e.includes("token")||e.includes("secret")||e.includes("credential")}function O(r){if(!r)return"unknown endpoint";try{const e=new URL(r,window.location.origin);for(const t of new Set(e.searchParams.keys()))$e(t)&&e.searchParams.set(t,"REDACTED");return`${e.pathname}${e.search}`}catch{return r.replace(/([?&]([^=&]+)=)([^&]*)/g,(e,t,s)=>{let i=s;try{i=decodeURIComponent(s)}catch{}return $e(i)?`${t}REDACTED`:e})}}function ke(r){if(r.request_error)return{label:"ERR",tone:"error",title:r.request_error};const e=r.inbound_resp_status??r.outbound_resp_status??r.status;if(e===null)return{label:"—",tone:"neutral",title:"No response status persisted"};const t=r.inbound_resp_status!==null?"Client response":r.outbound_resp_status!==null?"Provider response":"Request";return e>=400?{label:String(e),tone:"error",title:`${t}: ${e}`}:e>=300?{label:String(e),tone:"warning",title:`${t}: ${e}`}:{label:String(e),tone:"success",title:`${t}: ${e}`}}function R(r){return r.detail}function _(r,e){const t=r[e];return typeof t=="string"?t:void 0}function I(r,e){const t=r[e];return typeof t=="number"?t:void 0}const G="••••••••";function Y(r){const e=r.toLowerCase().replaceAll("_","-");return e==="authorization"||e==="proxy-authorization"||e==="cookie"||e==="set-cookie"||e.includes("api-key")||e.includes("token")||e.includes("secret")}function L(r){if(Array.isArray(r))return r.length===2&&typeof r[0]=="string"&&Y(r[0])?[r[0],G]:r.map(e=>L(e));if(r!==null&&typeof r=="object")return Object.fromEntries(Object.entries(r).map(([e,t])=>[e,Y(e)?G:L(t)]));if(typeof r=="string")try{return L(JSON.parse(r))}catch{return r.replace(/^([^:\r\n]+)(:\s*)(.*)$/gm,(e,t,s)=>Y(t.trim())?`${t}${s}${G}`:e)}return r}function ee(r){return Array.isArray(r)?r.map(e=>ee(e)):r!==null&&typeof r=="object"?Object.fromEntries(Object.entries(r).map(([e,t])=>[e,Xe(e)?L(t):ee(t)])):r}function Xe(r){const e=r.replace(/([a-z0-9])([A-Z])/g,"$1_$2").toLowerCase().replace(/[-\s]+/g,"_");return e==="headers"||e.endsWith("_headers")}function te(r){return Array.isArray(r)?r.map(e=>te(e)):r!==null&&typeof r=="object"?Object.fromEntries(Object.entries(r).map(([e,t])=>[e,e.toLowerCase().endsWith("_url")&&typeof t=="string"?O(t):te(t)])):r}function et(r){if(typeof r=="string")try{return JSON.stringify(JSON.parse(r),null,2)}catch{return r}return JSON.stringify(r,null,2)??String(r)}function tt(r){if(Array.isArray(r))return`${r.length} item${r.length===1?"":"s"}`;if(r!==null&&typeof r=="object"){const e=Object.keys(r).length;return`${e} field${e===1?"":"s"}`}return typeof r=="string"?`${new Blob([r]).size.toLocaleString()} bytes`:typeof r}class st extends y{static properties={label:{type:String},value:{attribute:!1},load_url:{type:String},is_headers:{type:Boolean},redact_record_headers:{type:Boolean},open:{type:Boolean,state:!0},wrap:{type:Boolean,state:!0},revealed:{type:Boolean,state:!0},copy_state:{type:String,state:!0},load_state:{type:String,state:!0},loaded_value:{attribute:!1,state:!0},error_message:{type:String,state:!0}};load_controller;copy_timeout;constructor(){super(),this.label="Payload",this.is_headers=!1,this.redact_record_headers=!1,this.open=!1,this.wrap=!0,this.revealed=!1,this.copy_state="idle",this.load_state="idle"}createRenderRoot(){return this}disconnectedCallback(){this.load_controller?.abort(),this.copy_timeout!==void 0&&window.clearTimeout(this.copy_timeout),super.disconnectedCallback()}willUpdate(e){!e.has("value")&&!e.has("load_url")||(this.load_controller?.abort(),this.load_controller=void 0,this.copy_timeout!==void 0&&(window.clearTimeout(this.copy_timeout),this.copy_timeout=void 0),this.open=!1,this.revealed=!1,this.copy_state="idle",this.load_state="idle",this.loaded_value=void 0,this.error_message=void 0)}effectiveValue(){return this.load_state==="ready"?this.loaded_value:this.value}displayedValue(){const e=this.effectiveValue(),t=this.redact_record_headers?te(e):e,s=this.revealed?t:this.redact_record_headers?ee(t):this.is_headers?L(t):t;return et(s)}toggleOpen(e){this.open=e.currentTarget.open,this.open&&this.value===void 0&&this.load_url&&this.load_state==="idle"&&this.loadPayload()}async loadPayload(){const e=this.load_url;if(!e)return;this.load_controller?.abort();const t=new AbortController;this.load_controller=t,this.load_state="loading",this.error_message=void 0;try{const s=await b(e,t.signal);if(this.load_controller!==t||this.load_url!==e)return;const i=new URL(e,window.location.origin).searchParams.get("field");if(!i||s.field!==i)throw new Error("Payload response did not match the requested field");this.loaded_value=s.value,this.load_state="ready"}catch(s){if(this.load_controller!==t||W(s))return;this.load_state="error",this.error_message=s instanceof Error?s.message:"Unable to load payload"}finally{this.load_controller===t&&(this.load_controller=void 0)}}async copyValue(){try{await navigator.clipboard.writeText(this.displayedValue()),this.copy_state="copied",this.copy_timeout!==void 0&&window.clearTimeout(this.copy_timeout),this.copy_timeout=window.setTimeout(()=>{this.copy_state="idle",this.copy_timeout=void 0},1500)}catch{this.copy_state="error"}}render(){if(!this.load_url&&(this.value===null||this.value===void 0||this.value===""))return d;const e=this.effectiveValue(),t=this.is_headers||this.redact_record_headers,s=this.load_state==="loading"?"Loading…":this.load_state==="error"?"Load failed":e===null?"No payload":e===void 0?"Load on open":tt(e);return n`
      <details class="payload-panel" ?open=${this.open} @toggle=${this.toggleOpen}>
        <summary>
          <span>${this.label}</span>
          <span class="payload-summary">${s}</span>
        </summary>
        ${this.open?this.load_state==="loading"?n`<div class="payload-state" role="status"><span class="spinner" aria-hidden="true"></span>Loading payload…</div>`:this.load_state==="error"?n`
                  <div class="payload-state payload-error" role="alert">
                    <span>${this.error_message}</span>
                    <button type="button" @click=${()=>{this.loadPayload()}}>Retry</button>
                  </div>
                `:e==null||e===""?n`<div class="payload-state">No payload was persisted.</div>`:n`
                    <div class="payload-toolbar">
                      <button type="button" @click=${()=>{this.copyValue()}}>
                        ${this.copy_state==="copied"?"Copied":this.copy_state==="error"?"Copy failed":"Copy"}
                      </button>
                      <button type="button" aria-pressed=${String(this.wrap)} @click=${()=>this.wrap=!this.wrap}>
                        ${this.wrap?"No wrap":"Wrap"}
                      </button>
                      ${t?n`
                            <button
                              type="button"
                              class=${this.revealed?"danger-button":""}
                              aria-pressed=${String(this.revealed)}
                              @click=${()=>this.revealed=!this.revealed}
                            >
                              ${this.revealed?"Hide sensitive":"Reveal sensitive"}
                            </button>
                          `:d}
                      <span class="payload-security-note">
                        ${t&&!this.revealed?"Sensitive headers redacted":""}
                      </span>
                    </div>
                    <pre class=${this.wrap?"wrap":"nowrap"}><code>${this.displayedValue()}</code></pre>
                  `:d}
      </details>
    `}}customElements.define("payload-panel",st);const q=[{id:"overview",label:"Overview"},{id:"client",label:"Client"},{id:"provider",label:"Provider"},{id:"raw",label:"Raw"}];function E(r){return r==null||r===""?"—":typeof r=="boolean"?r?"Yes":"No":String(r)}function it(r){if(r!==null&&typeof r=="object"&&!Array.isArray(r))return r;if(typeof r=="string")try{const e=JSON.parse(r);return e!==null&&typeof e=="object"&&!Array.isArray(e)?e:void 0}catch{return}}function ve(r,e,t){return it(r[e])?.[t]??r[t]}function v(r,e,t,s){return`/api/request-payload?${new URLSearchParams({day:r,request_id:e,row_id:t,field:s}).toString()}`}function be(r){return r===void 0?"neutral":r>=400?"error":r>=300?"warning":"success"}class rt extends y{static properties={detail:{attribute:!1},summary:{attribute:!1},state:{type:String},error_message:{type:String},active_tab:{type:String},timezone:{type:String}};createRenderRoot(){return this}openSession(e){this.dispatchEvent(new CustomEvent("open-session",{detail:e,bubbles:!0,composed:!0}))}retry(){this.dispatchEvent(new CustomEvent("detail-retry",{bubbles:!0,composed:!0}))}close(){this.dispatchEvent(new CustomEvent("detail-close",{bubbles:!0,composed:!0}))}selectTab(e){this.dispatchEvent(new CustomEvent("detail-tab-change",{detail:e,bubbles:!0,composed:!0}))}tabKeydown(e){const t=q.findIndex(a=>a.id===this.active_tab);let s;if(e.key==="ArrowRight"?s=(t+1)%q.length:e.key==="ArrowLeft"?s=(t-1+q.length)%q.length:e.key==="Home"?s=0:e.key==="End"&&(s=q.length-1),s===void 0)return;e.preventDefault();const i=q[s];this.selectTab(i.id),this.querySelectorAll("[role=tab]")[s]?.focus()}renderOverview(e){const t=I(e,"ts"),s=ve(e,"ctx_json","latency_ms"),i=ve(e,"params_json","stream"),o=[["Timestamp",t===void 0?void 0:x(t,this.timezone)],["Storage day",this.detail?.day],["Endpoint",e.endpoint],["Model",e.model],["Provider",e.provider_id],["Account",e.account_id],["Latency",typeof s=="number"?`${s} ms`:s],["Streaming",i]],a=I(e,"inbound_resp_status"),c=I(e,"outbound_resp_status"),l=I(e,"status");return n`
      <section class="flow-grid" aria-label="Request flow">
        <div>
          <span>Client request</span>
          <strong>${_(e,"inbound_req_method")??"—"}</strong>
        </div>
        <span class="flow-arrow" aria-hidden="true">→</span>
        <div>
          <span>Provider response</span>
          <strong class="status-text ${be(c)}">${E(c)}</strong>
        </div>
        <span class="flow-arrow" aria-hidden="true">→</span>
        <div>
          <span>Client response</span>
          <strong class="status-text ${be(a??l)}">
            ${E(a??l)}
          </strong>
        </div>
      </section>
      <dl class="metadata-grid">
        ${o.map(([h,u])=>n`
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
    `}renderClient(e,t,s,i){return n`
      <section class="payload-group">
        <div class="payload-group-heading">
          <div><span class="direction-label">Incoming</span><h3>Client request</h3></div>
          <span>${_(e,"inbound_req_method")??"—"} ${O(_(e,"inbound_req_url"))}</span>
        </div>
        <payload-panel
          label="Request headers"
          .value=${e.inbound_req_headers}
          .load_url=${v(t,s,i,"inbound_req_headers")}
          .is_headers=${!0}
        ></payload-panel>
        <payload-panel
          label="Request body"
          .value=${e.inbound_req_body}
          .load_url=${v(t,s,i,"inbound_req_body")}
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
          .load_url=${v(t,s,i,"inbound_resp_headers")}
          .is_headers=${!0}
        ></payload-panel>
        <payload-panel
          label="Response body"
          .value=${e.inbound_resp_body}
          .load_url=${v(t,s,i,"inbound_resp_body")}
        ></payload-panel>
      </section>
    `}renderProvider(e,t,s,i){return n`
      <section class="payload-group">
        <div class="payload-group-heading">
          <div><span class="direction-label">Outgoing</span><h3>Provider request</h3></div>
          <span>${_(e,"outbound_req_method")??"—"} ${O(_(e,"outbound_req_url"))}</span>
        </div>
        <payload-panel
          label="Request headers"
          .value=${e.outbound_req_headers}
          .load_url=${v(t,s,i,"outbound_req_headers")}
          .is_headers=${!0}
        ></payload-panel>
        <payload-panel
          label="Request body"
          .value=${e.outbound_req_body}
          .load_url=${v(t,s,i,"outbound_req_body")}
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
          .load_url=${v(t,s,i,"outbound_resp_headers")}
          .is_headers=${!0}
        ></payload-panel>
        <payload-panel
          label="Response body"
          .value=${e.outbound_resp_body}
          .load_url=${v(t,s,i,"outbound_resp_body")}
        ></payload-panel>
      </section>
    `}renderTab(e,t,s,i){switch(this.active_tab){case"client":return this.renderClient(e,t,s,i);case"provider":return this.renderProvider(e,t,s,i);case"raw":return n`
          <p class="raw-note">Network headers and bodies remain lazy and are not included in this overview record.</p>
          <payload-panel
            label="Persisted overview record"
            .value=${e}
            .redact_record_headers=${!0}
          ></payload-panel>
        `;default:return this.renderOverview(e)}}render(){if(!this.detail)return this.state==="loading"?n`
          <section class="detail-state" aria-live="polite">
            <button type="button" class="mobile-back-button" @click=${this.close}>← Requests</button>
            <span class="spinner" aria-hidden="true"></span>
            <p>Loading request detail…</p>
          </section>
        `:this.state==="error"?n`
          <section class="detail-state error-state" role="alert">
            <button type="button" class="mobile-back-button" @click=${this.close}>← Requests</button>
            <strong>Request detail could not be loaded</strong>
            <p>${this.error_message}</p>
            <button type="button" class="primary-button" @click=${this.retry}>Retry</button>
          </section>
        `:n`<section class="detail-state"><p>Select a request to inspect its route, payloads, and responses.</p></section>`;const e=this.detail.request,t=_(e,"request_id")??this.summary?.request_id??"unknown id",s=_(e,"session_id")??this.summary?.session_id??void 0,i=_(e,"inbound_req_method")??this.summary?.inbound_req_method??"REQUEST",o=O(_(e,"inbound_req_url")??this.summary?.inbound_req_url??_(e,"endpoint"));return n`
      <section class="detail-content">
        <header class="detail-header">
          <button type="button" class="mobile-back-button" @click=${this.close}>← Requests</button>
          <div class="detail-title">
            <p class="eyebrow">request · ${M(t)}</p>
            <h2><span>${i}</span> ${o}</h2>
            <p class="muted" title=${t}>${t}</p>
          </div>
          <div class="detail-actions">
            ${s?n`<button type="button" class="secondary-button" @click=${()=>this.openSession(s)}>Open session</button>`:d}
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
        ${this.state==="loading"?n`<div class="inline-state" role="status"><span class="spinner" aria-hidden="true"></span>Refreshing detail…</div>`:d}
        ${this.state==="error"?n`
              <div class="inline-error" role="alert">
                <span>${this.error_message}</span>
                <button type="button" @click=${this.retry}>Retry</button>
              </div>
            `:d}
        ${e.request_error?n`<div class="request-error" role="alert">${String(e.request_error)}</div>`:d}
        <div class="detail-tabs" role="tablist" aria-label="Request detail sections" @keydown=${this.tabKeydown}>
          ${q.map(a=>n`
              <button
                id="request-tab-${a.id}"
                type="button"
                role="tab"
                aria-selected=${String(this.active_tab===a.id)}
                aria-controls="request-panel-${a.id}"
                tabindex=${this.active_tab===a.id?"0":"-1"}
                @click=${()=>this.selectTab(a.id)}
              >
                ${a.label}
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
    `}}customElements.define("request-detail-view",rt);class ot extends y{static properties={requests:{attribute:!1},selected_key:{type:String},timezone:{type:String}};createRenderRoot(){return this}selectRequest(e){this.dispatchEvent(new CustomEvent("request-select",{detail:e,bubbles:!0,composed:!0}))}render(){const e=this.requests??[];return e.length===0?n`<p class="empty">No persisted requests match these filters.</p>`:n`
      <ul class="request-list" aria-label="Requests">
        ${e.map(t=>{const s=ke(t),i=this.selected_key===C(t),o=t.inbound_req_method??"REQUEST",a=Ye(t);return n`
            <li>
              <button
                type="button"
                class="request-row ${i?"selected":""}"
                data-request-key=${C(t)}
                aria-current=${i?"true":"false"}
                @click=${()=>this.selectRequest(t)}
              >
                <span class="request-row-time">${x(t.ts,this.timezone,!0)}</span>
                <span class="status ${s.tone}" title=${s.title}>${s.label}</span>
                <span class="request-row-main">
                  <span class="request-route"><strong>${o}</strong><span>${a}</span></span>
                  <span class="request-context">
                    <span>${t.model??"unknown model"}</span>
                    <span aria-hidden="true">·</span>
                    <span>${t.provider_id??"unknown provider"}</span>
                  </span>
                  <span class="request-identifiers">
                    <span title=${t.request_id}>req ${M(t.request_id)}</span>
                    ${t.session_id?n`<span title=${t.session_id}>session ${M(t.session_id)}</span>`:n`<span>no session</span>`}
                  </span>
                </span>
              </button>
            </li>
          `})}
      </ul>
    `}}customElements.define("request-list",ot);class at extends y{static properties={sessions:{attribute:!1},selected_session_id:{type:String},timezone:{type:String}};createRenderRoot(){return this}selectSession(e){this.dispatchEvent(new CustomEvent("session-select",{detail:e,bubbles:!0,composed:!0}))}render(){const e=this.sessions??[];return e.length===0?n`<p class="empty">No stored sessions yet.</p>`:n`
      <ul class="session-list" aria-label="Sessions">
        ${e.map(t=>n`
            <li>
              <button
                type="button"
                class="session-row ${this.selected_session_id===t.session_id?"selected":""}"
                aria-current=${this.selected_session_id===t.session_id?"true":"false"}
                @click=${()=>this.selectSession(t)}
              >
                <span class="session-count">${t.request_count}</span>
                <span class="session-row-main">
                  <strong>${t.model??t.endpoint??"session"}</strong>
                  <small>${t.provider_id??"unknown provider"} · ${x(t.last_ts,this.timezone)}</small>
                </span>
                <span class="session-row-id" title=${t.session_id}>${M(t.session_id)}</span>
              </button>
            </li>
          `)}
      </ul>
    `}}class nt extends y{static properties={detail:{attribute:!1},timezone:{type:String}};createRenderRoot(){return this}selectRequest(e){this.dispatchEvent(new CustomEvent("request-select",{detail:e,bubbles:!0,composed:!0}))}render(){if(!this.detail)return n`<section class="detail-state"><p>Select a session to see its request timeline.</p></section>`;const{session:e,requests:t}=this.detail;return n`
      <section class="detail-content">
        <header class="detail-header">
          <div class="detail-title">
            <p class="eyebrow">request-history timeline</p>
            <h2>${e.model??e.endpoint??"session"}</h2>
            <p class="muted">${e.session_id}</p>
          </div>
          <span class="session-count">${e.request_count}</span>
        </header>
        <dl class="metadata-grid">
          <div><dt>First seen</dt><dd>${x(e.first_ts,this.timezone)}</dd></div>
          <div><dt>Last seen</dt><dd>${x(e.last_ts,this.timezone)}</dd></div>
          <div><dt>Provider</dt><dd>${e.provider_id??"—"}</dd></div>
          <div><dt>Account</dt><dd>${e.account_id??"—"}</dd></div>
        </dl>
        <section class="timeline">
          <h3>Request timeline</h3>
          <ul>
            ${t.map(s=>{const i=ke(s);return n`
                <li>
                  <button type="button" class="timeline-row" @click=${()=>this.selectRequest(s)}>
                    <time>${x(s.ts,this.timezone)}</time>
                    <span class="status ${i.tone}" title=${i.title}>${i.label}</span>
                    <span>${s.model??s.endpoint??s.request_id}</span>
                    <small title=${s.request_id}>${M(s.request_id)}</small>
                  </button>
                </li>
              `})}
          </ul>
        </section>
      </section>
    `}}customElements.define("session-list",at);customElements.define("session-timeline",nt);const me=100;function w(r,e){return r instanceof Error?r.message:e}function lt(r){return r==="overview"||r==="client"||r==="provider"||r==="raw"}function V(){return{query:"",provider_id:"",status:"",errors_only:!1}}class dt extends y{static properties={active_view:{type:String},info:{attribute:!1},requests:{attribute:!1},request_days:{attribute:!1},selected_day:{type:String},selected_request:{attribute:!1},selected_request_id:{type:String},selected_request_row_id:{type:String},selected_request_detail:{attribute:!1},request_list_state:{type:String},request_list_error:{type:String},request_detail_state:{type:String},request_detail_error:{type:String},next_cursor:{type:String},loading_more:{type:Boolean},load_more_error:{type:String},search_query:{type:String},provider_id:{type:String},status_filter:{type:String},errors_only:{type:Boolean},applied_filters:{attribute:!1},active_detail_tab:{type:String},timezone:{type:String},request_days_loading:{type:Boolean},request_days_error:{type:String},sessions:{attribute:!1},selected_session:{attribute:!1},selected_session_detail:{attribute:!1},sessions_loading:{type:Boolean},sessions_error:{type:String},session_detail_loading:{type:Boolean},session_detail_error:{type:String}};request_load_id=0;request_detail_load_id=0;session_detail_load_id=0;request_days_load_id=0;sessions_loaded=!1;requested_request_id;requested_request_row_id;request_rows_context;request_controller;request_detail_controller;navigation_workflow_id=0;popstate_handler=()=>{this.restoreFromHistory()};constructor(){super(),this.active_view="requests",this.requests=[],this.request_days=[],this.sessions=[],this.request_list_state="idle",this.request_detail_state="idle",this.search_query="",this.provider_id="",this.status_filter="",this.errors_only=!1,this.applied_filters=V(),this.active_detail_tab="overview",this.timezone="local",this.loading_more=!1,this.request_days_loading=!1,this.sessions_loading=!1,this.session_detail_loading=!1}createRenderRoot(){return this}connectedCallback(){super.connectedCallback(),this.restoreUrlState(),window.addEventListener("popstate",this.popstate_handler),this.loadInitialData()}disconnectedCallback(){window.removeEventListener("popstate",this.popstate_handler),this.request_controller?.abort(),this.request_detail_controller?.abort(),super.disconnectedCallback()}restoreUrlState(){const e=new URLSearchParams(window.location.search);this.active_view=e.get("view")==="sessions"?"sessions":"requests";const t=e.get("day");this.selected_day=t&&/^\d{4}-\d{2}-\d{2}$/.test(t)?t:void 0,this.search_query=e.get("query")??"",this.provider_id=e.get("provider_id")??"";const s=e.get("status")??"";this.status_filter=/^\d{3}$/.test(s)?s:"",this.errors_only=e.get("errors_only")==="true"||e.get("errors_only")==="1",this.applied_filters={query:this.search_query,provider_id:this.provider_id,status:this.status_filter,errors_only:this.errors_only},this.requested_request_id=e.get("request_id")??void 0;const i=e.get("row_id");this.requested_request_row_id=i&&/^-?\d+$/.test(i)?i:void 0;const o=e.get("tab");this.active_detail_tab=lt(o)?o:"overview",this.timezone=e.get("timezone")==="utc"?"utc":"local"}selectedRequestDay(){return this.selected_request_detail?.day??this.selected_request?.day??this.selected_day}syncUrl(e="replace"){const t=new URLSearchParams;this.active_view!=="requests"&&t.set("view",this.active_view);const s=this.selected_request_id?this.selectedRequestDay():this.selected_day;s&&t.set("day",s),this.applied_filters.query&&t.set("query",this.applied_filters.query),this.applied_filters.provider_id&&t.set("provider_id",this.applied_filters.provider_id),this.applied_filters.status&&t.set("status",this.applied_filters.status),this.applied_filters.errors_only&&t.set("errors_only","true"),this.selected_request_id&&(t.set("request_id",this.selected_request_id),this.selected_request_row_id&&t.set("row_id",this.selected_request_row_id),t.set("tab",this.active_detail_tab)),t.set("timezone",this.timezone);const i=t.toString(),o=`${window.location.pathname}${i?`?${i}`:""}`;`${window.location.pathname}${window.location.search}`!==o&&(e==="push"?window.history.pushState(null,"",o):window.history.replaceState(null,"",o))}async loadInitialData(){const e=++this.navigation_workflow_id;this.loadInfo(),this.loadRequestDays(),await this.loadUrlState(e)}async restoreFromHistory(){const e=++this.navigation_workflow_id;this.request_controller?.abort(),this.request_detail_controller?.abort(),this.resetRequestSelection(),this.restoreUrlState(),this.requests=[],this.next_cursor=void 0,this.request_rows_context=void 0,await this.loadUrlState(e)}async loadUrlState(e){const t=this.requested_request_id,s=this.requested_request_row_id;this.active_view==="sessions"&&this.ensureSessionsLoaded();let i;if(this.selected_day?i=await this.loadRequests():(i=await this.loadLatestRequests(),i&&this.selected_day&&this.hasAppliedFilters()&&(i=await this.loadRequests())),!(!i||e!==this.navigation_workflow_id)&&t&&this.selected_day){const o=this.requests.find(a=>a.request_id===t&&(!s||a.row_id===s));await this.loadRequestDetail(this.selected_day,t,s??o?.row_id,o,!1,null)}}async loadInfo(){try{this.info=await b("/api/info")}catch{this.info=void 0}}async loadLatestRequests(){this.request_controller?.abort();const e=new AbortController;this.request_controller=e;const t=++this.request_load_id;this.requests=[],this.next_cursor=void 0,this.request_rows_context=void 0,this.request_list_state="loading",this.request_list_error=void 0;try{const s=await b(`/api/requests/latest?limit=${me}`,e.signal);return t!==this.request_load_id||this.request_controller!==e?!1:(this.selected_day=s.day??void 0,this.requests=s.requests,this.next_cursor=s.next_cursor??void 0,this.request_rows_context=this.requestContext(this.selected_day,V()),this.request_list_state="ready",this.syncUrl(),!0)}catch(s){return t===this.request_load_id&&!W(s)&&(this.request_list_state="error",this.request_list_error=w(s,"Unable to load recent requests")),!1}finally{this.request_controller===e&&(this.request_controller=void 0)}}requestContext(e=this.selected_day,t=this.applied_filters){return e?JSON.stringify([e,t.query,t.provider_id,t.status,t.errors_only]):void 0}requestParams(e,t,s){const i=new URLSearchParams({day:e,limit:String(me)});return t.query&&i.set("query",t.query),t.provider_id&&i.set("provider_id",t.provider_id),t.status&&i.set("status",t.status),t.errors_only&&i.set("errors_only","true"),s&&i.set("cursor",s),i}async loadRequests(e=!1){const t=this.selected_day;if(!t)return this.request_list_state="idle",this.requests=[],this.next_cursor=void 0,this.request_rows_context=void 0,!1;const s={...this.applied_filters},i=this.requestContext(t,s),o=e?this.next_cursor:void 0;if(e&&(!o||this.request_rows_context!==i))return!1;this.request_controller?.abort();const a=new AbortController;this.request_controller=a;const c=++this.request_load_id;e?(this.loading_more=!0,this.load_more_error=void 0):(this.loading_more=!1,this.request_rows_context!==i&&(this.requests=[],this.next_cursor=void 0,this.request_rows_context=void 0),this.request_list_state="loading",this.request_list_error=void 0,this.load_more_error=void 0);try{const l=await b(`/api/requests?${this.requestParams(t,s,o).toString()}`,a.signal);if(c!==this.request_load_id||this.request_controller!==a||this.requestContext()!==i)return!1;if(e){const h=new Set(this.requests.map(u=>C(u)));this.requests=[...this.requests,...l.requests.filter(u=>!h.has(C(u)))]}else this.requests=l.requests;return this.next_cursor=l.next_cursor??void 0,this.request_rows_context=i,this.request_list_state="ready",!0}catch(l){return c!==this.request_load_id||W(l)||(l instanceof Ee&&l.status===503&&this.markRequestDayUnavailable(t),e?this.load_more_error=w(l,"Unable to load more requests"):(this.request_list_state="error",this.request_list_error=w(l,"Unable to load requests"))),!1}finally{c===this.request_load_id&&(this.loading_more=!1),this.request_controller===a&&(this.request_controller=void 0)}}async loadRequestDays(){const e=++this.request_days_load_id;this.request_days_loading=!0,this.request_days_error=void 0;try{const t=await b("/api/request-days");e===this.request_days_load_id&&(this.request_days=t)}catch(t){e===this.request_days_load_id&&(this.request_days_error=w(t,"Unable to load request day states"))}finally{e===this.request_days_load_id&&(this.request_days_loading=!1)}}markRequestDayUnavailable(e){this.request_days.some(t=>t.day===e)?this.request_days=this.request_days.map(t=>t.day===e?{...t,state:"unavailable"}:t):this.request_days=[{day:e,state:"unavailable"},...this.request_days]}resetRequestSelection(){this.request_detail_controller?.abort(),this.request_detail_controller=void 0,this.request_detail_load_id+=1,this.selected_request=void 0,this.selected_request_id=void 0,this.selected_request_row_id=void 0,this.selected_request_detail=void 0,this.request_detail_state="idle",this.request_detail_error=void 0,this.active_detail_tab="overview"}async closeRequestDetail(){const e=this.selected_request_row_id&&this.selectedRequestDay()?C({day:this.selectedRequestDay(),row_id:this.selected_request_row_id}):void 0;if(++this.navigation_workflow_id,this.resetRequestSelection(),this.syncUrl("push"),!e||!window.matchMedia("(max-width: 680px)").matches)return;await this.updateComplete,[...this.querySelectorAll("request-list [data-request-key]")].find(s=>s.dataset.requestKey===e)?.focus()}async loadRequestDetail(e,t,s,i,o,a="replace"){this.request_detail_controller?.abort();const c=new AbortController;this.request_detail_controller=c;const l=++this.request_detail_load_id;this.selected_day=e,this.selected_request=i,this.selected_request_id=t,this.selected_request_row_id=s,o||(this.selected_request_detail=void 0),this.request_detail_state="loading",this.request_detail_error=void 0,a&&this.syncUrl(a);try{const h=new URLSearchParams({day:e,request_id:t});s&&h.set("row_id",s);const u=await b(`/api/request?${h.toString()}`,c.signal);if(l===this.request_detail_load_id&&this.request_detail_controller===c){const p=this.selected_request_row_id!==u.row_id;return this.selected_request_detail=u,this.selected_request_row_id=u.row_id,this.request_detail_state="ready",(a||p)&&this.syncUrl("replace"),!0}return!1}catch(h){return l===this.request_detail_load_id&&!W(h)&&(this.request_detail_state="error",this.request_detail_error=w(h,"Unable to load request detail")),!1}finally{this.request_detail_controller===c&&(this.request_detail_controller=void 0)}}async selectRequest(e){++this.navigation_workflow_id;const t=this.selected_request_id===e.request_id&&this.selected_request_detail?.day===e.day&&this.selected_request_detail.row_id===e.row_id,s=this.loadRequestDetail(e.day,e.request_id,e.row_id,e,t,"push");window.matchMedia("(max-width: 680px)").matches&&(await this.updateComplete,this.querySelector("request-detail-view .mobile-back-button")?.focus()),await s&&window.matchMedia("(max-width: 680px)").matches&&(await this.updateComplete,this.querySelector("request-detail-view .mobile-back-button")?.focus())}retryRequestDetail(){const e=this.selected_request_detail?.day??this.selected_request?.day??this.selected_day;e&&this.selected_request_id&&this.loadRequestDetail(e,this.selected_request_id,this.selected_request_row_id,this.selected_request,!!this.selected_request_detail,null)}selectDay(e){e!==this.selected_day&&(++this.navigation_workflow_id,this.selected_day=e,this.resetRequestSelection(),this.syncUrl("push"),this.loadRequests())}pickerDays(){return!this.selected_day||this.request_days.some(e=>e.day===this.selected_day)?this.request_days:[{day:this.selected_day,state:"available"},...this.request_days]}adjacentAvailableDay(e){const t=this.pickerDays().filter(i=>i.state==="available").map(i=>i.day).sort();if(!this.selected_day)return;const s=t.indexOf(this.selected_day);return s<0?void 0:t[s+e]}submitFilters(e){e.preventDefault(),++this.navigation_workflow_id,this.applied_filters={query:this.search_query.trim(),provider_id:this.provider_id.trim(),status:this.status_filter.trim(),errors_only:this.errors_only},this.resetRequestSelection(),this.syncUrl("push"),this.loadRequests()}clearFilters(){this.search_query="",this.provider_id="",this.status_filter="",this.errors_only=!1,this.applied_filters=V(),++this.navigation_workflow_id,this.resetRequestSelection(),this.syncUrl("push"),this.loadRequests()}hasAppliedFilters(){return!!(this.applied_filters.query||this.applied_filters.provider_id||this.applied_filters.status||this.applied_filters.errors_only)}filtersChanged(){return this.search_query.trim()!==this.applied_filters.query||this.provider_id.trim()!==this.applied_filters.provider_id||this.status_filter.trim()!==this.applied_filters.status||this.errors_only!==this.applied_filters.errors_only}providerOptions(){const e=new Set(this.requests.flatMap(t=>t.provider_id?[t.provider_id]:[]));return this.applied_filters.provider_id&&e.add(this.applied_filters.provider_id),[...e].sort()}async ensureSessionsLoaded(){if(!(this.sessions_loaded||this.sessions_loading)){this.sessions_loading=!0,this.sessions_error=void 0;try{this.sessions=await b("/api/sessions?limit=100"),this.sessions_loaded=!0}catch(e){this.sessions_error=w(e,"Unable to load sessions")}finally{this.sessions_loading=!1}}}retrySessions(){this.sessions_loaded=!1,this.ensureSessionsLoaded()}async loadSession(e,t){const s=++this.session_detail_load_id;this.selected_session=t,this.selected_session_detail=void 0,this.session_detail_loading=!0,this.session_detail_error=void 0;try{const i=await b(`/api/session?session_id=${encodeURIComponent(e)}&limit=500`);s===this.session_detail_load_id&&(this.selected_session=i.session,this.selected_session_detail=i)}catch(i){s===this.session_detail_load_id&&(this.session_detail_error=w(i,"Unable to load session timeline"))}finally{s===this.session_detail_load_id&&(this.session_detail_loading=!1)}}async openSession(e){++this.navigation_workflow_id,this.setActiveView("sessions",!1,"push");const t=this.sessions.find(s=>s.session_id===e);await this.loadSession(e,t)}async openRequest(e){const t=++this.navigation_workflow_id;this.setActiveView("requests",!0,null),!((this.selected_day!==e.day||this.hasAppliedFilters())&&(this.selected_day=e.day,this.search_query="",this.provider_id="",this.status_filter="",this.errors_only=!1,this.applied_filters=V(),this.resetRequestSelection(),!await this.loadRequests()||t!==this.navigation_workflow_id))&&await this.selectRequest(e)}setActiveView(e,t=!0,s="push"){s==="push"&&++this.navigation_workflow_id,this.active_view=e,s&&this.syncUrl(s),e==="sessions"&&t&&this.ensureSessionsLoaded()}setTimezone(e){this.timezone=e,this.syncUrl("push")}setDetailTab(e){this.active_detail_tab=e,this.syncUrl("push")}renderDayPicker(){const e=this.pickerDays(),t=this.adjacentAvailableDay(-1),s=this.adjacentAvailableDay(1);return n`
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
            ${this.selected_day?d:n`<option value="">No request day</option>`}
            ${e.map(i=>n`
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
    `}renderRequestToolbar(){const e=!!this.selected_day;return n`
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
              ${this.providerOptions().map(t=>n`<option value=${t}></option>`)}
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
          ${this.hasAppliedFilters()?n`<button type="button" class="text-button" @click=${this.clearFilters}>Clear</button>`:d}
        </form>
        ${this.request_days_error?n`<p class="toolbar-warning" role="status">Day scan: ${this.request_days_error}</p>`:d}
      </section>
    `}renderRequestSidebar(){const e=this.requests.length>0;return n`
      <div class="list-pane" aria-busy=${String(this.request_list_state==="loading")}>
        <header class="list-pane-header">
          <div>
            <strong>Requests</strong>
            <span>${this.requests.length.toLocaleString()} loaded${this.next_cursor?" · more available":""}</span>
          </div>
          ${this.hasAppliedFilters()?n`<span class="filter-indicator">Filtered</span>`:d}
        </header>
        ${this.request_list_state==="loading"?n`
              <div class="inline-state" role="status">
                <span class="spinner" aria-hidden="true"></span>${e?"Refreshing requests…":"Loading requests…"}
              </div>
            `:d}
        ${this.request_list_state==="error"?n`
              <div class="inline-error" role="alert">
                <span>${this.request_list_error}</span>
                <button type="button" @click=${()=>{this.loadRequests()}}>Retry</button>
              </div>
            `:d}
        ${e?n`
              <request-list
                .requests=${this.requests}
                .selected_key=${this.selectedRequestDay()&&this.selected_request_row_id?C({day:this.selectedRequestDay(),row_id:this.selected_request_row_id}):void 0}
                .timezone=${this.timezone}
                @request-select=${t=>{this.selectRequest(R(t))}}
              ></request-list>
            `:this.request_list_state==="ready"?n`<p class="empty">No persisted requests match these filters.</p>`:this.request_list_state==="idle"?n`<p class="empty">Choose an available request day.</p>`:d}
        ${this.load_more_error?n`
              <div class="inline-error load-more-error" role="alert">
                <span>${this.load_more_error}</span>
                <button type="button" @click=${()=>{this.loadRequests(!0)}}>Retry</button>
              </div>
            `:d}
        ${this.next_cursor&&e?n`
              <div class="list-footer">
                <button type="button" class="secondary-button" ?disabled=${this.loading_more} @click=${()=>{this.loadRequests(!0)}}>
                  ${this.loading_more?"Loading…":"Load more"}
                </button>
              </div>
            `:e&&this.request_list_state==="ready"?n`<p class="end-of-list">End of loaded day</p>`:d}
      </div>
    `}renderSessionsSidebar(){return this.sessions_loading?n`<div class="inline-state"><span class="spinner" aria-hidden="true"></span>Loading sessions…</div>`:this.sessions_error?n`
        <div class="inline-error" role="alert">
          <span>${this.sessions_error}</span>
          <button type="button" @click=${this.retrySessions}>Retry</button>
        </div>
      `:this.sessions_loaded?n`
      <session-list
        .sessions=${this.sessions}
        .selected_session_id=${this.selected_session?.session_id}
        .timezone=${this.timezone}
        @session-select=${e=>{this.loadSession(R(e).session_id,R(e))}}
      ></session-list>
    `:n`<button type="button" class="primary-button standalone-action" @click=${()=>{this.ensureSessionsLoaded()}}>
        Load session list
      </button>`}renderSessionDetail(){return n`
      ${this.session_detail_loading?n`<div class="inline-state"><span class="spinner" aria-hidden="true"></span>Loading session timeline…</div>`:d}
      ${this.session_detail_error?n`<div class="inline-error" role="alert"><span>${this.session_detail_error}</span></div>`:d}
      <session-timeline
        .detail=${this.selected_session_detail}
        .timezone=${this.timezone}
        @request-select=${e=>{this.openRequest(R(e))}}
      ></session-timeline>
    `}render(){const e=this.active_view==="sessions"?this.info?.sessions_db:this.info?.requests_dir,t=this.active_view==="requests"&&!!this.selected_request_id;return n`
      <header class="app-header">
        <div class="brand">
          <span class="brand-mark" aria-hidden="true">t</span>
          <div><h1>tokn viewer</h1><p>Local · read only</p></div>
        </div>
        <p class="sensitive-notice">History may contain sensitive prompts and responses.</p>
      </header>
      <main class="app-shell">
        <div class="shell-navigation">
          <nav class="view-navigation" aria-label="Viewer sections">
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
        ${this.active_view==="requests"?this.renderRequestToolbar():n`
              <section class="session-toolbar">
                <p>Session list from <code>sessions.db</code>; timeline payloads from request history.</p>
                <div class="timezone-toggle" role="group" aria-label="Timestamp timezone">
                  <button type="button" aria-pressed=${String(this.timezone==="local")} @click=${()=>this.setTimezone("local")}>Local</button>
                  <button type="button" aria-pressed=${String(this.timezone==="utc")} @click=${()=>this.setTimezone("utc")}>UTC</button>
                </div>
              </section>
            `}
        <section class="viewer-grid ${this.active_view==="requests"?"request-view":"session-view"} ${t?"has-selection":""}">
          <aside class="sidebar" aria-label=${this.active_view==="requests"?"Request list":"Session list"}>
            ${this.active_view==="requests"?this.renderRequestSidebar():this.renderSessionsSidebar()}
          </aside>
          <article class="detail-pane" aria-label=${this.active_view==="requests"?"Request detail":"Session detail"}>
            ${this.active_view==="requests"?n`
                  <request-detail-view
                    .detail=${this.selected_request_detail}
                    .summary=${this.selected_request}
                    .state=${this.request_detail_state}
                    .error_message=${this.request_detail_error}
                    .active_tab=${this.active_detail_tab}
                    .timezone=${this.timezone}
                    @detail-retry=${this.retryRequestDetail}
                    @detail-close=${()=>{this.closeRequestDetail()}}
                    @detail-tab-change=${s=>this.setDetailTab(R(s))}
                    @open-session=${s=>{this.openSession(R(s))}}
                  ></request-detail-view>
                `:this.renderSessionDetail()}
          </article>
        </section>
      </main>
    `}}customElements.define("viewer-app",dt);
