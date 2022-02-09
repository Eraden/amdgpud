echo Building binaries-$1.zip

zip binaries-$1.zip ./target/x86_64-unknown-linux-musl/release/amdfand;
zip binaries-$1.zip ./target/x86_64-unknown-linux-musl/release/amdmond;
zip binaries-$1.zip ./target/x86_64-unknown-linux-musl/release/amdvold;
zip binaries-$1.zip ./target/x86_64-unknown-linux-musl/release/amdgui-helper;
zip binaries-$1.zip ./target/amdguid-wayland.zip;
zip binaries-$1.zip ./target/amdguid-glium.zip
