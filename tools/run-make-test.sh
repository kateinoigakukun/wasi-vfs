#!/bin/bash

set -eu

TOOLS_DIR="$(cd "$(dirname "$0")" && pwd)"
TESTS_DIR="$(dirname "$TOOLS_DIR")/tests/run-make"
TMPDIR_BASE="$(mktemp -d)"

tests=$(ls -1F "$TESTS_DIR" | grep /)
tests_count="$(echo "$tests" | wc -l | awk '{ print $1 }')"
tests_count_width="$(printf $tests_count | wc -c | awk '{ print $1 }')"
tests_index=1

fail=0

while IFS= read -r test; do
  testdir="$TESTS_DIR/$test"
  printf "[%${tests_count_width}s/%${tests_count_width}s] Running $test " "$tests_index" "$tests_count"
  set +e
  (
    cd "$testdir" && \
    tmpdir="$TMPDIR_BASE/$(basename "$testdir")" && \
    mkdir -p "$tmpdir" && \
    make check --silent TMPDIR="$tmpdir"
  )
  status_code="$?"
  set -e
  if [[ "$status_code" -eq 0 ]]; then
    printf "[\e[32mOK\e[0m]\n"
  else
    printf "[\e[31mFAILED\e[0m]\n"
    fail=1
  fi
  tests_index=$((tests_index+1))
done <<< "$tests"

if [[ "$fail" -eq 0 ]]; then
  printf "\e[32mAll tests passed!\e[0m\n"
else
  printf "\e[31mSome tests failed!\e[0m\n"
  exit 1
fi
