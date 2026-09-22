#!/bin/sh
set -eu

mkdir -p "${PI_CODING_AGENT_DIR:?}"
cp /agent-test/models.json "$PI_CODING_AGENT_DIR/models.json"
exec pi "$@"
