//! State machine that emits a faithful Responses-API SSE lifecycle from a
//! generic stream of `IrDelta`s.
//!
//! Output items are emitted strictly sequentially: each item is fully closed
//! (its `*.done` events) before the next item's `output_item.added` is sent.

use super::event::SseEvent;
use crate::ir::IrDelta;
use serde_json::{json, Value};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug)]
enum OpenItem {
  Message {
    output_index: usize,
    item_id: String,
    text: String,
    part_open: bool,
  },
  FunctionCall {
    output_index: usize,
    chat_index: usize,
    item_id: String,
    call_id: String,
    name: String,
    arguments: String,
  },
  Reasoning {
    output_index: usize,
    item_id: String,
    summary: String,
    part_open: bool,
  },
}

impl OpenItem {
  fn output_index(&self) -> usize {
    match self {
      OpenItem::Message { output_index, .. }
      | OpenItem::FunctionCall { output_index, .. }
      | OpenItem::Reasoning { output_index, .. } => *output_index,
    }
  }

  fn item_id(&self) -> &str {
    match self {
      OpenItem::Message { item_id, .. }
      | OpenItem::FunctionCall { item_id, .. }
      | OpenItem::Reasoning { item_id, .. } => item_id,
    }
  }
}

pub struct ResponsesEmitter {
  id: String,
  model: String,
  created_at: u64,
  sequence: u64,
  next_output_index: usize,
  next_item_counter: usize,
  current: Option<OpenItem>,
  closed: Vec<Value>,
  created_emitted: bool,
  in_progress_emitted: bool,
  finished: bool,
  usage: Option<Value>,
  finish_reason: Option<String>,
}

