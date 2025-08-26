# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.7.0](https://github.com/joshrotenberg/docker-wrapper/compare/v0.6.0...v0.7.0) - 2025-08-26

### Added

- enhance port mapping API and add convenience aliases ([#152](https://github.com/joshrotenberg/docker-wrapper/pull/152))

## [0.6.0](https://github.com/joshrotenberg/docker-wrapper/compare/v0.5.0...v0.6.0) - 2025-08-25

### Added

- add custom image and platform support to all templates ([#148](https://github.com/joshrotenberg/docker-wrapper/pull/148))

### Other

- *(deps)* bump dtolnay/rust-toolchain ([#149](https://github.com/joshrotenberg/docker-wrapper/pull/149))

## [0.5.0] - 2025-08-25

### Added
- Container templates system with pre-configured defaults for common services
  - Redis templates: Basic, Stack, Cluster, Sentinel, Enterprise, and RedisInsight
  - Database templates: PostgreSQL, MySQL, MongoDB  
  - Web server template: Nginx
- Docker context management commands for remote Docker hosts
- Comprehensive testing documentation and patterns
- Hierarchical template organization with granular feature flags

### Changed
- Improved template architecture for better composability
- Enhanced documentation with template examples

### Fixed
- Template feature gates for conditional compilation

## [0.4.0](https://github.com/joshrotenberg/docker-wrapper/compare/v0.3.0...v0.4.0) (2025-08-23)


### ⚠ BREAKING CHANGES

* Initial public release
* Initial public release
* None - pure test additions
* Remove 'phase' terminology throughout codebase
* Remove volume_tmp method reference from tests
* Context system now uses structured ADRs instead of bespoke markdown files
* Remove competitive analysis from public documentation

### Features

* Add complete Docker Compose command coverage ([#90](https://github.com/joshrotenberg/docker-wrapper/issues/90)) ([7fdf0ec](https://github.com/joshrotenberg/docker-wrapper/commit/7fdf0ecc45598c8c7b95eb69d29ea3b4526b36ef)), closes [#87](https://github.com/joshrotenberg/docker-wrapper/issues/87)
* add comprehensive ContainerManager tests - major coverage improvement ([53f653c](https://github.com/joshrotenberg/docker-wrapper/commit/53f653c421cf41d79a58c84ca069e7a5a385dee1))
* add comprehensive image operations testing infrastructure ([180d553](https://github.com/joshrotenberg/docker-wrapper/commit/180d553569e12e64079081a680cb20f0b8ba586c))
* Add container lifecycle commands (stop, start, restart) ([#32](https://github.com/joshrotenberg/docker-wrapper/issues/32)) ([0406d95](https://github.com/joshrotenberg/docker-wrapper/commit/0406d95a18ae9ff04fd3cbc99611dc6cd263d20f))
* Add debugging and reliability features ([#51](https://github.com/joshrotenberg/docker-wrapper/issues/51)) ([#71](https://github.com/joshrotenberg/docker-wrapper/issues/71)) ([357a3a1](https://github.com/joshrotenberg/docker-wrapper/commit/357a3a130565317cffe959fc4fe118bb94680ab6))
* add dependency management, fix tests, and improve CI caching ([219090a](https://github.com/joshrotenberg/docker-wrapper/commit/219090a18671d45d9922b2b7fdeabb0f18874b6f))
* Add Docker Compose support with feature gating ([#57](https://github.com/joshrotenberg/docker-wrapper/issues/57)) ([d800225](https://github.com/joshrotenberg/docker-wrapper/commit/d80022577b6f68caad7f67b89d3e4e7f604dd3e9)), closes [#36](https://github.com/joshrotenberg/docker-wrapper/issues/36)
* Add Docker network and volume management support ([#64](https://github.com/joshrotenberg/docker-wrapper/issues/64)) ([c69070f](https://github.com/joshrotenberg/docker-wrapper/commit/c69070fa7227708e2820c6f4eca9bb0876c22fb8))
* Add platform detection and runtime abstraction ([#65](https://github.com/joshrotenberg/docker-wrapper/issues/65)) ([ea09fdd](https://github.com/joshrotenberg/docker-wrapper/commit/ea09fdd095f2e50d1523810d55fedae8d3835fc9)), closes [#44](https://github.com/joshrotenberg/docker-wrapper/issues/44)
* add release-please automation and refactor context system ([d2e911f](https://github.com/joshrotenberg/docker-wrapper/commit/d2e911fc095ee6ff27b9948810ccd97aca7a896f))
* Add streaming support for Docker command output ([#60](https://github.com/joshrotenberg/docker-wrapper/issues/60)) ([4642324](https://github.com/joshrotenberg/docker-wrapper/commit/464232457c9056ec876c95d2271316b0fe318d74))
* Add system cleanup and maintenance commands ([#50](https://github.com/joshrotenberg/docker-wrapper/issues/50)) ([#70](https://github.com/joshrotenberg/docker-wrapper/issues/70)) ([5e00e15](https://github.com/joshrotenberg/docker-wrapper/commit/5e00e157e64ce370bcb9707fae997c63c4c62fd3))
* Complete 100% Docker CLI coverage implementation ([#54](https://github.com/joshrotenberg/docker-wrapper/issues/54)) ([b3d1f35](https://github.com/joshrotenberg/docker-wrapper/commit/b3d1f35c680e3ea4d0b11bd3b4c77cb04aff9133))
* complete DockerCommandV2 migration for ALL remaining commands 🎉 ([#112](https://github.com/joshrotenberg/docker-wrapper/issues/112)) ([32b676c](https://github.com/joshrotenberg/docker-wrapper/commit/32b676c3351a6bbc5a23b8328de4cd7b7e89d6a5))
* fix all image operations and enable comprehensive testing ✅ ([e7ab93a](https://github.com/joshrotenberg/docker-wrapper/commit/e7ab93ab83d97defa77653d5c8a66d37686cd0e9))
* implement unified DockerCommand trait pattern ([#102](https://github.com/joshrotenberg/docker-wrapper/issues/102)) ([d7299fd](https://github.com/joshrotenberg/docker-wrapper/commit/d7299fdbd032fce615e1124934a1bf8800818463))
* Migrate build/image commands to DockerCommandV2 ([#108](https://github.com/joshrotenberg/docker-wrapper/issues/108)) ([f0e3d8d](https://github.com/joshrotenberg/docker-wrapper/commit/f0e3d8dffd7a22567348148fcdb16e57de472eb5))
* migrate container management commands to DockerCommandV2 ([#110](https://github.com/joshrotenberg/docker-wrapper/issues/110)) ([d8a0798](https://github.com/joshrotenberg/docker-wrapper/commit/d8a0798bdb2cb6344481052953a75cef452fd1e5))
* migrate container management compose commands to unified pattern ([#104](https://github.com/joshrotenberg/docker-wrapper/issues/104)) ([e54d2f3](https://github.com/joshrotenberg/docker-wrapper/commit/e54d2f35a14ee8d9997ec25d71428fb2c8b6cadd))
* migrate core compose commands to unified pattern ([#103](https://github.com/joshrotenberg/docker-wrapper/issues/103)) ([ec5aaed](https://github.com/joshrotenberg/docker-wrapper/commit/ec5aaedd1d474ace26d146b3893c6fa99d4e00fc)), closes [#94](https://github.com/joshrotenberg/docker-wrapper/issues/94)
* migrate core container commands to DockerCommandV2 ([#106](https://github.com/joshrotenberg/docker-wrapper/issues/106)) ([39d0f68](https://github.com/joshrotenberg/docker-wrapper/commit/39d0f6872f91c88f48ce2b4aed2bea6802b90de8))
* migrate image/container manipulation commands to DockerCommandV2 ([#111](https://github.com/joshrotenberg/docker-wrapper/issues/111)) ([487a0a3](https://github.com/joshrotenberg/docker-wrapper/commit/487a0a31781a4b3394b414cb435f719c1ceeea82))
* migrate registry/auth commands to DockerCommandV2 ([#107](https://github.com/joshrotenberg/docker-wrapper/issues/107)) ([fb3cad6](https://github.com/joshrotenberg/docker-wrapper/commit/fb3cad62da6dfd8dfc785355f679159ae9873423))
* migrate system/info commands to DockerCommandV2 ([#109](https://github.com/joshrotenberg/docker-wrapper/issues/109)) ([39ba990](https://github.com/joshrotenberg/docker-wrapper/commit/39ba990a57672810e79e8c0eda323a120a2f0e4f))
* migrate utility compose commands to unified pattern ([#105](https://github.com/joshrotenberg/docker-wrapper/issues/105)) ([7ae2dce](https://github.com/joshrotenberg/docker-wrapper/commit/7ae2dce4e0fc15e78d6655594f3a3adf2a86ef33))
* significantly improve test coverage and remove phase naming ([a5e0325](https://github.com/joshrotenberg/docker-wrapper/commit/a5e0325b17607837c6b15b7866e78892c4d2af21))


### Bug Fixes

* Add issues permission and skip-labeling for release-please ([#74](https://github.com/joshrotenberg/docker-wrapper/issues/74)) ([a19fb78](https://github.com/joshrotenberg/docker-wrapper/commit/a19fb78cb2250dacb956b18ff5ad292dff4580b2))
* Add missing toolchain parameter to rust-toolchain actions ([#92](https://github.com/joshrotenberg/docker-wrapper/issues/92)) ([79a0f19](https://github.com/joshrotenberg/docker-wrapper/commit/79a0f196e2ba0767c835b4a9a3eb003b47f23708))
* Pin rust-toolchain action to resolve CI failures ([#89](https://github.com/joshrotenberg/docker-wrapper/issues/89)) ([047fa81](https://github.com/joshrotenberg/docker-wrapper/commit/047fa819fad7e74599007ab87b4b8a9005b61cd4))
* remove duplicate ImageRef from types.rs to fix compilation ([78beff5](https://github.com/joshrotenberg/docker-wrapper/commit/78beff5bb79dac03e5a688510f290d0a544436c6))
* remove environment requirement from publish workflow ([#116](https://github.com/joshrotenberg/docker-wrapper/issues/116)) ([2e96b7e](https://github.com/joshrotenberg/docker-wrapper/commit/2e96b7e0c4e317da604a63fbee3780a4ef7b3af1))
* update Cargo.toml example names and fix unused variable warning ([5023ce7](https://github.com/joshrotenberg/docker-wrapper/commit/5023ce761ea5aa4be838eba2547008dcff49f53e))
* Update git-cliff-action to resolve Debian buster repository issues ([#66](https://github.com/joshrotenberg/docker-wrapper/issues/66)) ([51ba6b2](https://github.com/joshrotenberg/docker-wrapper/commit/51ba6b2733e7f60e0eb9f9138256176ce0b53491)), closes [#61](https://github.com/joshrotenberg/docker-wrapper/issues/61)
* Update release-please changelog type ([#29](https://github.com/joshrotenberg/docker-wrapper/issues/29)) ([9ec4185](https://github.com/joshrotenberg/docker-wrapper/commit/9ec418514f93e386fea006709785277331207dc7))
* Update release-please workflow ([#28](https://github.com/joshrotenberg/docker-wrapper/issues/28)) ([345ff3b](https://github.com/joshrotenberg/docker-wrapper/commit/345ff3b475f7b33a13881a526872bfdcd5b65db2))


### Performance Improvements

* Optimize CI workflow runtime ([#72](https://github.com/joshrotenberg/docker-wrapper/issues/72)) ([f86dd9e](https://github.com/joshrotenberg/docker-wrapper/commit/f86dd9eea2fcdbd456e56f960a152ad104ece979))


### Documentation

* add comprehensive Docker feature and test coverage matrix ([26e8d2e](https://github.com/joshrotenberg/docker-wrapper/commit/26e8d2e3fcff606d31accb84058c12b724620e4a))
* Add comprehensive Docker library comparison guide ([#67](https://github.com/joshrotenberg/docker-wrapper/issues/67)) ([eb59742](https://github.com/joshrotenberg/docker-wrapper/commit/eb597422ba50fc34aca00c72bb6589a81b26907a))
* Comprehensive documentation improvements ([#62](https://github.com/joshrotenberg/docker-wrapper/issues/62)) ([63df3b2](https://github.com/joshrotenberg/docker-wrapper/commit/63df3b277d38a6a44a342f9fb1ac8b83a0d8babf))
* create focused test-redis command implementation matrix ([040bb85](https://github.com/joshrotenberg/docker-wrapper/commit/040bb85f824e400ce4e64ea76c5c3b7b22fd639a))
* Fix bollard code examples and imports in comparison guide ([#68](https://github.com/joshrotenberg/docker-wrapper/issues/68)) ([52df4f5](https://github.com/joshrotenberg/docker-wrapper/commit/52df4f57f84835f90bca34b188fbe264a59b5bf7))
* prepare for 0.1.0 release - update dates, remove competitive analysis, reduce emoji usage ([6758490](https://github.com/joshrotenberg/docker-wrapper/commit/6758490abfdb35241d3224a8d2e35347f9565e53))


### Code Refactoring

* unify DockerCommand trait and remove V2 suffix ([#113](https://github.com/joshrotenberg/docker-wrapper/issues/113)) ([288c583](https://github.com/joshrotenberg/docker-wrapper/commit/288c583bad7846f01c03a24f13bad5de9d221ea3))

## [0.3.0](https://github.com/joshrotenberg/docker-wrapper/compare/v0.2.3...v0.3.0) (2025-08-23)


### ⚠ BREAKING CHANGES

* Initial public release
* Initial public release
* None - pure test additions
* Remove 'phase' terminology throughout codebase
* Remove volume_tmp method reference from tests
* Context system now uses structured ADRs instead of bespoke markdown files
* Remove competitive analysis from public documentation

### Features

* Add complete Docker Compose command coverage ([#90](https://github.com/joshrotenberg/docker-wrapper/issues/90)) ([7fdf0ec](https://github.com/joshrotenberg/docker-wrapper/commit/7fdf0ecc45598c8c7b95eb69d29ea3b4526b36ef)), closes [#87](https://github.com/joshrotenberg/docker-wrapper/issues/87)
* add comprehensive ContainerManager tests - major coverage improvement ([53f653c](https://github.com/joshrotenberg/docker-wrapper/commit/53f653c421cf41d79a58c84ca069e7a5a385dee1))
* add comprehensive image operations testing infrastructure ([180d553](https://github.com/joshrotenberg/docker-wrapper/commit/180d553569e12e64079081a680cb20f0b8ba586c))
* Add container lifecycle commands (stop, start, restart) ([#32](https://github.com/joshrotenberg/docker-wrapper/issues/32)) ([0406d95](https://github.com/joshrotenberg/docker-wrapper/commit/0406d95a18ae9ff04fd3cbc99611dc6cd263d20f))
* Add debugging and reliability features ([#51](https://github.com/joshrotenberg/docker-wrapper/issues/51)) ([#71](https://github.com/joshrotenberg/docker-wrapper/issues/71)) ([357a3a1](https://github.com/joshrotenberg/docker-wrapper/commit/357a3a130565317cffe959fc4fe118bb94680ab6))
* add dependency management, fix tests, and improve CI caching ([219090a](https://github.com/joshrotenberg/docker-wrapper/commit/219090a18671d45d9922b2b7fdeabb0f18874b6f))
* Add Docker Compose support with feature gating ([#57](https://github.com/joshrotenberg/docker-wrapper/issues/57)) ([d800225](https://github.com/joshrotenberg/docker-wrapper/commit/d80022577b6f68caad7f67b89d3e4e7f604dd3e9)), closes [#36](https://github.com/joshrotenberg/docker-wrapper/issues/36)
* Add Docker network and volume management support ([#64](https://github.com/joshrotenberg/docker-wrapper/issues/64)) ([c69070f](https://github.com/joshrotenberg/docker-wrapper/commit/c69070fa7227708e2820c6f4eca9bb0876c22fb8))
* Add platform detection and runtime abstraction ([#65](https://github.com/joshrotenberg/docker-wrapper/issues/65)) ([ea09fdd](https://github.com/joshrotenberg/docker-wrapper/commit/ea09fdd095f2e50d1523810d55fedae8d3835fc9)), closes [#44](https://github.com/joshrotenberg/docker-wrapper/issues/44)
* add release-please automation and refactor context system ([d2e911f](https://github.com/joshrotenberg/docker-wrapper/commit/d2e911fc095ee6ff27b9948810ccd97aca7a896f))
* Add streaming support for Docker command output ([#60](https://github.com/joshrotenberg/docker-wrapper/issues/60)) ([4642324](https://github.com/joshrotenberg/docker-wrapper/commit/464232457c9056ec876c95d2271316b0fe318d74))
* Add system cleanup and maintenance commands ([#50](https://github.com/joshrotenberg/docker-wrapper/issues/50)) ([#70](https://github.com/joshrotenberg/docker-wrapper/issues/70)) ([5e00e15](https://github.com/joshrotenberg/docker-wrapper/commit/5e00e157e64ce370bcb9707fae997c63c4c62fd3))
* Complete 100% Docker CLI coverage implementation ([#54](https://github.com/joshrotenberg/docker-wrapper/issues/54)) ([b3d1f35](https://github.com/joshrotenberg/docker-wrapper/commit/b3d1f35c680e3ea4d0b11bd3b4c77cb04aff9133))
* complete DockerCommandV2 migration for ALL remaining commands 🎉 ([#112](https://github.com/joshrotenberg/docker-wrapper/issues/112)) ([32b676c](https://github.com/joshrotenberg/docker-wrapper/commit/32b676c3351a6bbc5a23b8328de4cd7b7e89d6a5))
* fix all image operations and enable comprehensive testing ✅ ([e7ab93a](https://github.com/joshrotenberg/docker-wrapper/commit/e7ab93ab83d97defa77653d5c8a66d37686cd0e9))
* implement unified DockerCommand trait pattern ([#102](https://github.com/joshrotenberg/docker-wrapper/issues/102)) ([d7299fd](https://github.com/joshrotenberg/docker-wrapper/commit/d7299fdbd032fce615e1124934a1bf8800818463))
* Migrate build/image commands to DockerCommandV2 ([#108](https://github.com/joshrotenberg/docker-wrapper/issues/108)) ([f0e3d8d](https://github.com/joshrotenberg/docker-wrapper/commit/f0e3d8dffd7a22567348148fcdb16e57de472eb5))
* migrate container management commands to DockerCommandV2 ([#110](https://github.com/joshrotenberg/docker-wrapper/issues/110)) ([d8a0798](https://github.com/joshrotenberg/docker-wrapper/commit/d8a0798bdb2cb6344481052953a75cef452fd1e5))
* migrate container management compose commands to unified pattern ([#104](https://github.com/joshrotenberg/docker-wrapper/issues/104)) ([e54d2f3](https://github.com/joshrotenberg/docker-wrapper/commit/e54d2f35a14ee8d9997ec25d71428fb2c8b6cadd))
* migrate core compose commands to unified pattern ([#103](https://github.com/joshrotenberg/docker-wrapper/issues/103)) ([ec5aaed](https://github.com/joshrotenberg/docker-wrapper/commit/ec5aaedd1d474ace26d146b3893c6fa99d4e00fc)), closes [#94](https://github.com/joshrotenberg/docker-wrapper/issues/94)
* migrate core container commands to DockerCommandV2 ([#106](https://github.com/joshrotenberg/docker-wrapper/issues/106)) ([39d0f68](https://github.com/joshrotenberg/docker-wrapper/commit/39d0f6872f91c88f48ce2b4aed2bea6802b90de8))
* migrate image/container manipulation commands to DockerCommandV2 ([#111](https://github.com/joshrotenberg/docker-wrapper/issues/111)) ([487a0a3](https://github.com/joshrotenberg/docker-wrapper/commit/487a0a31781a4b3394b414cb435f719c1ceeea82))
* migrate registry/auth commands to DockerCommandV2 ([#107](https://github.com/joshrotenberg/docker-wrapper/issues/107)) ([fb3cad6](https://github.com/joshrotenberg/docker-wrapper/commit/fb3cad62da6dfd8dfc785355f679159ae9873423))
* migrate system/info commands to DockerCommandV2 ([#109](https://github.com/joshrotenberg/docker-wrapper/issues/109)) ([39ba990](https://github.com/joshrotenberg/docker-wrapper/commit/39ba990a57672810e79e8c0eda323a120a2f0e4f))
* migrate utility compose commands to unified pattern ([#105](https://github.com/joshrotenberg/docker-wrapper/issues/105)) ([7ae2dce](https://github.com/joshrotenberg/docker-wrapper/commit/7ae2dce4e0fc15e78d6655594f3a3adf2a86ef33))
* significantly improve test coverage and remove phase naming ([a5e0325](https://github.com/joshrotenberg/docker-wrapper/commit/a5e0325b17607837c6b15b7866e78892c4d2af21))


### Bug Fixes

* Add issues permission and skip-labeling for release-please ([#74](https://github.com/joshrotenberg/docker-wrapper/issues/74)) ([a19fb78](https://github.com/joshrotenberg/docker-wrapper/commit/a19fb78cb2250dacb956b18ff5ad292dff4580b2))
* Add missing toolchain parameter to rust-toolchain actions ([#92](https://github.com/joshrotenberg/docker-wrapper/issues/92)) ([79a0f19](https://github.com/joshrotenberg/docker-wrapper/commit/79a0f196e2ba0767c835b4a9a3eb003b47f23708))
* Pin rust-toolchain action to resolve CI failures ([#89](https://github.com/joshrotenberg/docker-wrapper/issues/89)) ([047fa81](https://github.com/joshrotenberg/docker-wrapper/commit/047fa819fad7e74599007ab87b4b8a9005b61cd4))
* remove duplicate ImageRef from types.rs to fix compilation ([78beff5](https://github.com/joshrotenberg/docker-wrapper/commit/78beff5bb79dac03e5a688510f290d0a544436c6))
* remove environment requirement from publish workflow ([#116](https://github.com/joshrotenberg/docker-wrapper/issues/116)) ([2e96b7e](https://github.com/joshrotenberg/docker-wrapper/commit/2e96b7e0c4e317da604a63fbee3780a4ef7b3af1))
* update Cargo.toml example names and fix unused variable warning ([5023ce7](https://github.com/joshrotenberg/docker-wrapper/commit/5023ce761ea5aa4be838eba2547008dcff49f53e))
* Update git-cliff-action to resolve Debian buster repository issues ([#66](https://github.com/joshrotenberg/docker-wrapper/issues/66)) ([51ba6b2](https://github.com/joshrotenberg/docker-wrapper/commit/51ba6b2733e7f60e0eb9f9138256176ce0b53491)), closes [#61](https://github.com/joshrotenberg/docker-wrapper/issues/61)
* Update release-please changelog type ([#29](https://github.com/joshrotenberg/docker-wrapper/issues/29)) ([9ec4185](https://github.com/joshrotenberg/docker-wrapper/commit/9ec418514f93e386fea006709785277331207dc7))
* Update release-please workflow ([#28](https://github.com/joshrotenberg/docker-wrapper/issues/28)) ([345ff3b](https://github.com/joshrotenberg/docker-wrapper/commit/345ff3b475f7b33a13881a526872bfdcd5b65db2))


### Performance Improvements

* Optimize CI workflow runtime ([#72](https://github.com/joshrotenberg/docker-wrapper/issues/72)) ([f86dd9e](https://github.com/joshrotenberg/docker-wrapper/commit/f86dd9eea2fcdbd456e56f960a152ad104ece979))


### Documentation

* add comprehensive Docker feature and test coverage matrix ([26e8d2e](https://github.com/joshrotenberg/docker-wrapper/commit/26e8d2e3fcff606d31accb84058c12b724620e4a))
* Add comprehensive Docker library comparison guide ([#67](https://github.com/joshrotenberg/docker-wrapper/issues/67)) ([eb59742](https://github.com/joshrotenberg/docker-wrapper/commit/eb597422ba50fc34aca00c72bb6589a81b26907a))
* Comprehensive documentation improvements ([#62](https://github.com/joshrotenberg/docker-wrapper/issues/62)) ([63df3b2](https://github.com/joshrotenberg/docker-wrapper/commit/63df3b277d38a6a44a342f9fb1ac8b83a0d8babf))
* create focused test-redis command implementation matrix ([040bb85](https://github.com/joshrotenberg/docker-wrapper/commit/040bb85f824e400ce4e64ea76c5c3b7b22fd639a))
* Fix bollard code examples and imports in comparison guide ([#68](https://github.com/joshrotenberg/docker-wrapper/issues/68)) ([52df4f5](https://github.com/joshrotenberg/docker-wrapper/commit/52df4f57f84835f90bca34b188fbe264a59b5bf7))
* prepare for 0.1.0 release - update dates, remove competitive analysis, reduce emoji usage ([6758490](https://github.com/joshrotenberg/docker-wrapper/commit/6758490abfdb35241d3224a8d2e35347f9565e53))


### Code Refactoring

* unify DockerCommand trait and remove V2 suffix ([#113](https://github.com/joshrotenberg/docker-wrapper/issues/113)) ([288c583](https://github.com/joshrotenberg/docker-wrapper/commit/288c583bad7846f01c03a24f13bad5de9d221ea3))

## [0.2.3](https://github.com/joshrotenberg/docker-wrapper/compare/v0.2.2...v0.2.3) (2025-08-23)


### Bug Fixes

* Add issues permission and skip-labeling for release-please ([#74](https://github.com/joshrotenberg/docker-wrapper/issues/74)) ([a19fb78](https://github.com/joshrotenberg/docker-wrapper/commit/a19fb78cb2250dacb956b18ff5ad292dff4580b2))

## [0.2.2](https://github.com/joshrotenberg/docker-wrapper/compare/v0.2.1...v0.2.2) (2025-08-22)


### Features

* Add debugging and reliability features ([#51](https://github.com/joshrotenberg/docker-wrapper/issues/51)) ([#71](https://github.com/joshrotenberg/docker-wrapper/issues/71)) ([357a3a1](https://github.com/joshrotenberg/docker-wrapper/commit/357a3a130565317cffe959fc4fe118bb94680ab6))
* Add system cleanup and maintenance commands ([#50](https://github.com/joshrotenberg/docker-wrapper/issues/50)) ([#70](https://github.com/joshrotenberg/docker-wrapper/issues/70)) ([5e00e15](https://github.com/joshrotenberg/docker-wrapper/commit/5e00e157e64ce370bcb9707fae997c63c4c62fd3))


### Performance Improvements

* Optimize CI workflow runtime ([#72](https://github.com/joshrotenberg/docker-wrapper/issues/72)) ([f86dd9e](https://github.com/joshrotenberg/docker-wrapper/commit/f86dd9eea2fcdbd456e56f960a152ad104ece979))


### Documentation

* Fix bollard code examples and imports in comparison guide ([#68](https://github.com/joshrotenberg/docker-wrapper/issues/68)) ([52df4f5](https://github.com/joshrotenberg/docker-wrapper/commit/52df4f57f84835f90bca34b188fbe264a59b5bf7))

## [0.2.1](https://github.com/joshrotenberg/docker-wrapper/compare/v0.2.0...v0.2.1) (2025-08-22)


### Features

* Add container lifecycle commands (stop, start, restart) ([#32](https://github.com/joshrotenberg/docker-wrapper/issues/32)) ([0406d95](https://github.com/joshrotenberg/docker-wrapper/commit/0406d95a18ae9ff04fd3cbc99611dc6cd263d20f))
* Add Docker Compose support with feature gating ([#57](https://github.com/joshrotenberg/docker-wrapper/issues/57)) ([d800225](https://github.com/joshrotenberg/docker-wrapper/commit/d80022577b6f68caad7f67b89d3e4e7f604dd3e9)), closes [#36](https://github.com/joshrotenberg/docker-wrapper/issues/36)
* Add Docker network and volume management support ([#64](https://github.com/joshrotenberg/docker-wrapper/issues/64)) ([c69070f](https://github.com/joshrotenberg/docker-wrapper/commit/c69070fa7227708e2820c6f4eca9bb0876c22fb8))
* Add platform detection and runtime abstraction ([#65](https://github.com/joshrotenberg/docker-wrapper/issues/65)) ([ea09fdd](https://github.com/joshrotenberg/docker-wrapper/commit/ea09fdd095f2e50d1523810d55fedae8d3835fc9)), closes [#44](https://github.com/joshrotenberg/docker-wrapper/issues/44)
* Add streaming support for Docker command output ([#60](https://github.com/joshrotenberg/docker-wrapper/issues/60)) ([4642324](https://github.com/joshrotenberg/docker-wrapper/commit/464232457c9056ec876c95d2271316b0fe318d74))
* Complete 100% Docker CLI coverage implementation ([#54](https://github.com/joshrotenberg/docker-wrapper/issues/54)) ([b3d1f35](https://github.com/joshrotenberg/docker-wrapper/commit/b3d1f35c680e3ea4d0b11bd3b4c77cb04aff9133))


### Bug Fixes

* Update git-cliff-action to resolve Debian buster repository issues ([#66](https://github.com/joshrotenberg/docker-wrapper/issues/66)) ([51ba6b2](https://github.com/joshrotenberg/docker-wrapper/commit/51ba6b2733e7f60e0eb9f9138256176ce0b53491)), closes [#61](https://github.com/joshrotenberg/docker-wrapper/issues/61)


### Documentation

* Add comprehensive Docker library comparison guide ([#67](https://github.com/joshrotenberg/docker-wrapper/issues/67)) ([eb59742](https://github.com/joshrotenberg/docker-wrapper/commit/eb597422ba50fc34aca00c72bb6589a81b26907a))
* Comprehensive documentation improvements ([#62](https://github.com/joshrotenberg/docker-wrapper/issues/62)) ([63df3b2](https://github.com/joshrotenberg/docker-wrapper/commit/63df3b277d38a6a44a342f9fb1ac8b83a0d8babf))

## [0.2.0](https://github.com/joshrotenberg/docker-wrapper/compare/v0.1.0...v0.2.0) (2025-07-27)


### ⚠ BREAKING CHANGES

* Initial public release
* Initial public release
* None - pure test additions
* Remove 'phase' terminology throughout codebase
* Remove volume_tmp method reference from tests
* Context system now uses structured ADRs instead of bespoke markdown files
* Remove competitive analysis from public documentation

### Features

* add comprehensive ContainerManager tests - major coverage improvement ([53f653c](https://github.com/joshrotenberg/docker-wrapper/commit/53f653c421cf41d79a58c84ca069e7a5a385dee1))
* add comprehensive image operations testing infrastructure ([180d553](https://github.com/joshrotenberg/docker-wrapper/commit/180d553569e12e64079081a680cb20f0b8ba586c))
* add dependency management, fix tests, and improve CI caching ([219090a](https://github.com/joshrotenberg/docker-wrapper/commit/219090a18671d45d9922b2b7fdeabb0f18874b6f))
* add release-please automation and refactor context system ([d2e911f](https://github.com/joshrotenberg/docker-wrapper/commit/d2e911fc095ee6ff27b9948810ccd97aca7a896f))
* fix all image operations and enable comprehensive testing ✅ ([e7ab93a](https://github.com/joshrotenberg/docker-wrapper/commit/e7ab93ab83d97defa77653d5c8a66d37686cd0e9))
* significantly improve test coverage and remove phase naming ([a5e0325](https://github.com/joshrotenberg/docker-wrapper/commit/a5e0325b17607837c6b15b7866e78892c4d2af21))


### Bug Fixes

* remove duplicate ImageRef from types.rs to fix compilation ([78beff5](https://github.com/joshrotenberg/docker-wrapper/commit/78beff5bb79dac03e5a688510f290d0a544436c6))
* update Cargo.toml example names and fix unused variable warning ([5023ce7](https://github.com/joshrotenberg/docker-wrapper/commit/5023ce761ea5aa4be838eba2547008dcff49f53e))
* Update release-please changelog type ([#29](https://github.com/joshrotenberg/docker-wrapper/issues/29)) ([9ec4185](https://github.com/joshrotenberg/docker-wrapper/commit/9ec418514f93e386fea006709785277331207dc7))
* Update release-please workflow ([#28](https://github.com/joshrotenberg/docker-wrapper/issues/28)) ([345ff3b](https://github.com/joshrotenberg/docker-wrapper/commit/345ff3b475f7b33a13881a526872bfdcd5b65db2))


### Documentation

* add comprehensive Docker feature and test coverage matrix ([26e8d2e](https://github.com/joshrotenberg/docker-wrapper/commit/26e8d2e3fcff606d31accb84058c12b724620e4a))
* create focused test-redis command implementation matrix ([040bb85](https://github.com/joshrotenberg/docker-wrapper/commit/040bb85f824e400ce4e64ea76c5c3b7b22fd639a))
* prepare for 0.1.0 release - update dates, remove competitive analysis, reduce emoji usage ([6758490](https://github.com/joshrotenberg/docker-wrapper/commit/6758490abfdb35241d3224a8d2e35347f9565e53))

## [Unreleased]

### Added
- Initial release of docker-wrapper
- Complete container lifecycle management
- Comprehensive image operations with registry support
- Advanced network management with custom drivers
- Full volume management with multiple backends
- Real-time Docker event monitoring and streaming
- Live container statistics with historical aggregation
- Type-safe APIs with builder patterns
- Production-ready error handling and resource cleanup

### Features by Phase

#### Phase 1: Foundation
- Docker client with automatic binary detection
- Process executor with async streaming support
- Core type system with newtype wrappers
- Comprehensive error handling with thiserror
- Basic container operations (run, stop, remove)

#### Phase 2: Container Lifecycle Management
- Advanced container builder with fluent API
- Environment variable and volume mounting
- Network attachment and resource limits
- Container execution with streaming I/O
- Health checking with multiple strategies
- Log streaming with filtering options
- Port management with dynamic allocation

#### Phase 3: Image, Network & Volume Management
- **Image Management**:
  - Image pulling with progress tracking
  - Image building from Dockerfiles
  - Image tagging, inspection, and removal
  - Registry authentication support
  - Image history and export/import operations

- **Network Management**:
  - Network creation with multiple drivers (bridge, overlay, macvlan)
  - Container network connection/disconnection
  - IPAM configuration with custom subnets
  - Network inspection and cleanup operations

- **Volume Management**:
  - Volume creation with multiple drivers
  - Volume mounting specifications
  - Volume inspection and cleanup
  - Usage statistics and batch operations

#### Phase 4: Advanced Features & Monitoring
- **Event Monitoring**:
  - Real-time Docker event streaming
  - Comprehensive event filtering (type, time, labels)
  - Container lifecycle event handling
  - Event waiting patterns for synchronization

- **Statistics Monitoring**:
  - Real-time container resource metrics
  - CPU, memory, network, and disk I/O tracking
  - Historical data aggregation with time windows
  - Resource threshold monitoring and alerts
  - System-wide Docker statistics

### Technical Achievements
- Zero unsafe code with memory safety guarantees
- Async-first design with tokio integration
- Streaming architecture with bounded memory usage
- Type-safe error handling with comprehensive context
- Resource cleanup automation with RAII patterns

## [0.1.0] - 2025-07-24

### Added
- Initial public release
- Complete Docker ecosystem management
- Production-ready monitoring capabilities
- Comprehensive documentation and examples

---

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for information on how to contribute to this project.
