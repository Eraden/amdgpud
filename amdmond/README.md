# AMD Monitoring daemon

## Watch mode

Tool will check temperature and prints:

* Minimal modulation
* Maximal modulation
* Current modulation
* Current fan speed in percentage (PWD / PWD MAX * 100) 
* Current value of each temperature sensor (typically temp1_input is which should be observed)

> `modulation` is a value between 0-255 which indicate how fast fan should be moving

```bash
/usr/bin/amdmond watch --format short
```

### Formats

There are 2 possible formats.

* `short` - very compact info
* `long` - more human-readable info 

## Log File mode

This tool can be used to track GPU temperature and amdfand speed curve management to prevent GPU card from generating
unnecessary noise.

It will create csv log file with:

* time
* temperature
* card modulation
* matrix point temperature
* matrix point speed

```bash
/usr/bin/amdmond log_file -s /var/log/amdmon.csv
```

## Install

```bash
cargo install amdmond
```

## Usage

### minimal:

```
amdmond log_file -s /var/log/amdmon.csv
```

Required arguments:

* `-s`, `--stat-file STAT-FILE`  Full path to statistics file

Optional arguments:

* `-h`, `--help`                 Help message
* `-v`, `--version`              Print version
* `-c`, `--config CONFIG`        Config location
* `-i`, `--interval INTERVAL`    Time between each check. 1000 is 1s, by default 5s
