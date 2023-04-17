local aur_template(appName, desc, ver, checksum, keywords, optdepends, pkg) =
  '# Maintainer Adrian Woźniak <adrian.wozniak@ita-prog.pl>\n' +
  '\n' +
  'pkgbase=' + appName + '\n' +
  'pkgname=' + appName + '-bin' +
  'pkgver=' + ver + '\n' +
  'pkgrel=1\n' +
  'pkgdesc=' + desc + '\n' +
  'url="https://github.com/Eraden/amdgpud"\n' +
  "license=('MIT' 'Apache-2.0')\n" +
  'source=( "https://github.com/Eraden/amdgpud/releases/download/' + pkg + '")\n' +
  "arch=('x86_64')\n" +
  'md5sums=( ' + checksum + ' )\n' +
  "keywords=( 'amdgpu' " + keywords + ')\n' +
  'optdepends=(' + optdepends + ')\n' +
  'conflicts=()\n' +
  'provides=()\n' +
  'build() { tar -xvf $srcdir/' + pkg + ' }\n' +
  'package() {' +
  '    cd $srcdir/\n' +
  '    mkdir -p $pkgdir/usr/bin/\n' +
  '    mkdir -p $pkgdir/usr/lib/systemd/system/\n' +
  '    for f in $(ls); do\n' +
  "        if [[ \"${f}\" =~ *\".service\' ]]; then\n" +
  '            install -m 0755 $srcdir/$f "$pkgdir/usr/lib/systemd/system/"\n' +
  '        else\n' +
  '            install -m 0755 $srcdir/$f $pkgdir/usr/bin\n' +
  '        fi\n' +
  '    done\n' +
  '}\n' +
  '#vim: syntax=sh\n';

local upload(target) = {
  name: 'upload',
  image: 'plugins/s3',
  environment: {
    ACCESS: { from_secret: 'access_key' },
    SECRET: { from_secret: 'secret_key' },
  },
  settings: {
    bucket: 'build',
    access_key: { from_secret: 'access_key' },
    secret_key: { from_secret: 'secret_key' },
    endpoint: 'https://minio.ita-prog.pl',
    source: 'build/*',
    target: target,
    path_style: true,
    debug: true,
  },
};

local gh_release() = {
  name: 'github-release',
  image: 'plugins/github-release',
  settings: {
    api_key: { from_secret: 'gh_secret_key' },
    files: 'build/*',
  },
  when: {
    event: 'tag',
  },
};

local restore_cache() = {
  name: 'restore-cache',
  image: 'meltwater/drone-cache:dev',
  pull: true,
  settings: {
    'access-key': { from_secret: 'access_key' },
    'secret-key': { from_secret: 'secret_access' },
    restore: true,
    'path-style': true,
    debug: true,
    bucket: 'drone-cache',
    region: 'eu-west-1',
    mount: ['target'],
  },
};

local rebuild_cache() = {
  name: 'rebuild-cache',
  image: 'meltwater/drone-cache:dev',
  pull: true,
  settings: {
    'access-key': { from_secret: 'access_key' },
    'secret-key': { from_secret: 'secret_access' },
    rebuild: true,
    'path-style': true,
    debug: true,
    bucket: 'drone-cache',
    region: 'eu-west-1',
    mount: ['target'],
  },
};

local buildGUIPkg(image, version, appName) = [
  'bash scripts/ci/build-gui-pkg ' + image + ' ' + version + ' ' + appName,
];

local buildDaemonPkg(image, version, appName) = [
  'bash scripts/ci/build-daemon-pkg ' + image + ' ' + version + ' ' + appName,
];

local UbuntuPipeline(image, version) = {
  kind: 'pipeline',
  name: 'ubuntu_' + version,
  steps: [
    restore_cache(),
    {
      name: 'test',
      image: 'ubuntu:' + image,
      environment: {
        AWS_ACCESS_KEY_ID: { from_secret: 'access_key' },
        AWS_SECRET_ACCESS_KEY: { from_secret: 'secret_access' },
        SCCACHE_ENDPOINT: 'https://minio.ita-prog.pl',
        SCCACHE_BUCKET: 'archlinux-cache',
      },
      commands: [
        'VERSION=' + version + ' ./scripts/ci/install-ubuntu-dependencies.sh',
        './scripts/ci/cargo-test.sh',
      ],
    },
    {
      name: 'build',
      image: 'ubuntu:' + image,
      environment: {
        AWS_ACCESS_KEY_ID: { from_secret: 'access_key' },
        AWS_SECRET_ACCESS_KEY: { from_secret: 'secret_access' },
        SCCACHE_ENDPOINT: 'https://minio.ita-prog.pl',
        SCCACHE_BUCKET: 'archlinux-cache',
      },
      commands: std.flattenArrays([
        [
          'VERSION=' + version + ' ./scripts/ci/install-ubuntu-dependencies.sh',
          'mkdir -p build',
          './scripts/ci/cargo-build.sh ubuntu ' + version,
        ],
        buildGUIPkg('ubuntu', version, 'amdguid-glium'),
        buildGUIPkg('ubuntu', version, 'amdguid-glow'),
        buildGUIPkg('ubuntu', version, 'amdguid-wayland'),
        buildDaemonPkg('ubuntu', version, 'amdmond'),
        buildDaemonPkg('ubuntu', version, 'amdvold'),
        buildDaemonPkg('ubuntu', version, 'amdfand'),
      ]),
    },
    rebuild_cache(),
    upload('/ubuntu-' + version),
    gh_release(),
  ],
};

local archlinuxDeps() = [
  'bash ./scripts/ci/install-archlinux-dependencies.sh',
  'rustup default nightly',
];

local ArchLinuxPipeline() = {
  kind: 'pipeline',
  name: 'archlinux',
  steps: [
    restore_cache(),
    {
      name: 'test',
      image: 'archlinux:latest',
      environment: {
        AWS_ACCESS_KEY_ID: { from_secret: 'access_key' },
        AWS_SECRET_ACCESS_KEY: { from_secret: 'secret_access' },
        SCCACHE_ENDPOINT: 'https://minio.ita-prog.pl',
        SCCACHE_BUCKET: 'archlinux-cache',
      },
      commands: std.flattenArrays([
        archlinuxDeps(),
        ['./scripts/ci/cargo-test.sh'],
      ]),
    },
    {
      name: 'build',
      image: 'archlinux:latest',
      environment: {
        AWS_ACCESS_KEY_ID: { from_secret: 'access_key' },
        AWS_SECRET_ACCESS_KEY: { from_secret: 'secret_access' },
        SCCACHE_ENDPOINT: 'https://minio.ita-prog.pl',
        SCCACHE_BUCKET: 'archlinux-cache',
      },
      commands: std.flattenArrays([
        archlinuxDeps(),
        [
          'mkdir -p build',
          './scripts/ci/cargo-build.sh archlinux latest',
        ],
        buildGUIPkg('archlinux', 'latest', 'amdguid-glium'),
        buildGUIPkg('archlinux', 'latest', 'amdguid-glow'),
        buildGUIPkg('archlinux', 'latest', 'amdguid-wayland'),
        buildDaemonPkg('archlinux', 'latest', 'amdmond'),
        buildDaemonPkg('archlinux', 'latest', 'amdvold'),
        buildDaemonPkg('archlinux', 'latest', 'amdfand'),
      ]),
    },
    rebuild_cache(),
    upload('/archlinux'),
    gh_release(),
  ],
};

[
  UbuntuPipeline('18.04', '18'),
  UbuntuPipeline('20.04', '20'),
  ArchLinuxPipeline(),
]
