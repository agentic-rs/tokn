import { useEffect, useState } from "react";
import { api } from "../lib/tauri";
import { useResource } from "../lib/useResource";
import { ErrorMessage, PageHeading } from "../components/Feedback";
export function Routing() {
  const resource = useResource(api.routing);
  const [draft, setDraft] = useState("");
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState("");
  const [message, setMessage] = useState("");
  useEffect(() => {
    if (resource.data) setDraft(resource.data.routing_toml);
  }, [resource.data]);
  const dirty = !!resource.data && draft !== resource.data.routing_toml;
  async function save() {
    if (!resource.data) return;
    setBusy(true);
    setError("");
    setMessage("");
    try {
      resource.setData(await api.saveRouting(resource.data, draft));
      setMessage(
        "Validated and saved. Apply to the gateway to activate these changes, or start it later.",
      );
    } catch (error) {
      setError(String(error));
    } finally {
      setBusy(false);
    }
  }
  async function reload() {
    setBusy(true);
    setError("");
    setMessage("");
    try {
      setMessage(await api.reload());
    } catch (error) {
      setError(String(error));
    } finally {
      setBusy(false);
    }
  }
  return (
    <>
      <PageHeading eyebrow="ROUTING POLICY" title="Routing">
        <span className="badge">
          {dirty ? "Unsaved changes" : "Saved configuration"}
        </span>
      </PageHeading>
      <p className="intro">
        Edit defaults, profiles and model scores (plus routes for v2). Exact
        model IDs and trailing wildcards such as <code>gpt-*</code> are
        supported; higher provider scores win.
      </p>
      <ErrorMessage error={error || resource.error} />
      {message && (
        <div className="notice" role="status">
          {message}
        </div>
      )}
      {!!resource.data?.overlay_paths.length && (
        <div className="notice">
          Editing the primary file. These fragments are applied afterwards and
          can override its profiles and scores:
          <ul>
            {resource.data.overlay_paths.map((path) => (
              <li key={path}>
                <code>{path}</code>
              </li>
            ))}
          </ul>
        </div>
      )}
      <section className="editor">
        <div className="editor-heading">
          <span>ROUTING.TOML</span>
          <span>Validated before saving</span>
        </div>
        <textarea
          aria-label="Routing configuration in TOML"
          spellCheck={false}
          value={draft}
          disabled={!resource.data || busy}
          onChange={(event) => {
            setDraft(event.target.value);
            setMessage("");
          }}
          placeholder={
            resource.loading
              ? "Loading configuration…"
              : '[model_scores."gpt-*"]\ncodex = 10'
          }
        />
      </section>
      <div className="routing-actions">
        <button
          disabled={busy || resource.loading}
          onClick={() => {
            if (
              !dirty ||
              window.confirm(
                "Discard your unsaved routing changes and reload from disk?",
              )
            ) {
              void resource.refresh();
              setMessage("");
            }
          }}
        >
          Reload from disk
        </button>
        <div className="actions">
          <button
            disabled={busy || dirty || !resource.data}
            onClick={() => void reload()}
          >
            Apply to gateway
          </button>
          <button
            className="primary"
            disabled={busy || !dirty}
            onClick={() => void save()}
          >
            {busy ? "Working…" : "Validate & save"}
          </button>
        </div>
      </div>
      <p className="path">{resource.data?.config_path}</p>
      <p className="muted">
        Listener and service settings stay in your config file. A rejected
        reload keeps the gateway’s previous configuration active.
      </p>
    </>
  );
}
