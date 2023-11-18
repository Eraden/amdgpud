#!/usr/bin/env zsh

set -x

ROOT=$(pwd)
AUR_TARGET=${ROOT}/target/aur

mkdir -p ${AUR_TARGET}

function resolveTarget {
  IF [[ "$1" == '' ]]; THEN
    echo "x86_64-unknown-linux-gnu"
  ELSE
    echo "x86_64-unknown-linux-musl"
  FI
}

function handleApplication {
  f=$1
  C=$2
  cd ${ROOT}/crates/${f}

  cargo build --release

  MUSL_CHECK=$(echo "$C" | grep "target = \"x86_64-unknown-linux-musl\"")
  TARGET="$(resolveTarget $MUSL_CHECK)"

  echo TARGET $TARGET

  mkdir -p target/${TARGET}/release
  cp ${ROOT}/target/${TARGET}/release/$f ./target/${TARGET}/release/$f
  cp ${ROOT}/target/${TARGET}/release/$f ./target/release/$f

  if [[ "${MUSL_CHECK}" == "" ]]; then
    # not MUSL
    cargo aur
  else
    # MUSL
    cargo aur --musl
  fi
  mkdir -p ${AUR_TARGET}/${f}
  cp *tar.gz ${AUR_TARGET}
  cp PKGBUILD ${AUR_TARGET}/${f}

  cd ${ROOT}
}

for f in $(ls crates);
do
  C=$(cat crates/$f/Cargo.toml)

  R=$(echo "$C" | grep -E "path = \"./src/main.rs\"")

  if [[ "$R" != "" ]]; then
    handleApplication $f $C
  fi
done
