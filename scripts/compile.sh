set -e +x

cd "$(git rev-parse --show-toplevel)"

cargo build --release --target x86_64-unknown-linux-musl --bin amdfand
cargo build --release --target x86_64-unknown-linux-musl --bin amdmond
cargo build --release --target x86_64-unknown-linux-musl --bin amdvold
cargo build --release --target x86_64-unknown-linux-musl --bin amdgui-helper
cargo build --release --target x86_64-unknown-linux-gnu --bin amdguid --no-default-features --features xorg
