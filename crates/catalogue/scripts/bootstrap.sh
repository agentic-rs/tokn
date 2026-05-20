#!/usr/bin/env bash
set -euo pipefail

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
repo_root="$(cd "$script_dir/../../.." && pwd)"
output="$repo_root/crates/catalogue/vendor/models.dev.json"
tmp="$output.tmp"

providers='[
  "openai",
  "deepseek",
  "github-copilot",
  "zai",
  "zai-coding-plan",
  "zhipuai",
  "zhipuai-coding-plan",
  "anthropic",
  "google",
  "alibaba",
  "minimax"
]'

mkdir -p "$(dirname "$output")"

curl -fsSL https://models.dev/api.json \
  | jq -c --argjson providers "$providers" '
      with_entries(select(.key as $k | $providers | index($k)))
    ' \
  > "$tmp"

mv "$tmp" "$output"
