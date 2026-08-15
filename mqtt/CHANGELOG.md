# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## `mqtt` - [1.0.2](https://github.com/Daminoup88/WattSeal/compare/mqtt-v0.1.4...mqtt-v1.0.2) - 2026-08-15

### Added
- Banner, purge, architecture
- Markdown files and better loading of UI settings from DB
- Icons and correct carbon intensities and electricity costs
- READMEs + architecture diagram

### Fixed
- *(ui)* Usage translation used the wrong function
- Linux tray icon or fall back to only collector / UI
- *(readme)* some url
- READMEs fix

### Other
- *(collector)* Go back to Winring0
- *(deps)* bump mockall from 0.14.0 to 0.15.0 ([#94](https://github.com/Daminoup88/WattSeal/pull/94))
- Split sensors and computed sensors ([#91](https://github.com/Daminoup88/WattSeal/pull/91))
- *(deps)* bump serde_json from 1.0.149 to 1.0.150 ([#76](https://github.com/Daminoup88/WattSeal/pull/76))
- Add Zed editor settings to format on save
- Replace Winring0 with Scaphandre driver ([#69](https://github.com/Daminoup88/WattSeal/pull/69))
- Mode to send metrics via mqtt ([#56](https://github.com/Daminoup88/WattSeal/pull/56))
- Add dashboard
- Overall architecture diagram
- README, roadmap, security
- Roadmap
- Windows defender warning
- Readme
- Mermaid chart fix
- Name, documentation, README
- Database utilities moved to the new `common` crate
- Finished splitting main.rs into multiple files:
- Sensor architecture
- Notification test
