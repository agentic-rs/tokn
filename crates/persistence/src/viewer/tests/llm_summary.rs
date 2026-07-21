use rusqlite::params;
use serde_json::json;

use crate::requests::open_day_db;

use super::super::get_request_llm_summary;
use super::support::{tempdir, write_request};

#[test]
fn summarizes_compressed_responses_messages_and_tools_with_bounded_previews() {
  let dir = tempdir();
  write_request(
    &dir,
    "2026-07-14",
    "request-llm-summary",
    1_784_444_800_000,
    Some("session-1"),
    Some("openai"),
  );
  let input = (0..8)
    .map(|index| json!({"role": "user", "content": [{"type": "input_text", "text": format!("message {index}")}]}))
    .chain([
      json!({"type": "function_call", "name": "lookup", "arguments": "{\"id\":1}"}),
      json!({"type": "function_call_output", "output": "result"}),
    ])
    .collect::<Vec<_>>();
  let request_body = serde_json::to_vec(&json!({
    "input": input,
    "tools": [
      {"type": "function", "function": {"name": "lookup"}},
      {"type": "custom", "name": "shell"}
    ]
  }))
  .unwrap();
  let compressed = zstd::stream::encode_all(request_body.as_slice(), 0).unwrap();
  let response = [
    "event: response.output_item.done",
    r#"data: {"type":"response.output_item.done","item":{"type":"message","role":"assistant","content":[{"type":"output_text","text":"finished"}]}}"#,
    "",
    "event: response.output_item.done",
    r#"data: {"type":"response.output_item.done","item":{"type":"custom_tool_call","name":"shell","input":"echo ok","status":"completed"}}"#,
    "",
  ]
  .join("\n");
  let conn = open_day_db(&dir.join("2026-07-14.db")).unwrap();
  conn
    .execute(
      "UPDATE request_downstream
       SET inbound_req_headers = ?2, inbound_req_body = ?3, inbound_resp_body = ?4
       WHERE request_id = ?1",
      params![
        "request-llm-summary",
        br#"{"content-encoding":"zstd","content-type":"application/json"}"#,
        compressed,
        response.as_bytes()
      ],
    )
    .unwrap();

  let summary = get_request_llm_summary(&dir, "2026-07-14", "request-llm-summary", None)
    .unwrap()
    .unwrap();
  assert_eq!(summary.messages_total, 9);
  assert_eq!(summary.messages.len(), 6);
  assert_eq!(summary.messages.first().unwrap().text.as_deref(), Some("message 3"));
  assert_eq!(summary.messages.last().unwrap().text.as_deref(), Some("finished"));
  assert_eq!(summary.messages.last().unwrap().phase, "output");
  assert_eq!(summary.tool_definitions_total, 2);
  assert_eq!(summary.tool_definitions[0].name, "lookup");
  assert_eq!(summary.tool_definitions[1].name, "shell");
  assert_eq!(summary.tool_calls_total, 2);
  assert_eq!(summary.tool_calls[0].name, "lookup");
  assert_eq!(summary.tool_calls[0].phase, "input");
  assert_eq!(summary.tool_calls[1].name, "shell");
  assert_eq!(summary.tool_calls[1].phase, "output");
  assert_eq!(summary.tool_results_total, 1);
  assert!(summary.warning.is_none());
}

#[test]
fn returns_a_warning_instead_of_failing_for_unsupported_content_encoding() {
  let dir = tempdir();
  write_request(
    &dir,
    "2026-07-14",
    "request-unsupported-encoding",
    1_784_444_800_000,
    None,
    Some("openai"),
  );
  let conn = open_day_db(&dir.join("2026-07-14.db")).unwrap();
  conn
    .execute(
      "UPDATE request_downstream
       SET inbound_req_headers = ?2, inbound_req_body = ?3
       WHERE request_id = ?1",
      params![
        "request-unsupported-encoding",
        br#"{"content-encoding":"br"}"#,
        b"opaque"
      ],
    )
    .unwrap();

  let summary = get_request_llm_summary(&dir, "2026-07-14", "request-unsupported-encoding", None)
    .unwrap()
    .unwrap();
  assert_eq!(summary.messages_total, 0);
  assert!(summary.warning.unwrap().contains("unsupported content encoding br"));
}
