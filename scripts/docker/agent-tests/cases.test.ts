import { describe, expect, test } from "bun:test";

import { defaultCases, parseCases, selectCases } from "./cases";

describe("agent-test matrix", () => {
  test("defaults cover text and real tool use for DeepSeek and Luna", () => {
    const cases = parseCases(defaultCases);
    expect(cases).toHaveLength(4);
    expect(new Set(cases.map((testCase) => `${testCase.model}/${testCase.probe}`)).size).toBe(4);
    expect(cases.filter((testCase) => testCase.model === "gpt-5.6-luna").every((testCase) => testCase.api === "responses")).toBe(true);
  });

  test("supports future agents and modes without silently mapping them to current ones", () => {
    const custom = { ...defaultCases[0], agent: "future-agent", mode: "future-mode" };
    expect(parseCases([custom])[0]).toEqual(custom);
  });

  test("accepts the Anthropic Messages API as an explicit endpoint", () => {
    expect(parseCases([{ ...defaultCases[0], api: "messages" }])[0].api).toBe("messages");
  });

  test("rejects invalid or ambiguous case declarations", () => {
    for (const value of [null, {}, [], [null], [defaultCases[0], defaultCases[0]]]) {
      expect(() => parseCases(value)).toThrow();
    }
    for (const patch of [
      { id: "../escape" }, { model: "codex/gpt-5.6-luna" }, { base_path: "https://example.org/v1" },
      { base_path: "//example.org/v1" }, { base_path: "/test/../v1" }, { base_path: "/test/v1?token=key" },
      { api: "auto" }, { probe: "fake-tool" }, { model: "" }, { display_name: "two\nlines" },
      { expected_text: " marker " }, { upstreamModel: "wrong_field" },
    ]) {
      expect(() => parseCases([{ ...defaultCases[0], ...patch }])).toThrow();
    }
    const { mode, ...missingMode } = defaultCases[0];
    expect(() => parseCases([missingMode])).toThrow("requires 'mode'");
  });

  test("selects explicit cases in matrix order and fails unknown selections", () => {
    expect(selectCases(defaultCases, [])).toEqual(defaultCases);
    expect(selectCases(defaultCases, [defaultCases[1].id, defaultCases[0].id])).toEqual(defaultCases.slice(0, 2));
    expect(selectCases(defaultCases, [defaultCases[0].id, defaultCases[0].id])).toEqual([defaultCases[0]]);
    expect(() => selectCases(defaultCases, [defaultCases[0].id, "typo"])).toThrow("Unknown agent-test case(s): typo");
  });
});
