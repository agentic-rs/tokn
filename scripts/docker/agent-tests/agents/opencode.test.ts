import { describe, expect, test } from "bun:test";
import { posix } from "node:path";

import { resolveAgent } from "./index";
import { opencode } from "./opencode";
import type { AgentTestCase } from "./types";

const testCase: AgentTestCase = {
  id: "opencode-test",
  agent: "opencode",
  mode: "api",
  model: "deepseek-v4-flash",
  base_path: "/test/v1",
  api: "chat_completions",
  probe: "text",
};
const marker = "TOKN_TEST_709f";

function events(...values: unknown[]): string {
  return values.map((value) => JSON.stringify(value)).join("\n");
}

function textEvent(text = marker) {
  return { type: "text", sessionID: "session-one", part: { type: "text", text } };
}

function finishEvent(reason = "stop") {
  return { type: "step_finish", sessionID: "session-one", part: { reason } };
}

function readEvent(status = "completed", filePath = "/agent-test/tool-fixture.txt", token = marker) {
  return {
    type: "tool_use",
    sessionID: "session-one",
    part: { tool: "read", state: { status, input: { filePath }, output: `<content>verification_token=${token}</content>` } },
  };
}

function evaluate(stdout: string, overrides: Partial<AgentTestCase> = {}, exit_code: number | null = 0, timed_out = false) {
  const selected = { ...testCase, ...overrides };
  const prepared = opencode.prepare(selected, { router_url: "http://127.0.0.1:4141", marker });
  return opencode.evaluate(selected, prepared, { stdout, stderr: "", exit_code, timed_out });
}

describe("OpenCode preparation", () => {
  test("produces isolated configuration and JSON invocation without embedding credentials", () => {
    const prepared = opencode.prepare(testCase, { router_url: "http://127.0.0.1:4141/", marker });
    const config = JSON.parse(prepared.files.find((file) => file.path === "opencode.json")!.content);
    expect(config.provider.tokn).toMatchObject({
      npm: "@ai-sdk/openai-compatible",
      options: { baseURL: "http://127.0.0.1:4141/test/v1", apiKey: "{env:TOKN_AGENT_TEST_API_KEY}" },
    });
    expect(config.permission).toBe("deny");
    expect(config.tools).toEqual({ "*": false });
    expect(config.enabled_providers).toEqual(["tokn"]);
    expect(config.plugin).toEqual([]);
    expect(config.mcp).toEqual({});
    expect(prepared.command).toEqual([
      "--pure", "run", "--format", "json", "--model", "tokn/deepseek-v4-flash", "--title", testCase.id, "--dir", "/workspace",
      expect.stringContaining(marker),
    ]);
    expect(prepared.working_dir).toBe("/workspace");
    expect(prepared.environment.OPENCODE_CONFIG).toBe("/agent-test/opencode.json");
    expect(prepared.environment.OPENCODE_DISABLE_PROJECT_CONFIG).toBe("1");
    expect(prepared.environment.OPENCODE_DISABLE_DEFAULT_PLUGINS).toBe("1");
    expect(prepared.environment.OPENCODE_DISABLE_EXTERNAL_SKILLS).toBe("1");
    expect(prepared.environment.TOKN_AGENT_TEST_API_KEY).toBeUndefined();
    expect(prepared.files.every((file) => !file.path.startsWith("/") && !file.path.includes(".."))).toBe(true);
  });

  test("selects Responses independently of the agent and upstream model ID", () => {
    const prepared = opencode.prepare({
      ...testCase, api: "responses", model: "gpt-5.6-luna", upstream_model: "codex/gpt-5.6-luna", display_name: "Luna",
    }, { router_url: "http://127.0.0.1:4141", marker });
    const config = JSON.parse(prepared.files[0].content);
    expect(config.provider.tokn.npm).toBe("@ai-sdk/openai");
    expect(config.provider.tokn.models["gpt-5.6-luna"]).toMatchObject({ name: "Luna", id: "codex/gpt-5.6-luna" });
    expect(prepared.command).toContain("tokn/gpt-5.6-luna");
  });

  test("keeps read answer only in its fixture and grants narrowly scoped read access", () => {
    const prepared = opencode.prepare({ ...testCase, probe: "read_tool" }, { router_url: "http://127.0.0.1:4141", marker });
    const config = JSON.parse(prepared.files[0].content);
    expect(prepared.files.find((file) => file.path === "prompt.txt")!.content).not.toContain(marker);
    expect(prepared.command.join(" ")).not.toContain(marker);
    expect(prepared.files.find((file) => file.path === "tool-fixture.txt")!.content).toContain(`verification_token=${marker}`);
    expect(config.permission).toEqual({
      "*": "deny",
      read: { "*": "deny", "agent-test/tool-fixture.txt": "allow" },
      external_directory: { "*": "deny", "/agent-test/*": "allow" },
    });
    expect(config.tools).toEqual({ "*": false, read: true });
    // Pinned OpenCode reads relative to the global non-Git worktree, not cwd.
    expect(config.permission.read[posix.relative("/", prepared.fixture_path!)]).toBe("allow");
    expect(config.permission.read["*"]).toBe("deny");
    expect(config.permission.read["agent-test/opencode.json"]).toBeUndefined();
  });

  test("generates fresh fixture markers unless the case explicitly supplies one", () => {
    const first = opencode.prepare(testCase, { router_url: "http://127.0.0.1:4141" });
    const second = opencode.prepare(testCase, { router_url: "http://127.0.0.1:4141" });
    expect(first.expected_text).not.toBe(second.expected_text);
    const fixed = opencode.prepare({ ...testCase, expected_text: "FIXED" }, { router_url: "http://127.0.0.1:4141", marker });
    expect(fixed.expected_text).toBe("FIXED");
  });

  test("does not pretend unsupported agents or modes are implemented", () => {
    expect(resolveAgent("opencode")).toBe(opencode);
    expect(() => opencode.prepare({ ...testCase, mode: "proxy" }, { router_url: "http://127.0.0.1:4141" }))
      .toThrow("does not support agent-test mode");
    expect(() => opencode.prepare({ ...testCase, api: "messages" }, { router_url: "http://127.0.0.1:4141" }))
      .toThrow("do not support the Messages API");
  });
});

