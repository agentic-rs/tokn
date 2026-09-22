import { describe, expect, test } from "bun:test";

import { claudeCode } from "./claude_code";
import { resolveAgent } from "./index";
import type { AgentTestCase } from "./types";

const marker = "TOKN_CLAUDE_709f";
const testCase: AgentTestCase = {
  id: "claude-code-test",
  agent: "claude-code",
  mode: "api",
  model: "claude-sonnet-4.6",
  base_path: "/test/v1",
  api: "messages",
  probe: "text",
};

function events(...values: unknown[]): string {
  return values.map((value) => JSON.stringify(value)).join("\n");
}

function successEvents(text = marker, middle: unknown[] = []) {
  return events(
    { type: "system", subtype: "init", session_id: "session-one", tools: [] },
    ...middle,
    { type: "result", subtype: "success", is_error: false, result: text, num_turns: 1, session_id: "session-one" },
  );
}

function readEvents(path = "/agent-test/tool-fixture.txt", token = marker, isError = false) {
  return [
    {
      type: "assistant", session_id: "session-one",
      message: { role: "assistant", content: [{ type: "tool_use", id: "tool-one", name: "Read", input: { file_path: path } }] },
    },
    {
      type: "user", session_id: "session-one",
      message: {
        role: "user",
        content: [{ type: "tool_result", tool_use_id: "tool-one", is_error: isError, content: `verification_token=${token}` }],
      },
    },
  ];
}

function evaluate(stdout: string, overrides: Partial<AgentTestCase> = {}, exit_code: number | null = 0, timed_out = false) {
  const selected = { ...testCase, ...overrides };
  const prepared = claudeCode.prepare(selected, { router_url: "http://127.0.0.1:4141", marker });
  return claudeCode.evaluate(selected, prepared, { stdout, stderr: "", exit_code, timed_out });
}

describe("Claude Code preparation", () => {
  test("configures Messages streaming with isolated state and no embedded credential", () => {
    const prepared = claudeCode.prepare(testCase, { router_url: "http://127.0.0.1:4141/", marker });
    expect(prepared.environment).toMatchObject({
      ANTHROPIC_BASE_URL: "http://127.0.0.1:4141/test",
      CLAUDE_CONFIG_DIR: "/tmp/claude-home",
      CLAUDE_CODE_DISABLE_NONESSENTIAL_TRAFFIC: "1",
      DISABLE_TELEMETRY: "1",
    });
    expect(prepared.environment.ANTHROPIC_AUTH_TOKEN).toBeUndefined();
    expect(prepared.command).toEqual([
      "--print", "--output-format", "stream-json", "--verbose", "--no-session-persistence", "--bare", "--restricted",
      "--model", "claude-sonnet-4.6", "--permission-mode", "dontAsk", "--permission-prompts", "none", "--max-turns", "3",
      "--tools", "", expect.stringContaining(marker),
    ]);
    expect(resolveAgent("claude-code")).toBe(claudeCode);
  });

  test("allows only Read for the read probe and keeps its token out of the prompt", () => {
    const prepared = claudeCode.prepare({
      ...testCase, probe: "read_tool", upstream_model: "github-copilot/claude-sonnet-4.6",
    }, { router_url: "http://127.0.0.1:4141", marker });
    expect(prepared.command).toContain("github-copilot/claude-sonnet-4.6");
    expect(prepared.command).toContain("--allowedTools");
    expect(prepared.command.join(" ")).not.toContain(marker);
    expect(prepared.files.find((file) => file.path === "tool-fixture.txt")!.content).toContain(marker);
  });

  test("rejects unsupported modes and wire APIs", () => {
    expect(() => claudeCode.prepare({ ...testCase, mode: "proxy" }, { router_url: "http://127.0.0.1:4141" }))
      .toThrow("does not support agent-test mode");
    for (const api of ["responses", "chat_completions"] as const) {
      expect(() => claudeCode.prepare({ ...testCase, api }, { router_url: "http://127.0.0.1:4141" }))
        .toThrow("require the Messages API");
    }
  });
});

describe("Claude Code stream verification", () => {
  test("accepts an exact completed response", () => {
    expect(evaluate(successEvents())).toEqual({
      success: true, text: marker, tool_calls: [], completed_steps: 1, session_ids: ["session-one"],
    });
  });

  test("accepts one matched completed Read", () => {
    const result = evaluate(successEvents(marker, readEvents()), { probe: "read_tool" });
    expect(result.success).toBe(true);
    expect(result.tool_calls[0]).toMatchObject({
      name: "Read", status: "completed", file_path: "/agent-test/tool-fixture.txt",
    });
  });

  test("rejects incomplete, malformed, failed, and unexpected output", () => {
    for (const stdout of ["", "{}", "not json", events({ type: "system", subtype: "init" })]) {
      expect(evaluate(stdout).success).toBe(false);
    }
    expect(evaluate(successEvents(), {}, 1).error).toContain("code 1");
    expect(evaluate(successEvents(), {}, null, true).error).toContain("timed out");
    expect(evaluate(successEvents(`extra ${marker}`)).error).toContain("exactly match");
    expect(evaluate(successEvents(marker, readEvents())).error).toContain("unexpectedly invoked");
    expect(evaluate(events(
      { type: "system", subtype: "init", session_id: "one" },
      { type: "result", subtype: "error", is_error: true, result: "secret", num_turns: 1 },
    )).error).toContain("error result");
  });

  test("rejects invalid read evidence", () => {
    const candidates = [
      [],
      readEvents("/agent-test/other.txt"),
      readEvents("/agent-test/tool-fixture.txt", "WRONG"),
      readEvents("/agent-test/tool-fixture.txt", marker, true),
      [...readEvents(), ...readEvents()],
      [{
        type: "assistant", session_id: "one",
        message: { content: [{ type: "tool_use", id: "other", name: "Bash", input: {} }] },
      }],
    ];
    for (const middle of candidates) {
      expect(evaluate(successEvents(marker, middle), { probe: "read_tool" }).success).toBe(false);
    }
  });
});
