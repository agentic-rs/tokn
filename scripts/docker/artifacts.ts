import { once } from "node:events";
import { createWriteStream, mkdirSync, readdirSync, rmSync, statSync } from "node:fs";
import { basename, join, resolve } from "node:path";

import { parseTaggedArgs, requireBranch, requirePositiveInteger, requireTag, UsageError } from "./args";
import { namesForTag } from "./containers";
import { container, defaultTag, gatewayArtifactName, gh, run, tmpRoot } from "./runtime";

type PullRequestView = {
  headRefName: string;
};

type WorkflowRun = {
  databaseId: number;
  headBranch: string;
  status: string;
  conclusion: string;
  url: string;
};

type RepoView = {
  nameWithOwner: string;
};

type WorkflowArtifact = {
  archive_download_url: string;
  expired: boolean;
  name: string;
  size_in_bytes: number;
};

type WorkflowArtifactsResponse = {
  artifacts: WorkflowArtifact[];
};

export async function loadImage(args: string[]): Promise<void> {
  const tagged = parseTaggedArgs(args, defaultTag);
  if (tagged.rest.length === 1) {
    loadTar(resolve(tagged.rest[0]), namesForTag(tagged.tag).gatewayImage);
    return;
  }
  if (tagged.rest.length === 2 && tagged.rest[0] === "--pr") {
    const prNumber = requirePositiveInteger(tagged.rest[1], "--pr");
    const targetTag = tagged.tag === defaultTag ? `pr-${prNumber}` : tagged.tag;
    await loadPrImage(prNumber, namesForTag(targetTag).gatewayImage);
    return;
  }
  if (tagged.rest.length === 2 && tagged.rest[0] === "--branch") {
    const branch = requireBranch(tagged.rest[1]);
    const targetTag = tagged.tag === defaultTag ? tagFromBranch(branch) : tagged.tag;
    await loadBranchImage(branch, namesForTag(targetTag).gatewayImage);
    return;
  }
  throw new UsageError("invalid load arguments");
}

function imageRefFromDockerLoad(output: string): string {
  const lines = output.split(/\r?\n/).map((line) => line.trim()).filter(Boolean);
  const loaded = lines
    .map((line) => line.match(/^Loaded image(?:\(s\))?(?: ID)?:\s+(.+)$/)?.[1])
    .filter((value): value is string => Boolean(value));
  const image = loaded.at(-1);
  if (!image) {
    throw new Error("could not find loaded image reference in docker load output");
  }
  return image;
}

function loadTar(tarPath: string, targetImage: string): void {
  console.log(`Loading ${basename(tarPath)}...`);
  const output = container(["load", "-i", tarPath], { capture: true });
  process.stdout.write(output);
  const loadedRef = imageRefFromDockerLoad(output);
  if (loadedRef !== targetImage) {
    container(["tag", loadedRef, targetImage]);
    console.log(`Tagged ${loadedRef} as ${targetImage}`);
  }
}

function tagFromBranch(branch: string): string {
  const tag = branch
    .toLowerCase()
    .replace(/[^a-z0-9_.-]+/g, "-")
    .replace(/^[^a-z0-9]+/, "")
    .replace(/[^a-z0-9]+$/, "");
  if (!tag) {
    throw new Error(`cannot derive a container tag from branch '${branch}'; pass --tag explicitly`);
  }
  return requireTag(tag);
}

async function loadPrImage(prNumber: number, targetImage: string): Promise<void> {
  const pr = JSON.parse(gh(["pr", "view", String(prNumber), "--json", "headRefName"], { capture: true })) as PullRequestView;
  if (!pr.headRefName) {
    throw new Error(`could not resolve head branch for PR #${prNumber}`);
  }

  await loadWorkflowImage(pr.headRefName, `PR #${prNumber} branch ${pr.headRefName}`, `tokn-pr-${prNumber}`, targetImage);
}

async function loadBranchImage(branch: string, targetImage: string): Promise<void> {
  await loadWorkflowImage(branch, `branch ${branch}`, `tokn-branch-${tagFromBranch(branch)}`, targetImage);
}

