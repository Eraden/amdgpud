![GitHub](https://img.shields.io/github/license/Eraden/amdgpud)

# AMDGPU Fan control service

Available commands:

* `service`        - Set fan speed depends on GPU temperature
* `set-automatic`  - Switch to GPU automatic fan speed control
* `set-manual`     - Switch to GPU manual fan speed control
* `available`      - Print available cards

#### amdfand set-automatic | set-manual [OPTIONS]

Optional arguments:

* -h, --help Help message
* -c, --card CARD GPU Card number

## Usage

```bash
cargo install amdfand

sudo amdfand monitor # print current temperature, current fan speed, min and max fan speed 
sudo amdfand service # check amdgpu temperature and adjust speed from config file 
```

## Config file

```toml
# /etc/amdfand/config.toml
log_level = "Error"
temp_input = "temp1_input"
update_rate = 4000 # time between checks in milliseconds

# GPU temperature to fan speed matrix
[[temp_matrix]]
temp = 4.0
speed = 4.0

[[temp_matrix]]
temp = 30.0
speed = 33.0

[[temp_matrix]]
temp = 45.0
speed = 50.0

[[temp_matrix]]
temp = 60.0
speed = 66.0

[[temp_matrix]]
temp = 65.0
speed = 69.0

[[temp_matrix]]
temp = 70.0
speed = 75.0

[[temp_matrix]]
temp = 75.0
speed = 89.0

[[temp_matrix]]
temp = 80.0
speed = 100.0

# GPU usage to fan speed matrix
[[usage_matrix]]
usage = 30.0
speed = 34.0

[[usage_matrix]]
usage = 65.0
speed = 60.0
```
