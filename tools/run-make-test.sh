#!/bin/bash

set -eu

TOOLS_DIR="$(cd "$(dirname "$0")" && pwd)"
TESTS_DIR="$(dirname "$TOOLS_DIR")/tests/run-make"
TMPDIR_BASE="$(mktemp -d)"

tests=$(ls -1F "$TESTS_DIR" | grep /)
tests_count="$(echo "$tests" | wc -l)"
tests_index=1

while IFS= read -r test; do
  testdir="$TESTS_DIR/$test"
  echo "[$tests_index/$tests_count] Running $test"
  (
    cd "$testdir" && \
    tmpdir="$TMPDIR_BASE/$(basename "$testdir")" && \
    mkdir -p "$tmpdir" && \
    make check --silent TMPDIR="$tmpdir"
  )
  tests_index=$((tests_index+1))
done <<< "$tests"
