![GitHub](https://img.shields.io/github/license/Eraden/amdgpud)

# AMDGPU Fan control service

Available commands:

* `monitor`        - Print current temp and fan speed
* `service`        - Set fan speed depends on GPU temperature
* `set-automatic`  - Switch to GPU automatic fan speed control
* `set-manual`     - Switch to GPU manual fan speed control
* `available`      - Print available cards

#### amdfand set-automatic | set-manual [OPTIONS]

Optional arguments:

* -h, --help       Help message
* -c, --card CARD  GPU Card number

## Usage

```bash
cargo install argonfand

sudo argonfand monitor # print current temperature, current fan speed, min and max fan speed 
sudo argonfand service # check amdgpu temperature and adjust speed from config file 
```

## Config file

```toml
# /etc/amdfand/config.toml
log_level = "Error"
temp_input = "temp1_input"

[[speed_matrix]]
temp = 4.0
speed = 4.0

[[speed_matrix]]
temp = 30.0
speed = 33.0

[[speed_matrix]]
temp = 45.0
speed = 50.0

[[speed_matrix]]
temp = 60.0
speed = 66.0

[[speed_matrix]]
temp = 65.0
speed = 69.0

[[speed_matrix]]
temp = 70.0
speed = 75.0

[[speed_matrix]]
temp = 75.0
speed = 89.0

[[speed_matrix]]
temp = 80.0
speed = 100.0
```

## Roadmap

* [ ] Write metrics
* [ ] Add support for multiple cards
  * Multiple services must recognize card even if there's multiple same version cards is installed
  * Support should be by using `--config` option
* [ ] CLI for fan config edit
* [ ] CLI for voltage edit
* [ ] GUI application using native Rust framework (ex. egui, druid)

## :bookmark: License

This work is dual-licensed under Apache 2.0 and MIT.
You can choose between one of them if you use this work.
