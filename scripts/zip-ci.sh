echo Building binaries for $1

cd "$(git rev-parse --show-toplevel)"

ROOT="$(git rev-parse --show-toplevel)"

./scripts/build.sh

function build_tag_gz() {
  zip_name=$1
  cd ${ROOT}
  for name in $*; do
    cp ${ROOT}/target/x86_64-unknown-linux-musl/release/${name} ${ROOT}/tmp
    cp ${ROOT}/services/${name}.service ./tmp

    cd ${ROOT}/tmp
    tar -cvf ${zip_name}.tar.gz ${name}.service ${name}
    cd ${ROOT}
  done

  cd ${ROOT}/tmp
  for name in $*; do
    rm ${name}.service ${name}
  done
  cd ${ROOT}
}

function tar_gui() {
  tar_name=$1

  cd ${ROOT}/tmp
  unzip ${tar_name}.zip

  cp ${ROOT}/target/x86_64-unknown-linux-musl/release/amdgui-helper ${ROOT}/tmp
  cp ${ROOT}/services/amdgui-helper.service ${ROOT}/tmp
  tar -cvf ${tar_name}.tar.gz amdgui-helper amdguid amdgui-helper.service
}

build_tag_gz amdfand
build_tag_gz amdmond
build_tag_gz amdvold

tar_gui agc

cd ${ROOT}/tmp

for f in $(ls *.tar.gz); do
  md5sum $f
done

cd ${ROOT}/tmp

zip -R ${1}.zip *.tar.gz

#for file in $(ls *.tar.gz);
#do
#  mv $file ${ROOT}/${1}-$file
#done
