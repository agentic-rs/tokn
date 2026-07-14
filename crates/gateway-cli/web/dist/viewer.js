(function(){const e=document.createElement("link").relList;if(e&&e.supports&&e.supports("modulepreload"))return;for(const i of document.querySelectorAll('link[rel="modulepreload"]'))s(i);new MutationObserver(i=>{for(const n of i)if(n.type==="childList")for(const o of n.addedNodes)o.tagName==="LINK"&&o.rel==="modulepreload"&&s(o)}).observe(document,{childList:!0,subtree:!0});function t(i){const n={};return i.integrity&&(n.integrity=i.integrity),i.referrerPolicy&&(n.referrerPolicy=i.referrerPolicy),i.crossOrigin==="use-credentials"?n.credentials="include":i.crossOrigin==="anonymous"?n.credentials="omit":n.credentials="same-origin",n}function s(i){if(i.ep)return;i.ep=!0;const n=t(i);fetch(i.href,n)}})();const N=globalThis,D=N.ShadowRoot&&(N.ShadyCSS===void 0||N.ShadyCSS.nativeShadow)&&"adoptedStyleSheets"in Document.prototype&&"replace"in CSSStyleSheet.prototype,te=Symbol(),V=new WeakMap;let de=class{constructor(e,t,s){if(this._$cssResult$=!0,s!==te)throw Error("CSSResult is not constructable. Use `unsafeCSS` or `css` instead.");this.cssText=e,this.t=t}get styleSheet(){let e=this.o;const t=this.t;if(D&&e===void 0){const s=t!==void 0&&t.length===1;s&&(e=V.get(t)),e===void 0&&((this.o=e=new CSSStyleSheet).replaceSync(this.cssText),s&&V.set(t,e))}return e}toString(){return this.cssText}};const ce=r=>new de(typeof r=="string"?r:r+"",void 0,te),he=(r,e)=>{if(D)r.adoptedStyleSheets=e.map(t=>t instanceof CSSStyleSheet?t:t.styleSheet);else for(const t of e){const s=document.createElement("style"),i=N.litNonce;i!==void 0&&s.setAttribute("nonce",i),s.textContent=t.cssText,r.appendChild(s)}},W=D?r=>r:r=>r instanceof CSSStyleSheet?(e=>{let t="";for(const s of e.cssRules)t+=s.cssText;return ce(t)})(r):r;const{is:ue,defineProperty:pe,getOwnPropertyDescriptor:$e,getOwnPropertyNames:_e,getOwnPropertySymbols:me,getPrototypeOf:fe}=Object,T=globalThis,J=T.trustedTypes,ve=J?J.emptyScript:"",ye=T.reactiveElementPolyfillSupport,C=(r,e)=>r,I={toAttribute(r,e){switch(e){case Boolean:r=r?ve:null;break;case Object:case Array:r=r==null?r:JSON.stringify(r)}return r},fromAttribute(r,e){let t=r;switch(e){case Boolean:t=r!==null;break;case Number:t=r===null?null:Number(r);break;case Object:case Array:try{t=JSON.parse(r)}catch{t=null}}return t}},se=(r,e)=>!ue(r,e),F={attribute:!0,type:String,converter:I,reflect:!1,useDefault:!1,hasChanged:se};Symbol.metadata??=Symbol("metadata"),T.litPropertyMetadata??=new WeakMap;let A=class extends HTMLElement{static addInitializer(e){this._$Ei(),(this.l??=[]).push(e)}static get observedAttributes(){return this.finalize(),this._$Eh&&[...this._$Eh.keys()]}static createProperty(e,t=F){if(t.state&&(t.attribute=!1),this._$Ei(),this.prototype.hasOwnProperty(e)&&((t=Object.create(t)).wrapped=!0),this.elementProperties.set(e,t),!t.noAccessor){const s=Symbol(),i=this.getPropertyDescriptor(e,s,t);i!==void 0&&pe(this.prototype,e,i)}}static getPropertyDescriptor(e,t,s){const{get:i,set:n}=$e(this.prototype,e)??{get(){return this[t]},set(o){this[t]=o}};return{get:i,set(o){const h=i?.call(this);n?.call(this,o),this.requestUpdate(e,h,s)},configurable:!0,enumerable:!0}}static getPropertyOptions(e){return this.elementProperties.get(e)??F}static _$Ei(){if(this.hasOwnProperty(C("elementProperties")))return;const e=fe(this);e.finalize(),e.l!==void 0&&(this.l=[...e.l]),this.elementProperties=new Map(e.elementProperties)}static finalize(){if(this.hasOwnProperty(C("finalized")))return;if(this.finalized=!0,this._$Ei(),this.hasOwnProperty(C("properties"))){const t=this.properties,s=[..._e(t),...me(t)];for(const i of s)this.createProperty(i,t[i])}const e=this[Symbol.metadata];if(e!==null){const t=litPropertyMetadata.get(e);if(t!==void 0)for(const[s,i]of t)this.elementProperties.set(s,i)}this._$Eh=new Map;for(const[t,s]of this.elementProperties){const i=this._$Eu(t,s);i!==void 0&&this._$Eh.set(i,t)}this.elementStyles=this.finalizeStyles(this.styles)}static finalizeStyles(e){const t=[];if(Array.isArray(e)){const s=new Set(e.flat(1/0).reverse());for(const i of s)t.unshift(W(i))}else e!==void 0&&t.push(W(e));return t}static _$Eu(e,t){const s=t.attribute;return s===!1?void 0:typeof s=="string"?s:typeof e=="string"?e.toLowerCase():void 0}constructor(){super(),this._$Ep=void 0,this.isUpdatePending=!1,this.hasUpdated=!1,this._$Em=null,this._$Ev()}_$Ev(){this._$ES=new Promise(e=>this.enableUpdating=e),this._$AL=new Map,this._$E_(),this.requestUpdate(),this.constructor.l?.forEach(e=>e(this))}addController(e){(this._$EO??=new Set).add(e),this.renderRoot!==void 0&&this.isConnected&&e.hostConnected?.()}removeController(e){this._$EO?.delete(e)}_$E_(){const e=new Map,t=this.constructor.elementProperties;for(const s of t.keys())this.hasOwnProperty(s)&&(e.set(s,this[s]),delete this[s]);e.size>0&&(this._$Ep=e)}createRenderRoot(){const e=this.shadowRoot??this.attachShadow(this.constructor.shadowRootOptions);return he(e,this.constructor.elementStyles),e}connectedCallback(){this.renderRoot??=this.createRenderRoot(),this.enableUpdating(!0),this._$EO?.forEach(e=>e.hostConnected?.())}enableUpdating(e){}disconnectedCallback(){this._$EO?.forEach(e=>e.hostDisconnected?.())}attributeChangedCallback(e,t,s){this._$AK(e,s)}_$ET(e,t){const s=this.constructor.elementProperties.get(e),i=this.constructor._$Eu(e,s);if(i!==void 0&&s.reflect===!0){const n=(s.converter?.toAttribute!==void 0?s.converter:I).toAttribute(t,s.type);this._$Em=e,n==null?this.removeAttribute(i):this.setAttribute(i,n),this._$Em=null}}_$AK(e,t){const s=this.constructor,i=s._$Eh.get(e);if(i!==void 0&&this._$Em!==i){const n=s.getPropertyOptions(i),o=typeof n.converter=="function"?{fromAttribute:n.converter}:n.converter?.fromAttribute!==void 0?n.converter:I;this._$Em=i;const h=o.fromAttribute(t,n.type);this[i]=h??this._$Ej?.get(i)??h,this._$Em=null}}requestUpdate(e,t,s,i=!1,n){if(e!==void 0){const o=this.constructor;if(i===!1&&(n=this[e]),s??=o.getPropertyOptions(e),!((s.hasChanged??se)(n,t)||s.useDefault&&s.reflect&&n===this._$Ej?.get(e)&&!this.hasAttribute(o._$Eu(e,s))))return;this.C(e,t,s)}this.isUpdatePending===!1&&(this._$ES=this._$EP())}C(e,t,{useDefault:s,reflect:i,wrapped:n},o){s&&!(this._$Ej??=new Map).has(e)&&(this._$Ej.set(e,o??t??this[e]),n!==!0||o!==void 0)||(this._$AL.has(e)||(this.hasUpdated||s||(t=void 0),this._$AL.set(e,t)),i===!0&&this._$Em!==e&&(this._$Eq??=new Set).add(e))}async _$EP(){this.isUpdatePending=!0;try{await this._$ES}catch(t){Promise.reject(t)}const e=this.scheduleUpdate();return e!=null&&await e,!this.isUpdatePending}scheduleUpdate(){return this.performUpdate()}performUpdate(){if(!this.isUpdatePending)return;if(!this.hasUpdated){if(this.renderRoot??=this.createRenderRoot(),this._$Ep){for(const[i,n]of this._$Ep)this[i]=n;this._$Ep=void 0}const s=this.constructor.elementProperties;if(s.size>0)for(const[i,n]of s){const{wrapped:o}=n,h=this[i];o!==!0||this._$AL.has(i)||h===void 0||this.C(i,void 0,n,h)}}let e=!1;const t=this._$AL;try{e=this.shouldUpdate(t),e?(this.willUpdate(t),this._$EO?.forEach(s=>s.hostUpdate?.()),this.update(t)):this._$EM()}catch(s){throw e=!1,this._$EM(),s}e&&this._$AE(t)}willUpdate(e){}_$AE(e){this._$EO?.forEach(t=>t.hostUpdated?.()),this.hasUpdated||(this.hasUpdated=!0,this.firstUpdated(e)),this.updated(e)}_$EM(){this._$AL=new Map,this.isUpdatePending=!1}get updateComplete(){return this.getUpdateComplete()}getUpdateComplete(){return this._$ES}shouldUpdate(e){return!0}update(e){this._$Eq&&=this._$Eq.forEach(t=>this._$ET(t,this[t])),this._$EM()}updated(e){}firstUpdated(e){}};A.elementStyles=[],A.shadowRootOptions={mode:"open"},A[C("elementProperties")]=new Map,A[C("finalized")]=new Map,ye?.({ReactiveElement:A}),(T.reactiveElementVersions??=[]).push("2.1.2");const L=globalThis,K=r=>r,H=L.trustedTypes,Z=H?H.createPolicy("lit-html",{createHTML:r=>r}):void 0,ie="$lit$",f=`lit$${Math.random().toFixed(9).slice(2)}$`,re="?"+f,ge=`<${re}>`,g=document,R=()=>g.createComment(""),P=r=>r===null||typeof r!="object"&&typeof r!="function",z=Array.isArray,be=r=>z(r)||typeof r?.[Symbol.iterator]=="function",k=`[ 	
\f\r]`,q=/<(?:(!--|\/[^a-zA-Z])|(\/?[a-zA-Z][^>\s]*)|(\/?$))/g,G=/-->/g,Q=/>/g,v=RegExp(`>|${k}(?:([^\\s"'>=/]+)(${k}*=${k}*(?:[^ 	
\f\r"'\`<>=]|("|')|))|$)`,"g"),X=/'/g,Y=/"/g,ne=/^(?:script|style|textarea|title)$/i,Ae=r=>(e,...t)=>({_$litType$:r,strings:e,values:t}),d=Ae(1),S=Symbol.for("lit-noChange"),c=Symbol.for("lit-nothing"),ee=new WeakMap,y=g.createTreeWalker(g,129);function oe(r,e){if(!z(r)||!r.hasOwnProperty("raw"))throw Error("invalid template strings array");return Z!==void 0?Z.createHTML(e):e}const we=(r,e)=>{const t=r.length-1,s=[];let i,n=e===2?"<svg>":e===3?"<math>":"",o=q;for(let h=0;h<t;h++){const a=r[h];let u,p,l=-1,_=0;for(;_<a.length&&(o.lastIndex=_,p=o.exec(a),p!==null);)_=o.lastIndex,o===q?p[1]==="!--"?o=G:p[1]!==void 0?o=Q:p[2]!==void 0?(ne.test(p[2])&&(i=RegExp("</"+p[2],"g")),o=v):p[3]!==void 0&&(o=v):o===v?p[0]===">"?(o=i??q,l=-1):p[1]===void 0?l=-2:(l=o.lastIndex-p[2].length,u=p[1],o=p[3]===void 0?v:p[3]==='"'?Y:X):o===Y||o===X?o=v:o===G||o===Q?o=q:(o=v,i=void 0);const m=o===v&&r[h+1].startsWith("/>")?" ":"";n+=o===q?a+ge:l>=0?(s.push(u),a.slice(0,l)+ie+a.slice(l)+f+m):a+f+(l===-2?h:m)}return[oe(r,n+(r[t]||"<?>")+(e===2?"</svg>":e===3?"</math>":"")),s]};class x{constructor({strings:e,_$litType$:t},s){let i;this.parts=[];let n=0,o=0;const h=e.length-1,a=this.parts,[u,p]=we(e,t);if(this.el=x.createElement(u,s),y.currentNode=this.el.content,t===2||t===3){const l=this.el.content.firstChild;l.replaceWith(...l.childNodes)}for(;(i=y.nextNode())!==null&&a.length<h;){if(i.nodeType===1){if(i.hasAttributes())for(const l of i.getAttributeNames())if(l.endsWith(ie)){const _=p[o++],m=i.getAttribute(l).split(f),O=/([.?@])?(.*)/.exec(_);a.push({type:1,index:n,name:O[2],strings:m,ctor:O[1]==="."?Ee:O[1]==="?"?qe:O[1]==="@"?Ce:M}),i.removeAttribute(l)}else l.startsWith(f)&&(a.push({type:6,index:n}),i.removeAttribute(l));if(ne.test(i.tagName)){const l=i.textContent.split(f),_=l.length-1;if(_>0){i.textContent=H?H.emptyScript:"";for(let m=0;m<_;m++)i.append(l[m],R()),y.nextNode(),a.push({type:2,index:++n});i.append(l[_],R())}}}else if(i.nodeType===8)if(i.data===re)a.push({type:2,index:n});else{let l=-1;for(;(l=i.data.indexOf(f,l+1))!==-1;)a.push({type:7,index:n}),l+=f.length-1}n++}}static createElement(e,t){const s=g.createElement("template");return s.innerHTML=e,s}}function E(r,e,t=r,s){if(e===S)return e;let i=s!==void 0?t._$Co?.[s]:t._$Cl;const n=P(e)?void 0:e._$litDirective$;return i?.constructor!==n&&(i?._$AO?.(!1),n===void 0?i=void 0:(i=new n(r),i._$AT(r,t,s)),s!==void 0?(t._$Co??=[])[s]=i:t._$Cl=i),i!==void 0&&(e=E(r,i._$AS(r,e.values),i,s)),e}class Se{constructor(e,t){this._$AV=[],this._$AN=void 0,this._$AD=e,this._$AM=t}get parentNode(){return this._$AM.parentNode}get _$AU(){return this._$AM._$AU}u(e){const{el:{content:t},parts:s}=this._$AD,i=(e?.creationScope??g).importNode(t,!0);y.currentNode=i;let n=y.nextNode(),o=0,h=0,a=s[0];for(;a!==void 0;){if(o===a.index){let u;a.type===2?u=new U(n,n.nextSibling,this,e):a.type===1?u=new a.ctor(n,a.name,a.strings,this,e):a.type===6&&(u=new Re(n,this,e)),this._$AV.push(u),a=s[++h]}o!==a?.index&&(n=y.nextNode(),o++)}return y.currentNode=g,i}p(e){let t=0;for(const s of this._$AV)s!==void 0&&(s.strings!==void 0?(s._$AI(e,s,t),t+=s.strings.length-2):s._$AI(e[t])),t++}}class U{get _$AU(){return this._$AM?._$AU??this._$Cv}constructor(e,t,s,i){this.type=2,this._$AH=c,this._$AN=void 0,this._$AA=e,this._$AB=t,this._$AM=s,this.options=i,this._$Cv=i?.isConnected??!0}get parentNode(){let e=this._$AA.parentNode;const t=this._$AM;return t!==void 0&&e?.nodeType===11&&(e=t.parentNode),e}get startNode(){return this._$AA}get endNode(){return this._$AB}_$AI(e,t=this){e=E(this,e,t),P(e)?e===c||e==null||e===""?(this._$AH!==c&&this._$AR(),this._$AH=c):e!==this._$AH&&e!==S&&this._(e):e._$litType$!==void 0?this.$(e):e.nodeType!==void 0?this.T(e):be(e)?this.k(e):this._(e)}O(e){return this._$AA.parentNode.insertBefore(e,this._$AB)}T(e){this._$AH!==e&&(this._$AR(),this._$AH=this.O(e))}_(e){this._$AH!==c&&P(this._$AH)?this._$AA.nextSibling.data=e:this.T(g.createTextNode(e)),this._$AH=e}$(e){const{values:t,_$litType$:s}=e,i=typeof s=="number"?this._$AC(e):(s.el===void 0&&(s.el=x.createElement(oe(s.h,s.h[0]),this.options)),s);if(this._$AH?._$AD===i)this._$AH.p(t);else{const n=new Se(i,this),o=n.u(this.options);n.p(t),this.T(o),this._$AH=n}}_$AC(e){let t=ee.get(e.strings);return t===void 0&&ee.set(e.strings,t=new x(e)),t}k(e){z(this._$AH)||(this._$AH=[],this._$AR());const t=this._$AH;let s,i=0;for(const n of e)i===t.length?t.push(s=new U(this.O(R()),this.O(R()),this,this.options)):s=t[i],s._$AI(n),i++;i<t.length&&(this._$AR(s&&s._$AB.nextSibling,i),t.length=i)}_$AR(e=this._$AA.nextSibling,t){for(this._$AP?.(!1,!0,t);e!==this._$AB;){const s=K(e).nextSibling;K(e).remove(),e=s}}setConnected(e){this._$AM===void 0&&(this._$Cv=e,this._$AP?.(e))}}class M{get tagName(){return this.element.tagName}get _$AU(){return this._$AM._$AU}constructor(e,t,s,i,n){this.type=1,this._$AH=c,this._$AN=void 0,this.element=e,this.name=t,this._$AM=i,this.options=n,s.length>2||s[0]!==""||s[1]!==""?(this._$AH=Array(s.length-1).fill(new String),this.strings=s):this._$AH=c}_$AI(e,t=this,s,i){const n=this.strings;let o=!1;if(n===void 0)e=E(this,e,t,0),o=!P(e)||e!==this._$AH&&e!==S,o&&(this._$AH=e);else{const h=e;let a,u;for(e=n[0],a=0;a<n.length-1;a++)u=E(this,h[s+a],t,a),u===S&&(u=this._$AH[a]),o||=!P(u)||u!==this._$AH[a],u===c?e=c:e!==c&&(e+=(u??"")+n[a+1]),this._$AH[a]=u}o&&!i&&this.j(e)}j(e){e===c?this.element.removeAttribute(this.name):this.element.setAttribute(this.name,e??"")}}class Ee extends M{constructor(){super(...arguments),this.type=3}j(e){this.element[this.name]=e===c?void 0:e}}class qe extends M{constructor(){super(...arguments),this.type=4}j(e){this.element.toggleAttribute(this.name,!!e&&e!==c)}}class Ce extends M{constructor(e,t,s,i,n){super(e,t,s,i,n),this.type=5}_$AI(e,t=this){if((e=E(this,e,t,0)??c)===S)return;const s=this._$AH,i=e===c&&s!==c||e.capture!==s.capture||e.once!==s.once||e.passive!==s.passive,n=e!==c&&(s===c||i);i&&this.element.removeEventListener(this.name,this,s),n&&this.element.addEventListener(this.name,this,e),this._$AH=e}handleEvent(e){typeof this._$AH=="function"?this._$AH.call(this.options?.host??this.element,e):this._$AH.handleEvent(e)}}class Re{constructor(e,t,s){this.element=e,this.type=6,this._$AN=void 0,this._$AM=t,this.options=s}get _$AU(){return this._$AM._$AU}_$AI(e){E(this,e)}}const Pe=L.litHtmlPolyfillSupport;Pe?.(x,U),(L.litHtmlVersions??=[]).push("3.3.3");const xe=(r,e,t)=>{const s=t?.renderBefore??e;let i=s._$litPart$;if(i===void 0){const n=t?.renderBefore??null;s._$litPart$=i=new U(e.insertBefore(R(),n),n,void 0,t??{})}return i._$AI(r),i};const B=globalThis;class $ extends A{constructor(){super(...arguments),this.renderOptions={host:this},this._$Do=void 0}createRenderRoot(){const e=super.createRenderRoot();return this.renderOptions.renderBefore??=e.firstChild,e}update(e){const t=this.render();this.hasUpdated||(this.renderOptions.isConnected=this.isConnected),super.update(e),this._$Do=xe(t,this.renderRoot,this.renderOptions)}connectedCallback(){super.connectedCallback(),this._$Do?.setConnected(!0)}disconnectedCallback(){super.disconnectedCallback(),this._$Do?.setConnected(!1)}render(){return S}}$._$litElement$=!0,$.finalized=!0,B.litElementHydrateSupport?.({LitElement:$});const Ue=B.litElementPolyfillSupport;Ue?.({LitElement:$});(B.litElementVersions??=[]).push("4.2.2");async function b(r){const e=await fetch(r,{cache:"no-store"});if(!e.ok){const t=await e.json().catch(()=>({}));throw new Error(t.error??`Request failed (${e.status})`)}return e.json()}function w(r){return new Intl.DateTimeFormat(void 0,{dateStyle:"medium",timeStyle:"medium"}).format(new Date(r))}function ae(r){return r===null?"—":String(r)}function le(r){return`${r.day}:${r.request_id}`}function j(r){return r.detail}class Oe extends ${static properties={label:{type:String},value:{attribute:!1}};createRenderRoot(){return this}render(){if(this.value===null||this.value===void 0||this.value==="")return c;const e=typeof this.value=="string"?this.value:JSON.stringify(this.value,null,2);return d`
      <section class="payload-section">
        <h3>${this.label}</h3>
        <pre>${e}</pre>
      </section>
    `}}class je extends ${static properties={requests:{attribute:!1},selected_key:{type:String}};requests=[];createRenderRoot(){return this}selectRequest(e){this.dispatchEvent(new CustomEvent("request-select",{detail:e,bubbles:!0,composed:!0}))}render(){return this.requests.length===0?d`<p class="empty">No persisted requests match this view.</p>`:d`
      <div class="list" role="list">
        ${this.requests.map(e=>d`
            <button
              class="list-row ${this.selected_key===le(e)?"selected":""}"
              @click=${()=>this.selectRequest(e)}
              role="listitem"
            >
              <span class="status ${e.status!==null&&e.status>=400?"error":""}">${ae(e.status)}</span>
              <span class="list-row-main">
                <strong>${e.model??e.endpoint??"unknown request"}</strong>
                <small>${e.provider_id??"unknown provider"} · ${w(e.ts)}</small>
              </span>
              <span class="list-row-meta">${e.session_id??e.request_id}</span>
            </button>
          `)}
      </div>
    `}}class Ne extends ${static properties={sessions:{attribute:!1},selected_session_id:{type:String}};sessions=[];createRenderRoot(){return this}selectSession(e){this.dispatchEvent(new CustomEvent("session-select",{detail:e,bubbles:!0,composed:!0}))}render(){return this.sessions.length===0?d`<p class="empty">No request records contain a session id yet.</p>`:d`
      <div class="list" role="list">
        ${this.sessions.map(e=>d`
            <button
              class="list-row ${this.selected_session_id===e.session_id?"selected":""}"
              @click=${()=>this.selectSession(e)}
              role="listitem"
            >
              <span class="session-count">${e.request_count}</span>
              <span class="list-row-main">
                <strong>${e.model??e.endpoint??"session"}</strong>
                <small>${e.provider_id??"unknown provider"} · ${w(e.last_ts)}</small>
              </span>
              <span class="list-row-meta">${e.session_id}</span>
            </button>
          `)}
      </div>
    `}}class He extends ${static properties={detail:{attribute:!1},selected_session_id:{type:String}};createRenderRoot(){return this}openSession(e){this.dispatchEvent(new CustomEvent("open-session",{detail:e,bubbles:!0,composed:!0}))}render(){if(!this.detail)return d`<section class="empty-detail"><p>Select a request to inspect its persisted metadata and bodies.</p></section>`;const e=this.detail.request,t=[["request_id",e.request_id],["day",this.detail.day],["timestamp",typeof e.ts=="number"?w(e.ts):e.ts],["endpoint",e.endpoint],["status",e.status],["provider",e.provider_id],["account",e.account_id],["model",e.model]],s=typeof e.session_id=="string"?e.session_id:void 0;return d`
      <section class="detail-header">
        <div>
          <p class="eyebrow">request</p>
          <h2>${String(e.model??e.endpoint??"request")}</h2>
          <p class="muted">${String(e.request_id??"unknown id")}</p>
        </div>
        ${s?d`<button class="link-button" @click=${()=>this.openSession(s)}>Open session</button>`:c}
      </section>
      <dl class="metadata-grid">
        ${t.map(([i,n])=>d`
            <div>
              <dt>${i}</dt>
              <dd>${n==null?"—":String(n)}</dd>
            </div>
          `)}
      </dl>
      ${e.request_error?d`<section class="error-message">${String(e.request_error)}</section>`:c}
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
    `}}class Te extends ${static properties={detail:{attribute:!1}};createRenderRoot(){return this}selectRequest(e){this.dispatchEvent(new CustomEvent("request-select",{detail:e,bubbles:!0,composed:!0}))}render(){if(!this.detail)return d`<section class="empty-detail"><p>Select a session to see its request timeline.</p></section>`;const{session:e,requests:t}=this.detail;return d`
      <section class="detail-header">
        <div>
          <p class="eyebrow">inferred session</p>
          <h2>${e.model??e.endpoint??"session"}</h2>
          <p class="muted">${e.session_id}</p>
        </div>
        <span class="session-count">${e.request_count}</span>
      </section>
      <dl class="metadata-grid">
        <div><dt>first seen</dt><dd>${w(e.first_ts)}</dd></div>
        <div><dt>last seen</dt><dd>${w(e.last_ts)}</dd></div>
        <div><dt>provider</dt><dd>${e.provider_id??"—"}</dd></div>
        <div><dt>account</dt><dd>${e.account_id??"—"}</dd></div>
      </dl>
      <section class="timeline">
        <h3>Request timeline</h3>
        ${t.map(s=>d`
            <button class="timeline-row" @click=${()=>this.selectRequest(s)}>
              <time>${w(s.ts)}</time>
              <span class="status ${s.status!==null&&s.status>=400?"error":""}">${ae(s.status)}</span>
              <span>${s.model??s.endpoint??s.request_id}</span>
              <small>${s.request_id}</small>
            </button>
          `)}
      </section>
    `}}class Me extends ${static properties={active_view:{type:String},info:{attribute:!1},requests:{attribute:!1},sessions:{attribute:!1},selected_request:{attribute:!1},selected_request_detail:{attribute:!1},selected_session:{attribute:!1},selected_session_detail:{attribute:!1},search_query:{type:String},loading:{type:Boolean},error_message:{type:String}};constructor(){super(),this.active_view="requests",this.requests=[],this.sessions=[],this.search_query="",this.loading=!0}createRenderRoot(){return this}connectedCallback(){super.connectedCallback(),this.loadInitialData()}async loadInitialData(){this.loading=!0,this.error_message=void 0;try{const[e,t,s]=await Promise.all([b("/api/info"),b("/api/requests?limit=100"),b("/api/sessions?limit=100")]);this.info=e,this.requests=t,this.sessions=s}catch(e){this.error_message=e instanceof Error?e.message:"Unable to load persisted history"}finally{this.loading=!1}}async loadRequests(){this.loading=!0,this.error_message=void 0;try{const e=this.search_query.trim(),t=e?`&query=${encodeURIComponent(e)}`:"";this.requests=await b(`/api/requests?limit=100${t}`),this.selected_request=void 0,this.selected_request_detail=void 0}catch(e){this.error_message=e instanceof Error?e.message:"Unable to load requests"}finally{this.loading=!1}}async selectRequest(e){this.selected_request=e,this.selected_request_detail=void 0,this.error_message=void 0;try{this.selected_request_detail=await b(`/api/request?day=${encodeURIComponent(e.day)}&request_id=${encodeURIComponent(e.request_id)}`)}catch(t){this.error_message=t instanceof Error?t.message:"Unable to load request details"}}async selectSession(e){this.selected_session=e,this.selected_session_detail=void 0,this.error_message=void 0;try{this.selected_session_detail=await b(`/api/session?session_id=${encodeURIComponent(e.session_id)}&limit=500`)}catch(t){this.error_message=t instanceof Error?t.message:"Unable to load session timeline"}}async openSession(e){const t=this.sessions.find(s=>s.session_id===e);if(!t){this.error_message="This request references a session that is no longer available in the request history.";return}this.active_view="sessions",await this.selectSession(t)}async openRequest(e){this.active_view="requests",await this.selectRequest(e)}setActiveView(e){this.active_view=e}submitSearch(e){e.preventDefault(),this.loadRequests()}updateSearch(e){this.search_query=e.target.value}render(){const e=this.selected_request?le(this.selected_request):void 0;return d`
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
          ${this.active_view==="requests"?d`<form @submit=${this.submitSearch}>
                <input
                  aria-label="Search requests"
                  .value=${this.search_query}
                  @input=${this.updateSearch}
                  placeholder="Search request, session, or model"
                />
                <button type="submit">Filter</button>
              </form>`:d`<p class="muted">Sessions are inferred from persisted request session ids.</p>`}
          <span class="data-path">${this.info?this.info.requests_dir:"Loading request history…"}</span>
        </section>
        ${this.error_message?d`<section class="error-banner">${this.error_message}</section>`:c}
        <section class="viewer-grid ${this.loading?"loading":""}">
          <aside class="sidebar">
            ${this.active_view==="requests"?d`<request-list
                  .requests=${this.requests}
                  .selected_key=${e}
                  @request-select=${t=>{this.selectRequest(j(t))}}
                ></request-list>`:d`<session-list
                  .sessions=${this.sessions}
                  .selected_session_id=${this.selected_session?.session_id}
                  @session-select=${t=>{this.selectSession(j(t))}}
                ></session-list>`}
          </aside>
          <article class="detail-pane">
            ${this.active_view==="requests"?d`<request-detail-view
                  .detail=${this.selected_request_detail}
                  @open-session=${t=>{this.openSession(j(t))}}
                ></request-detail-view>`:d`<session-timeline
                  .detail=${this.selected_session_detail}
                  @request-select=${t=>{this.openRequest(j(t))}}
                ></session-timeline>`}
          </article>
        </section>
      </main>
    `}}customElements.define("json-viewer",Oe);customElements.define("request-list",je);customElements.define("session-list",Ne);customElements.define("request-detail-view",He);customElements.define("session-timeline",Te);customElements.define("inspect-app",Me);
