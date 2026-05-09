ALTER TABLE requests ADD COLUMN latency_ms_nullable INTEGER;
UPDATE requests SET latency_ms_nullable = latency_ms;
ALTER TABLE requests DROP COLUMN latency_ms;
ALTER TABLE requests RENAME COLUMN latency_ms_nullable TO latency_ms;
ALTER TABLE requests ADD COLUMN latency_header_ms INTEGER;
