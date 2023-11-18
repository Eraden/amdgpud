PATH=$PATH:$HOME/.cargo/bin
rustup default nightly
rustup update
rustup component add rustfmt
rustup target install x86_64-unknown-linux-musl
cargo fmt -- --check
cargo test
