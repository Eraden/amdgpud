#!/usr/bin/env zsh

cargo build --release
strip target/x86_64-unknown-linux-musl/release/amdfand
upx --best --lzma target/x86_64-unknown-linux-musl/release/amdfand
