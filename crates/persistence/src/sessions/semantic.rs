use crate::{MessageRecord, PartRecord};
use bytes::Bytes;
use serde_json::Value;

pub(super) fn request_messages_from_json(endpoint: &str, value: &Value) -> Vec<MessageRecord> {
  let mut out = Vec::new();
  if let Some(instructions) = value.get("instructions").and_then(Value::as_str) {
    if !instructions.is_empty() {
      out.push(text_message("system", instructions));
    }
  }
  match endpoint {
    "chat_completions" | "chat/completions" => {
      extend_messages(&mut out, value.get("messages"));
    }
    "responses" => {
      if let Some(input) = value.get("input") {
        out.extend(input_messages(input));
      }
    }
    "messages" => {
      if let Some(system) = value.get("system").and_then(message_with_role("system")) {
        out.push(system);
      }
      extend_messages(&mut out, value.get("messages"));
    }
    _ => {}
  }
  out
}

fn extend_messages(out: &mut Vec<MessageRecord>, value: Option<&Value>) {
  if let Some(messages) = value.and_then(Value::as_array) {
    out.extend(messages.iter().filter_map(message_from_value));
  }
}

fn input_messages(input: &Value) -> Vec<MessageRecord> {
  match input {
    Value::String(text) => vec![text_message("user", text)],
    Value::Array(items) => items.iter().filter_map(message_from_value).collect(),
    Value::Object(_) => message_from_value(input).into_iter().collect(),
    _ => Vec::new(),
  }
}

fn message_with_role(role: &'static str) -> impl FnOnce(&Value) -> Option<MessageRecord> {
  move |value| match value {
    Value::String(text) if !text.is_empty() => Some(text_message(role, text)),
    Value::Array(_) | Value::Object(_) => {
      let parts = parts_from_value(value);
      (!parts.is_empty()).then(|| MessageRecord {
        role: role.to_string(),
        status: None,
        parts,
      })
    }
    _ => None,
  }
}

fn message_from_value(value: &Value) -> Option<MessageRecord> {
  message_from_value_with_default_role(value, "user")
}

fn message_from_value_with_default_role(value: &Value, default_role: &str) -> Option<MessageRecord> {
  let obj = value.as_object()?;
  let role = obj
    .get("role")
    .and_then(Value::as_str)
    .or_else(|| obj.get("type").and_then(Value::as_str))
    .unwrap_or(default_role);
  let parts = obj
    .get("content")
    .map(parts_from_value)
    .filter(|parts| !parts.is_empty())
    .unwrap_or_else(|| vec![json_part(value)]);
  Some(MessageRecord {
    role: role.to_string(),
    status: None,
    parts,
  })
}

fn parts_from_value(value: &Value) -> Vec<PartRecord> {
  match value {
    Value::String(text) => vec![PartRecord {
      part_type: "text".to_string(),
      content: Bytes::from(text.to_string()),
    }],
    Value::Array(parts) => parts.iter().map(part_from_value).collect(),
    Value::Object(_) => vec![part_from_value(value)],
    _ => Vec::new(),
  }
}

fn part_from_value(value: &Value) -> PartRecord {
  if let Some(text) = value
    .get("text")
    .and_then(Value::as_str)
    .or_else(|| value.get("input_text").and_then(Value::as_str))
    .or_else(|| value.get("output_text").and_then(Value::as_str))
  {
    return PartRecord {
      part_type: "text".to_string(),
      content: Bytes::from(text.to_string()),
    };
  }
  json_part(value)
}

fn json_part(value: &Value) -> PartRecord {
  let part_type = value.get("type").and_then(Value::as_str).unwrap_or("json").to_string();
  PartRecord {
    part_type,
    content: Bytes::from(serde_json::to_vec(value).unwrap_or_default()),
  }
}

fn text_message(role: &str, text: &str) -> MessageRecord {
  MessageRecord {
    role: role.to_string(),
    status: None,
    parts: vec![PartRecord {
      part_type: "text".to_string(),
      content: Bytes::from(text.to_string()),
    }],
  }
}

pub(super) fn response_messages_from_body(body: &[u8]) -> Vec<MessageRecord> {
  if body.is_empty() {
    return Vec::new();
  }
  if let Ok(value) = serde_json::from_slice::<Value>(body) {
    return response_messages_from_json(&value);
  }
  let Ok(text) = std::str::from_utf8(body) else {
    return Vec::new();
  };
  response_messages_from_sse(text)
}

fn response_messages_from_sse(text: &str) -> Vec<MessageRecord> {
  let mut completed = None;
  let mut deltas = String::new();
  let mut structured_deltas = Vec::new();
  for event in text.split("\n\n") {
    let (event_name, data) = parse_sse_event(event);
    if data.is_empty() || data == "[DONE]" {
      continue;
    }
    let Ok(value) = serde_json::from_str::<Value>(&data) else {
      continue;
    };
    if event_name == "response.completed" {
      completed = value.get("response").cloned().or(Some(value));
      continue;
    }
    collect_text_delta(&value, &mut deltas, &mut structured_deltas);
  }
  if let Some(value) = completed {
    let messages = response_messages_from_json(&value);
    if !messages.is_empty() {
      return messages;
    }
  }
  message_from_deltas(deltas, structured_deltas)
}

