#!/usr/bin/env zsh

BUILD_TYPE="$1"

set -e +x

ROOT="$(git rev-parse --show-toplevel)"

cd ${ROOT}

rm -Rf ${ROOT}/tmp
mkdir ${ROOT}/tmp

./scripts/compile.sh

strip target/x86_64-unknown-linux-musl/release/amdfand
strip target/x86_64-unknown-linux-musl/release/amdvold
strip target/x86_64-unknown-linux-musl/release/amdmond

#upx --best --lzma target/x86_64-unknown-linux-musl/release/amdfand
#upx --best --lzma target/x86_64-unknown-linux-musl/release/amdvold
#upx --best --lzma target/x86_64-unknown-linux-musl/release/amdmond

function build_and_zip() {
  feature=$1
  zip_name=$2

  cd ${ROOT}
  cargo build --release --target x86_64-unknown-linux-gnu --bin amdguid --no-default-features --features ${feature}
  strip target/x86_64-unknown-linux-gnu/release/amdguid
  #upx --best --lzma target/x86_64-unknown-linux-gnu/release/amdguid
  cp ./target/x86_64-unknown-linux-gnu/release/amdguid ./tmp
  cd ${ROOT}/tmp
  zip ${zip_name}.zip ./amdguid
  cd ${ROOT}
}

if [[ "$BUILD_TYPE" == 'local' ]];
then
  if [[ "$WAYLAND_DISPLAY" == "" ]];
  then
    build_and_zip xorg-glow amdguid-glow
  else
    build_and_zip wayland amdguid-wayland
  fi
else
  build_and_zip xorg-glium amdguid-glium
  build_and_zip xorg-glow amdguid-glow
  build_and_zip wayland amdguid-wayland
fi
