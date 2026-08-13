# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [1.0.2](https://github.com/Daminoup88/WattSeal/compare/v0.0.0...v1.0.2) - 2026-08-13

### Added

- *(collector)* Better driver error logging
- *(gpu)* Update GPU TDP table
- *(gpu)* GPU power estimation model when only usage available (applies for iGPU as well)
- *(gpu)* AMD GPU flexibility
- *(gpu)* Support NVML power samples as fallback
- *(collector)* Improved runtime logging
- *(collector)* NVIDIA GPU initialization logs and fallback with instant power
- *(collector)* AMD GPU initialization logs

### Fixed

- *(collector)* Info is uncountable
- *(cpu)* Correct MSR DRAM address
- *(collector)* PP0, PP1 and DRAM are now optional
- *(collector)* Double admin pop up if denied
- *(collector)* AMD RAPL counter on 64 bits

### Other

- Db optimization ([#115](https://github.com/Daminoup88/WattSeal/pull/115))
- *(collector)* Go back to Winring0
- *(gpu)* Cleaner error management for NVIDIA GPUs
- *(gpu)* Clearer logs and less critical errors for AMD GPUs
- Detect multiple GPUs ([#93](https://github.com/Daminoup88/WattSeal/pull/93))
- Split sensors and computed sensors ([#91](https://github.com/Daminoup88/WattSeal/pull/91))
- *(deps)* bump sysinfo from 0.39.2 to 0.39.3 ([#79](https://github.com/Daminoup88/WattSeal/pull/79))
- Replace power metrics by energy metrics ([#73](https://github.com/Daminoup88/WattSeal/pull/73))
- Better logging ([#72](https://github.com/Daminoup88/WattSeal/pull/72))
- *(deps)* bump sysinfo from 0.39.1 to 0.39.2 ([#71](https://github.com/Daminoup88/WattSeal/pull/71))
- Replace Winring0 with Scaphandre driver ([#69](https://github.com/Daminoup88/WattSeal/pull/69))
- *(deps)* bump sysinfo from 0.39.0 to 0.39.1 ([#66](https://github.com/Daminoup88/WattSeal/pull/66))
- *(deps)* bump sysinfo from 0.38.4 to 0.39.0 ([#62](https://github.com/Daminoup88/WattSeal/pull/62))
- Mode to send metrics via mqtt ([#56](https://github.com/Daminoup88/WattSeal/pull/56))
- *(deps)* bump windows from 0.57 to 0.62.2
