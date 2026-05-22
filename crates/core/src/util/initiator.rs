use serde_json::Value;

/// Best-effort initiator classifier for chat-style payloads.
///
/// Contract:
/// - `Some("agent")`: the payload carries concrete evidence that this is
///   part of an agent/tool loop.
/// - `None`: we do not assert `"agent"`. This covers both clear user turns
///   and structurally unknown payloads.
///
/// This helper intentionally never returns `"user"`. Callers that need an
/// outbound header can fall back to `"user"` at the edge, while persistence
/// can keep the absence of a concrete signal as `NULL`/missing.
pub fn classify_initiator(body: &Value) -> Option<&'static str> {
  let msgs = body.get("messages").and_then(|v| v.as_array())?;
  for m in msgs.iter().rev() {
    match m.get("role").and_then(|r| r.as_str()) {
      Some("system") => continue,
      Some("tool") => return Some("agent"),
      Some("assistant") => return Some("agent"),
      Some("user") => return None,
      _ => return None,
    }
  }
  None
}

/// Responses-API variant of [`classify_initiator`].
///
/// Shares the same contract:
/// - `Some("agent")`: concrete tool/assistant continuation evidence.
/// - `None`: no agent assertion, either because the payload is clearly a
///   user turn or because the shape is unknown/insufficiently structured.
pub fn classify_initiator_responses(body: &Value) -> Option<&'static str> {
  let input = body.get("input")?;
  if input.is_string() {
    return None;
  }
  let items = input.as_array()?;
  for it in items.iter().rev() {
    let typ = it.get("type").and_then(|t| t.as_str());
    if let Some(t) = typ {
      match t {
        "function_call_output" | "tool_result" | "computer_call_output" => return Some("agent"),
        "function_call" | "tool_call" | "reasoning" => return Some("agent"),
        "message" => {}
        _ => return None,
      }
    }
    match it.get("role").and_then(|r| r.as_str()) {
      Some("system") | Some("developer") => continue,
      Some("tool") => return Some("agent"),
      Some("assistant") => return Some("agent"),
      Some("user") => return None,
      _ => return None,
    }
  }
  None
}

#[cfg(test)]
mod responses_tests {
  use super::*;
  use serde_json::json;

  #[test]
  fn bare_string_input_is_user() {
    let b = json!({ "input": "hello" });
    assert_eq!(classify_initiator_responses(&b), None);
  }

  #[test]
  fn missing_input_defaults_to_user() {
    assert_eq!(classify_initiator_responses(&json!({})), None);
  }

  #[test]
  fn user_message_array_is_user() {
    let b = json!({ "input": [
        { "role": "system", "content": "x" },
        { "role": "user", "content": "hi" }
    ]});
    assert_eq!(classify_initiator_responses(&b), None);
  }

  #[test]
  fn tool_followup_is_agent() {
    let b = json!({ "input": [
        { "role": "user", "content": "x" },
        { "type": "function_call", "name": "f" },
        { "type": "function_call_output", "output": "42" }
    ]});
    assert_eq!(classify_initiator_responses(&b), Some("agent"));
  }

  #[test]
  fn non_array_input_is_unknown() {
    let b = json!({ "input": {"role": "user", "content": "x"} });
    assert_eq!(classify_initiator_responses(&b), None);
  }

  #[test]
  fn unknown_response_type_is_unknown() {
    let b = json!({ "input": [{ "type": "mystery" }] });
    assert_eq!(classify_initiator_responses(&b), None);
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use serde_json::json;

  #[test]
  fn user_turn() {
    let b = json!({"messages":[
        {"role":"system","content":"x"},
        {"role":"user","content":"hi"}
    ]});
    assert_eq!(classify_initiator(&b), None);
  }

  #[test]
  fn tool_followup_is_agent() {
    let b = json!({"messages":[
        {"role":"user","content":"do x"},
        {"role":"assistant","tool_calls":[{"id":"1"}]},
        {"role":"tool","tool_call_id":"1","content":"42"}
    ]});
    assert_eq!(classify_initiator(&b), Some("agent"));
  }

  #[test]
  fn after_assistant_is_agent() {
    let b = json!({"messages":[
        {"role":"user","content":"hi"},
        {"role":"assistant","content":"ok"}
    ]});
    assert_eq!(classify_initiator(&b), Some("agent"));
  }

  #[test]
  fn empty_defaults_to_user() {
    let b = json!({});
    assert_eq!(classify_initiator(&b), None);
  }

  #[test]
  fn unknown_chat_role_is_unknown() {
    let b = json!({"messages":[{"role":"mystery","content":"hi"}]});
    assert_eq!(classify_initiator(&b), None);
  }
}
