import { afterEach, beforeEach, describe, expect, test } from "bun:test";
import { mkdirSync, mkdtempSync, rmSync, symlinkSync, writeFileSync } from "node:fs";
import { homedir, tmpdir } from "node:os";
import { join, resolve } from "node:path";

import { defaultCases } from "./cases";
import { parseSuite, validateInputs } from "./suite";

let directory: string;

beforeEach(() => {
  directory = mkdtempSync(join(tmpdir(), "tokn-agent-test-suite-"));
  writeFileSync(join(directory, "config.toml"), "schema_version = 2\n");
  writeFileSync(join(directory, "auth.yaml"), "version: 1\naccounts: []\n");
});

afterEach(() => {
  rmSync(directory, { recursive: true, force: true });
});

function input(overrides: Record<string, unknown> = {}): Record<string, unknown> {
  return {
    schema_version: 1,
    gateway_image: "tokn-gateway-cli:test",
    config_file: "config.toml",
    cases: [defaultCases[0]],
    ...overrides,
  };
}

describe("agent-test suite configuration", () => {
  test("resolves relative inputs beside the suite and supplies isolated defaults", () => {
    const suite = parseSuite(input(), directory);
    expect(suite.config_file).toBe(join(directory, "config.toml"));
    expect(suite.auth_file).toBe(join(directory, "auth.yaml"));
    expect(suite.config_dir).toBeUndefined();
    expect(suite.auth_dir).toBeUndefined();
    expect(suite.router_url).toBe("http://127.0.0.1:4141");
    expect(suite.timeout_secs).toBe(120);
    expect(suite.serve_args).toEqual([]);
    expect(suite.agent_images).toEqual({});
    expect(suite.codex_disable_selinux_label).toBe(false);
    expect(suite.cases).toEqual([defaultCases[0]]);
    expect(() => validateInputs(suite)).not.toThrow();
  });

  test("requires an explicit boolean for the Codex SELinux workaround", () => {
    for (const value of [true, false]) {
      expect(parseSuite(input({ codex_disable_selinux_label: value }), directory).codex_disable_selinux_label).toBe(value);
    }
    for (const value of [null, "true", 1, [], {}]) {
      expect(() => parseSuite(input({ codex_disable_selinux_label: value }), directory)).toThrow("must be a boolean");
    }
  });

  test("discovers matching config fragments and auth shards independently", () => {
    mkdirSync(join(directory, "alternate.d"));
    mkdirSync(join(directory, "credentials"));
    mkdirSync(join(directory, "credentials", "auth.d"));
    const suite = parseSuite(input({
      config_file: "alternate.toml",
      auth_file: "credentials/accounts.yaml",
    }), directory);
    expect(suite.config_dir).toBe(join(directory, "alternate.d"));
    expect(suite.auth_dir).toBe(join(directory, "credentials", "auth.d"));
  });

  test("resolves explicit fragments, absolute files, and home-relative inputs", () => {
    const suite = parseSuite(input({
      config_file: join(directory, "config.toml"),
      auth_file: "~/agent-test-fixture/auth.yaml",
      config_dir: "../custom-config.d",
      auth_dir: "./custom-auth.d",
    }), directory);
    expect(suite.config_file).toBe(join(directory, "config.toml"));
    expect(suite.auth_file).toBe(join(homedir(), "agent-test-fixture", "auth.yaml"));
    expect(suite.config_dir).toBe(resolve(directory, "../custom-config.d"));
    expect(suite.auth_dir).toBe(join(directory, "custom-auth.d"));
  });

  test("accepts only HTTP loopback origins for the shared container namespace", () => {
    for (const router_url of ["http://127.0.0.1:4141", "http://localhost:5151/", "http://[::1]:4141"]) {
      expect(parseSuite(input({ router_url }), directory).router_url).toBe(new URL(router_url).origin);
    }
    for (const router_url of [
      "https://127.0.0.1:4141", "http://0.0.0.0:4141", "http://192.168.1.10:4141",
      "http://gateway:4141", "http://localhost.example.org:4141", "http://[::]:4141",
      "http://user:secret@127.0.0.1:4141", "http://127.0.0.1:4141/v1",
      "http://127.0.0.1:4141?api_key=secret", "http://127.0.0.1:4141#fragment", "not-a-url",
    ]) {
      expect(() => parseSuite(input({ router_url }), directory)).toThrow();
    }
  });

  test("rejects unknown fields and invalid case declarations before execution", () => {
    expect(() => parseSuite(input({ configFile: "typo.toml" }), directory)).toThrow("unknown suite field");
    for (const cases of [undefined, null, [], {}, [{ ...defaultCases[0], unknown_field: "typo" }], [defaultCases[0], defaultCases[0]]]) {
      expect(() => parseSuite(input({ cases }), directory)).toThrow();
    }
  });

  test("validates required fields, schema version, execution time, and image overrides", () => {
    for (const value of [null, [], "suite"]) {
      expect(() => parseSuite(value, directory)).toThrow("must be an object");
    }
    for (const overrides of [
      { schema_version: 2 }, { schema_version: "1" }, { gateway_image: "" },
      { config_file: "\0" }, { auth_file: 3 }, { config_dir: "" }, { auth_dir: null },
      { timeout_secs: 0 }, { timeout_secs: -1 }, { timeout_secs: 3601 },
      { timeout_secs: 1.5 }, { timeout_secs: "120" }, { agent_images: [] },
      { agent_images: { opencode: "" } },
    ]) {
      expect(() => parseSuite(input(overrides), directory)).toThrow();
    }
    for (const timeout_secs of [1, 3600]) {
      expect(parseSuite(input({ timeout_secs }), directory).timeout_secs).toBe(timeout_secs);
    }
    expect(parseSuite(input({ agent_images: { opencode: "opencode:custom" } }), directory).agent_images)
      .toEqual({ opencode: "opencode:custom" });
  });

  test("allows proxy overrides without permitting unrelated server arguments", () => {
    for (const serve_args of [[], ["--no-proxy"], ["--with-proxy"]]) {
      expect(parseSuite(input({ serve_args }), directory).serve_args).toEqual(serve_args);
    }
    for (const serve_args of ["--no-proxy", ["--host", "0.0.0.0"], ["--config", "/host/config.toml"], ["--no-proxy", 1]]) {
      expect(() => parseSuite(input({ serve_args }), directory)).toThrow("serve_args");
    }
  });
});

