#!/bin/bash

function main {
  export CARGO_TARGET_DIR="$(base_dir)/target/__compiletests"

  commands=(
    "cd derive_builder_test && rustup run nightly cargo test --test compiletests --features compiletests --color always"
  )

  dev/travis-run-all.sh "${commands[@]}"
}

function base_dir {
  if hash readlink 2>/dev/null; then
    # use readlink, if installed, to follow symlinks
    local __DIR="$(dirname "$(readlink -f "$0")")"
  else
    local __DIR="$(dirname "$0")"
  fi
  echo ${__DIR}
}

# cd into the crate root
(cd "$(base_dir)/.." && main $@)
