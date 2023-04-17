# amdgui-helper

Daemon with elevated privileges to scan for `amdfand` daemons, reload them and save config files

You can communicate with it using sock file `/tmp/amdgui-helper.sock` using helper `Command` from `amdgpu`.

Each connection is single use and will be terminated after sending `Response`.
