-- Canonical current schema for requests/<YYYY-MM-DD>.db.
-- Regenerated whenever a new NNN_*.sql migration is added so that fresh
-- day files can jump straight here instead of replaying history.
-- Must remain equivalent to the cumulative effect of 001..NNN.

CREATE TABLE request_connection (
  request_id TEXT PRIMARY KEY,
  ts INTEGER NOT NULL,
  endpoint TEXT,
  status INTEGER,
  request_error TEXT,
  latency_ms INTEGER,
  latency_header_ms INTEGER,
  user TEXT,
  peer_addr TEXT,
  local_addr TEXT,
  mode TEXT,
  behave_as TEXT,
  method TEXT
);
CREATE INDEX idx_request_connection_ts ON request_connection(ts);
CREATE INDEX idx_request_connection_local_addr ON request_connection(local_addr);

CREATE TABLE request_metadata (
  request_id TEXT PRIMARY KEY,
  session_id TEXT,
  account_id TEXT,
  provider_id TEXT,
  model TEXT,
  initiator TEXT,
  stream INTEGER,
  input_tok INTEGER,
  output_tok INTEGER,
  cached_tok INTEGER,
  reasoning_tok INTEGER
);
CREATE INDEX idx_request_metadata_session ON request_metadata(session_id);
CREATE INDEX idx_request_metadata_account ON request_metadata(account_id);
CREATE INDEX idx_request_metadata_provider ON request_metadata(provider_id);

CREATE TABLE request_downstream (
  request_id TEXT PRIMARY KEY,
  inbound_req_method TEXT,
  inbound_req_url TEXT,
  inbound_req_headers BLOB,
  inbound_req_body BLOB,
  inbound_resp_status INTEGER,
  inbound_resp_headers BLOB,
  inbound_resp_body BLOB
);

CREATE TABLE request_upstream (
  request_id TEXT PRIMARY KEY,
  outbound_req_method TEXT,
  outbound_req_url TEXT,
  outbound_req_headers BLOB,
  outbound_req_body BLOB,
  outbound_resp_status INTEGER,
  outbound_resp_headers BLOB,
  outbound_resp_body BLOB
);

CREATE VIEW requests AS
SELECT
  c.rowid AS idx,
  c.ts,
  m.session_id,
  c.request_id,
  c.request_error,
  c.endpoint,
  m.account_id,
  m.provider_id,
  m.model,
  m.initiator,
  c.status,
  m.stream,
  c.latency_ms,
  c.latency_header_ms,
  m.input_tok,
  m.output_tok,
  m.cached_tok,
  m.reasoning_tok,
  c.peer_addr,
  c.method,
  c.user,
  c.local_addr,
  c.mode,
  c.behave_as,
  d.inbound_req_method,
  d.inbound_req_url,
  d.inbound_req_headers,
  d.inbound_req_body,
  u.outbound_req_method,
  u.outbound_req_url,
  u.outbound_req_headers,
  u.outbound_req_body,
  u.outbound_resp_status,
  u.outbound_resp_headers,
  u.outbound_resp_body,
  d.inbound_resp_status,
  d.inbound_resp_headers,
  d.inbound_resp_body
FROM request_connection c
LEFT JOIN request_metadata m ON m.request_id = c.request_id
LEFT JOIN request_downstream d ON d.request_id = c.request_id
LEFT JOIN request_upstream u ON u.request_id = c.request_id;