fn parse_sse_event(event: &str) -> (&str, String) {
  let mut event_name = "";
  let mut data = String::new();
  for line in event.lines() {
    if let Some(value) = line.strip_prefix("event:") {
      event_name = value.trim();
    } else if let Some(value) = line.strip_prefix("data:") {
      if !data.is_empty() {
        data.push('\n');
      }
      data.push_str(value.trim());
    }
  }
  (event_name, data)
}

fn collect_text_delta(value: &Value, text: &mut String, structured: &mut Vec<PartRecord>) {
  if let Some(delta) = value.get("delta") {
    if let Some(delta) = delta.as_str() {
      text.push_str(delta);
    } else if let Some(delta) = delta.as_object() {
      if let Some(value) = delta.get("text").and_then(Value::as_str) {
        text.push_str(value);
      }
      if let Some(value) = delta.get("partial_json").and_then(Value::as_str) {
        structured.push(PartRecord {
          part_type: "input_json_delta".to_string(),
          content: Bytes::from(value.to_string()),
        });
      }
    }
  }
  if let Some(content_block) = value.get("content_block") {
    if content_block.get("type").and_then(Value::as_str) != Some("text") {
      structured.push(json_part(content_block));
    }
  }
  if let Some(choices) = value.get("choices").and_then(Value::as_array) {
    for delta in choices.iter().filter_map(|choice| choice.get("delta")) {
      if let Some(content) = delta.get("content").and_then(Value::as_str) {
        text.push_str(content);
      }
      if delta.get("tool_calls").is_some() || delta.get("function_call").is_some() {
        structured.push(json_part(delta));
      }
    }
  }
}

fn message_from_deltas(text: String, structured: Vec<PartRecord>) -> Vec<MessageRecord> {
  if text.is_empty() && structured.is_empty() {
    return Vec::new();
  }
  let mut parts = Vec::new();
  if !text.is_empty() {
    parts.push(PartRecord {
      part_type: "text".to_string(),
      content: Bytes::from(text),
    });
  }
  parts.extend(structured);
  vec![MessageRecord {
    role: "assistant".to_string(),
    status: None,
    parts,
  }]
}

fn response_messages_from_json(value: &Value) -> Vec<MessageRecord> {
  if let Some(output) = value.get("output").and_then(Value::as_array) {
    let messages: Vec<_> = output
      .iter()
      .filter_map(|value| message_from_value_with_default_role(value, "assistant"))
      .collect();
    if !messages.is_empty() {
      return messages;
    }
  }
  if let Some(text) = value.get("output_text").and_then(Value::as_str) {
    return vec![text_message("assistant", text)];
  }
  if let Some(choices) = value.get("choices").and_then(Value::as_array) {
    let messages: Vec<_> = choices
      .iter()
      .filter_map(|choice| choice.get("message").or_else(|| choice.get("delta")))
      .filter_map(|value| message_from_value_with_default_role(value, "assistant"))
      .collect();
    if !messages.is_empty() {
      return messages;
    }
  }
  if value.get("content").is_some() {
    if let Some(message) = message_from_value_with_default_role(value, "assistant") {
      return vec![message];
    }
  }
  Vec::new()
}

#[cfg(test)]
mod tests {
  use super::*;
  use serde_json::json;

  #[test]
  fn parses_requests_for_all_supported_endpoints() {
    let chat = request_messages_from_json(
      "chat_completions",
      &json!({"messages": [{"role": "user", "content": "chat"}]}),
    );
    let responses = request_messages_from_json("responses", &json!({"instructions": "system", "input": "response"}));
    let messages = request_messages_from_json(
      "messages",
      &json!({
        "system": [{"type": "text", "text": "system"}],
        "messages": [{"role": "user", "content": [{"type": "text", "text": "message"}]}]
      }),
    );

    assert_message(&chat[0], "user", "chat");
    assert_message(&responses[0], "system", "system");
    assert_message(&responses[1], "user", "response");
    assert_message(&messages[0], "system", "system");
    assert_message(&messages[1], "user", "message");
  }

  #[test]
  fn parses_buffered_chat_and_messages_responses() {
    let chat = response_messages_from_body(br#"{"choices":[{"message":{"content":"chat reply"}}]}"#);
    let messages = response_messages_from_body(
      br#"{"type":"message","role":"assistant","content":[{"type":"text","text":"message reply"}]}"#,
    );

    assert_message(&chat[0], "assistant", "chat reply");
    assert_message(&messages[0], "assistant", "message reply");
  }

  #[test]
  fn parses_streamed_deltas_for_all_supported_endpoints() {
    let responses =
      response_messages_from_body(b"event: response.output_text.delta\ndata: {\"delta\":\"response\"}\n\n");
    let chat =
      response_messages_from_body(b"data: {\"choices\":[{\"delta\":{\"content\":\"chat\"}}]}\n\ndata: [DONE]\n\n");
    let messages = response_messages_from_body(
      b"event: content_block_delta\ndata: {\"delta\":{\"type\":\"text_delta\",\"text\":\"message\"}}\n\n",
    );

    assert_message(&responses[0], "assistant", "response");
    assert_message(&chat[0], "assistant", "chat");
    assert_message(&messages[0], "assistant", "message");
  }

  fn assert_message(message: &MessageRecord, role: &str, text: &str) {
    assert_eq!(message.role, role);
    assert_eq!(message.parts.len(), 1);
    assert_eq!(message.parts[0].part_type, "text");
    assert_eq!(message.parts[0].content.as_ref(), text.as_bytes());
  }
}
