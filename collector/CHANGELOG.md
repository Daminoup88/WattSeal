# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## `collector` - [1.0.2](https://github.com/Daminoup88/WattSeal/compare/collector-v0.0.0...collector-v1.0.2) - 2026-08-15

### Added
- *(collector)* Better driver error logging
- *(gpu)* Update GPU TDP table
- *(gpu)* GPU power estimation model when only usage available (applies for iGPU as well)
- *(gpu)* AMD GPU flexibility
- *(gpu)* Support NVML power samples as fallback
- *(collector)* Improved runtime logging
- *(collector)* NVIDIA GPU initialization logs and fallback with instant power
- *(collector)* AMD GPU initialization logs
- Banner, purge, architecture
- Markdown files and better loading of UI settings from DB
- implement close dialog for UI and background collector, integrated gpu estimation
- *(collector)* Disk and network power
- All time data for each component, refresh processes, refresh button
- Cross-platform with any sysinfo-supported system
- Cross-platform on linux, estimation if not admin
- *(ui)* Hardware info page
- *(database)* all_time_data fetched and updated in database
- *(database)* Hardware info added
- *(collector)* implemented detected_hardware_list
- *(collector)* new data all_time and general_hardware_infos
- feat(collector + database) : new sensors and data
- *(collector)* Template to add RAM, disk and network data using sysinfo
- *(database)* averaging data
- *(collector)* major changes to push processes data as a sensor
- *(collector)* VRAM per process added
- *(collector)* per process data collection
- *(common)* Select last N events from all sensor tables
- *(common)* Select last N entries function and flexible table names
- READMEs + architecture diagram
- Static database path
- Multi-threaded collector + UI

### Fixed
- *(collector)* Info is uncountable
- *(cpu)* Correct MSR DRAM address
- *(collector)* PP0, PP1 and DRAM are now optional
- *(collector)* Double admin pop up if denied
- *(collector)* AMD RAPL counter on 64 bits
- resolve gpu sensor initialization error ([#34](https://github.com/Daminoup88/WattSeal/pull/34))
- Minor changes
- MacOS build
- *(collector)* gpu usage
- *(database)* all time data auto update on single record
- *(collector)* massive optimization for processes
- *(collector)* Winring works on first launch at startup
- *(database)* purge
- *(main)* Sleep period adjusted
- *(database)* insert sql and Format change
- *(type)* TotalData and merge
- *(database)* process_data table creation
- *(database)* process_data insertion
- *(collector)* display

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
- *(deps)* bump nvml-wrapper from 0.12.0 to 0.12.1 ([#33](https://github.com/Daminoup88/WattSeal/pull/33))
- *(deps)* bump display-info from 0.5.8 to 0.5.9 ([#31](https://github.com/Daminoup88/WattSeal/pull/31))
- *(deps)* bump sysinfo from 0.38.0 to 0.38.4 ([#28](https://github.com/Daminoup88/WattSeal/pull/28))
- *(deps)* bump sysinfo from 0.37.2 to 0.38.0 ([#16](https://github.com/Daminoup88/WattSeal/pull/16))
- *(deps)* bump nvml-wrapper from 0.11.0 to 0.12.0 ([#15](https://github.com/Daminoup88/WattSeal/pull/15))
- *(deps)* bump display-info from 0.5.7 to 0.5.8 ([#14](https://github.com/Daminoup88/WattSeal/pull/14))
- *(deps)* bump chrono from 0.4.43 to 0.4.44 ([#13](https://github.com/Daminoup88/WattSeal/pull/13))
- File lock on db to prevent multiple instances
- Extended release logs & unify cargo tomls
- Keep cargo version to the tag version
- Name, documentation, README
- Clean warnings, release profile, optimize database, remove logs, init logs in release mode
- *(collector)* Integrated GPU stored in GPU table instead of CPU
- BREAKING CHANGE: Correct all time data and collection duration
- *(database)* Change process_usage_watt name to process_power_watts
- BREAKING CHANGE: Simplify hardware info
- *(collector)* Hardware info splitted in the sensors
- Removed many crash possibilities except when the database cannot be created
- *(collector)* Optimize and centralize CPU usage retrieval
- BREAKING CHANGE: Per process GPU usage (only NVIDIA supported)
- *(collector)* Move process.rs inside sensors
- *(common)* Refactor the table / data architecture for easier future integration
- Merge branch 'main' of https://github.com/Daminoup88/ProjetE5
- *(common)* Optional CPUData usage
- GPU intel usage percent
- GPU power and display
- BREAKING CHANGE: Total power table
- *(deps)* Remove unused dependencies
- BREAKING CHANGE: Timestamp stored as i64 in database
- *(collector)* Clear unused comments and deps
- cache workflow
- Common types only accessed in `database` module
- Database utilities moved to the new `common` crate
- Collector logic moved to lib inside CollectorApp struct
- Delete unused `differs` function
- Database tables and insertion
- Clean the Sensor trait
- One event for several measures at a time
- initializing database and pushing the data
- few changes database
- database sql functions
- ALL values of cpu and gpu consumption and as Options
- accurate CPU usage in percents calcultated
- cpu value ok
- gpu and cpu collected in main
- formatted
- CPU and GPU detection, data structs
- merging
- Rust automatic formatting (needs to be run with cargo +nightly fmt)
- Merge branch 'main' of https://github.com/Daminoup88/ProjetE5
- Cargo workspace with collector + UI