describe("OpenCode JSONL verification", () => {
  test("accepts a completed exact text response and normalizes session IDs", () => {
    const result = evaluate(events(textEvent(), finishEvent()));
    expect(result).toEqual({ success: true, text: marker, tool_calls: [], completed_steps: 1, session_ids: ["session-one"] });
  });

  test("accepts a completed read and terminal response after the tool step", () => {
    const result = evaluate(events(readEvent(), finishEvent("tool-calls"), textEvent(), finishEvent()), { probe: "read_tool" });
    expect(result.success).toBe(true);
    expect(result.completed_steps).toBe(2);
    expect(result.tool_calls[0]).toMatchObject({ name: "read", status: "completed", file_path: "/agent-test/tool-fixture.txt" });
  });

  test("fails on an error event even if the process exits zero and text matched", () => {
    const result = evaluate(events(textEvent(), finishEvent(), { type: "error", error: { data: { message: "secret=DO_NOT_REPORT" } } }));
    expect(result.success).toBe(false);
    expect(result.error).toContain("error event");
    expect(JSON.stringify(result)).not.toContain("DO_NOT_REPORT");
  });

  test("fails empty, malformed, and structurally invalid output", () => {
    for (const stdout of ["", "not json", "{}", "null", "[]", events({ type: "text", part: {} }, finishEvent())]) {
      expect(evaluate(stdout).success).toBe(false);
    }
    expect(evaluate(`${events(textEvent(), finishEvent())}\n{`).error).toContain("Invalid OpenCode JSON");
  });

  test("requires terminal completion rather than only a tool step or token limit", () => {
    for (const stdout of [events(textEvent()), events(textEvent(), finishEvent("tool-calls")), events(textEvent(), finishEvent("length"))]) {
      expect(evaluate(stdout).error).toContain("terminal step");
    }
    expect(evaluate(events(textEvent(), finishEvent(), { type: "step_start" })).success).toBe(false);
  });

  test("rejects extra prose, unexpected tools, nonzero exit, and timeout", () => {
    expect(evaluate(events(textEvent(`Here: ${marker}`), finishEvent())).error).toContain("exactly match");
    expect(evaluate(events(readEvent(), textEvent(), finishEvent())).error).toContain("unexpectedly invoked");
    expect(evaluate(events(textEvent(), finishEvent()), {}, 1).error).toContain("code 1");
    expect(evaluate(events(textEvent(), finishEvent()), {}, null, true).error).toContain("timed out");
  });

  test("rejects guessed answers, failed reads, wrong files, missing token, and extra tools", () => {
    const candidates = [
      [],
      [readEvent("error")],
      [readEvent("running")],
      [readEvent("completed", "/agent-test/other.txt")],
      [readEvent("completed", "/agent-test/tool-fixture.txt", "WRONG")],
      [readEvent(), readEvent()],
      [readEvent(), { type: "tool_use", part: { tool: "bash", state: { status: "completed" } } }],
    ];
    for (const calls of candidates) {
      expect(evaluate(events(...calls, textEvent(), finishEvent()), { probe: "read_tool" }).success).toBe(false);
    }
  });
});
