import { useEffect, useState } from "react";
import { listen } from "@tauri-apps/api/event";
import { ArrowUpRight, Power, Square, RefreshCw } from "lucide-react";
import { api } from "../lib/tauri";
import { useResource } from "../lib/useResource";
import type { GatewayStatus } from "../lib/types";
import { ErrorMessage, PageHeading, number } from "../components/Feedback";
export function Overview() {
  const status = useResource(api.status);
  const usage = useResource(api.usage);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState("");
  useEffect(() => {
    const unlisten = listen<GatewayStatus>("gateway-status", (event) =>
      status.setData(event.payload),
    );
    void unlisten.catch((error) => setError(String(error)));
    return () => {
      void unlisten.then((stop) => stop()).catch(() => {});
    };
  }, [status.setData]);
  async function act(action: () => Promise<unknown>) {
    setBusy(true);
    setError("");
    try {
      await action();
      await status.refresh();
    } catch (error) {
      setError(String(error));
    } finally {
      await status.refresh();
      setBusy(false);
    }
  }
  const totals = (usage.data ?? []).reduce(
    (sum, row) => ({
      requests: sum.requests + row.requests,
      tokens: sum.tokens + row.input_tokens + row.output_tokens,
      cached: sum.cached + row.cached_tokens,
    }),
    { requests: 0, tokens: 0, cached: 0 },
  );
  const running = status.data?.state === "running";
  return (
    <>
      <PageHeading eyebrow="YOUR LOCAL GATEWAY" title="Overview">
        <button
          onClick={() => {
            void status.refresh();
            void usage.refresh();
          }}
          disabled={busy}
        >
          <RefreshCw size={15} />
          Refresh
        </button>
      </PageHeading>
      <ErrorMessage
        error={error || status.error || status.data?.last_error || ""}
      />
      <section className="gateway-card">
        <div className="gateway-top">
          <span className={`status-pill ${running ? "online" : ""}`}>
            <i />
            {status.loading
              ? "Checking connection"
              : (status.data?.state ?? "Unavailable")}
          </span>
          <span className="muted">LOCAL INSTANCE</span>
        </div>
        <h2>{status.data?.address ?? "Your gateway, on your machine."}</h2>
        <p>
          {status.data?.ownership === "external"
            ? "Started outside this app. You can view activity and apply routing changes here."
            : status.data?.ownership === "managed"
              ? "Managed by Tokn Desktop. Closing the app will gracefully stop this gateway."
              : "Start the bundled gateway using your existing Tokn configuration."}
        </p>
        <div className="actions">
          <button
            className="primary"
            disabled={busy || !status.data || status.data.ownership !== "none"}
            onClick={() => void act(api.start)}
          >
            <Power size={16} />
            Start gateway
          </button>
          <button
            disabled={busy || status.data?.ownership !== "managed"}
            onClick={() => void act(api.stop)}
          >
            <Square size={14} />
            {busy ? "Working…" : "Stop gateway"}
          </button>
        </div>
      </section>
      <div className="section-label">
        <h3>Activity</h3>
        <span>Past 24 hours · recorded usage</span>
      </div>
      <ErrorMessage error={usage.error} />
      <div className="stats">
        {[
          ["Requests", totals.requests],
          ["Input + output tokens", totals.tokens],
          ["Cache read tokens", totals.cached],
        ].map(([label, value]) => (
          <section className="stat" key={label}>
            <p>{label}</p>
            <strong>
              {usage.loading ? "…" : usage.error ? "—" : number(Number(value))}
            </strong>
            <ArrowUpRight size={18} />
          </section>
        ))}
      </div>
      <section className="note">
        <span className="note-mark">↳</span>
        <div>
          <h3>One gateway. Every agent.</h3>
          <p>
            Use the same local endpoint across your agents. Provider selection
            follows your profiles and model scores.
          </p>
        </div>
      </section>
    </>
  );
}
