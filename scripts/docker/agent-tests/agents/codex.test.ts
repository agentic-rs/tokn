import { describe, expect, test } from "bun:test";

import { codex } from "./codex";
import { resolveAgent } from "./index";
import type { AgentTestCase } from "./types";

const marker = "TOKN_CODEX_709f";
const testCase: AgentTestCase = {
  id: "codex-test",
  agent: "codex",
  mode: "api",
  model: "gpt-5.4",
  base_path: "/test/v1",
  api: "responses",
  probe: "text",
};

function events(...values: unknown[]): string {
  return values.map((value) => JSON.stringify(value)).join("\n");
}

function successEvents(text = marker, items: unknown[] = []) {
  return events(
    { type: "thread.started", thread_id: "thread-one" },
    { type: "turn.started" },
    ...items,
    { type: "item.completed", item: { id: "answer", type: "agent_message", text } },
    { type: "turn.completed", usage: {} },
  );
}

function commandEvent(path = "/agent-test/tool-fixture.txt", token = marker, status = "completed") {
  return {
    type: "item.completed",
    item: {
      id: "command", type: "command_execution", command: `cat ${path}`, status,
      aggregated_output: `verification_token=${token}\n`, exit_code: status === "completed" ? 0 : 1,
    },
  };
}

function evaluate(stdout: string, overrides: Partial<AgentTestCase> = {}, exit_code: number | null = 0, timed_out = false) {
  const selected = { ...testCase, ...overrides };
  const prepared = codex.prepare(selected, { router_url: "http://127.0.0.1:4141", marker });
  return codex.evaluate(selected, prepared, { stdout, stderr: "", exit_code, timed_out });
}

describe("Codex preparation", () => {
  test("configures a private Responses provider without persisting a session", () => {
    const prepared = codex.prepare(testCase, { router_url: "http://127.0.0.1:4141/", marker });
    expect(prepared.command).toEqual([
      "exec", "--json", "--skip-git-repo-check", "--ephemeral", "--ignore-user-config", "--ignore-rules",
      "--sandbox", "read-only", "--ask-for-approval", "never", "--cd", "/workspace", "--model", "gpt-5.4",
      "--config", 'model_provider="tokn"', "--config",
      expect.stringContaining('base_url = "http://127.0.0.1:4141/test/v1"'), expect.stringContaining(marker),
    ]);
    expect(prepared.command.join(" ")).toContain('env_key = "TOKN_AGENT_TEST_API_KEY"');
    expect(prepared.environment).toEqual({ CODEX_HOME: "/tmp/codex-home" });
    expect(prepared.environment.TOKN_AGENT_TEST_API_KEY).toBeUndefined();
    expect(resolveAgent("codex")).toBe(codex);
  });

  test("uses a qualified upstream model and isolates the read fixture", () => {
    const prepared = codex.prepare({
      ...testCase, probe: "read_tool", upstream_model: "github-copilot/gpt-5.4",
    }, { router_url: "http://127.0.0.1:4141", marker });
    expect(prepared.command).toContain("github-copilot/gpt-5.4");
    expect(prepared.command.join(" ")).not.toContain(marker);
    expect(prepared.files.find((file) => file.path === "tool-fixture.txt")!.content).toContain(marker);
    expect(prepared.fixture_path).toBe("/agent-test/tool-fixture.txt");
  });

  test("rejects unsupported modes and wire APIs", () => {
    expect(() => codex.prepare({ ...testCase, mode: "proxy" }, { router_url: "http://127.0.0.1:4141" }))
      .toThrow("does not support agent-test mode");
    for (const api of ["chat_completions", "messages"] as const) {
      expect(() => codex.prepare({ ...testCase, api }, { router_url: "http://127.0.0.1:4141" }))
        .toThrow("require the Responses API");
    }
  });
});

describe("Codex JSONL verification", () => {
  test("accepts an exact completed response", () => {
    expect(evaluate(successEvents())).toEqual({
      success: true, text: marker, tool_calls: [], completed_steps: 1, session_ids: ["thread-one"],
    });
  });

  test("accepts one completed fixture read", () => {
    const result = evaluate(successEvents(marker, [commandEvent()]), { probe: "read_tool" });
    expect(result.success).toBe(true);
    expect(result.tool_calls[0]).toMatchObject({
      name: "command_execution", status: "completed", file_path: "/agent-test/tool-fixture.txt",
    });
  });

  test("rejects incomplete, malformed, failed, and unexpected output", () => {
    for (const stdout of ["", "{}", "not json", events({ type: "thread.started", thread_id: "one" })]) {
      expect(evaluate(stdout).success).toBe(false);
    }
    expect(evaluate(successEvents(), {}, 1).error).toContain("code 1");
    expect(evaluate(successEvents(), {}, null, true).error).toContain("timed out");
    expect(evaluate(events({ type: "error", message: "secret" }), {}).error).toContain("error event");
    expect(evaluate(successEvents(`extra ${marker}`)).error).toContain("exactly match");
    expect(evaluate(successEvents(marker, [commandEvent()])).error).toContain("unexpectedly invoked");
  });

  test("rejects invalid read evidence", () => {
    const candidates = [
      [],
      [commandEvent("/agent-test/other.txt")],
      [commandEvent("/agent-test/tool-fixture.txt", "WRONG")],
      [commandEvent("/agent-test/tool-fixture.txt", marker, "failed")],
      [commandEvent(), commandEvent()],
      [{ type: "item.completed", item: { type: "web_search", status: "completed" } }],
    ];
    for (const items of candidates) {
      expect(evaluate(successEvents(marker, items), { probe: "read_tool" }).success).toBe(false);
    }
  });
});
