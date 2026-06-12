use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_json::{Map, Value};

use tokn_endpoint_core::Extras;

/// One block in a Messages API content array.
#[derive(Clone, Debug)]
pub enum ContentBlock {
  Text {
    text: String,
    /// Anthropic prompt-caching directive (e.g. `{"type":"ephemeral"}`).
    cache_control: Option<Value>,
    extras: Extras,
  },
  Thinking {
    thinking: String,
    signature: Option<String>,
    cache_control: Option<Value>,
    extras: Extras,
  },
  RedactedThinking {
    fields: Extras,
  },
  ToolUse {
    id: String,
    name: String,
    input: Value,
    cache_control: Option<Value>,
    extras: Extras,
  },
  ToolResult {
    tool_use_id: String,
    content: Value,
    is_error: Option<bool>,
    cache_control: Option<Value>,
    extras: Extras,
  },
  Image {
    source: Value,
    cache_control: Option<Value>,
    extras: Extras,
  },
  Document {
    source: Value,
    cache_control: Option<Value>,
    extras: Extras,
  },
  Other(Value),
}

/// Delta variants emitted inside `content_block_delta` events.
#[derive(Clone, Debug)]
pub enum ContentBlockDelta {
  TextDelta { text: String, extras: Extras },
  ThinkingDelta { thinking: String, extras: Extras },
  SignatureDelta { signature: String, extras: Extras },
  InputJsonDelta { partial_json: String, extras: Extras },
  Other(Value),
}

impl Serialize for ContentBlock {
  fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
    match self {
      Self::Text {
        text,
        cache_control,
        extras,
      } => {
        let mut obj = object_with_type("text", extras);
        obj.insert("text".into(), Value::String(text.clone()));
        insert_opt(&mut obj, "cache_control", cache_control);
        Value::Object(obj).serialize(serializer)
      }
      Self::Thinking {
        thinking,
        signature,
        cache_control,
        extras,
      } => {
        let mut obj = object_with_type("thinking", extras);
        obj.insert("thinking".into(), Value::String(thinking.clone()));
        insert_opt_string(&mut obj, "signature", signature);
        insert_opt(&mut obj, "cache_control", cache_control);
        Value::Object(obj).serialize(serializer)
      }
      Self::RedactedThinking { fields } => {
        Value::Object(object_with_type("redacted_thinking", fields)).serialize(serializer)
      }
      Self::ToolUse {
        id,
        name,
        input,
        cache_control,
        extras,
      } => {
        let mut obj = object_with_type("tool_use", extras);
        obj.insert("id".into(), Value::String(id.clone()));
        obj.insert("name".into(), Value::String(name.clone()));
        obj.insert("input".into(), input.clone());
        insert_opt(&mut obj, "cache_control", cache_control);
        Value::Object(obj).serialize(serializer)
      }
      Self::ToolResult {
        tool_use_id,
        content,
        is_error,
        cache_control,
        extras,
      } => {
        let mut obj = object_with_type("tool_result", extras);
        obj.insert("tool_use_id".into(), Value::String(tool_use_id.clone()));
        obj.insert("content".into(), content.clone());
        if let Some(is_error) = is_error {
          obj.insert("is_error".into(), Value::Bool(*is_error));
        }
        insert_opt(&mut obj, "cache_control", cache_control);
        Value::Object(obj).serialize(serializer)
      }
      Self::Image {
        source,
        cache_control,
        extras,
      } => {
        let mut obj = object_with_type("image", extras);
        obj.insert("source".into(), source.clone());
        insert_opt(&mut obj, "cache_control", cache_control);
        Value::Object(obj).serialize(serializer)
      }
      Self::Document {
        source,
        cache_control,
        extras,
      } => {
        let mut obj = object_with_type("document", extras);
        obj.insert("source".into(), source.clone());
        insert_opt(&mut obj, "cache_control", cache_control);
        Value::Object(obj).serialize(serializer)
      }
      Self::Other(value) => value.serialize(serializer),
    }
  }
}

