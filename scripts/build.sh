set -e +x

cd "$(git rev-parse --show-toplevel)"

./scripts/compile.sh

strip target/x86_64-unknown-linux-musl/release/amdfand
strip target/x86_64-unknown-linux-musl/release/amdvold
strip target/x86_64-unknown-linux-musl/release/amdmond

#upx --best --lzma target/x86_64-unknown-linux-musl/release/amdfand
#upx --best --lzma target/x86_64-unknown-linux-musl/release/amdvold
#upx --best --lzma target/x86_64-unknown-linux-musl/release/amdmond

cargo build --release --target x86_64-unknown-linux-gnu --bin amdguid --no-default-features --features xorg-glium
strip target/x86_64-unknown-linux-gnu/release/amdguid
#upx --best --lzma target/x86_64-unknown-linux-gnu/release/amdguid
zip ./target/amdguid-glium.zip ./target/x86_64-unknown-linux-gnu/release/amdguid

cargo build --release --target x86_64-unknown-linux-gnu --bin amdguid --no-default-features --features xorg-glow
strip target/x86_64-unknown-linux-gnu/release/amdguid
#upx --best --lzma target/x86_64-unknown-linux-gnu/release/amdguid
zip ./target/amdguid-glow.zip ./target/x86_64-unknown-linux-gnu/release/amdguid

cargo build --release --target x86_64-unknown-linux-gnu --bin amdguid --no-default-features --features wayland
strip target/x86_64-unknown-linux-gnu/release/amdguid
#upx --best --lzma target/x86_64-unknown-linux-gnu/release/amdguid
zip ./target/amdguid-wayland.zip ./target/x86_64-unknown-linux-gnu/release/amdguid
