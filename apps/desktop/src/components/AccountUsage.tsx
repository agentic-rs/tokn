import { Empty, ErrorMessage, number } from "./Feedback";
import { api } from "../lib/tauri";
import { useResource } from "../lib/useResource";
export function AccountUsage() {
  const usage = useResource(api.usage);
  return (
    <>
      <ErrorMessage error={usage.error} />
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
