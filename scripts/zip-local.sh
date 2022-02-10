cd "$(git rev-parse --show-toplevel)"

echo Building archlinux.zip

rm -Rf ./tmp/*.zip

cp ./target/x86_64-unknown-linux-musl/release/amdfand ./tmp
cp ./target/x86_64-unknown-linux-musl/release/amdmond ./tmp
cp ./target/x86_64-unknown-linux-musl/release/amdvold ./tmp
cp ./target/x86_64-unknown-linux-musl/release/amdgui-helper ./tmp
cp ./target/amdguid-wayland.zip ./tmp
cp ./target/amdguid-glium.zip ./tmp
cp ./target/amdguid-glow.zip ./tmp

cd ./tmp

zip -R archlinux.zip *
