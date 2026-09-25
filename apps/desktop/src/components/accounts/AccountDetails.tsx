import { useState } from "react";
import { api } from "../../lib/tauri";
import type {
  AccountActivation,
  AccountProbe,
  AccountSummary,
} from "../../lib/types";
import { ErrorMessage } from "../Feedback";

const date = (seconds: number | null) =>
  seconds ? new Date(seconds * 1000).toLocaleString() : "Not reported";
const percent = (value: number) => Math.max(0, Math.min(100, value));

export function AccountDetails({
  account,
  onSaved,
  onChecked,
}: {
  account: AccountSummary;
  onSaved: () => Promise<void>;
  onChecked: () => Promise<void>;
}) {
  const [label, setLabel] = useState(account.label ?? "");
  const [activation, setActivation] = useState<AccountActivation>(
    account.enabled ? account.tier : "disabled",
  );
  const [probe, setProbe] = useState<AccountProbe>();
  const [busy, setBusy] = useState("");
  const [error, setError] = useState("");
  const [removing, setRemoving] = useState(false);
  async function run(action: string, operation: () => Promise<void>) {
    setBusy(action);
    setError("");
    try {
      await operation();
    } catch (error) {
      setError(String(error));
    } finally {
      setBusy("");
    }
  }
  async function check(force: boolean) {
    await run(
      force ? "Refreshing credentials…" : "Checking provider…",
      async () => {
        setProbe(await api.probeAccount(account.id, force));
        await onChecked();
      },
    );
  }
  return (
    <section className="account-panel" aria-label={`Manage ${account.id}`}>
      <h2>{account.label || account.id}</h2>
      <p className="muted">
        {account.provider} · {account.id}
      </p>
      <ErrorMessage error={error} />
      <dl className="account-facts">
        <div>
          <dt>Identity</dt>
          <dd>{account.username ?? "Not reported"}</dd>
        </div>
        <div>
          <dt>Credential</dt>
          <dd>
            {account.credential_kind.replaceAll("_", " ")} ·{" "}
            {account.credential_status}
          </dd>
        </div>
        <div>
          <dt>Expires</dt>
          <dd>{date(account.expires_at)}</dd>
        </div>
        <div>
          <dt>Last refresh</dt>
          <dd>{date(account.last_refresh)}</dd>
        </div>
      </dl>
      <p className="muted">
        Stored status is local metadata. Check the provider for current
        authentication and quota.
      </p>
      <div className="actions account-actions">
        <button disabled={!!busy} onClick={() => void check(false)}>
          Check status and quota
        </button>
        {account.can_refresh && (
          <button disabled={!!busy} onClick={() => void check(true)}>
            Refresh credentials
          </button>
        )}
      </div>
      {busy && <p role="status">{busy}</p>}
      {probe && (
        <section className="quota-panel" aria-label="Provider quota">
          <h3>Provider quota</h3>
          <p>
            Authentication: {probe.authentication} · Checked{" "}
            {date(probe.checked_at)}
          </p>
          {probe.message && <p role="status">{probe.message}</p>}
          {probe.quota_status === "unsupported" ? (
            <p>This provider does not report quota.</p>
          ) : probe.quota_status === "unavailable" ? (
            <p>Quota unavailable; this does not mean zero remaining.</p>
          ) : (
            <>
              {probe.plan && <p>Plan: {probe.plan}</p>}
              {probe.headline && <p>{probe.headline}</p>}
              {probe.metered && (
                <div>
                  <p>
                    {probe.metered.label}:{" "}
                    {probe.metered.remaining.toLocaleString()} remaining
                    {probe.metered.entitlement != null
                      ? ` / ${probe.metered.entitlement.toLocaleString()}`
                      : " (unlimited allowance)"}
                  </p>
                  {probe.metered.entitlement != null &&
                    probe.metered.entitlement > 0 && (
                      <progress
                        aria-label={`${probe.metered.label} remaining`}
                        max={100}
                        value={percent(
                          (probe.metered.remaining /
                            probe.metered.entitlement) *
                            100,
                        )}
                      />
                    )}
                </div>
              )}
              {probe.secondary.map((bucket, index) => {
                const used =
                  bucket.percent_used ??
                  (bucket.total && bucket.used != null
                    ? (bucket.used / bucket.total) * 100
                    : undefined);
                return (
                  <div key={`${bucket.label}-${index}`}>
                    <p>
                      {bucket.label}:{" "}
                      {used != null
                        ? `${used.toFixed(1)}% used`
                        : bucket.used != null
                          ? `${bucket.used.toLocaleString()} used`
                          : "Usage not reported"}
                      {bucket.total
                        ? ` / ${bucket.total.toLocaleString()}`
                        : ""}
                    </p>
                    {used != null && (
                      <progress
                        aria-label={`${bucket.label} used`}
                        max={100}
                        value={percent(used)}
                      />
                    )}
                    {bucket.reset_at_ms && (
                      <small>
                        Resets {new Date(bucket.reset_at_ms).toLocaleString()}
                      </small>
                    )}
                  </div>
                );
              })}
              {probe.reset_date && <p>Resets {probe.reset_date}</p>}
            </>
          )}
        </section>
      )}
      <fieldset disabled={!!busy} className="account-form">
        <label>
          Label
          <input
            value={label}
            onChange={(event) => setLabel(event.target.value)}
          />
        </label>
        <label>
          Activation
          <select
            value={activation}
            onChange={(event) =>
              setActivation(event.target.value as AccountActivation)
            }
          >
            <option value="active">Active</option>
            <option value="fallback">Fallback</option>
            <option value="disabled">Disabled</option>
          </select>
        </label>
        <p className="muted">
          Active accounts are tried first. Fallback accounts are used when
          active accounts cannot serve a request. This setting is shared across
          profiles.
        </p>
        <button
          className="primary"
          onClick={() =>
            void run("Saving account…", async () => {
              await api.editAccount({
                action: "update",
                id: account.id,
                label: label.trim() || null,
                activation,
              });
              await onSaved();
            })
          }
        >
          Save changes
        </button>
      </fieldset>
      <div className="account-remove">
        {!removing ? (
          <button disabled={!!busy} onClick={() => setRemoving(true)}>
            Remove account…
          </button>
        ) : (
          <>
            <p>
              Remove <strong>{account.id}</strong> from its local credential
              file? Routes explicitly selecting it may stop working. This does
              not revoke access at the provider.
            </p>
            <div className="actions">
              <button disabled={!!busy} onClick={() => setRemoving(false)}>
                Keep account
              </button>
              <button
                className="danger"
                disabled={!!busy}
                onClick={() =>
                  void run("Removing account…", async () => {
                    await api.editAccount({ action: "remove", id: account.id });
                    await onSaved();
                  })
                }
              >
                Remove account
              </button>
            </div>
          </>
        )}
      </div>
    </section>
  );
}