impl ResponsesEmitter {
  pub fn new(id: String, model: String) -> Self {
    let created_at = SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0);
    Self {
      id,
      model,
      created_at,
      sequence: 0,
      next_output_index: 0,
      next_item_counter: 0,
      current: None,
      closed: Vec::new(),
      created_emitted: false,
      in_progress_emitted: false,
      finished: false,
      usage: None,
      finish_reason: None,
    }
  }

  pub fn update_id(&mut self, id: String) {
    self.id = id;
  }
  pub fn update_model(&mut self, model: String) {
    self.model = model;
  }

  fn next_seq(&mut self) -> u64 {
    let n = self.sequence;
    self.sequence += 1;
    n
  }

  fn synth_item_id(&mut self, prefix: &str) -> String {
    self.next_item_counter += 1;
    format!("{prefix}_{}", self.next_item_counter)
  }

  fn ensure_started(&mut self, out: &mut Vec<SseEvent>) {
    if !self.created_emitted {
      let seq = self.next_seq();
      out.push(SseEvent::json(
        Some("response.created"),
        json!({
          "type": "response.created",
          "sequence_number": seq,
          "response": self.response_snapshot("in_progress", false),
        }),
      ));
      self.created_emitted = true;
    }
    if !self.in_progress_emitted {
      let seq = self.next_seq();
      out.push(SseEvent::json(
        Some("response.in_progress"),
        json!({
          "type": "response.in_progress",
          "sequence_number": seq,
          "response": self.response_snapshot("in_progress", false),
        }),
      ));
      self.in_progress_emitted = true;
    }
  }

  fn response_snapshot(&self, status: &str, completed: bool) -> Value {
    let mut response = json!({
      "id": self.id,
      "object": "response",
      "created_at": self.created_at,
      "status": status,
      "model": self.model,
      "output": if completed { Value::Array(self.closed.clone()) } else { Value::Array(Vec::new()) },
      "usage": self.usage.clone().unwrap_or(Value::Null),
    });
    if completed {
      if let Some(obj) = response.as_object_mut() {
        obj.insert("completed_at".into(), json!(self.created_at));
      }
    }
    response
  }

  pub fn emit(&mut self, deltas: &[IrDelta]) -> Vec<SseEvent> {
    if deltas.is_empty() {
      return Vec::new();
    }
    let mut out = Vec::new();
    self.ensure_started(&mut out);
    for delta in deltas {
      match delta {
        IrDelta::Text(text) => self.handle_text(text, &mut out),
        IrDelta::Reasoning(text) => self.handle_reasoning(text, &mut out),
        IrDelta::ToolCall { index, id, name, arguments_delta } => {
          self.handle_tool_call(*index, id.clone(), name.clone(), arguments_delta, &mut out);
        }
        IrDelta::Usage(usage) => {
          self.usage = Some(json!({
            "input_tokens": usage.input_tokens,
            "output_tokens": usage.output_tokens,
            "total_tokens": usage.total_tokens,
          }));
        }
        IrDelta::Finish(reason) => {
          self.finish_reason = reason.clone();
        }
      }
    }
    out
  }

  fn close_current(&mut self, out: &mut Vec<SseEvent>) {
    if let Some(item) = self.current.take() {
      self.close_item(item, out);
    }
  }

  fn handle_text(&mut self, text: &str, out: &mut Vec<SseEvent>) {
    if !matches!(self.current, Some(OpenItem::Message { .. })) {
      self.close_current(out);
      let output_index = self.next_output_index;
      self.next_output_index += 1;
      let item_id = self.synth_item_id("msg");
      let seq = self.next_seq();
      out.push(SseEvent::json(
        Some("response.output_item.added"),
        json!({
          "type": "response.output_item.added",
          "sequence_number": seq,
          "output_index": output_index,
          "item": {
            "id": item_id,
            "type": "message",
            "status": "in_progress",
            "role": "assistant",
            "content": [],
          }
        }),
      ));
      let seq = self.next_seq();
      out.push(SseEvent::json(
        Some("response.content_part.added"),
        json!({
          "type": "response.content_part.added",
          "sequence_number": seq,
          "item_id": item_id,
          "output_index": output_index,
          "content_index": 0,
          "part": {"type": "output_text", "text": "", "annotations": []},
        }),
      ));
      self.current = Some(OpenItem::Message {
        output_index,
        item_id,
        text: String::new(),
        part_open: true,
      });
    }
    let (output_index, item_id) = if let Some(OpenItem::Message { output_index, item_id, text: buf, .. }) = &mut self.current {
      buf.push_str(text);
      (*output_index, item_id.clone())
    } else {
      unreachable!()
    };
    let seq = self.next_seq();
    out.push(SseEvent::json(
      Some("response.output_text.delta"),
      json!({
        "type": "response.output_text.delta",
        "sequence_number": seq,
        "item_id": item_id,
        "output_index": output_index,
        "content_index": 0,
        "delta": text,
      }),
    ));
  }

  fn handle_reasoning(&mut self, text: &str, out: &mut Vec<SseEvent>) {
    if !matches!(self.current, Some(OpenItem::Reasoning { .. })) {
      self.close_current(out);
      let output_index = self.next_output_index;
      self.next_output_index += 1;
      let item_id = self.synth_item_id("rs");
      let seq = self.next_seq();
      out.push(SseEvent::json(
        Some("response.output_item.added"),
        json!({
          "type": "response.output_item.added",
          "sequence_number": seq,
          "output_index": output_index,
          "item": {
            "id": item_id,
            "type": "reasoning",
            "status": "in_progress",
            "content": [],
            "summary": [],
          }
        }),
      ));
      self.current = Some(OpenItem::Reasoning {
        output_index,
        item_id,
        summary: String::new(),
        part_open: true,
      });
    }
    let (output_index, item_id) = if let Some(OpenItem::Reasoning { output_index, item_id, summary, .. }) = &mut self.current {
      summary.push_str(text);
      (*output_index, item_id.clone())
    } else {
      unreachable!()
    };
    let seq = self.next_seq();
    out.push(SseEvent::json(
      Some("response.reasoning_text.delta"),
      json!({
        "type": "response.reasoning_text.delta",
        "sequence_number": seq,
        "item_id": item_id,
        "output_index": output_index,
        "content_index": 0,
        "delta": text,
      }),
    ));
  }

  fn handle_tool_call(
    &mut self,
    chat_index: usize,
    id_hint: Option<String>,
    name_hint: Option<String>,
    args_delta: &str,
    out: &mut Vec<SseEvent>,
  ) {
    let same_call = matches!(&self.current, Some(OpenItem::FunctionCall { chat_index: ci, .. }) if *ci == chat_index);
    if !same_call {
      self.close_current(out);
      let output_index = self.next_output_index;
      self.next_output_index += 1;
      let item_id = self.synth_item_id("fc");
      let call_id = id_hint.clone().unwrap_or_else(|| item_id.clone());
      let name = name_hint.clone().unwrap_or_default();
      let seq = self.next_seq();
      out.push(SseEvent::json(
        Some("response.output_item.added"),
        json!({
          "type": "response.output_item.added",
          "sequence_number": seq,
          "output_index": output_index,
          "item": {
            "id": item_id,
            "type": "function_call",
            "status": "in_progress",
            "call_id": call_id,
            "name": name,
            "arguments": "",
          }
        }),
      ));
      self.current = Some(OpenItem::FunctionCall {
        output_index,
        chat_index,
        item_id,
        call_id,
        name,
        arguments: String::new(),
      });
    }
    let (output_index, item_id) = if let Some(OpenItem::FunctionCall { output_index, item_id, name, call_id, arguments, .. }) = &mut self.current {
      arguments.push_str(args_delta);
      if let Some(n) = name_hint {
        if name.is_empty() {
          *name = n;
        }
      }
      if let Some(id) = id_hint {
        if call_id == item_id.as_str() || call_id.is_empty() {
          *call_id = id;
        }
      }
      (*output_index, item_id.clone())
    } else {
      unreachable!()
    };
    if !args_delta.is_empty() {
      let seq = self.next_seq();
      out.push(SseEvent::json(
        Some("response.function_call_arguments.delta"),
        json!({
          "type": "response.function_call_arguments.delta",
          "sequence_number": seq,
          "item_id": item_id,
          "output_index": output_index,
          "delta": args_delta,
        }),
      ));
    }
  }

  pub fn finish(&mut self) -> Vec<SseEvent> {
    if self.finished {
      return Vec::new();
    }
    self.finished = true;
    let mut out = Vec::new();
    self.ensure_started(&mut out);
    self.close_current(&mut out);
    let seq = self.next_seq();
    out.push(SseEvent::json(
      Some("response.completed"),
      json!({
        "type": "response.completed",
        "sequence_number": seq,
        "response": self.response_snapshot("completed", true),
      }),
    ));
    out
  }

  fn close_item(&mut self, item: OpenItem, out: &mut Vec<SseEvent>) {
    let output_index = item.output_index();
    let item_id = item.item_id().to_string();
    match item {
      OpenItem::Message { text, part_open, .. } => {
        if part_open {
          let seq = self.next_seq();
          out.push(SseEvent::json(
            Some("response.output_text.done"),
            json!({
              "type": "response.output_text.done",
              "sequence_number": seq,
              "item_id": item_id,
              "output_index": output_index,
              "content_index": 0,
              "text": text,
            }),
          ));
          let seq = self.next_seq();
          out.push(SseEvent::json(
            Some("response.content_part.done"),
            json!({
              "type": "response.content_part.done",
              "sequence_number": seq,
              "item_id": item_id,
              "output_index": output_index,
              "content_index": 0,
              "part": {"type": "output_text", "text": text, "annotations": []},
            }),
          ));
        }
        let final_item = json!({
          "id": item_id,
          "type": "message",
          "status": "completed",
          "role": "assistant",
          "content": [{"type": "output_text", "text": text, "annotations": []}],
        });
        let seq = self.next_seq();
        out.push(SseEvent::json(
          Some("response.output_item.done"),
          json!({
            "type": "response.output_item.done",
            "sequence_number": seq,
            "output_index": output_index,
            "item": final_item.clone(),
          }),
        ));
        self.closed.push(final_item);
      }
      OpenItem::FunctionCall { call_id, name, arguments, .. } => {
        let seq = self.next_seq();
        out.push(SseEvent::json(
          Some("response.function_call_arguments.done"),
          json!({
            "type": "response.function_call_arguments.done",
            "sequence_number": seq,
            "item_id": item_id,
            "output_index": output_index,
            "arguments": arguments,
          }),
        ));
        let final_item = json!({
          "id": item_id,
          "type": "function_call",
          "status": "completed",
          "call_id": call_id,
          "name": name,
          "arguments": arguments,
        });
        let seq = self.next_seq();
        out.push(SseEvent::json(
          Some("response.output_item.done"),
          json!({
            "type": "response.output_item.done",
            "sequence_number": seq,
            "output_index": output_index,
            "item": final_item.clone(),
          }),
        ));
        self.closed.push(final_item);
      }
      OpenItem::Reasoning { summary: text, part_open, .. } => {
        if part_open {
          let seq = self.next_seq();
          out.push(SseEvent::json(
            Some("response.reasoning_text.done"),
            json!({
              "type": "response.reasoning_text.done",
              "sequence_number": seq,
              "item_id": item_id,
              "output_index": output_index,
              "content_index": 0,
              "text": text,
            }),
          ));
        }
        let final_item = json!({
          "id": item_id,
          "type": "reasoning",
          "status": "completed",
          "content": [{"type": "reasoning_text", "text": text}],
          "summary": [],
        });
        let seq = self.next_seq();
        out.push(SseEvent::json(
          Some("response.output_item.done"),
          json!({
            "type": "response.output_item.done",
            "sequence_number": seq,
            "output_index": output_index,
            "item": final_item.clone(),
          }),
        ));
        self.closed.push(final_item);
      }
    }
  }
}
