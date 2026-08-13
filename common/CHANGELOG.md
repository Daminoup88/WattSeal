# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [1.0.2](https://github.com/Daminoup88/WattSeal/compare/v0.1.0...v1.0.2) - 2026-08-13

### Added

- *(migration)* Previous GPU entries set to previous GPU 1
- *(ui)* Prices in local currency
- *(collector)* Improved runtime logging

### Fixed

- *(ui)* Variable duration based power also in batched update
- *(collector)* Info is uncountable
- *(collector)* PP0, PP1 and DRAM are now optional
- *(ui)* If several GPUs, display the sum

### Other

- release-plz config
- Db optimization ([#115](https://github.com/Daminoup88/WattSeal/pull/115))
- Detect multiple GPUs ([#93](https://github.com/Daminoup88/WattSeal/pull/93))
- Split sensors and computed sensors ([#91](https://github.com/Daminoup88/WattSeal/pull/91))
- *(deps)* bump rusqlite from 0.39.0 to 0.40.1 ([#81](https://github.com/Daminoup88/WattSeal/pull/81))
- *(deps)* bump file_icon_provider from 1.0.0 to 1.0.1 ([#78](https://github.com/Daminoup88/WattSeal/pull/78))
- *(deps)* bump serde_json from 1.0.149 to 1.0.150 ([#76](https://github.com/Daminoup88/WattSeal/pull/76))
- Replace power metrics by energy metrics ([#73](https://github.com/Daminoup88/WattSeal/pull/73))
- Better logging ([#72](https://github.com/Daminoup88/WattSeal/pull/72))
- Mode to send metrics via mqtt ([#56](https://github.com/Daminoup88/WattSeal/pull/56))
