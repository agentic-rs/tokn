import { randomUUID } from "node:crypto";

import type { TrialCase } from "./types";

export const fixturePath = "/trial/tool-fixture.txt";

export function preparePrompt(trial: TrialCase, marker?: string): {
  expected_text: string;
  prompt: string;
  fixture?: { path: string; content: string };
} {
  const expectedText = trial.expected_text ?? marker ?? `TOKN_${randomUUID().replaceAll("-", "")}`;
  if (trial.probe === "read_tool") {
    return {
      expected_text: expectedText,
      prompt: `Use the read tool exactly once to read ${fixturePath}. Do not guess the contents or use other tools. ` +
        "After reading, reply with only the value of verification_token, without quotes, Markdown, or explanation.",
      fixture: { path: "tool-fixture.txt", content: `Tokn integration fixture\nverification_token=${expectedText}\n` },
    };
  }
  return {
    expected_text: expectedText,
    prompt: `Reply with exactly ${expectedText}, without quotes, Markdown, explanation, or any other text. Do not use tools.`,
  };
}
