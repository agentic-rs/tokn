#!/bin/sh
set -eu

mkdir -p "${PI_CODING_AGENT_DIR:?}"
cp /trial/models.json "$PI_CODING_AGENT_DIR/models.json"
exec pi "$@"
