#!/bin/bash

function main {
  export CARGO_TARGET_DIR="../target/__nightlytests"

  commands=(
    "cd derive_builder && rustup run nightly cargo test --all --color always --features \"skeptic_tests nightlytests\""
  )

  dev/travis-run-all.sh "${commands[@]}"
}

function base_dir {
  if [ uname -s != "Darwin" ] -a hash readlink 2>/dev/null; then
    # use readlink, if installed, to follow symlinks
    local __DIR="$(dirname "$(readlink -f "$0")")"
  else
    local __DIR="$(dirname "$0")"
  fi
  echo ${__DIR}
}

# cd into the crate root
(cd "$(base_dir)/.." && main $@)
