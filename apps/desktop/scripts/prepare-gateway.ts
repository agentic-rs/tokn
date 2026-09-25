import { execFileSync } from "node:child_process";
import { copyFileSync, mkdirSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { resolve } from "node:path";
const root = fileURLToPath(new URL("../../../", import.meta.url));
const release = process.argv.includes("--release");
const target = execFileSync("rustc", ["--print", "host-tuple"], {
  encoding: "utf8",
}).trim();
if (!target.includes("apple-darwin"))
  throw new Error("This first desktop release supports macOS only.");
const metadata = JSON.parse(
  execFileSync("cargo", ["metadata", "--no-deps", "--format-version", "1"], {
    cwd: root,
    encoding: "utf8",
  }),
);
execFileSync(
  "cargo",
  [
    "build",
    "--locked",
    "-p",
    "tokn-gateway-cli",
    "--bin",
    "tokn-gateway",
    ...(release ? ["--release"] : []),
  ],
  { cwd: root, stdio: "inherit" },
);
const destination = resolve(root, "apps/desktop/src-tauri/binaries");
mkdirSync(destination, { recursive: true });
copyFileSync(
  resolve(
    metadata.target_directory,
    release ? "release" : "debug",
    "tokn-gateway",
  ),
  resolve(destination, `tokn-gateway-${target}`),
);
