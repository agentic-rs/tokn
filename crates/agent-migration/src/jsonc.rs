use anyhow::{Context, Result};
use serde_json::Value;
use std::path::Path;

pub(crate) fn read_json_or_jsonc(path: &Path) -> Result<Value> {
  let raw = std::fs::read_to_string(path).with_context(|| format!("reading {}", path.display()))?;
  parse_json_or_jsonc(&raw, path).with_context(|| format!("parsing {}", path.display()))
}

pub(crate) fn parse_json_or_jsonc(raw: &str, path: &Path) -> Result<Value> {
  if path.extension().and_then(|extension| extension.to_str()) != Some("jsonc") {
    return Ok(serde_json::from_str(raw)?);
  }
  let parsed = serde_json::from_str(raw).or_else(|_| {
    let without_comments = strip_jsonc_comments(raw);
    let without_trailing_commas = strip_jsonc_trailing_commas(&without_comments);
    serde_json::from_str(&without_trailing_commas)
  })?;
  Ok(parsed)
}

fn strip_jsonc_comments(raw: &str) -> String {
  let mut out = String::with_capacity(raw.len());
  let mut chars = raw.chars().peekable();
  let mut in_string = false;
  let mut escaped = false;

  while let Some(ch) = chars.next() {
    if in_string {
      out.push(ch);
      if escaped {
        escaped = false;
      } else if ch == '\\' {
        escaped = true;
      } else if ch == '"' {
        in_string = false;
      }
      continue;
    }

    match ch {
      '"' => {
        in_string = true;
        out.push(ch);
      }
      '/' if chars.peek() == Some(&'/') => {
        chars.next();
        for next in chars.by_ref() {
          if next == '\n' {
            out.push('\n');
            break;
          }
        }
      }
      '/' if chars.peek() == Some(&'*') => {
        chars.next();
        let mut previous = '\0';
        for next in chars.by_ref() {
          if next == '\n' {
            out.push('\n');
          }
          if previous == '*' && next == '/' {
            break;
          }
          previous = next;
        }
      }
      _ => out.push(ch),
    }
  }

  out
}

fn strip_jsonc_trailing_commas(raw: &str) -> String {
  let mut out = String::with_capacity(raw.len());
  let chars = raw.chars().collect::<Vec<_>>();
  let mut index = 0;
  let mut in_string = false;
  let mut escaped = false;

  while index < chars.len() {
    let ch = chars[index];
    if in_string {
      out.push(ch);
      if escaped {
        escaped = false;
      } else if ch == '\\' {
        escaped = true;
      } else if ch == '"' {
        in_string = false;
      }
      index += 1;
      continue;
    }

    if ch == '"' {
      in_string = true;
      out.push(ch);
      index += 1;
      continue;
    }

    if ch == ',' {
      let next = chars[index + 1..].iter().find(|next| !next.is_whitespace());
      if matches!(next, Some('}' | ']')) {
        index += 1;
        continue;
      }
    }

    out.push(ch);
    index += 1;
  }

  out
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn parses_jsonc_comments_and_trailing_commas() {
    let raw = r#"
{
  // line comment
  "url": "https://example.com//not-a-comment",
  /*
   * block comment
   */
  "items": [
    "a",
  ],
}
"#;

    let parsed = parse_json_or_jsonc(raw, Path::new("opencode.jsonc")).unwrap();

    assert_eq!(parsed["url"], "https://example.com//not-a-comment");
    assert_eq!(parsed["items"][0], "a");
  }

  #[test]
  fn json_path_rejects_jsonc_syntax() {
    let err = parse_json_or_jsonc("{// comment\n}", Path::new("opencode.json")).unwrap_err();

    assert!(err.downcast_ref::<serde_json::Error>().is_some());
  }
}
