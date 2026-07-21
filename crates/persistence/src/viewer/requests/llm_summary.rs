use flate2::read::GzDecoder;
use rusqlite::types::ValueRef;
use rusqlite::{params_from_iter, OptionalExtension};
use serde::Serialize;
use serde_json::Value;
use std::io::Read;
use std::path::Path;

use super::detail::{quote_identifier, request_lookup_condition};
use crate::viewer::database::open_readonly;
use crate::viewer::days::request_day_files;
use crate::viewer::schema::RequestSchema;
use crate::Result;

const MAX_DECODED_BODY_BYTES: u64 = 8 * 1024 * 1024;
const MAX_MESSAGE_PREVIEWS: usize = 6;
const MAX_TOOL_DEFINITIONS: usize = 12;
const MAX_TOOL_CALLS: usize = 8;
const MAX_PREVIEW_CHARS: usize = 280;
const MAX_LABEL_CHARS: usize = 120;

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct LlmMessagePreview {
  pub role: String,
  pub phase: String,
  pub text: Option<String>,
  pub truncated: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct LlmToolDefinitionSummary {
  pub name: String,
  pub kind: String,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct LlmToolCallSummary {
  pub name: String,
  pub kind: String,
  pub phase: String,
  pub status: Option<String>,
  pub argument_bytes: usize,
}

#[derive(Debug, Clone, Default, Serialize, PartialEq, Eq)]
pub struct LlmRequestContentSummary {
  pub messages_total: usize,
  pub messages: Vec<LlmMessagePreview>,
  pub tool_definitions_total: usize,
  pub tool_definitions: Vec<LlmToolDefinitionSummary>,
  pub tool_calls_total: usize,
  pub tool_calls: Vec<LlmToolCallSummary>,
  pub tool_results_total: usize,
  pub warning: Option<String>,
}

#[derive(Default)]
struct SummaryBuilder {
  messages: Vec<LlmMessagePreview>,
  tool_definitions: Vec<LlmToolDefinitionSummary>,
  tool_calls: Vec<LlmToolCallSummary>,
  tool_results_total: usize,
  warnings: Vec<String>,
}

impl SummaryBuilder {
  fn finish(mut self) -> LlmRequestContentSummary {
    let messages_total = self.messages.len();
    let tool_definitions_total = self.tool_definitions.len();
    let tool_calls_total = self.tool_calls.len();
    retain_last(&mut self.messages, MAX_MESSAGE_PREVIEWS);
    self.tool_definitions.truncate(MAX_TOOL_DEFINITIONS);
    retain_last(&mut self.tool_calls, MAX_TOOL_CALLS);
    LlmRequestContentSummary {
      messages_total,
      messages: self.messages,
      tool_definitions_total,
      tool_definitions: self.tool_definitions,
      tool_calls_total,
      tool_calls: self.tool_calls,
      tool_results_total: self.tool_results_total,
      warning: (!self.warnings.is_empty()).then(|| self.warnings.join("; ")),
    }
  }

  fn warning(&mut self, context: &str, error: impl std::fmt::Display) {
    self.warnings.push(format!("{context}: {error}"));
  }
}

/// Return a bounded semantic summary of the persisted LLM request and response
/// payloads. Large bodies stay inside the persistence layer and are never
/// returned wholesale by this query.
pub fn get_request_llm_summary(
  requests_dir: &Path,
  day: &str,
  request_id: &str,
  row_id: Option<i64>,
) -> Result<Option<LlmRequestContentSummary>> {
  let Some(day_file) = request_day_files(requests_dir)?
    .into_iter()
    .find(|file| file.day == day)
  else {
    return Ok(None);
  };
  let Some(conn) = open_readonly(&day_file.path)? else {
    return Ok(None);
  };

  let schema = RequestSchema::read(&conn)?;
  let request_condition = request_lookup_condition(schema, row_id.is_some());
  let fields = [
    "inbound_req_headers",
    "inbound_req_body",
    "outbound_req_headers",
    "outbound_req_body",
    "inbound_resp_body",
    "outbound_resp_body",
  ];
  let projection = fields.map(quote_identifier).join(", ");
  let mut values = vec![rusqlite::types::Value::Text(request_id.to_string())];
  if let Some(row_id) = row_id {
    values.push(rusqlite::types::Value::Integer(row_id));
  }
  let mut statement = conn.prepare(&format!(
    "SELECT {projection} FROM requests WHERE {request_condition} LIMIT 1"
  ))?;
  let row = statement
    .query_row(params_from_iter(values.iter()), |row| {
      Ok([
        sql_bytes(row.get_ref(0)?),
        sql_bytes(row.get_ref(1)?),
        sql_bytes(row.get_ref(2)?),
        sql_bytes(row.get_ref(3)?),
        sql_bytes(row.get_ref(4)?),
        sql_bytes(row.get_ref(5)?),
      ])
    })
    .optional()?;
  let Some([inbound_headers, inbound_request, outbound_headers, outbound_request, inbound_response, outbound_response]) =
    row
  else {
    return Ok(None);
  };

  let (request_headers, request_body) = if !inbound_request.is_empty() {
    (&inbound_headers, &inbound_request)
  } else {
    (&outbound_headers, &outbound_request)
  };
  let response_body = if !inbound_response.is_empty() {
    &inbound_response
  } else {
    &outbound_response
  };

  let mut summary = SummaryBuilder::default();
  if !request_body.is_empty() {
    match decode_json_request(request_headers, request_body) {
      Ok(request) => summarize_request(&request, &mut summary),
      Err(error) => summary.warning("request summary unavailable", error),
    }
  }
  if !response_body.is_empty() {
    summarize_response(response_body, &mut summary);
  }
  Ok(Some(summary.finish()))
}

fn sql_bytes(value: ValueRef<'_>) -> Vec<u8> {
  match value {
    ValueRef::Text(value) | ValueRef::Blob(value) => value.to_vec(),
    _ => Vec::new(),
  }
}

fn decode_json_request(headers: &[u8], body: &[u8]) -> std::result::Result<Value, String> {
  let headers = serde_json::from_slice::<Value>(headers).unwrap_or(Value::Null);
  let encoding = header_str(&headers, "content-encoding").unwrap_or("identity");
  let decoded = match encoding.trim().to_ascii_lowercase().as_str() {
    "" | "identity" => bounded_read(body)?,
    "gzip" => bounded_read(GzDecoder::new(body))?,
    "zstd" => {
      let decoder = zstd::stream::read::Decoder::new(body).map_err(|error| error.to_string())?;
      bounded_read(decoder)?
    }
    other => return Err(format!("unsupported content encoding {other}")),
  };
  serde_json::from_slice(&decoded).map_err(|error| error.to_string())
}

fn bounded_read(reader: impl Read) -> std::result::Result<Vec<u8>, String> {
  let mut output = Vec::new();
  reader
    .take(MAX_DECODED_BODY_BYTES + 1)
    .read_to_end(&mut output)
    .map_err(|error| error.to_string())?;
  if output.len() as u64 > MAX_DECODED_BODY_BYTES {
    return Err(format!("decoded body exceeds {MAX_DECODED_BODY_BYTES} bytes"));
  }
  Ok(output)
}

fn header_str<'a>(headers: &'a Value, name: &str) -> Option<&'a str> {
  headers
    .as_object()?
    .iter()
    .find(|(key, _)| key.eq_ignore_ascii_case(name))
    .and_then(|(_, value)| value.as_str())
}

