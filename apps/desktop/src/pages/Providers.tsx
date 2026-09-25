import { api } from "../lib/tauri";
import { useResource } from "../lib/useResource";
import {
  Empty,
  ErrorMessage,
  PageHeading,
  number,
} from "../components/Feedback";
export function Providers() {
  const accounts = useResource(api.accounts);
  const usage = useResource(api.usage);
  return (
    <>
      <PageHeading eyebrow="CONNECTED ACCOUNTS" title="Providers">
        <button
          onClick={() => {
            void accounts.refresh();
            void usage.refresh();
          }}
        >
          Refresh
        </button>
      </PageHeading>
      <p className="intro">
        Accounts from your local credential store. Manage sign-in with the Tokn
        CLI.
      </p>
      <ErrorMessage error={accounts.error} />
      <ErrorMessage error={usage.error} />
      {accounts.loading ? (
        <Empty>Loading accounts…</Empty>
      ) : !accounts.data?.length ? (
        <Empty>
          No connected accounts. Add an account with the Tokn CLI to get
          started.
        </Empty>
      ) : (
        <div className="provider-grid">
          {accounts.data.map((account) => {
            const rows = (usage.data ?? []).filter(
              (row) =>
                row.account === account.id && row.provider === account.provider,
            );
            return (
              <section className="provider-card" key={account.id}>
                <div className="provider-icon">
                  {account.provider.slice(0, 2).toUpperCase()}
                </div>
                <span className={`badge ${account.enabled ? "enabled" : ""}`}>
                  {account.enabled ? account.tier : "disabled"}
                </span>
                <h2>{account.label || account.id}</h2>
                <p>{account.provider}</p>
                <div className="provider-footer">
                  <span>Requests · 24h</span>
                  <strong>
                    {usage.error
                      ? "Unavailable"
                      : usage.loading
                        ? "…"
                        : number(
                            rows.reduce((sum, row) => sum + row.requests, 0),
                          )}
                  </strong>
                </div>
              </section>
            );
          })}
        </div>
      )}
      <div className="section-label">
        <h3>Model usage</h3>
        <span>Recorded locally · not remaining provider quota</span>
      </div>
      {usage.loading ? (
        <Empty>Loading usage…</Empty>
      ) : !usage.data?.length ? (
        <Empty>No recorded usage in the past 24 hours.</Empty>
      ) : (
        <div className="table-wrap">
          <table>
            <thead>
              <tr>
                <th>Model / account</th>
                <th>Requests</th>
                <th>Input</th>
                <th>Output</th>
                <th>Cache read</th>
              </tr>
            </thead>
            <tbody>
              {usage.data.map((row, index) => (
                <tr key={index}>
                  <td>
                    {row.model}
                    <small>
                      {row.provider ?? "Unassigned"} /{" "}
                      {row.account ?? "Unassigned"}
                    </small>
                  </td>
                  <td>{number(row.requests)}</td>
                  <td>{number(row.input_tokens)}</td>
                  <td>{number(row.output_tokens)}</td>
                  <td>{number(row.cached_tokens)}</td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      )}
    </>
  );
}