async function loadWorkflowImage(
  branch: string,
  description: string,
  downloadPrefix: string,
  targetImage: string,
): Promise<void> {
  const runs = JSON.parse(
    gh(
      [
        "run",
        "list",
        "--branch",
        branch,
        "--status",
        "success",
        "--json",
        "databaseId,headBranch,status,conclusion,url",
        "--limit",
        "20",
      ],
      { capture: true },
    ),
  ) as WorkflowRun[];
  const run = runs.find((candidate) => candidate.conclusion === "success");
  if (!run) {
    throw new Error(`could not find a successful workflow run for ${description}`);
  }

  const downloadDir = join(tmpRoot, `${downloadPrefix}-${run.databaseId}`);
  rmSync(downloadDir, { recursive: true, force: true });
  mkdirSync(downloadDir, { recursive: true });
  console.log(`Downloading ${gatewayArtifactName} from ${run.url}...`);
  await downloadWorkflowArtifact(run.databaseId, downloadDir);
  const tarPath = findFirstFile(downloadDir, (path) => path.endsWith(".tar"));
  if (!tarPath) {
    throw new Error(`downloaded artifact did not contain a .tar file under ${downloadDir}`);
  }
  loadTar(tarPath, targetImage);
}

async function downloadWorkflowArtifact(runId: number, downloadDir: string): Promise<void> {
  const repo = JSON.parse(gh(["repo", "view", "--json", "nameWithOwner"], { capture: true })) as RepoView;
  const artifacts = JSON.parse(
    gh(["api", `repos/${repo.nameWithOwner}/actions/runs/${runId}/artifacts`], { capture: true }),
  ) as WorkflowArtifactsResponse;
  const artifact = artifacts.artifacts.find((candidate) => candidate.name === gatewayArtifactName && !candidate.expired);
  if (!artifact) {
    throw new Error(`could not find non-expired ${gatewayArtifactName} artifact for workflow run ${runId}`);
  }

  const zipPath = join(downloadDir, `${gatewayArtifactName}.zip`);
  await downloadWithProgress(artifact.archive_download_url, zipPath, artifact.size_in_bytes);
  run("unzip", ["-q", zipPath, "-d", downloadDir]);
}

async function downloadWithProgress(url: string, destination: string, expectedBytes: number): Promise<void> {
  const token = gh(["auth", "token"], { capture: true }).trim();
  const response = await fetch(url, {
    headers: {
      Accept: "application/vnd.github+json",
      Authorization: `Bearer ${token}`,
      "X-GitHub-Api-Version": "2022-11-28",
    },
  });
  if (!response.ok) {
    throw new Error(`artifact download failed (${response.status} ${response.statusText})`);
  }
  if (!response.body) {
    throw new Error("artifact download response did not include a body");
  }

  const totalBytes = Number(response.headers.get("content-length")) || expectedBytes;
  const reader = response.body.getReader();
  const writer = createWriteStream(destination);
  let receivedBytes = 0;
  let lastRenderedAt = 0;

  const renderProgress = (force: boolean) => {
    const now = Date.now();
    if (!force && now - lastRenderedAt < 250) {
      return;
    }
    lastRenderedAt = now;
    const renderedTotal = totalBytes > 0 ? formatBytes(totalBytes) : "unknown";
    const percent = totalBytes > 0 ? ` ${Math.floor((receivedBytes / totalBytes) * 100)}%` : "";
    process.stderr.write(`\r  ${formatBytes(receivedBytes)} / ${renderedTotal}${percent}`);
  };

  try {
    for (;;) {
      const { done, value } = await reader.read();
      if (done) break;
      receivedBytes += value.byteLength;
      if (!writer.write(value)) {
        await once(writer, "drain");
      }
      renderProgress(false);
    }
    writer.end();
    await once(writer, "finish");
    renderProgress(true);
    process.stderr.write("\n");
  } catch (err) {
    writer.destroy();
    throw err;
  }
}

function formatBytes(bytes: number): string {
  if (bytes < 1024) {
    return `${bytes} B`;
  }
  const units = ["KiB", "MiB", "GiB"];
  let value = bytes / 1024;
  for (const unit of units) {
    if (value < 1024 || unit === units.at(-1)) {
      return `${value.toFixed(value >= 10 ? 1 : 2)} ${unit}`;
    }
    value /= 1024;
  }
  return `${bytes} B`;
}

function findFirstFile(root: string, predicate: (path: string) => boolean): string | undefined {
  for (const entry of readdirSync(root)) {
    const path = join(root, entry);
    const stat = statSync(path);
    if (stat.isDirectory()) {
      const found = findFirstFile(path, predicate);
      if (found) return found;
    } else if (stat.isFile() && predicate(path)) {
      return path;
    }
  }
  return undefined;
}
