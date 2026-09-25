import { randomUUID } from "node:crypto";

import type { AgentTestCase } from "./types";

export const fixturePath = "/agent-test/tool-fixture.txt";

export function readPrompt(instruction: string): string {
  return `${instruction} Do not guess the contents or use other tools. ` +
    "After reading, reply with only the value of verification_token, without quotes, Markdown, or explanation.";
}

export function preparePrompt(testCase: AgentTestCase, marker?: string): {
  expected_text: string;
  prompt: string;
  fixture?: { path: string; content: string };
} {
  const expectedText = testCase.expected_text ?? marker ?? `TOKN_${randomUUID().replaceAll("-", "")}`;
  if (testCase.probe === "read_tool") {
    return {
      expected_text: expectedText,
      prompt: readPrompt(`Use the read tool exactly once to read ${fixturePath}.`),
      fixture: { path: "tool-fixture.txt", content: `Tokn integration fixture\nverification_token=${expectedText}\n` },
    };
  }
  return {
    expected_text: expectedText,
    prompt: `Reply with exactly ${expectedText}, without quotes, Markdown, explanation, or any other text. Do not use tools.`,
  };
}
