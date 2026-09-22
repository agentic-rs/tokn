#!/bin/sh
set -eu

mkdir -p "${DSH_HOME:?}"
cp /trial/settings.yaml "$DSH_HOME/settings.yaml"
exec dsh "$@"
