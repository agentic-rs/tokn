#!/bin/sh
set -eu

mkdir -p "${DSH_HOME:?}"
cp /agent-test/settings.yaml "$DSH_HOME/settings.yaml"
exec dsh "$@"
