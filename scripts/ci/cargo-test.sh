set -ex

PATH=$PATH:$HOME/.cargo/bin
rustup default nightly
rustup update
rustup component add rustfmt
rustup target install x86_64-unknown-linux-musl
cargo fmt -- --check
cargo test --release

for p in $(ls crates);
do
    if [[ "$p" != "amdgpu" && "$p" != "amdgui" ]]; then
        echo "Testing $p"
        rm -Rf target
        cargo test --release -p $p
    fi
done
