#!/bin/sh
set -eu

export ANTHROPIC_AUTH_TOKEN="${TOKN_AGENT_TEST_API_KEY:?}"
mkdir -p "${CLAUDE_CONFIG_DIR:?}"
exec claude "$@"
