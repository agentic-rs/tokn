import { describe, expect, test } from "bun:test";

import { resolveAgent } from "./index";
import { pi } from "./pi";
import type { AgentTestCase } from "./types";

const marker = "TOKN_PI_709f";
const testCase: AgentTestCase = {
  id: "pi-test",
  agent: "pi",
  mode: "api",
  model: "deepseek-v4-flash",
  base_path: "/test/v1",
  api: "chat_completions",
  probe: "text",
};

function events(...values: unknown[]): string {
  return values.map((value) => JSON.stringify(value)).join("\n");
}

function completeEvents(text = marker, extra: unknown[] = []): string {
  return events(
    { type: "session", version: 3, id: "pi-session" },
    { type: "agent_start" },
    ...extra,
    { type: "message_end", message: { role: "assistant", content: [{ type: "text", text }] } },
    { type: "turn_end", message: {}, toolResults: [] },
    { type: "agent_end", messages: [] },
  );
}

function readEvents(path = "/agent-test/tool-fixture.txt", token = marker, isError = false) {
  return [{
    type: "tool_execution_start",
    toolCallId: "tool-one",
    toolName: "read",
    args: { path },
  }, {
    type: "tool_execution_end",
    toolCallId: "tool-one",
    toolName: "read",
    result: { content: [{ type: "text", text: `verification_token=${token}` }] },
    isError,
  }];
}

function evaluate(stdout: string, overrides: Partial<AgentTestCase> = {}, exit_code: number | null = 0, timed_out = false) {
  const selected = { ...testCase, ...overrides };
  const prepared = pi.prepare(selected, { router_url: "http://127.0.0.1:4141", marker });
  return pi.evaluate(selected, prepared, { stdout, stderr: "", exit_code, timed_out });
}

describe("Pi preparation", () => {
  test("configures an isolated Chat Completions provider without embedding credentials", () => {
    const prepared = pi.prepare(testCase, { router_url: "http://127.0.0.1:4141/", marker });
    const config = JSON.parse(prepared.files.find((file) => file.path === "models.json")!.content);
    expect(config.providers.tokn).toMatchObject({
      baseUrl: "http://127.0.0.1:4141/test/v1",
      api: "openai-completions",
      apiKey: "$TOKN_AGENT_TEST_API_KEY",
    });
    expect(config.providers.tokn.models[0]).toMatchObject({ id: testCase.model, name: testCase.model });
    expect(prepared.command).toEqual([
      "--mode", "json", "--no-session", "--no-approve", "--provider", "tokn", "--model", testCase.model,
      "--no-tools", expect.stringContaining(marker),
    ]);
    expect(prepared.environment).toEqual({ PI_CODING_AGENT_DIR: "/tmp/pi-agent", PI_OFFLINE: "1" });
    expect(prepared.files.every((file) => !file.content.includes("Bearer "))).toBe(true);
    expect(resolveAgent("pi")).toBe(pi);
  });

  test("selects Responses and the qualified upstream model independently", () => {
    const prepared = pi.prepare({
      ...testCase,
      api: "responses",
      model: "gpt-5.6-luna",
      upstream_model: "codex/gpt-5.6-luna",
      display_name: "Luna",
    }, { router_url: "http://127.0.0.1:4141", marker });
    const provider = JSON.parse(prepared.files[0].content).providers.tokn;
    expect(provider.api).toBe("openai-responses");
    expect(provider.models[0]).toMatchObject({ id: "codex/gpt-5.6-luna", name: "Luna" });
    expect(prepared.command).toContain("codex/gpt-5.6-luna");
  });

  test("keeps the read marker out of the prompt and enables only read", () => {
    const prepared = pi.prepare({ ...testCase, probe: "read_tool" }, { router_url: "http://127.0.0.1:4141", marker });
    expect(prepared.command).toContain("--tools");
    expect(prepared.command).toContain("read");
    expect(prepared.command.join(" ")).not.toContain(marker);
    expect(prepared.files.find((file) => file.path === "tool-fixture.txt")!.content).toContain(marker);
    expect(prepared.fixture_path).toBe("/agent-test/tool-fixture.txt");
  });

  test("rejects unsupported modes", () => {
    expect(() => pi.prepare({ ...testCase, mode: "proxy" }, { router_url: "http://127.0.0.1:4141" }))
      .toThrow("does not support agent-test mode");
    expect(() => pi.prepare({ ...testCase, api: "messages" }, { router_url: "http://127.0.0.1:4141" }))
      .toThrow("do not support the Messages API");
  });
});

describe("Pi JSONL verification", () => {
  test("accepts a complete exact text session", () => {
    expect(evaluate(completeEvents())).toEqual({
      success: true,
      text: marker,
      tool_calls: [],
      completed_steps: 1,
      session_ids: ["pi-session"],
    });
  });

  test("accepts exactly one successful fixture read", () => {
    const result = evaluate(completeEvents(marker, readEvents()), { probe: "read_tool" });
    expect(result.success).toBe(true);
    expect(result.tool_calls[0]).toMatchObject({
      name: "read",
      status: "completed",
      file_path: "/agent-test/tool-fixture.txt",
    });
  });

  test("requires parseable lifecycle events and an exact final response", () => {
    for (const stdout of ["", "not json", events({ type: "session", id: "x" }), completeEvents(`extra ${marker}`)]) {
      expect(evaluate(stdout).success).toBe(false);
    }
    expect(evaluate(`${completeEvents()}\n{`).error).toContain("Invalid Pi JSON");
  });

  test("rejects unexpected or invalid tool calls", () => {
    expect(evaluate(completeEvents(marker, readEvents())).error).toContain("unexpectedly invoked");
    for (const events of [
      readEvents("/agent-test/other.txt"),
      readEvents("/agent-test/tool-fixture.txt", "WRONG"),
      readEvents("/agent-test/tool-fixture.txt", marker, true),
      readEvents().map((event) => ({ ...event, toolName: "bash" })),
    ]) {
      expect(evaluate(completeEvents(marker, events), { probe: "read_tool" }).success).toBe(false);
    }
    expect(evaluate(completeEvents(marker, [...readEvents(), ...readEvents()]), { probe: "read_tool" }).success).toBe(false);
  });

  test("reports assistant errors without copying provider details", () => {
    const stdout = events(
      { type: "session", version: 3, id: "pi-session" },
      { type: "agent_start" },
      { type: "message_end", message: { role: "assistant", content: [], stopReason: "error", errorMessage: "secret" } },
      { type: "turn_end" },
      { type: "agent_end" },
    );
    expect(evaluate(stdout).error).toContain("assistant error");
    expect(JSON.stringify(evaluate(stdout))).not.toContain("secret");
  });

  test("reports process failure and timeout without copying provider output", () => {
    expect(evaluate("secret provider error", {}, 1).error).toBe("Pi exited with code 1");
    expect(evaluate("secret provider error", {}, null, true).error).toBe("Pi agent test timed out");
  });
});
