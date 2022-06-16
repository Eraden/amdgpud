#!/usr/bin/env zsh

version=$1

if [[ "$version" == "" ]];
then
  echo "VERSION is needed"
  exit 1
fi

set -e +x

cd "$(git rev-parse --show-toplevel)"

ROOT="$(git rev-parse --show-toplevel)"

echo Building archlinux.tar.gz

./scripts/build.sh

build_tag_gz() {
  zip_name=$1
  cd ${ROOT}
  for name in $*; do
    cp ${ROOT}/target/x86_64-unknown-linux-musl/release/${name} ${ROOT}/tmp
    cp ${ROOT}/services/${name}.service ./tmp

    cd ${ROOT}/tmp
    tar -cvf ${zip_name}-${version}.tar.gz ${name}.service ${name}
    cd ${ROOT}
  done

  cd ${ROOT}/tmp
  for name in $*; do
    rm ${name}.service ${name}
  done
  cd ${ROOT}
}

tar_gui() {
  tar_name=$1

  cd ${ROOT}/tmp
  unzip ${tar_name}.zip
  rm ${tar_name}.zip

  cp ${ROOT}/target/x86_64-unknown-linux-musl/release/amdgui-helper ${ROOT}/tmp
  cp ${ROOT}/services/amdgui-helper.service ${ROOT}/tmp
  tar -cvf ${tar_name}-${version}.tar.gz amdgui-helper amdguid amdgui-helper.service

  rm amdgui-helper
  rm amdgui-helper.service
  rm amdguid
}

build_tag_gz amdfand
build_tag_gz amdmond
build_tag_gz amdvold

tar_gui amdguid-wayland
tar_gui amdguid-glium
tar_gui amdguid-glow

cd ${ROOT}/tmp

for f in $(ls *.tar.gz); do
  md5sum $f
done

tar -cvf archlinux-${version}.tar.gz *.tar.gz
