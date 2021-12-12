# AMD graphic card voltage manager

This tool can be used to overclock you AMD graphic card on Linux

## Install

```bash
cargo install amdvold
```

## Usage

Available commands:

* `setup-info` - prints information how to enable voltage management on Linux (see Requirements) 
* `print-states` - prints current card states
* `change-state` - change card voltage states
* `apply-changes` - apply changes

## Changing states

Positional arguments:
* `index`                    Profile number
* `module`                   Either memory or engine
* `frequency`                New GPU module frequency
* `voltage`                  New GPU module voltage

Optional arguments:
* `-a`, `--apply-immediately`  Apply changes immediately after change

Example:

```bash
amdvold 1 engine 1450MHz 772mV
```

## Requirements

To enable AMD GPU voltage manipulation kernel parameter must be added, please do one of the following:

* In GRUB add to "GRUB_CMDLINE_LINUX_DEFAULT" following text "amdgpu.ppfeaturemask=0xffffffff", example:

  GRUB_CMDLINE_LINUX_DEFAULT="loglevel=3 cryptdevice=/dev/nvme0n1p3:cryptroot amdgpu.ppfeaturemask=0xffffffff psi=1"

  Easiest way is to modify "/etc/default/grub" and generate new grub config.

* If you have hooks enabled add in "/etc/modprobe.d/amdgpu.conf" to "options" following text "amdgpu.ppfeaturemask=0xffffffff", example:

  options amdgpu si_support=1 cik_support=1 vm_fragment_size=9 audio=0 dc=0 aspm=0 ppfeaturemask=0xffffffff

  (only "ppfeaturemask=0xffffffff" is required and if you don't have "options amdgpu" you can just add "options amdgpu ppfeaturemask=0xffffffff")