import { expect, test } from "bun:test";

import { createEngine } from "./engine";

test("command transport captures streams and preserves a nonzero process status", async () => {
  const result = await createEngine(process.execPath).run(["-e", "console.log(process.env.TOKN_TEST_VALUE); console.error('diagnostic'); process.exit(7)"], { env: { TOKN_TEST_VALUE: "fixture" } });
  expect(result).toEqual({ stdout: "fixture\n", stderr: "diagnostic\n", exit_code: 7, timed_out: false });
});

test("command timeout terminates the waiting client", async () => {
  const result = await createEngine(process.execPath).run(["-e", "await Bun.sleep(60000)"], { timeout_ms: 100 });
  expect(result.timed_out).toBe(true);
  expect(result.exit_code).not.toBe(0);
});

test("cancellation interrupts an in-flight command and rejects with the caller's reason", async () => {
  const controller = new AbortController();
  const timer = setTimeout(() => controller.abort(new Error("cancelled by test")), 100);
  try {
    await expect(createEngine(process.execPath).run(["-e", "await Bun.sleep(60000)"], { signal: controller.signal })).rejects.toThrow("cancelled by test");
  } finally {
    clearTimeout(timer);
  }
});
