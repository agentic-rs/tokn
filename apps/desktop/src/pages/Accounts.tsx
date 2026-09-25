import { useState } from "react";
import { api } from "../lib/tauri";
import { useResource } from "../lib/useResource";
import { Empty, ErrorMessage, PageHeading } from "../components/Feedback";
import { AccountUsage } from "../components/AccountUsage";
import { AddAccount } from "../components/accounts/AddAccount";
import { AccountDetails } from "../components/accounts/AccountDetails";

export function Accounts() {
  const accounts = useResource(api.accounts);
  const [selected, setSelected] = useState("");
  const [adding, setAdding] = useState(false);
  const [message, setMessage] = useState("");
  const [search, setSearch] = useState("");
  const account = accounts.data?.find((account) => account.id === selected);
  const visible = (accounts.data ?? []).filter((account) =>
    `${account.id} ${account.label ?? ""} ${account.provider} ${account.username ?? ""}`
      .toLowerCase()
      .includes(search.toLowerCase()),
  );
  const providers = [...new Set(visible.map((account) => account.provider))];
  async function saved() {
    await accounts.refresh();
    setMessage("Account saved.");
    try {
      const status = await api.status();
      if (status.state === "running") {
        await api.reload();
        setMessage("Account saved and gateway reloaded.");
      } else
        setMessage("Account saved. The gateway will use it on its next start.");
    } catch {
      setMessage(
        "Account saved, but the gateway could not reload. Reload or restart it to apply the change.",
      );
    }
  }
  return (
    <>
      <PageHeading eyebrow="CREDENTIALS AND ACCESS" title="Accounts">
        <div className="actions">
          <button onClick={() => void accounts.refresh()}>Refresh list</button>
          <button className="primary" onClick={() => setAdding(true)}>
            Add account
          </button>
        </div>
      </PageHeading>
      <p className="intro">
        Manage provider credentials, activation, and remaining quota.
      </p>
      <ErrorMessage error={accounts.error} />
      {message && (
        <div className="notice" role="status">
          {message}
        </div>
      )}
      {adding && (
        <AddAccount onSaved={saved} onClose={() => setAdding(false)} />
      )}
      <label className="account-search">
        Search accounts
        <input
          type="search"
          value={search}
          onChange={(event) => setSearch(event.target.value)}
          placeholder="Provider, account, or identity"
        />
      </label>
      {accounts.loading && !accounts.data ? (
        <Empty>Loading accounts…</Empty>
      ) : !accounts.data?.length ? (
        <Empty>
          No accounts yet. Choose Add account to sign in or import a credential.
        </Empty>
      ) : !visible.length ? (
        <Empty>No matching accounts.</Empty>
      ) : (
        <div className="accounts-layout">
          <div className="account-list">
            {providers.map((provider) => (
              <section key={provider}>
                <h3>{provider}</h3>
                <div className="account-group">
                  {visible
                    .filter((item) => item.provider === provider)
                    .map((item) => (
                      <button
                        key={item.id}
                        className={`account-row ${selected === item.id ? "selected" : ""}`}
                        aria-pressed={selected === item.id}
                        onClick={() => setSelected(item.id)}
                      >
                        <span>
                          <strong>{item.label || item.id}</strong>
                          <small>{item.username || item.id}</small>
                        </span>
                        <span
                          className={`badge ${item.enabled ? "enabled" : ""}`}
                        >
                          {item.enabled ? item.tier : "disabled"}
                        </span>
                      </button>
                    ))}
                </div>
              </section>
            ))}
          </div>
          {account ? (
            <AccountDetails
              key={`${account.id}-${account.label}-${account.enabled}-${account.tier}`}
              account={account}
              onSaved={saved}
              onChecked={accounts.refresh}
            />
          ) : (
            <Empty>
              Select an account to manage credentials, activation, and quota.
            </Empty>
          )}
        </div>
      )}
      <AccountUsage />
    </>
  );
}
