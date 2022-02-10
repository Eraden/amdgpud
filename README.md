![GitHub](https://img.shields.io/github/license/Eraden/amdgpud)
[![amdgpud](https://badgen.net/badge/Discord%20Activity/Currently%20Online/green?icon=discord)](https://discord.gg/nCyndSCJ)

# AMD GPU management tools

This repository holds couple tools for AMD graphic cards

* `amdfand` - fan speed daemon
* `amdvold` - voltage and overclocking tool
* `amdmond` - monitor daemon
* `amdguid` - GUI manager
* `amdgui-helper` - daemon with elevated privileges to scan for `amdfand` daemons, reload them and save config files

For more information please check README each of them.

## Roadmap

* [X] Add support for multiple cards
    * Multiple services must recognize card even if there's multiple same version cards is installed
    * Support should be by using `--config` option
* [ ] CLI for fan config edit
* [ ] CLI for voltage edit
* [X] GUI application using native Rust framework (ex. egui, druid)

## License

This work is dual-licensed under Apache 2.0 and MIT. You can choose between one of them if you use this work.
