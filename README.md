![GitHub](https://img.shields.io/github/license/Eraden/amdgpud)

# AMD GPU management tools

This repository holds couple tools for AMD graphic cards

* `amdfand` - fan speed daemon
* `amdvold` - voltage and overclocking tool
* `amdmond` - monitor daemon

For more information please check README each of them.

## Roadmap

* [ ] Add support for multiple cards
    * Multiple services must recognize card even if there's multiple same version cards is installed
    * Support should be by using `--config` option
* [ ] CLI for fan config edit
* [ ] CLI for voltage edit
* [ ] GUI application using native Rust framework (ex. egui, druid)

## :bookmark: License

This work is dual-licensed under Apache 2.0 and MIT. You can choose between one of them if you use this work.
