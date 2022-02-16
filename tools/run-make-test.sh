#!/bin/bash

set -eu

TOOLS_DIR="$(cd "$(dirname "$0")" && pwd)"
TESTS_DIR="$(dirname "$TOOLS_DIR")/tests/run-make"
TMPDIR_BASE="$(mktemp -d)"

for testdir in "$TESTS_DIR"/*; do
    if [ -d "$testdir" ]; then
        pushd "$testdir"
        tmpdir="$TMPDIR_BASE/$(basename "$testdir")"
        mkdir -p "$tmpdir"
        make check TMPDIR="$tmpdir"
        popd
    fi
done
