import { useRef, useState } from "react";
import { api } from "../lib/tauri";
import { useResource } from "../lib/useResource";
import type { RequestDetail, RequestSummary } from "../lib/types";
import { Empty, ErrorMessage, PageHeading } from "../components/Feedback";
export function History() {
  const history = useResource(api.history);
  const [selected, setSelected] = useState<RequestDetail | null>(null);
  const [error, setError] = useState("");
  const [loading, setLoading] = useState(false);
  const generation = useRef(0);
  async function select(row: RequestSummary) {
    const current = ++generation.current;
    setError("");
    setLoading(true);
    setSelected(null);
    try {
      const detail = await api.detail(row);
      if (current === generation.current) {
        setSelected(detail);
        if (!detail) setError("This request is no longer available.");
      }
    } catch (error) {
      if (current === generation.current) setError(String(error));
    } finally {
      if (current === generation.current) setLoading(false);
    }
  }
  return (
    <>
      <PageHeading eyebrow="LOCAL REQUEST LOG" title="History">
        <button onClick={() => void history.refresh()}>Refresh</button>
      </PageHeading>
      <p className="intro">
        Latest 100 requests from the most recent recorded day. Select a request
        to inspect its metadata.
      </p>
      <ErrorMessage error={history.error || error} />
      {history.loading ? (
        <Empty>Loading request history…</Empty>
      ) : !history.data?.requests.length ? (
        <Empty>
          No requests recorded yet. Activity will appear here when request
          persistence is enabled.
        </Empty>
      ) : (
        <div className="table-wrap">
          <table>
            <thead>
              <tr>
                <th>Time</th>
                <th>Model / provider</th>
                <th>Endpoint</th>
                <th>Status</th>
                <th />
              </tr>
            </thead>
            <tbody>
              {history.data.requests.map((row) => (
                <tr key={`${row.day}:${row.row_id}`}>
                  <td className="mono">
                    {new Date(row.ts).toLocaleTimeString()}
                  </td>
                  <td>
                    {row.model ?? "Unknown model"}
                    <small>{row.provider_id ?? "No provider"}</small>
                  </td>
                  <td>
                    <code>{row.endpoint ?? "—"}</code>
                  </td>
                  <td>
                    <span
                      className={`badge ${row.request_error || (row.status ?? 0) >= 400 ? "failed" : "enabled"}`}
                    >
                      {row.request_error ? "Error" : (row.status ?? "—")}
                    </span>
                  </td>
                  <td>
                    <button
                      onClick={() => void select(row)}
                      aria-label={`Inspect request ${row.request_id}`}
                    >
                      Inspect
                    </button>
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      )}
      {loading && <Empty>Loading request details…</Empty>}
      {selected && (
        <section className="detail">
          <div className="section-label">
            <h3>Request details</h3>
            <button onClick={() => setSelected(null)}>Close</button>
          </div>
          <pre>{JSON.stringify(selected.request, null, 2)}</pre>
        </section>
      )}
    </>
  );
}