impl<'de> Deserialize<'de> for ContentBlock {
  fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
    let value = Value::deserialize(deserializer)?;
    let Some(kind) = value.get("type").and_then(Value::as_str) else {
      return Ok(Self::Other(value));
    };
    let mut obj = into_object(value.clone());
    obj.remove("type");
    match kind {
      "text" => {
        let text = take_required_string::<D>(&mut obj, "text")?;
        let cache_control = obj.remove("cache_control");
        Ok(Self::Text {
          text,
          cache_control,
          extras: obj,
        })
      }
      "thinking" => {
        let thinking = take_required_string::<D>(&mut obj, "thinking")?;
        let signature = take_optional_string(&mut obj, "signature");
        let cache_control = obj.remove("cache_control");
        Ok(Self::Thinking {
          thinking,
          signature,
          cache_control,
          extras: obj,
        })
      }
      "redacted_thinking" => Ok(Self::RedactedThinking { fields: obj }),
      "tool_use" => {
        let id = take_required_string::<D>(&mut obj, "id")?;
        let name = take_required_string::<D>(&mut obj, "name")?;
        let input = obj.remove("input").unwrap_or(Value::Null);
        let cache_control = obj.remove("cache_control");
        Ok(Self::ToolUse {
          id,
          name,
          input,
          cache_control,
          extras: obj,
        })
      }
      "tool_result" => {
        let tool_use_id = take_required_string::<D>(&mut obj, "tool_use_id")?;
        let content = obj.remove("content").unwrap_or(Value::Null);
        let is_error = obj.remove("is_error").and_then(|value| value.as_bool());
        let cache_control = obj.remove("cache_control");
        Ok(Self::ToolResult {
          tool_use_id,
          content,
          is_error,
          cache_control,
          extras: obj,
        })
      }
      "image" => {
        let source = obj.remove("source").unwrap_or(Value::Null);
        let cache_control = obj.remove("cache_control");
        Ok(Self::Image {
          source,
          cache_control,
          extras: obj,
        })
      }
      "document" => {
        let source = obj.remove("source").unwrap_or(Value::Null);
        let cache_control = obj.remove("cache_control");
        Ok(Self::Document {
          source,
          cache_control,
          extras: obj,
        })
      }
      _ => Ok(Self::Other(value)),
    }
  }
}

impl Serialize for ContentBlockDelta {
  fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
    match self {
      Self::TextDelta { text, extras } => serialize_string_delta(serializer, "text_delta", "text", text, extras),
      Self::ThinkingDelta { thinking, extras } => {
        serialize_string_delta(serializer, "thinking_delta", "thinking", thinking, extras)
      }
      Self::SignatureDelta { signature, extras } => {
        serialize_string_delta(serializer, "signature_delta", "signature", signature, extras)
      }
      Self::InputJsonDelta { partial_json, extras } => {
        serialize_string_delta(serializer, "input_json_delta", "partial_json", partial_json, extras)
      }
      Self::Other(value) => value.serialize(serializer),
    }
  }
}

impl<'de> Deserialize<'de> for ContentBlockDelta {
  fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
    let value = Value::deserialize(deserializer)?;
    let Some(kind) = value.get("type").and_then(Value::as_str) else {
      return Ok(Self::Other(value));
    };
    let mut obj = into_object(value.clone());
    obj.remove("type");
    match kind {
      "text_delta" => {
        let text = take_required_string::<D>(&mut obj, "text")?;
        Ok(Self::TextDelta { text, extras: obj })
      }
      "thinking_delta" => {
        let thinking = take_required_string::<D>(&mut obj, "thinking")?;
        Ok(Self::ThinkingDelta { thinking, extras: obj })
      }
      "signature_delta" => {
        let signature = take_required_string::<D>(&mut obj, "signature")?;
        Ok(Self::SignatureDelta { signature, extras: obj })
      }
      "input_json_delta" => {
        let partial_json = take_required_string::<D>(&mut obj, "partial_json")?;
        Ok(Self::InputJsonDelta {
          partial_json,
          extras: obj,
        })
      }
      _ => Ok(Self::Other(value)),
    }
  }
}

fn serialize_string_delta<S: Serializer>(
  serializer: S,
  kind: &str,
  field: &str,
  value: &str,
  extras: &Extras,
) -> Result<S::Ok, S::Error> {
  let mut obj = object_with_type(kind, extras);
  obj.insert(field.into(), Value::String(value.into()));
  Value::Object(obj).serialize(serializer)
}

fn object_with_type(kind: &str, extras: &Extras) -> Map<String, Value> {
  let mut obj = extras.clone();
  obj.insert("type".into(), Value::String(kind.into()));
  obj
}

fn into_object(value: Value) -> Map<String, Value> {
  match value {
    Value::Object(obj) => obj,
    other => {
      let mut obj = Map::new();
      obj.insert("value".into(), other);
      obj
    }
  }
}

fn insert_opt(obj: &mut Map<String, Value>, key: &str, value: &Option<Value>) {
  if let Some(value) = value {
    obj.insert(key.into(), value.clone());
  }
}

fn insert_opt_string(obj: &mut Map<String, Value>, key: &str, value: &Option<String>) {
  if let Some(value) = value {
    obj.insert(key.into(), Value::String(value.clone()));
  }
}

fn take_required_string<'de, D: Deserializer<'de>>(
  obj: &mut Map<String, Value>,
  key: &'static str,
) -> Result<String, D::Error> {
  obj
    .remove(key)
    .and_then(|value| value.as_str().map(str::to_string))
    .ok_or_else(|| serde::de::Error::missing_field(key))
}

fn take_optional_string(obj: &mut Map<String, Value>, key: &str) -> Option<String> {
  obj.remove(key).and_then(|value| value.as_str().map(str::to_string))
}
