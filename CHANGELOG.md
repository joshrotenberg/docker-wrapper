# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).
## [Unreleased]

## [0.10.0](https://github.com/joshrotenberg/docker-wrapper/compare/v0.9.1...v0.10.0) - 2026-01-05

### Added

- enhance RedisClusterTemplate for CI and hybrid environments ([#211](https://github.com/joshrotenberg/docker-wrapper/pull/211))

### Fixed

- use SIGKILL and StopCommand in lifecycle example ([#212](https://github.com/joshrotenberg/docker-wrapper/pull/212))
- use serial_test for env var tests and improve MySQL readiness check ([#213](https://github.com/joshrotenberg/docker-wrapper/pull/213))
- complete template feature flag coverage and add CI verification ([#206](https://github.com/joshrotenberg/docker-wrapper/pull/206))
- template-redis-enterprise feature gate and example API updates ([#204](https://github.com/joshrotenberg/docker-wrapper/pull/204))

### Bug Fixes

- Use which crate for cross-platform Docker binary lookup  ([d17edce](https://github.com/joshrotenberg/docker-wrapper/commit/d17edceb456668dc8855c13fd0e7a17551040d83))
- Improve template reliability on slower systems  ([766a3dd](https://github.com/joshrotenberg/docker-wrapper/commit/766a3dd5a17ee5a021a814e485565204e74ad428))
- Handle IMAGE header in docker images output  ([8a0cdb4](https://github.com/joshrotenberg/docker-wrapper/commit/8a0cdb4ff67963f12381cd059313a5f8c4a58bb1))

### Documentation

- Streamline README and improve rustdoc  ([5f1945d](https://github.com/joshrotenberg/docker-wrapper/commit/5f1945df794b65fa6521ae218508fc842aa206c8))

### Features

- Add configurable timeouts for command execution  ([b729956](https://github.com/joshrotenberg/docker-wrapper/commit/b729956ab3fec22342b205f91461fb80fd1b3a7c))
- Add Docker Swarm command support  ([4213d52](https://github.com/joshrotenberg/docker-wrapper/commit/4213d529fc562ab07ec6c0f7d0e2ecd2643a24c9))

### Miscellaneous

- Release v0.9.0  ([324bc09](https://github.com/joshrotenberg/docker-wrapper/commit/324bc09500c57518ed2ca9bb397f4801106d3bb8))

### Refactoring

- Reorganize module structure for consistency  ([6fc529c](https://github.com/joshrotenberg/docker-wrapper/commit/6fc529cc3126e6d9542986969cad53df341f076d))

### Testing

- Add property-based testing with proptest  ([6cb673f](https://github.com/joshrotenberg/docker-wrapper/commit/6cb673f033c6414d65f9693081ed3eef58e282a0))

### Ci

- Bump actions/upload-artifact from 5 to 6  ([b602ee3](https://github.com/joshrotenberg/docker-wrapper/commit/b602ee3d56d6f526a2824a9bbadf893132baa9e8))
- Bump actions/cache from 4 to 5  ([b184cc3](https://github.com/joshrotenberg/docker-wrapper/commit/b184cc387ba72ef239290a499212be2dad3e6e30))

## [0.9.1] - 2026-01-04

### Features

- Add comprehensive tracing instrumentation ([#197](https://github.com/joshrotenberg/docker-wrapper/pull/197))

### CI

- Add examples compilation check to CI workflows ([#198](https://github.com/joshrotenberg/docker-wrapper/pull/198))

## [0.8.4] - 2025-12-11

### Bug Fixes

- Use container hostnames for Redis cluster initialization  ([c6b8e2b](https://github.com/joshrotenberg/docker-wrapper/commit/c6b8e2b7f76bf7c5ee8caccaea08a45fc4ae55a4))
- Use self.execute_command to avoid docker docker bug  ([c76debc](https://github.com/joshrotenberg/docker-wrapper/commit/c76debce238c879ad3fc69612bbd2f5f0e6d196c))

### Documentation

- Update README version numbers, license, and remove outdated planning doc  ([bbe861e](https://github.com/joshrotenberg/docker-wrapper/commit/bbe861e31995a317ed3afdacfd39236d86220c8e))

### Miscellaneous

- Release v0.8.4  ([e234941](https://github.com/joshrotenberg/docker-wrapper/commit/e234941a03df387c71e032b6a080a2f29e889481))

### Testing

- Add command execution verification tests  ([cdd6eb2](https://github.com/joshrotenberg/docker-wrapper/commit/cdd6eb29b564c69d3cf1a1b312c265ac0717e0d0))

### Ci

- Bump actions/checkout from 4 to 6  ([b2be6c2](https://github.com/joshrotenberg/docker-wrapper/commit/b2be6c28168ebe192977162a35b31d2be59b8c9d))
- Bump actions/upload-artifact from 4 to 5  ([3576dce](https://github.com/joshrotenberg/docker-wrapper/commit/3576dce837595c0613daccd6a654dec55128f9a3))

## [0.8.3] - 2025-10-15

### Features

- Enhance templates with readiness checks and complete Redis Developer CLI tool  ([938dac4](https://github.com/joshrotenberg/docker-wrapper/commit/938dac4ae0a33e648a142df8619f9af7018ef465))
- Add YAML configuration support  ([c35550c](https://github.com/joshrotenberg/docker-wrapper/commit/c35550c56fdfc00fea1929ad8ae5aae08701d098))
- YAML configuration and comprehensive documentation  ([bf0bc80](https://github.com/joshrotenberg/docker-wrapper/commit/bf0bc80812bbc387ae36396b078e8f519dc2233b))

### Miscellaneous

- Release v0.8.3  ([f9b7f21](https://github.com/joshrotenberg/docker-wrapper/commit/f9b7f21376532f2673df4dc6748ec8fa6a4f0384))

### Refactoring

- Remove redis-dev example (extracted to standalone redis-up project)  ([57d24c2](https://github.com/joshrotenberg/docker-wrapper/commit/57d24c213e76a4432427bcc5e72edf0917cf4067))

## [0.8.2] - 2025-08-27

### Bug Fixes

- Enable all features for docs.rs to show template documentation  ([d8fdf25](https://github.com/joshrotenberg/docker-wrapper/commit/d8fdf25d88b4fb9f52bedea26d70756888b40492))

### Miscellaneous

- Release v0.8.2  ([ac39f27](https://github.com/joshrotenberg/docker-wrapper/commit/ac39f27095465b83d1d1cab7b354055bbc6c14f5))

## [0.8.1] - 2025-08-27

### Documentation

- Comprehensive template documentation and integration tests  ([9e39b34](https://github.com/joshrotenberg/docker-wrapper/commit/9e39b345dbb671012067c631ff30a86afbfeab7f))

### Miscellaneous

- Release v0.8.1  ([4996392](https://github.com/joshrotenberg/docker-wrapper/commit/4996392ae512745429c28ac73dfa86874f53e178))

## [0.8.0] - 2025-08-26

### Documentation

- Clarify DockerCommand trait requirement and fix ContainerId usage  ([680bbfa](https://github.com/joshrotenberg/docker-wrapper/commit/680bbfa38c2c25ba895325c4a31f26cd9b730d8b))

### Features

- [**breaking**] Replace curl with reqwest for Redis Enterprise bootstrap  ([669ec0c](https://github.com/joshrotenberg/docker-wrapper/commit/669ec0c0cda137a1cc95cf9b670bdce2320b680a))

### Miscellaneous

- Release v0.7.1  ([00631d1](https://github.com/joshrotenberg/docker-wrapper/commit/00631d11750e2b3f72ad93be5db7285d7b6c2d14))

## [0.7.0] - 2025-08-26

### Features

- Enhance port mapping API and add convenience aliases  ([e7d7ee3](https://github.com/joshrotenberg/docker-wrapper/commit/e7d7ee3ed818e25246a96a17083bd7c3661d2da5))

### Miscellaneous

- Release v0.7.0  ([f5ba765](https://github.com/joshrotenberg/docker-wrapper/commit/f5ba7651d75a9125409f724c0a08f5fbab4f4995))

## [0.6.0] - 2025-08-25

### Features

- Add custom image and platform support to all templates  ([d5dfa1c](https://github.com/joshrotenberg/docker-wrapper/commit/d5dfa1c90e67421c47cfeaffab38c5692d5850a3))

### Miscellaneous

- Release v0.6.0  ([e48bcc1](https://github.com/joshrotenberg/docker-wrapper/commit/e48bcc186790511b1fdab4832c14f92661e8d9d7))

### Ci

- Bump dtolnay/rust-toolchain  ([3109499](https://github.com/joshrotenberg/docker-wrapper/commit/310949976ffefc610909b484b96db0838fc17c6b))
- Bump dtolnay/rust-toolchain  ([a70ab40](https://github.com/joshrotenberg/docker-wrapper/commit/a70ab403b06752abd62dea992ff58ed9ad8f6cbe))

## [0.5.0] - 2025-08-25

### Bug Fixes

- Install cargo-machete and cargo-outdated in CI  ([9f6d437](https://github.com/joshrotenberg/docker-wrapper/commit/9f6d4377206c5513a51a9de21afe060a96758e8a))
- Enable changelog updates in release-plz for proper version detection  ([6cb9dd4](https://github.com/joshrotenberg/docker-wrapper/commit/6cb9dd40d235b2ff43b039aaf984c43ca7a4a051))
- Remove invalid changelog_config field from release-plz.toml  ([c0ff58e](https://github.com/joshrotenberg/docker-wrapper/commit/c0ff58ebac0c5f6f7fc71b1b7cb98fe8c89ff408))

### Features

- Add docker context management commands  ([d341f02](https://github.com/joshrotenberg/docker-wrapper/commit/d341f0291b79ef7393494f8932b25c1ed59ec01e))
- Add templates feature for pre-configured Docker containers  ([1feb49a](https://github.com/joshrotenberg/docker-wrapper/commit/1feb49a6f373846f3192aba9967b4055d2648dc1))
- Add Redis Cluster template with multi-node setup  ([4310ad7](https://github.com/joshrotenberg/docker-wrapper/commit/4310ad768b803fc42cc1adcfc7cc4ae9babf25ef))
- Redis template improvements with Stack support and separate RedisInsight  ([a492e42](https://github.com/joshrotenberg/docker-wrapper/commit/a492e42c998f095ae57904f1d9449b2bcf454e5f))
- Add Redis Sentinel template with documentation updates  ([8676ec6](https://github.com/joshrotenberg/docker-wrapper/commit/8676ec6f86c2322c5fcf1fdddd9a107069d821a0))
- Implement Redis Enterprise template and comprehensive testing documentation  ([9181d9f](https://github.com/joshrotenberg/docker-wrapper/commit/9181d9fdfc17739098f9841880cce6f35dbfe455))

### Miscellaneous

- Release v0.5.0  ([96ef2e8](https://github.com/joshrotenberg/docker-wrapper/commit/96ef2e8f95d1dfc5171ddd0ff55fc058260f538d))

### Refactoring

- Implement hierarchical template organization with granular feature flags  ([cf6ffd2](https://github.com/joshrotenberg/docker-wrapper/commit/cf6ffd2d43385b7ff92b0f897be891b830f59cdf))

### Testing

- Comprehensive integration test coverage improvements  ([4e842f4](https://github.com/joshrotenberg/docker-wrapper/commit/4e842f4bad1d0d4bd78e94c9fccc8527fe4adef4))

## [0.4.1] - 2025-08-24

### Documentation

- Update README with current version and features  ([bc9e830](https://github.com/joshrotenberg/docker-wrapper/commit/bc9e83066af4fa5dc4382f91b31f4f9185c7804a))

### Features

- Add network/volume examples and rustdoc improvements  ([8d34131](https://github.com/joshrotenberg/docker-wrapper/commit/8d3413184e463c47e67dbcb7b0c41d854d79dfd6))

### Miscellaneous

- Release v0.4.1  ([59d4d8c](https://github.com/joshrotenberg/docker-wrapper/commit/59d4d8cf368406b37fbc04554935b37852396d3d))

## [0.4.0] - 2025-08-24

### Bug Fixes

- Add issues permission and skip-labeling for release-please  ([a19fb78](https://github.com/joshrotenberg/docker-wrapper/commit/a19fb78cb2250dacb956b18ff5ad292dff4580b2))
- Pin rust-toolchain action to resolve CI failures  ([047fa81](https://github.com/joshrotenberg/docker-wrapper/commit/047fa819fad7e74599007ab87b4b8a9005b61cd4))
- Add missing toolchain parameter to rust-toolchain actions  ([79a0f19](https://github.com/joshrotenberg/docker-wrapper/commit/79a0f196e2ba0767c835b4a9a3eb003b47f23708))
- Remove environment requirement from publish workflow  ([2e96b7e](https://github.com/joshrotenberg/docker-wrapper/commit/2e96b7e0c4e317da604a63fbee3780a4ef7b3af1))
- Correct release-plz action path to release-plz/action@v0.5  ([d6471db](https://github.com/joshrotenberg/docker-wrapper/commit/d6471db742ab57947b380ba8e7c560fbb8443dda))
- Remove invalid pr_body_template from package section  ([b456e77](https://github.com/joshrotenberg/docker-wrapper/commit/b456e77e6135f01fcf53dcd08975d189b844108c))
- Remove CLAUDE.md from git tracking  ([f28375a](https://github.com/joshrotenberg/docker-wrapper/commit/f28375a5fd52dde59ef71bba346240344d4d6ed1))

### Features

- Add complete Docker Compose command coverage  ([7fdf0ec](https://github.com/joshrotenberg/docker-wrapper/commit/7fdf0ecc45598c8c7b95eb69d29ea3b4526b36ef))
- Implement unified DockerCommand trait pattern  ([d7299fd](https://github.com/joshrotenberg/docker-wrapper/commit/d7299fdbd032fce615e1124934a1bf8800818463))
- Migrate core compose commands to unified pattern  ([ec5aaed](https://github.com/joshrotenberg/docker-wrapper/commit/ec5aaedd1d474ace26d146b3893c6fa99d4e00fc))
- Migrate utility compose commands to unified pattern  ([7ae2dce](https://github.com/joshrotenberg/docker-wrapper/commit/7ae2dce4e0fc15e78d6655594f3a3adf2a86ef33))
- Migrate container management compose commands to unified pattern  ([e54d2f3](https://github.com/joshrotenberg/docker-wrapper/commit/e54d2f35a14ee8d9997ec25d71428fb2c8b6cadd))
- Migrate core container commands to DockerCommandV2  ([39d0f68](https://github.com/joshrotenberg/docker-wrapper/commit/39d0f6872f91c88f48ce2b4aed2bea6802b90de8))
- Migrate registry/auth commands to DockerCommandV2  ([fb3cad6](https://github.com/joshrotenberg/docker-wrapper/commit/fb3cad62da6dfd8dfc785355f679159ae9873423))
- Migrate build/image commands to DockerCommandV2  ([f0e3d8d](https://github.com/joshrotenberg/docker-wrapper/commit/f0e3d8dffd7a22567348148fcdb16e57de472eb5))
- Migrate system/info commands to DockerCommandV2  ([39ba990](https://github.com/joshrotenberg/docker-wrapper/commit/39ba990a57672810e79e8c0eda323a120a2f0e4f))
- Migrate container management commands to DockerCommandV2  ([d8a0798](https://github.com/joshrotenberg/docker-wrapper/commit/d8a0798bdb2cb6344481052953a75cef452fd1e5))
- Migrate image/container manipulation commands to DockerCommandV2  ([487a0a3](https://github.com/joshrotenberg/docker-wrapper/commit/487a0a31781a4b3394b414cb435f719c1ceeea82))
- Complete DockerCommandV2 migration for ALL remaining commands 🎉  ([32b676c](https://github.com/joshrotenberg/docker-wrapper/commit/32b676c3351a6bbc5a23b8328de4cd7b7e89d6a5))
- Add docker builder commands for build cache management  ([157c830](https://github.com/joshrotenberg/docker-wrapper/commit/157c830fb230e50fbf26497a162f1bd60dcb576b))

### Miscellaneous

- Release 0.2.3  ([ef6c8c5](https://github.com/joshrotenberg/docker-wrapper/commit/ef6c8c5b070253223042e11737a08f2e7b872191))
- Organize documentation into docs directory  ([5d8d108](https://github.com/joshrotenberg/docker-wrapper/commit/5d8d108b73f583abd491016db398a9248f214576))
- Release 0.3.0  ([f86c4cc](https://github.com/joshrotenberg/docker-wrapper/commit/f86c4cc422ab4f5f98819315336aaccd2e3dc5fd))
- Migrate from release-please to release-plz  ([54e7fef](https://github.com/joshrotenberg/docker-wrapper/commit/54e7fefceb97ff9f7677dd2acf9f438864845a87))
- Release 0.4.0  ([cf879a7](https://github.com/joshrotenberg/docker-wrapper/commit/cf879a7260e66dfcaab4631904a6ea29effe208f))

### Refactoring

- Unify DockerCommand trait and remove V2 suffix  ([288c583](https://github.com/joshrotenberg/docker-wrapper/commit/288c583bad7846f01c03a24f13bad5de9d221ea3))

## [0.2.2] - 2025-08-23

### Documentation

- Fix bollard code examples and imports in comparison guide  ([52df4f5](https://github.com/joshrotenberg/docker-wrapper/commit/52df4f57f84835f90bca34b188fbe264a59b5bf7))

### Features

- Add system cleanup and maintenance commands   ([5e00e15](https://github.com/joshrotenberg/docker-wrapper/commit/5e00e157e64ce370bcb9707fae997c63c4c62fd3))
- Add debugging and reliability features   ([357a3a1](https://github.com/joshrotenberg/docker-wrapper/commit/357a3a130565317cffe959fc4fe118bb94680ab6))

### Miscellaneous

- Release 0.2.2  ([e726e2c](https://github.com/joshrotenberg/docker-wrapper/commit/e726e2c6f4dacb46dddbda6e5a2f8b782313e002))

### Performance

- Optimize CI workflow runtime  ([f86dd9e](https://github.com/joshrotenberg/docker-wrapper/commit/f86dd9eea2fcdbd456e56f960a152ad104ece979))

## [0.2.1] - 2025-08-22

### Bug Fixes

- Update git-cliff-action to resolve Debian buster repository issues  ([51ba6b2](https://github.com/joshrotenberg/docker-wrapper/commit/51ba6b2733e7f60e0eb9f9138256176ce0b53491))

### Documentation

- Comprehensive documentation improvements  ([63df3b2](https://github.com/joshrotenberg/docker-wrapper/commit/63df3b277d38a6a44a342f9fb1ac8b83a0d8babf))
- Add comprehensive Docker library comparison guide  ([eb59742](https://github.com/joshrotenberg/docker-wrapper/commit/eb597422ba50fc34aca00c72bb6589a81b26907a))

### Features

- Add container lifecycle commands (stop, start, restart)  ([0406d95](https://github.com/joshrotenberg/docker-wrapper/commit/0406d95a18ae9ff04fd3cbc99611dc6cd263d20f))
- Complete 100% Docker CLI coverage implementation  ([b3d1f35](https://github.com/joshrotenberg/docker-wrapper/commit/b3d1f35c680e3ea4d0b11bd3b4c77cb04aff9133))
- Add Docker Compose support with feature gating  ([d800225](https://github.com/joshrotenberg/docker-wrapper/commit/d80022577b6f68caad7f67b89d3e4e7f604dd3e9))
- Add streaming support for Docker command output  ([4642324](https://github.com/joshrotenberg/docker-wrapper/commit/464232457c9056ec876c95d2271316b0fe318d74))
- Add Docker network and volume management support  ([c69070f](https://github.com/joshrotenberg/docker-wrapper/commit/c69070fa7227708e2820c6f4eca9bb0876c22fb8))
- Add platform detection and runtime abstraction  ([ea09fdd](https://github.com/joshrotenberg/docker-wrapper/commit/ea09fdd095f2e50d1523810d55fedae8d3835fc9))

### Miscellaneous

- Release 0.2.1  ([6fe198b](https://github.com/joshrotenberg/docker-wrapper/commit/6fe198b65d53722865c664998e8a8d217eb9823e))

### Ci

- Bump actions/checkout from 4 to 5  ([781b496](https://github.com/joshrotenberg/docker-wrapper/commit/781b496f9813f5f514086b6a0a3550f41c2570c1))

## [0.2.0] - 2025-07-27

### Bug Fixes

- Update Cargo.toml example names and fix unused variable warning ([5023ce7](https://github.com/joshrotenberg/docker-wrapper/commit/5023ce761ea5aa4be838eba2547008dcff49f53e))
- Remove duplicate ImageRef from types.rs to fix compilation ([78beff5](https://github.com/joshrotenberg/docker-wrapper/commit/78beff5bb79dac03e5a688510f290d0a544436c6))
- [**breaking**] Update release-please workflow  ([345ff3b](https://github.com/joshrotenberg/docker-wrapper/commit/345ff3b475f7b33a13881a526872bfdcd5b65db2))
- [**breaking**] Update release-please changelog type  ([9ec4185](https://github.com/joshrotenberg/docker-wrapper/commit/9ec418514f93e386fea006709785277331207dc7))

### Documentation

- [**breaking**] Prepare for 0.1.0 release - update dates, remove competitive analysis, reduce emoji usage ([6758490](https://github.com/joshrotenberg/docker-wrapper/commit/6758490abfdb35241d3224a8d2e35347f9565e53))
- Add comprehensive Docker feature and test coverage matrix ([26e8d2e](https://github.com/joshrotenberg/docker-wrapper/commit/26e8d2e3fcff606d31accb84058c12b724620e4a))
- Create focused test-redis command implementation matrix ([040bb85](https://github.com/joshrotenberg/docker-wrapper/commit/040bb85f824e400ce4e64ea76c5c3b7b22fd639a))

### Features

- [**breaking**] Add release-please automation and refactor context system ([d2e911f](https://github.com/joshrotenberg/docker-wrapper/commit/d2e911fc095ee6ff27b9948810ccd97aca7a896f))
- [**breaking**] Add dependency management, fix tests, and improve CI caching ([219090a](https://github.com/joshrotenberg/docker-wrapper/commit/219090a18671d45d9922b2b7fdeabb0f18874b6f))
- [**breaking**] Significantly improve test coverage and remove phase naming ([a5e0325](https://github.com/joshrotenberg/docker-wrapper/commit/a5e0325b17607837c6b15b7866e78892c4d2af21))
- [**breaking**] Add comprehensive ContainerManager tests - major coverage improvement ([53f653c](https://github.com/joshrotenberg/docker-wrapper/commit/53f653c421cf41d79a58c84ca069e7a5a385dee1))
- Add comprehensive image operations testing infrastructure ([180d553](https://github.com/joshrotenberg/docker-wrapper/commit/180d553569e12e64079081a680cb20f0b8ba586c))
- Fix all image operations and enable comprehensive testing ✅ ([e7ab93a](https://github.com/joshrotenberg/docker-wrapper/commit/e7ab93ab83d97defa77653d5c8a66d37686cd0e9))

### Miscellaneous

- Release 0.2.0  ([9d1de8e](https://github.com/joshrotenberg/docker-wrapper/commit/9d1de8e5dc9bde98d9dcc141a97b72ba3d7103b7))

<!-- generated by git-cliff -->
