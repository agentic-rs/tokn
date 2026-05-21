-- Squashed requests migrations from snapshot v0.0.0 to snapshot v0.1.1.
-- Covers schema versions 0001 through 0007.

ALTER TABLE requests ADD COLUMN request_id TEXT;
ALTER TABLE requests ADD COLUMN request_error TEXT;

-- Rename prompt_tok/completion_tok to input_tok/output_tok and add
-- cached_tok and reasoning_tok columns for breakdown of usage tokens.
ALTER TABLE requests RENAME COLUMN prompt_tok TO input_tok;
ALTER TABLE requests RENAME COLUMN completion_tok TO output_tok;
ALTER TABLE requests ADD COLUMN cached_tok INTEGER;
ALTER TABLE requests ADD COLUMN reasoning_tok INTEGER;

ALTER TABLE requests ADD COLUMN latency_ms_nullable INTEGER;
UPDATE requests SET latency_ms_nullable = latency_ms;
ALTER TABLE requests DROP COLUMN latency_ms;
ALTER TABLE requests RENAME COLUMN latency_ms_nullable TO latency_ms;

ALTER TABLE requests ADD COLUMN status_nullable INTEGER;
UPDATE requests SET status_nullable = status;
ALTER TABLE requests DROP COLUMN status;
ALTER TABLE requests RENAME COLUMN status_nullable TO status;

ALTER TABLE requests ADD COLUMN stream_nullable INTEGER;
UPDATE requests SET stream_nullable = stream;
ALTER TABLE requests DROP COLUMN stream;
ALTER TABLE requests RENAME COLUMN stream_nullable TO stream;

DROP INDEX idx_requests_session;
ALTER TABLE requests ADD COLUMN session_id_nullable TEXT;
UPDATE requests SET session_id_nullable = session_id;
ALTER TABLE requests DROP COLUMN session_id;
ALTER TABLE requests RENAME COLUMN session_id_nullable TO session_id;
CREATE INDEX idx_requests_session ON requests(session_id);

ALTER TABLE requests ADD COLUMN inbound_req_headers_nullable BLOB;
UPDATE requests SET inbound_req_headers_nullable = inbound_req_headers;
ALTER TABLE requests DROP COLUMN inbound_req_headers;
ALTER TABLE requests RENAME COLUMN inbound_req_headers_nullable TO inbound_req_headers;

ALTER TABLE requests ADD COLUMN inbound_req_body_nullable BLOB;
UPDATE requests SET inbound_req_body_nullable = inbound_req_body;
ALTER TABLE requests DROP COLUMN inbound_req_body;
ALTER TABLE requests RENAME COLUMN inbound_req_body_nullable TO inbound_req_body;

ALTER TABLE requests ADD COLUMN inbound_resp_headers_nullable BLOB;
UPDATE requests SET inbound_resp_headers_nullable = inbound_resp_headers;
ALTER TABLE requests DROP COLUMN inbound_resp_headers;
ALTER TABLE requests RENAME COLUMN inbound_resp_headers_nullable TO inbound_resp_headers;

ALTER TABLE requests ADD COLUMN inbound_resp_body_nullable BLOB;
UPDATE requests SET inbound_resp_body_nullable = inbound_resp_body;
ALTER TABLE requests DROP COLUMN inbound_resp_body;
ALTER TABLE requests RENAME COLUMN inbound_resp_body_nullable TO inbound_resp_body;

ALTER TABLE requests ADD COLUMN latency_header_ms INTEGER;
CREATE UNIQUE INDEX idx_requests_request_id ON requests(request_id);

ALTER TABLE requests ADD COLUMN source TEXT;
ALTER TABLE requests ADD COLUMN method TEXT;

ALTER TABLE requests RENAME COLUMN source TO peer_addr;
ALTER TABLE requests ADD COLUMN user TEXT;
ALTER TABLE requests ADD COLUMN local_addr TEXT;
ALTER TABLE requests ADD COLUMN mode TEXT;
ALTER TABLE requests ADD COLUMN behave_as TEXT;

