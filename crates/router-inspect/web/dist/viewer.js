(function(){const e=document.createElement("link").relList;if(e&&e.supports&&e.supports("modulepreload"))return;for(const i of document.querySelectorAll('link[rel="modulepreload"]'))s(i);new MutationObserver(i=>{for(const n of i)if(n.type==="childList")for(const o of n.addedNodes)o.tagName==="LINK"&&o.rel==="modulepreload"&&s(o)}).observe(document,{childList:!0,subtree:!0});function t(i){const n={};return i.integrity&&(n.integrity=i.integrity),i.referrerPolicy&&(n.referrerPolicy=i.referrerPolicy),i.crossOrigin==="use-credentials"?n.credentials="include":i.crossOrigin==="anonymous"?n.credentials="omit":n.credentials="same-origin",n}function s(i){if(i.ep)return;i.ep=!0;const n=t(i);fetch(i.href,n)}})();const j=globalThis,L=j.ShadowRoot&&(j.ShadyCSS===void 0||j.ShadyCSS.nativeShadow)&&"adoptedStyleSheets"in Document.prototype&&"replace"in CSSStyleSheet.prototype,te=Symbol(),V=new WeakMap;let ce=class{constructor(e,t,s){if(this._$cssResult$=!0,s!==te)throw Error("CSSResult is not constructable. Use `unsafeCSS` or `css` instead.");this.cssText=e,this.t=t}get styleSheet(){let e=this.o;const t=this.t;if(L&&e===void 0){const s=t!==void 0&&t.length===1;s&&(e=V.get(t)),e===void 0&&((this.o=e=new CSSStyleSheet).replaceSync(this.cssText),s&&V.set(t,e))}return e}toString(){return this.cssText}};const he=r=>new ce(typeof r=="string"?r:r+"",void 0,te),ue=(r,e)=>{if(L)r.adoptedStyleSheets=e.map(t=>t instanceof CSSStyleSheet?t:t.styleSheet);else for(const t of e){const s=document.createElement("style"),i=j.litNonce;i!==void 0&&s.setAttribute("nonce",i),s.textContent=t.cssText,r.appendChild(s)}},W=L?r=>r:r=>r instanceof CSSStyleSheet?(e=>{let t="";for(const s of e.cssRules)t+=s.cssText;return he(t)})(r):r;const{is:pe,defineProperty:_e,getOwnPropertyDescriptor:$e,getOwnPropertyNames:fe,getOwnPropertySymbols:ye,getPrototypeOf:me}=Object,H=globalThis,J=H.trustedTypes,ve=J?J.emptyScript:"",ge=H.reactiveElementPolyfillSupport,R=(r,e)=>r,M={toAttribute(r,e){switch(e){case Boolean:r=r?ve:null;break;case Object:case Array:r=r==null?r:JSON.stringify(r)}return r},fromAttribute(r,e){let t=r;switch(e){case Boolean:t=r!==null;break;case Number:t=r===null?null:Number(r);break;case Object:case Array:try{t=JSON.parse(r)}catch{t=null}}return t}},se=(r,e)=>!pe(r,e),F={attribute:!0,type:String,converter:M,reflect:!1,useDefault:!1,hasChanged:se};Symbol.metadata??=Symbol("metadata"),H.litPropertyMetadata??=new WeakMap;let q=class extends HTMLElement{static addInitializer(e){this._$Ei(),(this.l??=[]).push(e)}static get observedAttributes(){return this.finalize(),this._$Eh&&[...this._$Eh.keys()]}static createProperty(e,t=F){if(t.state&&(t.attribute=!1),this._$Ei(),this.prototype.hasOwnProperty(e)&&((t=Object.create(t)).wrapped=!0),this.elementProperties.set(e,t),!t.noAccessor){const s=Symbol(),i=this.getPropertyDescriptor(e,s,t);i!==void 0&&_e(this.prototype,e,i)}}static getPropertyDescriptor(e,t,s){const{get:i,set:n}=$e(this.prototype,e)??{get(){return this[t]},set(o){this[t]=o}};return{get:i,set(o){const h=i?.call(this);n?.call(this,o),this.requestUpdate(e,h,s)},configurable:!0,enumerable:!0}}static getPropertyOptions(e){return this.elementProperties.get(e)??F}static _$Ei(){if(this.hasOwnProperty(R("elementProperties")))return;const e=me(this);e.finalize(),e.l!==void 0&&(this.l=[...e.l]),this.elementProperties=new Map(e.elementProperties)}static finalize(){if(this.hasOwnProperty(R("finalized")))return;if(this.finalized=!0,this._$Ei(),this.hasOwnProperty(R("properties"))){const t=this.properties,s=[...fe(t),...ye(t)];for(const i of s)this.createProperty(i,t[i])}const e=this[Symbol.metadata];if(e!==null){const t=litPropertyMetadata.get(e);if(t!==void 0)for(const[s,i]of t)this.elementProperties.set(s,i)}this._$Eh=new Map;for(const[t,s]of this.elementProperties){const i=this._$Eu(t,s);i!==void 0&&this._$Eh.set(i,t)}this.elementStyles=this.finalizeStyles(this.styles)}static finalizeStyles(e){const t=[];if(Array.isArray(e)){const s=new Set(e.flat(1/0).reverse());for(const i of s)t.unshift(W(i))}else e!==void 0&&t.push(W(e));return t}static _$Eu(e,t){const s=t.attribute;return s===!1?void 0:typeof s=="string"?s:typeof e=="string"?e.toLowerCase():void 0}constructor(){super(),this._$Ep=void 0,this.isUpdatePending=!1,this.hasUpdated=!1,this._$Em=null,this._$Ev()}_$Ev(){this._$ES=new Promise(e=>this.enableUpdating=e),this._$AL=new Map,this._$E_(),this.requestUpdate(),this.constructor.l?.forEach(e=>e(this))}addController(e){(this._$EO??=new Set).add(e),this.renderRoot!==void 0&&this.isConnected&&e.hostConnected?.()}removeController(e){this._$EO?.delete(e)}_$E_(){const e=new Map,t=this.constructor.elementProperties;for(const s of t.keys())this.hasOwnProperty(s)&&(e.set(s,this[s]),delete this[s]);e.size>0&&(this._$Ep=e)}createRenderRoot(){const e=this.shadowRoot??this.attachShadow(this.constructor.shadowRootOptions);return ue(e,this.constructor.elementStyles),e}connectedCallback(){this.renderRoot??=this.createRenderRoot(),this.enableUpdating(!0),this._$EO?.forEach(e=>e.hostConnected?.())}enableUpdating(e){}disconnectedCallback(){this._$EO?.forEach(e=>e.hostDisconnected?.())}attributeChangedCallback(e,t,s){this._$AK(e,s)}_$ET(e,t){const s=this.constructor.elementProperties.get(e),i=this.constructor._$Eu(e,s);if(i!==void 0&&s.reflect===!0){const n=(s.converter?.toAttribute!==void 0?s.converter:M).toAttribute(t,s.type);this._$Em=e,n==null?this.removeAttribute(i):this.setAttribute(i,n),this._$Em=null}}_$AK(e,t){const s=this.constructor,i=s._$Eh.get(e);if(i!==void 0&&this._$Em!==i){const n=s.getPropertyOptions(i),o=typeof n.converter=="function"?{fromAttribute:n.converter}:n.converter?.fromAttribute!==void 0?n.converter:M;this._$Em=i;const h=o.fromAttribute(t,n.type);this[i]=h??this._$Ej?.get(i)??h,this._$Em=null}}requestUpdate(e,t,s,i=!1,n){if(e!==void 0){const o=this.constructor;if(i===!1&&(n=this[e]),s??=o.getPropertyOptions(e),!((s.hasChanged??se)(n,t)||s.useDefault&&s.reflect&&n===this._$Ej?.get(e)&&!this.hasAttribute(o._$Eu(e,s))))return;this.C(e,t,s)}this.isUpdatePending===!1&&(this._$ES=this._$EP())}C(e,t,{useDefault:s,reflect:i,wrapped:n},o){s&&!(this._$Ej??=new Map).has(e)&&(this._$Ej.set(e,o??t??this[e]),n!==!0||o!==void 0)||(this._$AL.has(e)||(this.hasUpdated||s||(t=void 0),this._$AL.set(e,t)),i===!0&&this._$Em!==e&&(this._$Eq??=new Set).add(e))}async _$EP(){this.isUpdatePending=!0;try{await this._$ES}catch(t){Promise.reject(t)}const e=this.scheduleUpdate();return e!=null&&await e,!this.isUpdatePending}scheduleUpdate(){return this.performUpdate()}performUpdate(){if(!this.isUpdatePending)return;if(!this.hasUpdated){if(this.renderRoot??=this.createRenderRoot(),this._$Ep){for(const[i,n]of this._$Ep)this[i]=n;this._$Ep=void 0}const s=this.constructor.elementProperties;if(s.size>0)for(const[i,n]of s){const{wrapped:o}=n,h=this[i];o!==!0||this._$AL.has(i)||h===void 0||this.C(i,void 0,n,h)}}let e=!1;const t=this._$AL;try{e=this.shouldUpdate(t),e?(this.willUpdate(t),this._$EO?.forEach(s=>s.hostUpdate?.()),this.update(t)):this._$EM()}catch(s){throw e=!1,this._$EM(),s}e&&this._$AE(t)}willUpdate(e){}_$AE(e){this._$EO?.forEach(t=>t.hostUpdated?.()),this.hasUpdated||(this.hasUpdated=!0,this.firstUpdated(e)),this.updated(e)}_$EM(){this._$AL=new Map,this.isUpdatePending=!1}get updateComplete(){return this.getUpdateComplete()}getUpdateComplete(){return this._$ES}shouldUpdate(e){return!0}update(e){this._$Eq&&=this._$Eq.forEach(t=>this._$ET(t,this[t])),this._$EM()}updated(e){}firstUpdated(e){}};q.elementStyles=[],q.shadowRootOptions={mode:"open"},q[R("elementProperties")]=new Map,q[R("finalized")]=new Map,ge?.({ReactiveElement:q}),(H.reactiveElementVersions??=[]).push("2.1.2");const I=globalThis,K=r=>r,N=I.trustedTypes,Z=N?N.createPolicy("lit-html",{createHTML:r=>r}):void 0,ie="$lit$",y=`lit$${Math.random().toFixed(9).slice(2)}$`,re="?"+y,be=`<${re}>`,b=document,C=()=>b.createComment(""),P=r=>r===null||typeof r!="object"&&typeof r!="function",B=Array.isArray,qe=r=>B(r)||typeof r?.[Symbol.iterator]=="function",D=`[ 	
\f\r]`,E=/<(?:(!--|\/[^a-zA-Z])|(\/?[a-zA-Z][^>\s]*)|(\/?$))/g,G=/-->/g,Q=/>/g,m=RegExp(`>|${D}(?:([^\\s"'>=/]+)(${D}*=${D}*(?:[^ 	
\f\r"'\`<>=]|("|')|))|$)`,"g"),X=/'/g,Y=/"/g,ne=/^(?:script|style|textarea|title)$/i,Ae=r=>(e,...t)=>({_$litType$:r,strings:e,values:t}),l=Ae(1),w=Symbol.for("lit-noChange"),d=Symbol.for("lit-nothing"),ee=new WeakMap,g=b.createTreeWalker(b,129);function oe(r,e){if(!B(r)||!r.hasOwnProperty("raw"))throw Error("invalid template strings array");return Z!==void 0?Z.createHTML(e):e}const we=(r,e)=>{const t=r.length-1,s=[];let i,n=e===2?"<svg>":e===3?"<math>":"",o=E;for(let h=0;h<t;h++){const a=r[h];let u,p,c=-1,$=0;for(;$<a.length&&(o.lastIndex=$,p=o.exec(a),p!==null);)$=o.lastIndex,o===E?p[1]==="!--"?o=G:p[1]!==void 0?o=Q:p[2]!==void 0?(ne.test(p[2])&&(i=RegExp("</"+p[2],"g")),o=m):p[3]!==void 0&&(o=m):o===m?p[0]===">"?(o=i??E,c=-1):p[1]===void 0?c=-2:(c=o.lastIndex-p[2].length,u=p[1],o=p[3]===void 0?m:p[3]==='"'?Y:X):o===Y||o===X?o=m:o===G||o===Q?o=E:(o=m,i=void 0);const f=o===m&&r[h+1].startsWith("/>")?" ":"";n+=o===E?a+be:c>=0?(s.push(u),a.slice(0,c)+ie+a.slice(c)+y+f):a+y+(c===-2?h:f)}return[oe(r,n+(r[t]||"<?>")+(e===2?"</svg>":e===3?"</math>":"")),s]};class U{constructor({strings:e,_$litType$:t},s){let i;this.parts=[];let n=0,o=0;const h=e.length-1,a=this.parts,[u,p]=we(e,t);if(this.el=U.createElement(u,s),g.currentNode=this.el.content,t===2||t===3){const c=this.el.content.firstChild;c.replaceWith(...c.childNodes)}for(;(i=g.nextNode())!==null&&a.length<h;){if(i.nodeType===1){if(i.hasAttributes())for(const c of i.getAttributeNames())if(c.endsWith(ie)){const $=p[o++],f=i.getAttribute(c).split(y),x=/([.?@])?(.*)/.exec($);a.push({type:1,index:n,name:x[2],strings:f,ctor:x[1]==="."?Ee:x[1]==="?"?Re:x[1]==="@"?Ce:T}),i.removeAttribute(c)}else c.startsWith(y)&&(a.push({type:6,index:n}),i.removeAttribute(c));if(ne.test(i.tagName)){const c=i.textContent.split(y),$=c.length-1;if($>0){i.textContent=N?N.emptyScript:"";for(let f=0;f<$;f++)i.append(c[f],C()),g.nextNode(),a.push({type:2,index:++n});i.append(c[$],C())}}}else if(i.nodeType===8)if(i.data===re)a.push({type:2,index:n});else{let c=-1;for(;(c=i.data.indexOf(y,c+1))!==-1;)a.push({type:7,index:n}),c+=y.length-1}n++}}static createElement(e,t){const s=b.createElement("template");return s.innerHTML=e,s}}function S(r,e,t=r,s){if(e===w)return e;let i=s!==void 0?t._$Co?.[s]:t._$Cl;const n=P(e)?void 0:e._$litDirective$;return i?.constructor!==n&&(i?._$AO?.(!1),n===void 0?i=void 0:(i=new n(r),i._$AT(r,t,s)),s!==void 0?(t._$Co??=[])[s]=i:t._$Cl=i),i!==void 0&&(e=S(r,i._$AS(r,e.values),i,s)),e}class Se{constructor(e,t){this._$AV=[],this._$AN=void 0,this._$AD=e,this._$AM=t}get parentNode(){return this._$AM.parentNode}get _$AU(){return this._$AM._$AU}u(e){const{el:{content:t},parts:s}=this._$AD,i=(e?.creationScope??b).importNode(t,!0);g.currentNode=i;let n=g.nextNode(),o=0,h=0,a=s[0];for(;a!==void 0;){if(o===a.index){let u;a.type===2?u=new k(n,n.nextSibling,this,e):a.type===1?u=new a.ctor(n,a.name,a.strings,this,e):a.type===6&&(u=new Pe(n,this,e)),this._$AV.push(u),a=s[++h]}o!==a?.index&&(n=g.nextNode(),o++)}return g.currentNode=b,i}p(e){let t=0;for(const s of this._$AV)s!==void 0&&(s.strings!==void 0?(s._$AI(e,s,t),t+=s.strings.length-2):s._$AI(e[t])),t++}}class k{get _$AU(){return this._$AM?._$AU??this._$Cv}constructor(e,t,s,i){this.type=2,this._$AH=d,this._$AN=void 0,this._$AA=e,this._$AB=t,this._$AM=s,this.options=i,this._$Cv=i?.isConnected??!0}get parentNode(){let e=this._$AA.parentNode;const t=this._$AM;return t!==void 0&&e?.nodeType===11&&(e=t.parentNode),e}get startNode(){return this._$AA}get endNode(){return this._$AB}_$AI(e,t=this){e=S(this,e,t),P(e)?e===d||e==null||e===""?(this._$AH!==d&&this._$AR(),this._$AH=d):e!==this._$AH&&e!==w&&this._(e):e._$litType$!==void 0?this.$(e):e.nodeType!==void 0?this.T(e):qe(e)?this.k(e):this._(e)}O(e){return this._$AA.parentNode.insertBefore(e,this._$AB)}T(e){this._$AH!==e&&(this._$AR(),this._$AH=this.O(e))}_(e){this._$AH!==d&&P(this._$AH)?this._$AA.nextSibling.data=e:this.T(b.createTextNode(e)),this._$AH=e}$(e){const{values:t,_$litType$:s}=e,i=typeof s=="number"?this._$AC(e):(s.el===void 0&&(s.el=U.createElement(oe(s.h,s.h[0]),this.options)),s);if(this._$AH?._$AD===i)this._$AH.p(t);else{const n=new Se(i,this),o=n.u(this.options);n.p(t),this.T(o),this._$AH=n}}_$AC(e){let t=ee.get(e.strings);return t===void 0&&ee.set(e.strings,t=new U(e)),t}k(e){B(this._$AH)||(this._$AH=[],this._$AR());const t=this._$AH;let s,i=0;for(const n of e)i===t.length?t.push(s=new k(this.O(C()),this.O(C()),this,this.options)):s=t[i],s._$AI(n),i++;i<t.length&&(this._$AR(s&&s._$AB.nextSibling,i),t.length=i)}_$AR(e=this._$AA.nextSibling,t){for(this._$AP?.(!1,!0,t);e!==this._$AB;){const s=K(e).nextSibling;K(e).remove(),e=s}}setConnected(e){this._$AM===void 0&&(this._$Cv=e,this._$AP?.(e))}}class T{get tagName(){return this.element.tagName}get _$AU(){return this._$AM._$AU}constructor(e,t,s,i,n){this.type=1,this._$AH=d,this._$AN=void 0,this.element=e,this.name=t,this._$AM=i,this.options=n,s.length>2||s[0]!==""||s[1]!==""?(this._$AH=Array(s.length-1).fill(new String),this.strings=s):this._$AH=d}_$AI(e,t=this,s,i){const n=this.strings;let o=!1;if(n===void 0)e=S(this,e,t,0),o=!P(e)||e!==this._$AH&&e!==w,o&&(this._$AH=e);else{const h=e;let a,u;for(e=n[0],a=0;a<n.length-1;a++)u=S(this,h[s+a],t,a),u===w&&(u=this._$AH[a]),o||=!P(u)||u!==this._$AH[a],u===d?e=d:e!==d&&(e+=(u??"")+n[a+1]),this._$AH[a]=u}o&&!i&&this.j(e)}j(e){e===d?this.element.removeAttribute(this.name):this.element.setAttribute(this.name,e??"")}}class Ee extends T{constructor(){super(...arguments),this.type=3}j(e){this.element[this.name]=e===d?void 0:e}}class Re extends T{constructor(){super(...arguments),this.type=4}j(e){this.element.toggleAttribute(this.name,!!e&&e!==d)}}class Ce extends T{constructor(e,t,s,i,n){super(e,t,s,i,n),this.type=5}_$AI(e,t=this){if((e=S(this,e,t,0)??d)===w)return;const s=this._$AH,i=e===d&&s!==d||e.capture!==s.capture||e.once!==s.once||e.passive!==s.passive,n=e!==d&&(s===d||i);i&&this.element.removeEventListener(this.name,this,s),n&&this.element.addEventListener(this.name,this,e),this._$AH=e}handleEvent(e){typeof this._$AH=="function"?this._$AH.call(this.options?.host??this.element,e):this._$AH.handleEvent(e)}}class Pe{constructor(e,t,s){this.element=e,this.type=6,this._$AN=void 0,this._$AM=t,this.options=s}get _$AU(){return this._$AM._$AU}_$AI(e){S(this,e)}}const Ue=I.litHtmlPolyfillSupport;Ue?.(U,k),(I.litHtmlVersions??=[]).push("3.3.3");const ke=(r,e,t)=>{const s=t?.renderBefore??e;let i=s._$litPart$;if(i===void 0){const n=t?.renderBefore??null;s._$litPart$=i=new k(e.insertBefore(C(),n),n,void 0,t??{})}return i._$AI(r),i};const z=globalThis;class _ extends q{constructor(){super(...arguments),this.renderOptions={host:this},this._$Do=void 0}createRenderRoot(){const e=super.createRenderRoot();return this.renderOptions.renderBefore??=e.firstChild,e}update(e){const t=this.render();this.hasUpdated||(this.renderOptions.isConnected=this.isConnected),super.update(e),this._$Do=ke(t,this.renderRoot,this.renderOptions)}connectedCallback(){super.connectedCallback(),this._$Do?.setConnected(!0)}disconnectedCallback(){super.disconnectedCallback(),this._$Do?.setConnected(!1)}render(){return w}}_._$litElement$=!0,_.finalized=!0,z.litElementHydrateSupport?.({LitElement:_});const xe=z.litElementPolyfillSupport;xe?.({LitElement:_});(z.litElementVersions??=[]).push("4.2.2");class ae extends Error{status;constructor(e,t){super(t),this.name="HttpError",this.status=e}}async function v(r){const e=await fetch(r,{cache:"no-store"});if(!e.ok){const t=await e.json().catch(()=>({}));throw new ae(e.status,t.error??`Request failed (${e.status})`)}return e.json()}function A(r){return new Intl.DateTimeFormat(void 0,{dateStyle:"medium",timeStyle:"medium"}).format(new Date(r))}function le(r){return r===null?"—":String(r)}function de(r){return`${r.day}:${r.request_id}`}function O(r){return r.detail}class Oe extends _{static properties={label:{type:String},value:{attribute:!1}};createRenderRoot(){return this}render(){if(this.value===null||this.value===void 0||this.value==="")return d;const e=typeof this.value=="string"?this.value:JSON.stringify(this.value,null,2);return l`
      <section class="payload-section">
        <h3>${this.label}</h3>
        <pre>${e}</pre>
      </section>
    `}}class je extends _{static properties={requests:{attribute:!1},selected_key:{type:String}};createRenderRoot(){return this}selectRequest(e){this.dispatchEvent(new CustomEvent("request-select",{detail:e,bubbles:!0,composed:!0}))}render(){const e=this.requests??[];return e.length===0?l`<p class="empty">No persisted requests match this view.</p>`:l`
      <div class="list" role="list">
        ${e.map(t=>l`
            <button
              class="list-row ${this.selected_key===de(t)?"selected":""}"
              @click=${()=>this.selectRequest(t)}
              role="listitem"
            >
              <span class="status ${t.status!==null&&t.status>=400?"error":""}">${le(t.status)}</span>
              <span class="list-row-main">
                <strong>${t.model??t.endpoint??"unknown request"}</strong>
                <small>${t.provider_id??"unknown provider"} · ${A(t.ts)}</small>
              </span>
              <span class="list-row-meta">${t.session_id??t.request_id}</span>
            </button>
          `)}
      </div>
    `}}class Ne extends _{static properties={sessions:{attribute:!1},selected_session_id:{type:String}};createRenderRoot(){return this}selectSession(e){this.dispatchEvent(new CustomEvent("session-select",{detail:e,bubbles:!0,composed:!0}))}render(){const e=this.sessions??[];return e.length===0?l`<p class="empty">No request records contain a session id yet.</p>`:l`
      <div class="list" role="list">
        ${e.map(t=>l`
            <button
              class="list-row ${this.selected_session_id===t.session_id?"selected":""}"
              @click=${()=>this.selectSession(t)}
              role="listitem"
            >
              <span class="session-count">${t.request_count}</span>
              <span class="list-row-main">
                <strong>${t.model??t.endpoint??"session"}</strong>
                <small>${t.provider_id??"unknown provider"} · ${A(t.last_ts)}</small>
              </span>
              <span class="list-row-meta">${t.session_id}</span>
            </button>
          `)}
      </div>
    `}}class He extends _{static properties={detail:{attribute:!1},selected_session_id:{type:String}};createRenderRoot(){return this}openSession(e){this.dispatchEvent(new CustomEvent("open-session",{detail:e,bubbles:!0,composed:!0}))}render(){if(!this.detail)return l`<section class="empty-detail"><p>Select a request to inspect its persisted metadata and bodies.</p></section>`;const e=this.detail.request,t=[["request_id",e.request_id],["day",this.detail.day],["timestamp",typeof e.ts=="number"?A(e.ts):e.ts],["endpoint",e.endpoint],["status",e.status],["provider",e.provider_id],["account",e.account_id],["model",e.model]],s=typeof e.session_id=="string"?e.session_id:void 0;return l`
      <section class="detail-header">
        <div>
          <p class="eyebrow">request</p>
          <h2>${String(e.model??e.endpoint??"request")}</h2>
          <p class="muted">${String(e.request_id??"unknown id")}</p>
        </div>
        ${s?l`<button class="link-button" @click=${()=>this.openSession(s)}>Open session</button>`:d}
      </section>
      <dl class="metadata-grid">
        ${t.map(([i,n])=>l`
            <div>
              <dt>${i}</dt>
              <dd>${n==null?"—":String(n)}</dd>
            </div>
          `)}
      </dl>
      ${e.request_error?l`<section class="error-message">${String(e.request_error)}</section>`:d}
      <json-viewer label="Inbound request headers" .value=${e.inbound_req_headers}></json-viewer>
      <json-viewer label="Inbound request" .value=${e.inbound_req_body}></json-viewer>
      <json-viewer label="Outbound request headers" .value=${e.outbound_req_headers}></json-viewer>
      <json-viewer label="Outbound request" .value=${e.outbound_req_body}></json-viewer>
      <json-viewer label="Outbound response headers" .value=${e.outbound_resp_headers}></json-viewer>
      <json-viewer label="Outbound response" .value=${e.outbound_resp_body}></json-viewer>
      <json-viewer label="Inbound response headers" .value=${e.inbound_resp_headers}></json-viewer>
      <json-viewer label="Inbound response" .value=${e.inbound_resp_body}></json-viewer>
      <json-viewer label="Request parameters" .value=${e.params_json}></json-viewer>
      <json-viewer label="Usage" .value=${e.usage_json}></json-viewer>
      <json-viewer label="Request context" .value=${e.ctx_json}></json-viewer>
    `}}class Te extends _{static properties={detail:{attribute:!1}};createRenderRoot(){return this}selectRequest(e){this.dispatchEvent(new CustomEvent("request-select",{detail:e,bubbles:!0,composed:!0}))}render(){if(!this.detail)return l`<section class="empty-detail"><p>Select a session to see its request timeline.</p></section>`;const{session:e,requests:t}=this.detail;return l`
      <section class="detail-header">
        <div>
          <p class="eyebrow">inferred session</p>
          <h2>${e.model??e.endpoint??"session"}</h2>
          <p class="muted">${e.session_id}</p>
        </div>
        <span class="session-count">${e.request_count}</span>
      </section>
      <dl class="metadata-grid">
        <div><dt>first seen</dt><dd>${A(e.first_ts)}</dd></div>
        <div><dt>last seen</dt><dd>${A(e.last_ts)}</dd></div>
        <div><dt>provider</dt><dd>${e.provider_id??"—"}</dd></div>
        <div><dt>account</dt><dd>${e.account_id??"—"}</dd></div>
      </dl>
      <section class="timeline">
        <h3>Request timeline</h3>
        ${t.map(s=>l`
            <button class="timeline-row" @click=${()=>this.selectRequest(s)}>
              <time>${A(s.ts)}</time>
              <span class="status ${s.status!==null&&s.status>=400?"error":""}">${le(s.status)}</span>
              <span>${s.model??s.endpoint??s.request_id}</span>
              <small>${s.request_id}</small>
            </button>
          `)}
      </section>
    `}}class De extends _{static properties={active_view:{type:String},info:{attribute:!1},requests:{attribute:!1},request_days:{attribute:!1},selected_day:{type:String},sessions:{attribute:!1},selected_request:{attribute:!1},selected_request_detail:{attribute:!1},selected_session:{attribute:!1},selected_session_detail:{attribute:!1},search_query:{type:String},loading:{type:Boolean},request_days_loading:{type:Boolean},request_days_error:{type:String},sessions_loading:{type:Boolean},sessions_error:{type:String},error_message:{type:String}};request_load_id=0;request_detail_load_id=0;session_detail_load_id=0;request_days_load_id=0;sessions_loaded=!1;constructor(){super(),this.active_view="requests",this.requests=[],this.request_days=[],this.sessions=[],this.search_query="",this.loading=!0,this.request_days_loading=!1,this.sessions_loading=!1}createRenderRoot(){return this}connectedCallback(){super.connectedCallback(),this.loadInitialData()}async loadInitialData(){const e=++this.request_load_id;this.loading=!0,this.error_message=void 0;const[t,s]=await Promise.allSettled([v("/api/info"),v("/api/requests/latest?limit=100")]);t.status==="fulfilled"&&(this.info=t.value),s.status==="fulfilled"&&e===this.request_load_id&&(this.selected_day=s.value.day??void 0,this.requests=s.value.requests,this.clearRequestSelection());const i=t.status==="rejected"?t.reason:s.status==="rejected"?s.reason:void 0;i&&(this.error_message=i instanceof Error?i.message:"Unable to load persisted history"),e===this.request_load_id&&(this.loading=!1),this.loadRequestDays()}async loadRequestDays(){const e=++this.request_days_load_id;this.request_days_loading=!0,this.request_days_error=void 0;try{const t=await v("/api/request-days");e===this.request_days_load_id&&(this.request_days=t)}catch(t){e===this.request_days_load_id&&(this.request_days_error=t instanceof Error?t.message:"Unable to load request day states")}finally{e===this.request_days_load_id&&(this.request_days_loading=!1)}}markRequestDayUnavailable(e){if(this.request_days.find(s=>s.day===e)){this.request_days=this.request_days.map(s=>s.day===e?{...s,state:"unavailable"}:s);return}this.request_days=[{day:e,state:"unavailable"},...this.request_days]}clearRequestSelection(){this.request_detail_load_id+=1,this.selected_request=void 0,this.selected_request_detail=void 0}async loadRequests(){const e=this.selected_day;if(!e){this.requests=[],this.clearRequestSelection();return}const t=++this.request_load_id;this.loading=!0,this.error_message=void 0,this.clearRequestSelection(),this.requests=[];try{const s=new URLSearchParams({day:e,limit:"100"}),i=this.search_query.trim();i&&s.set("query",i);const n=await v(`/api/requests?${s.toString()}`);if(t!==this.request_load_id)return;this.requests=n}catch(s){t===this.request_load_id&&(s instanceof ae&&s.status===503&&this.selected_day===e&&this.markRequestDayUnavailable(e),this.error_message=s instanceof Error?s.message:"Unable to load requests")}finally{t===this.request_load_id&&(this.loading=!1)}}selectDay(e){this.selected_day=e,this.loadRequests()}async selectRequest(e){const t=++this.request_detail_load_id;this.selected_request=e,this.selected_request_detail=void 0,this.error_message=void 0;try{const s=await v(`/api/request?day=${encodeURIComponent(e.day)}&request_id=${encodeURIComponent(e.request_id)}`);t===this.request_detail_load_id&&(this.selected_request_detail=s)}catch(s){t===this.request_detail_load_id&&(this.error_message=s instanceof Error?s.message:"Unable to load request details")}}async ensureSessionsLoaded(){if(!(this.sessions_loaded||this.sessions_loading)){this.sessions_loading=!0,this.sessions_error=void 0;try{this.sessions=await v("/api/sessions?limit=100"),this.sessions_loaded=!0}catch(e){this.sessions_error=e instanceof Error?e.message:"Unable to load sessions"}finally{this.sessions_loading=!1}}}async loadSession(e,t){const s=++this.session_detail_load_id;this.selected_session=t,this.selected_session_detail=void 0,this.error_message=void 0;try{const i=await v(`/api/session?session_id=${encodeURIComponent(e)}&limit=500`);s===this.session_detail_load_id&&(this.selected_session=i.session,this.selected_session_detail=i)}catch(i){s===this.session_detail_load_id&&(this.error_message=i instanceof Error?i.message:"Unable to load session timeline")}}async selectSession(e){await this.loadSession(e.session_id,e)}async openSession(e){this.setActiveView("sessions",!1);const t=this.sessions.find(s=>s.session_id===e);await this.loadSession(e,t)}async openRequest(e){this.setActiveView("requests");const t=this.selected_day!==e.day,s=!!this.search_query.trim();(t||s)&&(this.selected_day=e.day,this.search_query="",await this.loadRequests()),await this.selectRequest(e)}setActiveView(e,t=!0){this.active_view=e,e==="sessions"&&t&&this.ensureSessionsLoaded()}submitSearch(e){e.preventDefault(),this.loadRequests()}updateSearch(e){this.search_query=e.target.value}pickerDays(){return!this.selected_day||this.request_days.some(e=>e.day===this.selected_day)?this.request_days:[{day:this.selected_day,state:"available"},...this.request_days]}renderDayPicker(){const e=this.pickerDays();return l`
      <div class="day-picker-group">
        <div class="day-picker-heading">
          <span class="day-picker-label">Request day (UTC)</span>
          <button
            class="day-refresh"
            ?disabled=${this.request_days_loading}
            title="Refresh request day availability"
            @click=${()=>{this.loadRequestDays()}}
          >
            Refresh
          </button>
        </div>
        <div class="day-picker" role="group" aria-label="Request day">
          ${e.length>0?e.map(t=>{const s=t.state==="available",i=t.day===this.selected_day,n=t.state==="empty"?"Empty":t.state==="unavailable"?"Unavailable":void 0,o=t.state==="empty"?"No persisted requests for this day":t.state==="unavailable"?"This request day could not be read":`Show requests from ${t.day}`;return l`
                  <button
                    class="day-button ${t.state} ${i?"selected":""}"
                    ?disabled=${!s}
                    aria-pressed=${String(i)}
                    title=${o}
                    @click=${()=>this.selectDay(t.day)}
                  >
                    <span>${t.day}</span>${n?l`<small>${n}</small>`:d}
                  </button>
                `}):l`<span class="day-picker-empty">${this.request_days_loading?"Checking request days…":"No persisted request days."}</span>`}
          ${this.request_days_loading&&e.length>0?l`<span class="day-picker-status">Checking days…</span>`:d}
        </div>
        ${this.request_days_error?l`<p class="day-picker-error">${this.request_days_error}</p>`:d}
      </div>
    `}renderSessionsSidebar(){return this.sessions_loading?l`<p class="empty">Loading sessions…</p>`:this.sessions_error?l`
        <section class="sidebar-message">
          <p class="sidebar-warning">${this.sessions_error}</p>
          <button class="link-button" @click=${()=>{this.ensureSessionsLoaded()}}>Retry loading sessions</button>
        </section>
      `:this.sessions_loaded?l`<session-list
      .sessions=${this.sessions}
      .selected_session_id=${this.selected_session?.session_id}
      @session-select=${e=>{this.selectSession(O(e))}}
    ></session-list>`:l`
        <section class="sidebar-message">
          <p class="empty">The session list has not been loaded.</p>
          <button class="link-button" @click=${()=>{this.ensureSessionsLoaded()}}>Load session list</button>
        </section>
      `}render(){const e=this.selected_request?de(this.selected_request):void 0,t=!!this.selected_day;return l`
      <header class="app-header">
        <div>
          <p class="eyebrow">local, read-only viewer</p>
          <h1>tokn inspect</h1>
        </div>
        <p class="sensitive-notice">History may contain sensitive prompts and responses.</p>
      </header>
      <main class="app-shell">
        <nav class="tabs" aria-label="Inspector views">
          <button class=${this.active_view==="requests"?"active":""} @click=${()=>this.setActiveView("requests")}>Requests</button>
          <button class=${this.active_view==="sessions"?"active":""} @click=${()=>this.setActiveView("sessions")}>Sessions</button>
        </nav>
        <section class="toolbar">
          <div class="toolbar-controls">
            ${this.active_view==="requests"?l`
                  ${this.renderDayPicker()}
                  <form class="request-search" @submit=${this.submitSearch}>
                    <input
                      aria-label="Search requests"
                      .value=${this.search_query}
                      @input=${this.updateSearch}
                      ?disabled=${!t}
                      placeholder=${t?"Search request, session, or model":"Choose an available request day"}
                    />
                    <button type="submit" ?disabled=${!t}>Filter</button>
                  </form>
                `:l`<p class="muted">Sessions are inferred from persisted request session ids.</p>`}
          </div>
          <span class="data-path" title=${this.info?.requests_dir??""}>${this.info?this.info.requests_dir:"Loading request history…"}</span>
        </section>
        ${this.error_message?l`<section class="error-banner">${this.error_message}</section>`:d}
        <section class="viewer-grid ${this.loading?"loading":""}" aria-busy=${String(this.loading)}>
          <aside class="sidebar">
            ${this.active_view==="requests"?this.loading?l`<p class="empty">Loading requests…</p>`:l`<request-list
                    .requests=${this.requests}
                    .selected_key=${e}
                    @request-select=${s=>{this.selectRequest(O(s))}}
                  ></request-list>`:this.renderSessionsSidebar()}
          </aside>
          <article class="detail-pane">
            ${this.active_view==="requests"?l`<request-detail-view
                  .detail=${this.selected_request_detail}
                  @open-session=${s=>{this.openSession(O(s))}}
                ></request-detail-view>`:l`<session-timeline
                  .detail=${this.selected_session_detail}
                  @request-select=${s=>{this.openRequest(O(s))}}
                ></session-timeline>`}
          </article>
        </section>
      </main>
    `}}customElements.define("json-viewer",Oe);customElements.define("request-list",je);customElements.define("session-list",Ne);customElements.define("request-detail-view",He);customElements.define("session-timeline",Te);customElements.define("inspect-app",De);
