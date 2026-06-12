use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_json::{Map, Value};

use tokn_endpoint_core::Extras;

/// Content part for an input message.
#[derive(Clone, Debug)]
pub enum InputContentPart {
  InputText { text: String, extras: Extras },
  InputImage { fields: Extras },
  InputAudio { fields: Extras },
  InputFile { fields: Extras },
  Other(Value),
}

/// Content part attached to an assistant `message` output item.
#[derive(Clone, Debug)]
pub enum OutputContentPart {
  OutputText {
    text: String,
    annotations: Vec<Value>,
    logprobs: Option<Value>,
    extras: Extras,
  },
  Refusal {
    refusal: String,
    extras: Extras,
  },
  Other(Value),
}

/// One element of a `reasoning` item's `content` or `summary` arrays.
#[derive(Clone, Debug)]
pub enum ReasoningPart {
  ReasoningText { text: String, extras: Extras },
  SummaryText { text: String, extras: Extras },
  Text { text: String, extras: Extras },
  Other(Value),
}

impl Serialize for InputContentPart {
  fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
    match self {
      Self::InputText { text, extras } => {
        let mut obj = object_with_type("input_text", extras);
        obj.insert("text".into(), Value::String(text.clone()));
        Value::Object(obj).serialize(serializer)
      }
      Self::InputImage { fields } => Value::Object(object_with_type("input_image", fields)).serialize(serializer),
      Self::InputAudio { fields } => Value::Object(object_with_type("input_audio", fields)).serialize(serializer),
      Self::InputFile { fields } => Value::Object(object_with_type("input_file", fields)).serialize(serializer),
      Self::Other(value) => value.serialize(serializer),
    }
  }
}

impl<'de> Deserialize<'de> for InputContentPart {
  fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
    let value = Value::deserialize(deserializer)?;
    let Some(kind) = value.get("type").and_then(Value::as_str) else {
      return Ok(Self::Other(value));
    };
    let mut obj = into_object(value.clone());
    obj.remove("type");
    match kind {
      "input_text" => {
        let text = take_required_string::<D>(&mut obj, "text")?;
        Ok(Self::InputText { text, extras: obj })
      }
      "input_image" => Ok(Self::InputImage { fields: obj }),
      "input_audio" => Ok(Self::InputAudio { fields: obj }),
      "input_file" => Ok(Self::InputFile { fields: obj }),
      _ => Ok(Self::Other(value)),
    }
  }
}

impl Serialize for OutputContentPart {
  fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
    match self {
      Self::OutputText {
        text,
        annotations,
        logprobs,
        extras,
      } => {
        let mut obj = object_with_type("output_text", extras);
        obj.insert("text".into(), Value::String(text.clone()));
        if !annotations.is_empty() {
          obj.insert("annotations".into(), Value::Array(annotations.clone()));
        }
        if let Some(logprobs) = logprobs {
          obj.insert("logprobs".into(), logprobs.clone());
        }
        Value::Object(obj).serialize(serializer)
      }
      Self::Refusal { refusal, extras } => {
        let mut obj = object_with_type("refusal", extras);
        obj.insert("refusal".into(), Value::String(refusal.clone()));
        Value::Object(obj).serialize(serializer)
      }
      Self::Other(value) => value.serialize(serializer),
    }
  }
}

impl<'de> Deserialize<'de> for OutputContentPart {
  fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
    let value = Value::deserialize(deserializer)?;
    let Some(kind) = value.get("type").and_then(Value::as_str) else {
      return Ok(Self::Other(value));
    };
    let mut obj = into_object(value.clone());
    obj.remove("type");
    match kind {
      "output_text" => {
        let text = take_required_string::<D>(&mut obj, "text")?;
        let annotations = take_array(&mut obj, "annotations");
        let logprobs = obj.remove("logprobs");
        Ok(Self::OutputText {
          text,
          annotations,
          logprobs,
          extras: obj,
        })
      }
      "refusal" => {
        let refusal = take_required_string::<D>(&mut obj, "refusal")?;
        Ok(Self::Refusal { refusal, extras: obj })
      }
      _ => Ok(Self::Other(value)),
    }
  }
}

impl Serialize for ReasoningPart {
  fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
    match self {
      Self::ReasoningText { text, extras } => serialize_text_part(serializer, "reasoning_text", text, extras),
      Self::SummaryText { text, extras } => serialize_text_part(serializer, "summary_text", text, extras),
      Self::Text { text, extras } => serialize_text_part(serializer, "text", text, extras),
      Self::Other(value) => value.serialize(serializer),
    }
  }
}

impl<'de> Deserialize<'de> for ReasoningPart {
  fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
    let value = Value::deserialize(deserializer)?;
    let Some(kind) = value.get("type").and_then(Value::as_str) else {
      return Ok(Self::Other(value));
    };
    let mut obj = into_object(value.clone());
    obj.remove("type");
    match kind {
      "reasoning_text" => {
        let text = take_required_string::<D>(&mut obj, "text")?;
        Ok(Self::ReasoningText { text, extras: obj })
      }
      "summary_text" => {
        let text = take_required_string::<D>(&mut obj, "text")?;
        Ok(Self::SummaryText { text, extras: obj })
      }
      "text" => {
        let text = take_required_string::<D>(&mut obj, "text")?;
        Ok(Self::Text { text, extras: obj })
      }
      _ => Ok(Self::Other(value)),
    }
  }
}

fn serialize_text_part<S: Serializer>(
  serializer: S,
  kind: &str,
  text: &str,
  extras: &Extras,
) -> Result<S::Ok, S::Error> {
  let mut obj = object_with_type(kind, extras);
  obj.insert("text".into(), Value::String(text.into()));
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

fn take_required_string<'de, D: Deserializer<'de>>(
  obj: &mut Map<String, Value>,
  key: &'static str,
) -> Result<String, D::Error> {
  obj
    .remove(key)
    .and_then(|value| value.as_str().map(str::to_string))
    .ok_or_else(|| serde::de::Error::missing_field(key))
}

fn take_array(obj: &mut Map<String, Value>, key: &str) -> Vec<Value> {
  match obj.remove(key) {
    Some(Value::Array(values)) => values,
    Some(value) => vec![value],
    None => Vec::new(),
  }
}