fn summarize_request(request: &Value, summary: &mut SummaryBuilder) {
  summarize_tool_definitions(request.get("tools"), summary);
  if let Some(input) = request.get("input").and_then(Value::as_array) {
    summarize_items(input, "input", summary);
  }
  if let Some(messages) = request.get("messages").and_then(Value::as_array) {
    summarize_items(messages, "input", summary);
  }
  if let Some(system) = request.get("system") {
    push_message("system", "input", system, summary);
  }
}

fn summarize_response(body: &[u8], summary: &mut SummaryBuilder) {
  if body.len() as u64 > MAX_DECODED_BODY_BYTES {
    summary.warning("response summary unavailable", "body exceeds summary limit");
    return;
  }
  if let Ok(response) = serde_json::from_slice::<Value>(body) {
    summarize_response_value(&response, summary);
    return;
  }

  let Ok(text) = std::str::from_utf8(body) else {
    summary.warning("response summary unavailable", "body is not UTF-8 JSON or SSE");
    return;
  };
  let mut found_event = false;
  for line in text.lines().filter_map(|line| line.strip_prefix("data: ")) {
    if line == "[DONE]" {
      continue;
    }
    let Ok(event) = serde_json::from_str::<Value>(line) else {
      continue;
    };
    found_event = true;
    summarize_response_event(&event, summary);
  }
  if !found_event {
    summary.warning("response summary unavailable", "body is not JSON or recognized SSE");
  }
}

fn summarize_response_value(response: &Value, summary: &mut SummaryBuilder) {
  if let Some(output) = response.get("output").and_then(Value::as_array) {
    summarize_items(output, "output", summary);
  }
  if let Some(content) = response.get("content").and_then(Value::as_array) {
    push_message("assistant", "output", &Value::Array(content.clone()), summary);
    summarize_items(content, "output", summary);
  }
  if let Some(choices) = response.get("choices").and_then(Value::as_array) {
    for message in choices.iter().filter_map(|choice| choice.get("message")) {
      summarize_item(message, "output", summary);
    }
  }
}

