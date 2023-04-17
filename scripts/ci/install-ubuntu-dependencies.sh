#!/usr/bin/env bash

ln -fs /usr/share/zoneinfo/America/New_York /etc/localtime

apt-get update
DEBIAN_FRONTEND=noninteractive apt-get install -y tzdata
dpkg-reconfigure --frontend noninteractive tzdata
apt-get install --yes curl gnupg clang gcc cmake build-essential git python3 zip tar wget
cp /usr/bin/clang /usr/bin/cc
cp /usr/bin/python3 /usr/bin/python
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs -o /tmp/install-rustup
chmod +x /tmp/install-rustup
/tmp/install-rustup -y

export PATH=$PATH:~/.cargo/bin

wget -O - https://apt.kitware.com/keys/kitware-archive-latest.asc 2>/dev/null | gpg --dearmor - | tee /usr/share/keyrings/kitware-archive-keyring.gpg >/dev/null
if [[ "${VERSION}" == "18" ]]; then
    echo 'deb [signed-by=/usr/share/keyrings/kitware-archive-keyring.gpg] https://apt.kitware.com/ubuntu/ bionic main' | tee /etc/apt/sources.list.d/kitware.list >/dev/null || echo 1
fi

apt-key adv --keyserver keyserver.ubuntu.com --recv-keys 42D5A192B819C5DA || echo 0
apt-get update || echo 0
apt-get install --yes upx-ucl xcb libxcb-shape0 libxcb-xfixes0 libxcb-record0 libxcb-shape0-dev libxcb-xfixes0-dev libxcb-record0-dev || echo 0
