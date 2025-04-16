#!/usr/bin/env zsh

rustup default nightly

set -e +x

ROOT="$(git rev-parse --show-toplevel)"

cd ${ROOT}

rm -Rf ${ROOT}/tmp
mkdir -p ${ROOT}/tmp

./scripts/compile.sh

strip target/x86_64-unknown-linux-musl/release/amdfand
strip target/x86_64-unknown-linux-musl/release/amdvold
strip target/x86_64-unknown-linux-musl/release/amdmond

function build() {
  zip_name=$1

  cd ${ROOT}

  strip target/x86_64-unknown-linux-gnu/release/$zip_name
  cp ./target/x86_64-unknown-linux-gnu/release/$zip_name ./tmp/amdguid

  cd ${ROOT}/tmp
  zip ${zip_name}.zip ./amdguid
  cd ${ROOT}
}

build agc
