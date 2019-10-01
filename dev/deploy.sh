#!/bin/bash

function main {
  export CARGO_TARGET_DIR="../.deploy/"

  if [ -z "$CRATES_IO_TOKEN" ]; then
    echo "EE: The variable \`CRATES_IO_TOKEN\` must be set!"
    exit 1;
  fi

  (cd derive_builder_core && rustup run nightly cargo publish --token "$CRATES_IO_TOKEN")

  # Wait for `derive_builder_core` to be available on crates.io before publishing `derive_builder`
  sleep 10

  (cd derive_builder      && rustup run nightly cargo publish --token "$CRATES_IO_TOKEN")
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
