# Changelog

## [2.0.0](https://github.com/jolars/tomat/compare/v1.0.0...v2.0.0) (2025-10-29)

### ⚠ BREAKING CHANGES

* change `break-time` to `break` in cli and config

### Features

* change `break-time` to `break` in cli and config ([ef325fc](https://github.com/jolars/tomat/commit/ef325fce879110399537bfa3eadfaf5671b00a2d))
* enable sound notifications ([bd28eb0](https://github.com/jolars/tomat/commit/bd28eb0bce434d384d414a687d84aeaaef8d30d5))
* implement pause and resume commands ([f63662b](https://github.com/jolars/tomat/commit/f63662bc56625c2861b2b3e9bb889663d716a9d8))
* improve timer precision ([4cd6c16](https://github.com/jolars/tomat/commit/4cd6c1685cbfbf5943bcee137799701649f721ce))

### Bug Fixes

* avoid socket race condition ([eefe788](https://github.com/jolars/tomat/commit/eefe788f6843646beaf8d52b3053f7b230bde177))
* don't play audio notifications during testing ([253df25](https://github.com/jolars/tomat/commit/253df253925b779fdd615407b4498ef6ae88320f))
* make timer persist after restarts ([b0706c4](https://github.com/jolars/tomat/commit/b0706c43e3c196455cc16a5da0d885d4ae7e52b2))
* preserve timer progress across pause/resume cycles ([333b03d](https://github.com/jolars/tomat/commit/333b03dba08981a5928073c8f003e61360931267))
* remove unused arguments from `tomat toggle` ([53c3065](https://github.com/jolars/tomat/commit/53c3065590cec7f265da07109251ccba806cdb1c))

## 1.0.0 (2025-10-29)

### Features

* add daemon, service file, and binary ([baea76c](https://github.com/jolars/tomat/commit/baea76c4070405388c0df5a787db82844ff6e3ec))
* add notifications via `notify-rust` ([435e8dc](https://github.com/jolars/tomat/commit/435e8dc14d5bd41942cc9af0810b6c1bf071c5b2))
* add support for configuration through a TOML file ([a3cb56d](https://github.com/jolars/tomat/commit/a3cb56d01ebb9e45bb1628c71a0c6caba94de6ee))
* add toggle subcommand ([17f03c8](https://github.com/jolars/tomat/commit/17f03c8e9bb44cbca89ae8ed976cf9ccdd03d50c))
* allow floats in durations ([b61372f](https://github.com/jolars/tomat/commit/b61372fca2a8fa7e6569e50035bfed776ef15dda))
* enable full daemon control ([68df527](https://github.com/jolars/tomat/commit/68df527a06ab9e257bbcc74f891198d1e45d9a28))
* setup basic package infrastructure ([20401a8](https://github.com/jolars/tomat/commit/20401a8393d47e65d5d6b22d62cdf3c68b152613))
* support long breaks ([91d544b](https://github.com/jolars/tomat/commit/91d544bbbc99ad788e45898780fbf94b6a668287))

### Bug Fixes

* add AppKit framework for macos ([9220354](https://github.com/jolars/tomat/commit/9220354bca90178e1c408fd15fdab1fc2c46a9c9))
* flush to writer ([493f1f0](https://github.com/jolars/tomat/commit/493f1f00e115a7e427be194422d6c7003555a764))