describe("agent-test input validation", () => {
  test("requires visible shutdown logs in either schema", () => {
    for (const section of ["[logging]", "schema_version = 2\n[service.logging]"]) {
      writeFileSync(join(directory, "config.toml"), `${section}\ntarget = "file"\n`);
      expect(() => validateInputs(parseSuite(input(), directory))).toThrow("verify persistence shutdown");
    }
  });
  test("requires files for roots and directories for fragments", () => {
    mkdirSync(join(directory, "config.d"));
    mkdirSync(join(directory, "auth.d"));
    expect(() => validateInputs(parseSuite(input(), directory))).not.toThrow();
    for (const overrides of [
      { config_file: "missing.toml" }, { auth_file: "missing.yaml" },
      { config_file: "config.d" }, { auth_file: "auth.d" },
      { config_dir: "config.toml" }, { auth_dir: "auth.yaml" },
      { config_dir: "missing.d" }, { auth_dir: "missing.d" },
    ]) {
      expect(() => validateInputs(parseSuite(input(overrides), directory))).toThrow();
    }
  });

  test("rejects commas in Docker mount source paths, including symlink targets", () => {
    const comma_file = join(directory, "auth,unsafe.yaml");
    writeFileSync(comma_file, "version: 1\naccounts: []\n");
    const comma_dir = join(directory, "auth,unsafe.d");
    mkdirSync(comma_dir);
    symlinkSync(comma_file, join(directory, "linked-auth.yaml"));
    symlinkSync(comma_dir, join(directory, "linked-auth.d"));
    for (const overrides of [
      { auth_file: comma_file }, { auth_dir: comma_dir },
      { auth_file: "linked-auth.yaml" }, { auth_dir: "linked-auth.d" },
    ]) {
      expect(() => validateInputs(parseSuite(input(overrides), directory))).toThrow("cannot contain commas");
    }
  });

  test("accepts default persistence paths with recording enabled in either schema", () => {
    for (const config of [
      "schema_version = 2\n[service.persistence]\nenabled = true\nrecord_sessions = true\n",
      "[db]\nenabled = true\nrecord_sessions = true\n",
    ]) {
      writeFileSync(join(directory, "config.toml"), config);
      expect(() => validateInputs(parseSuite(input(), directory))).not.toThrow();
    }
  });

  test("refuses disabled persistence or session recording in either schema", () => {
    for (const prefix of ["schema_version = 2\n[service.persistence]\n", "[db]\n"]) {
      for (const setting of ["enabled = false", "record_sessions = false"]) {
        writeFileSync(join(directory, "config.toml"), `${prefix}${setting}\n`);
        expect(() => validateInputs(parseSuite(input(), directory))).toThrow("persistence and session recording enabled");
      }
    }
  });

  test("refuses every explicit persistence path override in either schema", () => {
    for (const prefix of ["schema_version = 2\n[service.persistence]\n", "[db]\n"]) {
      for (const field of ["db_path", "usage_db_path", "sessions_db_path", "requests_dir"]) {
        for (const path of ["/host/router/storage", "relative-storage"]) {
          writeFileSync(join(directory, "config.toml"), `${prefix}${field} = "${path}"\n`);
          expect(() => validateInputs(parseSuite(input(), directory))).toThrow(`remove ${field}`);
        }
      }
    }
  });

  test("rejects malformed TOML before creating container resources", () => {
    writeFileSync(join(directory, "config.toml"), 'schema_version = "unterminated\n');
    expect(() => validateInputs(parseSuite(input(), directory))).toThrow();
  });
});
