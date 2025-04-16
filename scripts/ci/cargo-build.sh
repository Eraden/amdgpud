#!/usr/bin/env bash

set -eux

OS=$1
VER=$2

export PATH=$PATH:$HOME/.cargo/bin

rustup default nightly
rustup target install x86_64-unknown-linux-musl
rustup update

# export SCCACHE_ENDPOINT=https://minio.ita-prog.pl:443
# export SCCACHE_BUCKET=${OS}-cache
# export RUSTC_WRAPPER=$(which sccache)
# export AWS_PROFILE=minio
unset RUSTC_WRAPPER

cargo metadata --format-version 1 --no-deps | jq -S -M -c ".packages | map({ version: .version, name: .name })" > metadata.json

cargo build --release --bin amdmond
cargo build --release --bin amdfand
cargo build --release --bin amdports
cargo build --release --bin amdgui-helper

if command -v cargo-chef
then
    cargo chef cook --release --recipe-path recipe.json --bin agc
    cargo chef cook --release --recipe-path recipe.json --bin amdvold
else
    cargo build --release --bin agc
    cargo build --release --bin amdvold
fi

ls -al
ls -al target
