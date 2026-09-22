import { existsSync, readFileSync, realpathSync, statSync } from "node:fs";
import { homedir } from "node:os";
import { basename, dirname, extname, resolve } from "node:path";

import { parseCases } from "./cases";
import type { AgentTestCase } from "./agents/types";

export type AgentTestSuite = {
  schema_version: 1;
  gateway_image: string;
  config_file: string;
  auth_file: string;
  config_dir?: string;
  auth_dir?: string;
  router_url: string;
  serve_args: string[];
  timeout_secs: number;
  agent_images: Record<string, string>;
  cases: AgentTestCase[];
};

function object(value: unknown, label: string): Record<string, unknown> {
  if (!value || typeof value !== "object" || Array.isArray(value)) throw new Error(`${label} must be an object`);
  return value as Record<string, unknown>;
}

function string(value: unknown, label: string): string {
  if (typeof value !== "string" || !value.trim() || value.includes("\0")) throw new Error(`${label} must be a nonempty string`);
  return value;
}

function localPath(value: string, base_dir: string): string {
  return resolve(base_dir, value.startsWith("~/") ? resolve(homedir(), value.slice(2)) : value);
}

export function parseSuite(value: unknown, base_dir: string): AgentTestSuite {
  const input = object(value, "suite");
  const fields = new Set(["schema_version", "gateway_image", "config_file", "auth_file", "config_dir", "auth_dir", "router_url", "serve_args", "timeout_secs", "agent_images", "cases"]);
  for (const field of Object.keys(input)) if (!fields.has(field)) throw new Error(`unknown suite field: ${field}`);
  if (input.schema_version !== 1) throw new Error("suite schema_version must be 1");
  const config_file = localPath(string(input.config_file, "config_file"), base_dir);
  const auth_file = localPath(string(input.auth_file ?? resolve(dirname(config_file), "auth.yaml"), "auth_file"), base_dir);
  const router_url = string(input.router_url ?? "http://127.0.0.1:4141", "router_url");
  const url = new URL(router_url);
  if (url.protocol !== "http:" || !["127.0.0.1", "localhost", "[::1]"].includes(url.hostname) || url.username || url.password || url.search || url.hash || url.pathname !== "/") {
    throw new Error("router_url must be an HTTP loopback origin inside the gateway container");
  }
  const serve_args = input.serve_args ?? [];
  if (!Array.isArray(serve_args) || !serve_args.every((arg) => typeof arg === "string" && ["--no-proxy", "--with-proxy"].includes(arg))) {
    throw new Error("serve_args currently supports only --no-proxy and --with-proxy; listener policy belongs in config");
  }
  if (new Set(serve_args).size > 1) throw new Error("serve_args cannot enable and disable the proxy together");
  const timeout_secs = input.timeout_secs ?? 120;
  if (!Number.isInteger(timeout_secs) || Number(timeout_secs) < 1 || Number(timeout_secs) > 3600) throw new Error("timeout_secs must be an integer from 1 to 3600");
  const agent_images = Object.fromEntries(Object.entries(object(input.agent_images ?? {}, "agent_images")).map(([key, value]) => [key, string(value, `agent_images.${key}`)]));
  const stem = basename(config_file, extname(config_file));
  const default_config_dir = resolve(dirname(config_file), `${stem}.d`);
  const optionalDir = (value: unknown, fallback: string, label: string) => value === undefined
    ? existsSync(fallback) ? fallback : undefined
    : localPath(string(value, label), base_dir);
  return {
    schema_version: 1,
    gateway_image: string(input.gateway_image, "gateway_image"),
    config_file, auth_file,
    config_dir: optionalDir(input.config_dir, default_config_dir, "config_dir"),
    auth_dir: optionalDir(input.auth_dir, resolve(dirname(auth_file), "auth.d"), "auth_dir"),
    router_url: url.origin,
    serve_args,
    timeout_secs: Number(timeout_secs),
    agent_images,
    cases: parseCases(input.cases),
  };
}

export function loadSuite(path: string): AgentTestSuite {
  return parseSuite(JSON.parse(readFileSync(path, "utf8")), dirname(resolve(path)));
}

export function validateInputs(suite: AgentTestSuite): void {
  for (const path of [suite.config_file, suite.auth_file]) {
    if (!statSync(path).isFile()) throw new Error(`expected an input file: ${path}`);
    if (realpathSync(path).includes(",")) throw new Error("Docker bind mount paths cannot contain commas");
  }
  for (const path of [suite.config_dir, suite.auth_dir].filter((path): path is string => Boolean(path))) {
    if (!statSync(path).isDirectory()) throw new Error(`expected an input directory: ${path}`);
    if (realpathSync(path).includes(",")) throw new Error("Docker bind mount paths cannot contain commas");
  }
  const config = Bun.TOML.parse(readFileSync(suite.config_file, "utf8")) as Record<string, unknown>;
  const service = (config.service ?? {}) as Record<string, unknown>;
  const logging = (config.schema_version === 2 ? service.logging : config.logging) as Record<string, unknown> | undefined;
  if (logging?.target === "file") throw new Error("agent tests require logging.target = stderr or both to verify persistence shutdown");
  const persistence = (config.schema_version === 2 ? service.persistence : config.db) as Record<string, unknown> | undefined;
  if (persistence?.enabled === false || persistence?.record_sessions === false) throw new Error("agent tests require persistence and session recording enabled");
  // Export names must match the private volume. Host-specific absolute paths
  // cannot be silently reused inside the container.
  for (const field of ["db_path", "usage_db_path", "sessions_db_path", "requests_dir"]) {
    if (persistence?.[field] !== undefined) throw new Error(`agent-test config must use default persistence paths; remove ${field} in a dedicated config`);
  }
}
