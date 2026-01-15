# Changelog

## [2.8.0](https://github.com/jolars/tomat/compare/v2.7.0...v2.8.0) (2026-01-15)

### Features

* add `sound.mode` option ([66b537a](https://github.com/jolars/tomat/commit/66b537a80a6adce43be1a75b0fff91fa2725276a))
* add hooks for end of states ([494c8be](https://github.com/jolars/tomat/commit/494c8be4b7f299514e8a8d4018c3c5f7b0eae99a))
* make 0 timeout for hooks mean no timeout ([9c4514a](https://github.com/jolars/tomat/commit/9c4514afe6486aad645e4af705971272871ca7bb))

### Bug Fixes

* await resume for hooks if auto advancing ([15ba349](https://github.com/jolars/tomat/commit/15ba3497162af5c943ef70f8f2a0220f88cbe6c2))
* execute skip hooks before phase transition ([9ffc487](https://github.com/jolars/tomat/commit/9ffc487f5e652620ac1b62d5acf4878d16111abb))
* remove dead `on_complete` hook code ([ae61f6e](https://github.com/jolars/tomat/commit/ae61f6ebd2cbb3d7fa24f06f345594ce351f6a40))
* track cli reference and exclude from cargo build ([4addeaa](https://github.com/jolars/tomat/commit/4addeaa1d1edd58645fc4754e35d9e6059d2d8a4))

## [2.7.0](https://github.com/jolars/tomat/compare/v2.6.0...v2.7.0) (2026-01-13)

### Features

* add granular options for auto advancing ([80d7f31](https://github.com/jolars/tomat/commit/80d7f31bde1ee83b6da1d4b0c0bbc8c0357b4211))
* add support for customizing notification urgency ([337ebd1](https://github.com/jolars/tomat/commit/337ebd17372ec5aac99e6aa0b18ac1649c7957c6))
* allow customization of notification messages ([feb3b13](https://github.com/jolars/tomat/commit/feb3b131237cc1365bf7948f9bd8a91333e3178e))

### Performance Improvements

* reduce size of sound alerts ([755b791](https://github.com/jolars/tomat/commit/755b791a12733de600326ee9e475cf49d908ea4b))

## [2.6.0](https://github.com/jolars/tomat/compare/v2.5.0...v2.6.0) (2026-01-09)

### Features

* add a system for calling hooks upon state changes ([0573d57](https://github.com/jolars/tomat/commit/0573d57eacad41d12a8891cff595711fea49704c))
* reduce default `tomat watch` interval to 0.25s ([5735a89](https://github.com/jolars/tomat/commit/5735a8926aabebdd2f9a1e729da8fc34a3ee41ff))

### Bug Fixes

* execute hook on work start ([bf0b375](https://github.com/jolars/tomat/commit/bf0b3753e229361689ffc4ab66815eaccc386a52))

## [2.5.0](https://github.com/jolars/tomat/compare/v2.4.0...v2.5.0) (2025-11-29)

### Features

* add shell completion ([#6](https://github.com/jolars/tomat/issues/6)) ([2c7b8e9](https://github.com/jolars/tomat/commit/2c7b8e9ee8b5042554b50db3c11d3d286b03ba09))

### Bug Fixes

* work around race conditions for spawning servers ([c5b9113](https://github.com/jolars/tomat/commit/c5b9113e9afc5bfcd836b27cb474b4e4372348ef))

### Performance Improvements

* reduce startup delay in server verification ([0e02b9d](https://github.com/jolars/tomat/commit/0e02b9de950064cb35fe32b6bb0b868ec7e4b876))

## [2.4.0](https://github.com/jolars/tomat/compare/v2.3.0...v2.4.0) (2025-11-04)

### Features

* add `tomato watch` command ([fd45b67](https://github.com/jolars/tomat/commit/fd45b67d1b4d8d715017e887411d6f27a5af909e))
* add format option ([2422eae](https://github.com/jolars/tomat/commit/2422eaeba066c7d5d192a066e45f325ed8fdb7be))

### Bug Fixes

* correctly get status output when paused ([6006947](https://github.com/jolars/tomat/commit/60069475e61fa7a6a8ece6b67148092cb5dcb699))

## [2.3.0](https://github.com/jolars/tomat/compare/v2.2.0...v2.3.0) (2025-10-31)

### Features

* add `tomat daemon install/uninstall` ([f414020](https://github.com/jolars/tomat/commit/f4140205b2d21bc55474b81975fad1417a3f8fe8))

### Bug Fixes

* bump version in Cargo.toml ([6c90065](https://github.com/jolars/tomat/commit/6c900657cd52472a22103d38af49527c4d6218cb))
* fix linting error ([61568b3](https://github.com/jolars/tomat/commit/61568b30b270cc5e837edf7b151cb4a24d2ef9f3))

## [2.2.0](https://github.com/jolars/tomat/compare/v2.1.0...v2.2.0) (2025-10-31)

### Features

* add `--output` option ([8ccda87](https://github.com/jolars/tomat/commit/8ccda87a4f112384cef26b02d03fea875de2adfb))
* add `i3status-rs` to output formats ([0515bd9](https://github.com/jolars/tomat/commit/0515bd994c22d120cf1faaf3c4a03cec70c6e78a))
* change default timer duration to 5 seconds ([5d96283](https://github.com/jolars/tomat/commit/5d96283bb153b222a1acf1a5f9db73a272320c68))

### Bug Fixes

* fix lint errors ([94b4175](https://github.com/jolars/tomat/commit/94b41756677d7a4fc1f17b43c850561238944d48))

## [2.1.0](https://github.com/jolars/tomat/compare/v2.0.0...v2.1.0) (2025-10-30)

### Features

* add icon and enable it in notifications ([70c047a](https://github.com/jolars/tomat/commit/70c047a20ab08744300ebec2f072a59a15e7bb87))
* change summary field in notification to "Tomato" ([e9e7c70](https://github.com/jolars/tomat/commit/e9e7c7024776f1123471f47f384ef6926ada0c98))
* **config:** add configuration section for notifications ([ff7f8ce](https://github.com/jolars/tomat/commit/ff7f8ce7b4f2852ee7704bb3c0d91c074ab7d87c))
* update logo ([ee3559d](https://github.com/jolars/tomat/commit/ee3559db697a7d13ab10dcfd6d66402f9ad61b64))

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
