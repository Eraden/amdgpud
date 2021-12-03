#!/usr/bin/env zsh

git_root() { echo "$(git rev-parse --show-toplevel)" }

cd $(git_root)/amdgpu
cargo publish

cd $(git_root)/amdgpu-config
cargo publish

cd $(git_root)/amdfand
cargo publish

cd $(git_root)/amdvold
cargo publish

cd $(git_root)/amdmond
cargo publish