fn summarize_response_event(event: &Value, summary: &mut SummaryBuilder) {
  match event.get("type").and_then(Value::as_str) {
    Some("response.output_item.done") => {
      if let Some(item) = event.get("item") {
        summarize_item(item, "output", summary);
      }
    }
    Some("message_stop") => {
      if let Some(message) = event.get("message") {
        summarize_item(message, "output", summary);
      }
    }
    _ => {}
  }
}

fn summarize_items(items: &[Value], phase: &str, summary: &mut SummaryBuilder) {
  for item in items {
    summarize_item(item, phase, summary);
  }
}

fn summarize_item(item: &Value, phase: &str, summary: &mut SummaryBuilder) {
  let item_type = item.get("type").and_then(Value::as_str).unwrap_or("");
  if is_tool_call_type(item_type) {
    push_tool_call(item, item_type, phase, summary);
    return;
  }
  if is_tool_result_type(item_type) {
    summary.tool_results_total += 1;
    return;
  }

  let role = item.get("role").and_then(Value::as_str).or(match item_type {
    "compaction" => Some("compaction"),
    "message" => Some("assistant"),
    _ => None,
  });
  if let Some(role) = role {
    push_message(role, phase, item.get("content").unwrap_or(item), summary);
  }

  if let Some(tool_calls) = item.get("tool_calls").and_then(Value::as_array) {
    for tool_call in tool_calls {
      push_tool_call(tool_call, "function_call", phase, summary);
    }
  }
  if role == Some("tool") {
    summary.tool_results_total += 1;
  }
}

fn push_message(role: &str, phase: &str, content: &Value, summary: &mut SummaryBuilder) {
  let text = extract_text(content);
  let (text, truncated) = text
    .as_deref()
    .map(truncate_preview)
    .map_or((None, false), |(text, truncated)| (Some(text), truncated));
  summary.messages.push(LlmMessagePreview {
    role: truncate_label(role),
    phase: phase.to_string(),
    text,
    truncated,
  });
}

fn summarize_tool_definitions(tools: Option<&Value>, summary: &mut SummaryBuilder) {
  let Some(tools) = tools.and_then(Value::as_array) else {
    return;
  };
  for tool in tools {
    let kind = tool.get("type").and_then(Value::as_str).unwrap_or("tool");
    let name = tool
      .get("name")
      .or_else(|| tool.pointer("/function/name"))
      .and_then(Value::as_str)
      .unwrap_or(kind);
    summary.tool_definitions.push(LlmToolDefinitionSummary {
      name: truncate_label(name),
      kind: truncate_label(kind),
    });
  }
}

fn push_tool_call(item: &Value, item_type: &str, phase: &str, summary: &mut SummaryBuilder) {
  let name = item
    .get("name")
    .or_else(|| item.pointer("/function/name"))
    .and_then(Value::as_str)
    .unwrap_or(item_type);
  let arguments = item
    .get("input")
    .or_else(|| item.get("arguments"))
    .or_else(|| item.pointer("/function/arguments"));
  let argument_bytes = arguments.map(serialized_len).unwrap_or(0);
  summary.tool_calls.push(LlmToolCallSummary {
    name: truncate_label(name),
    kind: truncate_label(item_type),
    phase: phase.to_string(),
    status: item.get("status").and_then(Value::as_str).map(truncate_label),
    argument_bytes,
  });
}

fn serialized_len(value: &Value) -> usize {
  value.as_str().map(str::len).unwrap_or_else(|| value.to_string().len())
}

fn is_tool_call_type(item_type: &str) -> bool {
  matches!(item_type, "custom_tool_call" | "function_call" | "tool_call")
}

fn is_tool_result_type(item_type: &str) -> bool {
  matches!(
    item_type,
    "custom_tool_call_output" | "function_call_output" | "tool_result"
  )
}

fn extract_text(value: &Value) -> Option<String> {
  match value {
    Value::String(text) => Some(text.clone()),
    Value::Array(parts) => {
      let texts = parts.iter().filter_map(extract_text).collect::<Vec<_>>();
      (!texts.is_empty()).then(|| texts.join("\n"))
    }
    Value::Object(object) => ["text", "input_text", "output_text", "content"]
      .into_iter()
      .find_map(|field| object.get(field).and_then(extract_text)),
    _ => None,
  }
}

fn truncate_preview(value: &str) -> (String, bool) {
  let value = value.trim();
  if value.chars().count() <= MAX_PREVIEW_CHARS {
    return (value.to_string(), false);
  }
  let mut output = value.chars().take(MAX_PREVIEW_CHARS).collect::<String>();
  output.push('…');
  (output, true)
}

fn truncate_label(value: &str) -> String {
  let value = value.trim();
  if value.chars().count() <= MAX_LABEL_CHARS {
    return value.to_string();
  }
  let mut output = value.chars().take(MAX_LABEL_CHARS).collect::<String>();
  output.push('…');
  output
}

fn retain_last<T>(values: &mut Vec<T>, limit: usize) {
  if values.len() > limit {
    values.drain(..values.len() - limit);
  }
}
