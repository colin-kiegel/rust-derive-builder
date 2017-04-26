#!/bin/bash

function main {
  export CARGO_TARGET_DIR="../target/__nightlytests"

  commands=(
    "cd derive_builder_core  && cargo clippy -- -Dclippy"
    "cd derive_builder_macro && cargo clippy -- -Dclippy"
    "cd derive_builder       && cargo clippy -- -Dclippy"
    "cd testsuite            && cargo clippy -- -Dclippy"
    "cd derive_builder_core  && cargo fmt -- --write-mode diff"
    "cd derive_builder_macro && cargo fmt -- --write-mode diff"
    "cd derive_builder       && cargo fmt -- --write-mode diff"
    "cd testsuite            && cargo fmt -- --write-mode diff"
  )

  dev/travis-run-all.sh "${commands[@]}"
}

function base_dir {
  if [ $(uname -s) != "Darwin" ] && hash readlink 2>/dev/null; then
    # use readlink, if installed, to follow symlinks
    local __DIR="$(dirname "$(readlink -f "$0")")"
  else
    local __DIR="$(dirname "$0")"
  fi
  echo ${__DIR}
}

# cd into the crate root
(cd "$(base_dir)/.." && main $@)
