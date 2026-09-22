import { describe, expect, test } from "bun:test";

import { dsh } from "./dsh";
import { resolveAgent } from "./index";
import type { AgentTestCase } from "./types";

const marker = "TOKN_DSH_709f";
const testCase: AgentTestCase = {
  id: "dsh-test",
  agent: "dsh",
  mode: "api",
  model: "deepseek-v4-flash",
  base_path: "/test/v1",
  api: "chat_completions",
  probe: "text",
};

describe("DSH preparation", () => {
  test("configures headless mode through an environment credential reference", () => {
    const prepared = dsh.prepare(testCase, { router_url: "http://127.0.0.1:4141/", marker });
    const settings = prepared.files.find((file) => file.path === "settings.yaml")!.content;
    expect(settings).toContain('provider: "tokn"');
    expect(settings).toContain('apiKeyEnv: "TOKN_AGENT_TEST_API_KEY"');
    expect(settings).toContain('api: "openai-completions"');
    expect(settings).toContain('baseURL: "http://127.0.0.1:4141/test/v1"');
    expect(settings).toContain(`- id: "${testCase.model}"`);
    expect(settings).not.toContain(marker);
    expect(prepared.command).toEqual(["--profile", "headless", expect.stringContaining(marker)]);
    expect(prepared.environment).toEqual({
      DSH_HOME: "/tmp/dsh-home",
      DSH_PERMISSION_MODE: "workspace-write",
      DSH_TELEMETRY_DISABLED: "1",
    });
    expect(resolveAgent("dsh")).toBe(dsh);
  });

  test("selects Responses and a qualified upstream model", () => {
    const prepared = dsh.prepare({
      ...testCase,
      api: "responses",
      model: "gpt-5.6-luna",
      upstream_model: "codex/gpt-5.6-luna",
    }, { router_url: "http://127.0.0.1:4141", marker });
    const settings = prepared.files[0].content;
    expect(settings).toContain('api: "openai-responses"');
    expect(settings).toContain('model: "codex/gpt-5.6-luna"');
    expect(settings).toContain('- id: "codex/gpt-5.6-luna"');
  });

  test("rejects modes and probes that cannot be verified", () => {
    expect(() => dsh.prepare({ ...testCase, mode: "proxy" }, { router_url: "http://127.0.0.1:4141" }))
      .toThrow("does not support agent-test mode");
    expect(() => dsh.prepare({ ...testCase, probe: "read_tool" }, { router_url: "http://127.0.0.1:4141" }))
      .toThrow("only text probes are supported");
  });
});

describe("DSH headless verification", () => {
  const prepared = dsh.prepare(testCase, { router_url: "http://127.0.0.1:4141", marker });

  test("accepts only the exact final response", () => {
    expect(dsh.evaluate(testCase, prepared, { stdout: `${marker}\n`, stderr: "reasoning", exit_code: 0 })).toEqual({
      success: true,
      text: marker,
      tool_calls: [],
      completed_steps: 1,
      session_ids: [],
    });
    expect(dsh.evaluate(testCase, prepared, { stdout: `extra ${marker}`, stderr: "", exit_code: 0 }).success).toBe(false);
    expect(dsh.evaluate(testCase, prepared, { stdout: "", stderr: "", exit_code: 0 }).success).toBe(false);
  });

  test("reports process failure and timeout without copying stderr", () => {
    expect(dsh.evaluate(testCase, prepared, { stdout: "", stderr: "secret", exit_code: 2 }).error).toBe("DSH exited with code 2");
    expect(dsh.evaluate(testCase, prepared, { stdout: "", stderr: "secret", exit_code: null, timed_out: true }).error)
      .toBe("DSH agent test timed out");
  });
});
