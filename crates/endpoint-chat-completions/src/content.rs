use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_json::{Map, Value};

use tokn_endpoint_core::Extras;

/// Chat content can be either a plain string or a list of typed parts.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ChatContent {
  Text(String),
  Parts(Vec<ContentPart>),
}

impl Default for ChatContent {
  fn default() -> Self {
    Self::Text(String::new())
  }
}

/// One element of a structured chat message content array.
#[derive(Clone, Debug)]
pub enum ContentPart {
  Text { text: String, extras: Extras },
  ImageUrl { image_url: Value, extras: Extras },
  InputAudio { input_audio: Value, extras: Extras },
  Other(Value),
}

impl Serialize for ContentPart {
  fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
    match self {
      Self::Text { text, extras } => {
        let mut obj = object_with_type("text", extras);
        obj.insert("text".into(), Value::String(text.clone()));
        Value::Object(obj).serialize(serializer)
      }
      Self::ImageUrl { image_url, extras } => {
        let mut obj = object_with_type("image_url", extras);
        obj.insert("image_url".into(), image_url.clone());
        Value::Object(obj).serialize(serializer)
      }
      Self::InputAudio { input_audio, extras } => {
        let mut obj = object_with_type("input_audio", extras);
        obj.insert("input_audio".into(), input_audio.clone());
        Value::Object(obj).serialize(serializer)
      }
      Self::Other(value) => value.serialize(serializer),
    }
  }
}

impl<'de> Deserialize<'de> for ContentPart {
  fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
    let value = Value::deserialize(deserializer)?;
    let Some(kind) = value.get("type").and_then(Value::as_str) else {
      return Ok(Self::Other(value));
    };
    match kind {
      "text" => {
        let mut obj = into_object(value);
        obj.remove("type");
        let text = take_required_string::<D>(&mut obj, "text")?;
        Ok(Self::Text { text, extras: obj })
      }
      "image_url" => {
        let mut obj = into_object(value);
        obj.remove("type");
        let image_url = obj.remove("image_url").unwrap_or(Value::Null);
        Ok(Self::ImageUrl { image_url, extras: obj })
      }
      "input_audio" => {
        let mut obj = into_object(value);
        obj.remove("type");
        let input_audio = obj.remove("input_audio").unwrap_or(Value::Null);
        Ok(Self::InputAudio {
          input_audio,
          extras: obj,
        })
      }
      _ => Ok(Self::Other(value)),
    }
  }
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
