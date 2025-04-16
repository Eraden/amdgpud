./scripts/build.sh

sudo cp ./target/x86_64-unknown-linux-musl/release/amdfand /usr/bin
sudo cp ./target/x86_64-unknown-linux-musl/release/amdmond /usr/bin
sudo cp ./target/x86_64-unknown-linux-musl/release/amdgui-helper /usr/bin
sudo cp ./target/x86_64-unknown-linux-gnu/release/agc /usr/bin

sudo cp services/amdfand.service /usr/lib/systemd/system/amdfand.service
sudo cp services/amdmond.service /usr/lib/systemd/system/amdmond.service
sudo cp services/amdgui-helper.service /usr/lib/systemd/system/amdgui-helper.service

sudo systemctl enable --now amdgui-helper
sudo systemctl enable --now amdfand
sudo systemctl enable --now amdmond
