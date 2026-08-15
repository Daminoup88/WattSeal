# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## `common` - [1.0.2](https://github.com/Daminoup88/WattSeal/compare/common-v0.1.0...common-v1.0.2) - 2026-08-15

### Added
- *(migration)* Previous GPU entries set to previous GPU 1
- *(ui)* Prices in local currency
- *(collector)* Improved runtime logging
- Better logs
- Markdown files and better loading of UI settings from DB
- Icons and correct carbon intensities and electricity costs
- *(ui)* Finalized translations, some UI improvements
- *(ui)* Persistent theme and electricity price
- implement close dialog for UI and background collector, integrated gpu estimation
- feat(ui: Carbon intensity, more time ranges, purged processes
- All time data for each component, refresh processes, refresh button
- Cross-platform with any sysinfo-supported system
- *(ui)* App translations English and French
- *(ui)* All time data and carbon emissions
- *(ui)* Hardware info page
- *(ui)* Top processes in UI
- *(database)* all_time_data fetched and updated in database
- *(database)* Hardware info added
- *(collector)* implemented detected_hardware_list
- *(ui)* Processes card
- *(collector)* new data all_time and general_hardware_infos
- *(common)* Icon fetched from the exe path
- *(database)* Real-time select average & helpers
- *(data)* Zero method to get SensorData initialized at 0
- *(ui)* Secondary data extended
- feat(collector + database) : new sensors and data
- *(collector)* Template to add RAM, disk and network data using sysinfo
- *(database)* averaging data
- *(common)* Database functions
- *(ui)* Visually change time range and metric type
- *(collector)* major changes to push processes data as a sensor
- *(common)* Generic functions for SensorData
- *(common)* Select last N events from all sensor tables
- *(ui)* Print last DB event in terminal
- *(common)* Table list in memory and select last cpu events
- *(common)* Select last N entries function and flexible table names

### Fixed
- *(ui)* Variable duration based power also in batched update
- *(collector)* Info is uncountable
- *(collector)* PP0, PP1 and DRAM are now optional
- *(ui)* If several GPUs, display the sum
- Minor changes
- *(db)* Ensure hardware_info table created
- *(ui)* Energy or power display and calculation from DB for hour rows
- *(database)* all time data auto update on single record
- *(collector)* massive optimization for processes
- *(database)* Added required functions to retrieve in UI
- *(database)* purge
- *(database)* pondération pour le calcul du total_power_average
- *(database)* insert sql and Format change
- *(database)* added collumn
- *(type)* TotalData and merge
- *(database)* process_data table creation
- *(database)* process_data insertion
- *(collector)* display

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
- *(deps)* bump rusqlite from 0.38.0 to 0.39.0 ([#29](https://github.com/Daminoup88/WattSeal/pull/29))
- *(deps)* bump chrono from 0.4.43 to 0.4.44 ([#13](https://github.com/Daminoup88/WattSeal/pull/13))
- File lock on db to prevent multiple instances
- Extended release logs & unify cargo tomls
- Keep cargo version to the tag version
- Name, documentation, README
- Clean warnings, release profile, optimize database, remove logs, init logs in release mode
- BREAKING CHANGE: Correct all time data and collection duration
- *(database)* Change process_usage_watt name to process_power_watts
- BREAKING CHANGE: Simplify hardware info
- BREAKING CHANGE: major optimization of the purge. New column in timestamp table
- *(collector)* Hardware info splitted in the sensors
- Removed many crash possibilities except when the database cannot be created
- BREAKING CHANGE: Per process GPU usage (only NVIDIA supported)
- *(common)* Refactor the table / data architecture for easier future integration
- *(common)* Optional CPUData usage
- *(common)* Function to extract correct data type from a param DB query
- BREAKING CHANGE: Total power table
- BREAKING CHANGE: Timestamp stored as i64 in database
- *(common)* Database `entries` module created
- Merge with main v1
- *(ui)* Update to iced 0.14 and plotters_iced2
- Update rusqlite requirement from 0.37.0 to 0.38.0 ([#6](https://github.com/Daminoup88/WattSeal/pull/6))
- SensorChart number of series no longer generic
- Database utilities moved to the new `common` crate