CREATE TABLE IF NOT EXISTS metrics (
  id INTEGER PRIMARY KEY,
  ts INTEGER NOT NULL,
  request_id TEXT,
  user TEXT,
  peer_addr TEXT,
  local_addr TEXT,
  mode TEXT,
  behave_as TEXT,
  method TEXT,
  path TEXT,
  url TEXT,
  status INTEGER,
  request_error TEXT,
  account_id TEXT,
  provider_id TEXT,
  latency_ms INTEGER,

  inbound_req_method   TEXT,
  inbound_req_url      TEXT,
  inbound_req_headers  BLOB,

  inbound_resp_status  INTEGER,
  inbound_resp_headers BLOB,
  inbound_resp_body    BLOB
);

CREATE INDEX IF NOT EXISTS idx_metrics_ts       ON metrics(ts);
CREATE INDEX IF NOT EXISTS idx_metrics_local_addr ON metrics(local_addr);
CREATE INDEX IF NOT EXISTS idx_metrics_provider ON metrics(provider_id);
CREATE INDEX IF NOT EXISTS idx_metrics_account  ON metrics(account_id);
CREATE UNIQUE INDEX IF NOT EXISTS idx_metrics_request_id ON metrics(request_id) WHERE request_id IS NOT NULL;

ALTER TABLE requests RENAME TO requests_legacy;

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

INSERT INTO request_connection (
  rowid,
  request_id,
  ts,
  endpoint,
  status,
  request_error,
  latency_ms,
  latency_header_ms,
  user,
  peer_addr,
  local_addr,
  mode,
  behave_as,
  method
)
SELECT
  id,
  CASE WHEN request_id IS NULL OR request_id = '' THEN 'legacy:' || id ELSE request_id END,
  ts,
  endpoint,
  status,
  request_error,
  latency_ms,
  latency_header_ms,
  user,
  peer_addr,
  local_addr,
  mode,
  behave_as,
  method
FROM requests_legacy;

INSERT INTO request_metadata (
  request_id,
  session_id,
  account_id,
  provider_id,
  model,
  initiator,
  stream,
  input_tok,
  output_tok,
  cached_tok,
  reasoning_tok
)
SELECT
  CASE WHEN request_id IS NULL OR request_id = '' THEN 'legacy:' || id ELSE request_id END,
  session_id,
  account_id,
  provider_id,
  model,
  initiator,
  stream,
  input_tok,
  output_tok,
  cached_tok,
  reasoning_tok
FROM requests_legacy;

INSERT INTO request_downstream (
  request_id,
  inbound_req_method,
  inbound_req_url,
  inbound_req_headers,
  inbound_req_body,
  inbound_resp_status,
  inbound_resp_headers,
  inbound_resp_body
)
SELECT
  CASE WHEN request_id IS NULL OR request_id = '' THEN 'legacy:' || id ELSE request_id END,
  inbound_req_method,
  inbound_req_url,
  inbound_req_headers,
  inbound_req_body,
  inbound_resp_status,
  inbound_resp_headers,
  inbound_resp_body
FROM requests_legacy;

INSERT INTO request_upstream (
  request_id,
  outbound_req_method,
  outbound_req_url,
  outbound_req_headers,
  outbound_req_body,
  outbound_resp_status,
  outbound_resp_headers,
  outbound_resp_body
)
SELECT
  CASE WHEN request_id IS NULL OR request_id = '' THEN 'legacy:' || id ELSE request_id END,
  outbound_req_method,
  outbound_req_url,
  outbound_req_headers,
  outbound_req_body,
  outbound_resp_status,
  outbound_resp_headers,
  outbound_resp_body
FROM requests_legacy;

DROP TABLE requests_legacy;
DROP TABLE IF EXISTS metrics;

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
