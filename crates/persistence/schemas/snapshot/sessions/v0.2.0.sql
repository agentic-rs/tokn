-- Canonical current schema for sessions.db.
-- Regenerated whenever a new NNN_*.sql migration is added so that fresh
-- installs can jump straight here instead of replaying history.
-- Must remain equivalent to the cumulative effect of 001..002.
--
-- Mental model: a session is an ordered list of *parts*. A "message" is
-- merely a logical grouping of consecutive parts that share a role / turn,
-- exposed via the `message_seq` column rather than a separate table.

CREATE TABLE sessions (
  id            TEXT PRIMARY KEY,
  first_seen_ts INTEGER NOT NULL,
  last_seen_ts  INTEGER NOT NULL,
  source        TEXT    NOT NULL,        -- 'header' | 'auto'
  account_id    TEXT,
  provider_id   TEXT,
  model         TEXT,
  message_count INTEGER NOT NULL DEFAULT 0,
  part_count    INTEGER NOT NULL DEFAULT 0
);
CREATE INDEX idx_sessions_last ON sessions(last_seen_ts);

-- Content-addressed part store; identical parts dedupe across the whole DB.
CREATE TABLE part_blobs (
  hash      TEXT PRIMARY KEY,            -- sha256(part_type || 0x00 || content)
  part_type TEXT NOT NULL,               -- 'text' | 'image_url' | 'tool_use' | ...
  content   BLOB NOT NULL
);

-- Each row = one part of a session, in order.
CREATE TABLE session_parts (
  session_id  TEXT    NOT NULL REFERENCES sessions(id),
  part_seq    INTEGER NOT NULL,          -- monotonic 0..N within the session
  message_seq INTEGER NOT NULL,          -- groups parts of the same message
  part_index  INTEGER NOT NULL,          -- position within the message
  ts          INTEGER NOT NULL,
  endpoint    TEXT    NOT NULL,
  role        TEXT    NOT NULL,
  status      INTEGER,
  part_hash   TEXT    NOT NULL REFERENCES part_blobs(hash),
  PRIMARY KEY (session_id, part_seq)
);
CREATE INDEX idx_session_parts_msg  ON session_parts(session_id, message_seq, part_index);
CREATE INDEX idx_session_parts_hash ON session_parts(part_hash);

-- Tree-shaped semantic session store. Each node maps to one observed
-- request/response boundary; reducers may store only the suffix relative to
-- the chosen parent, but they never create synthetic intermediate nodes.
CREATE TABLE session_nodes (
  id                     TEXT PRIMARY KEY,
  session_id             TEXT    NOT NULL REFERENCES sessions(id),
  parent_id              TEXT    REFERENCES session_nodes(id),
  request_id             TEXT    NOT NULL,
  ts                     INTEGER NOT NULL,
  endpoint               TEXT    NOT NULL,
  status                 INTEGER,
  account_id             TEXT,
  provider_id            TEXT,
  model                  TEXT,
  reduction_kind         TEXT    NOT NULL,
  parent_source          TEXT    NOT NULL,
  common_prefix_messages INTEGER NOT NULL DEFAULT 0,
  request_message_count  INTEGER NOT NULL DEFAULT 0,
  response_message_count INTEGER NOT NULL DEFAULT 0,
  UNIQUE(session_id, request_id)
);
CREATE INDEX idx_session_nodes_session_parent ON session_nodes(session_id, parent_id);
CREATE INDEX idx_session_nodes_session_ts ON session_nodes(session_id, ts);

CREATE TABLE node_messages (
  id          TEXT PRIMARY KEY,
  node_id     TEXT    NOT NULL REFERENCES session_nodes(id),
  side        TEXT    NOT NULL,           -- 'request' | 'response'
  message_seq INTEGER NOT NULL,           -- order within this node's delta
  role        TEXT    NOT NULL,
  status      INTEGER,
  UNIQUE(node_id, side, message_seq)
);
CREATE INDEX idx_node_messages_node_side ON node_messages(node_id, side, message_seq);

CREATE TABLE node_parts (
  message_id TEXT    NOT NULL REFERENCES node_messages(id),
  part_index INTEGER NOT NULL,
  part_hash  TEXT    NOT NULL REFERENCES part_blobs(hash),
  PRIMARY KEY(message_id, part_index)
);
CREATE INDEX idx_node_parts_hash ON node_parts(part_hash);

CREATE TABLE session_heads (
  session_id TEXT    PRIMARY KEY REFERENCES sessions(id),
  node_id    TEXT    NOT NULL REFERENCES session_nodes(id),
  updated_ts INTEGER NOT NULL
);

-- Subagent/session topology metadata. This is deliberately separate from
-- session_nodes.parent_id, which is the conversation-tree edge.
CREATE TABLE session_relations (
  parent_session_id TEXT    NOT NULL,
  child_session_id  TEXT    NOT NULL,
  relation_kind     TEXT    NOT NULL,
  first_seen_ts     INTEGER NOT NULL,
  last_seen_ts      INTEGER NOT NULL,
  source            TEXT    NOT NULL,
  PRIMARY KEY(parent_session_id, child_session_id, relation_kind)
);
