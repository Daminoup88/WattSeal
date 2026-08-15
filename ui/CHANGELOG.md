# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## `ui` - [1.0.2](https://github.com/Daminoup88/WattSeal/compare/ui-v0.1.0...ui-v1.0.2) - 2026-08-15

### Added
- *(i18n)* add Simplified Chinese (ZH) UI localization ([#80](https://github.com/Daminoup88/WattSeal/pull/80))
- *(ui)* app translations Romanian
- *(ui)* Number formatting
- *(ui)* Prices in local currency
- *(ui)* Translate settings picklists
- Better logs
- Markdown files and better loading of UI settings from DB
- Icons and correct carbon intensities and electricity costs
- *(ui)* Finalized translations, some UI improvements
- *(ui)* Persistent theme and electricity price
- *(ui)* Footer and icons font
- implement close dialog for UI and background collector, integrated gpu estimation
- feat(ui: Carbon intensity, more time ranges, purged processes
- All time data for each component, refresh processes, refresh button
- Cross-platform with any sysinfo-supported system
- *(ui)* Educational content with `?`
- *(ui)* App translations English and French
- *(ui)* Settings modal
- *(ui)* All time data and carbon emissions
- *(ui)* Hardware info grid
- *(ui)* Hardware info page
- *(ui)* Top processes in UI
- feat(ui) Processes icons
- *(ui)* Processes card
- *(common)* Icon fetched from the exe path
- *(ui)* Minor changes
- *(ui)* Secondary data extended
- *(ui)* Order cards by CPU, GPU, RAM...
- *(ui)* Eco default theme
- *(ui)* Component cards button and picklist style
- *(ui)* Component cards with chart
- *(ui)* New messages
- *(ui)* Fetch more database data in App
- *(ui)* Dashboard functional time range with data populated
- *(ui)* Isolate push to history and push latest data
- *(ui)* Helper functions for TimeRange
- *(ui)* Time range enum moved to types file
- *(ui)* Set all chart line types
- *(ui)* Step chart line support
- *(ui)* Toggler style
- *(ui)* Visually change time range and metric type
- *(ui)* Change time range pick list and metric type button
- *(ui)* Ordered component cards
- *(ui)* Remove multiple axes support & move sensordata mapping from the chart component
- *(ui)* Power cards initialized at startup and data loaded from the new total table
- *(ui)* Dashboard page
- *(ui)* Add styles for each element
- *(ui)* Print last DB event in terminal
- Series in a HashMap in the chart + mock UI/DB conn
- Database access created in UI (unwrap)
- READMEs + architecture diagram

### Fixed
- *(ui)* Apply number formatting
- *(ui)* Small settings alignment fix
- *(ui)* Variable duration based power also in batched update
- *(ui)* Report issue link
- *(ui)* Usage translation used the wrong function
- *(readme)* some url
- *(ui)* theme design
- *(ui)* Power or energy correct chart legend
- *(ui)* Energy or power display and calculation from DB for hour rows
- *(ui)* Old bug that linked the first data with the latest at startup
- *(ui)* Change line color on theme change
- *(database)* purge
- *(ui)* Line type changes for all metric types & time range selector not aligned anymore if no metric button
- *(ui)* Use local timezone datetime
- *(ui)* Chart tooltip position fixed
- *(ui)* Data charted on exact second
- *(ui)* Component prune history
- *(ui)* Dynamic chart range always start at 0
- *(ui)* Cursor position correctly detected for tooltip
- *(ui)* Remove debug data point
- *(ui)* Fix tooltip right / left switch
- *(ui)* Fix tooltip position
- *(ui)* Call update on tooltip change to redraw
- READMEs fix

### Other
- release-plz config
- *(ui)* Minor syntactical changes in translations
- Db optimization ([#115](https://github.com/Daminoup88/WattSeal/pull/115))
- Split sensors and computed sensors ([#91](https://github.com/Daminoup88/WattSeal/pull/91))
- Replace power metrics by energy metrics ([#73](https://github.com/Daminoup88/WattSeal/pull/73))
- *(deps)* bump chrono from 0.4.43 to 0.4.44 ([#13](https://github.com/Daminoup88/WattSeal/pull/13))
- Extended release logs & unify cargo tomls
- Keep cargo version to the tag version
- Name, documentation, README
- Clean warnings, release profile, optimize database, remove logs, init logs in release mode
- *(collector)* Integrated GPU stored in GPU table instead of CPU
- *(ui)* Info page simplified
- *(ui)* Removed lifetime annotations in chart and components
- Removed many crash possibilities except when the database cannot be created
- *(ui)* Move the data ownership to the app
- *(common)* Refactor the table / data architecture for easier future integration
- *(ui)* Clean dependencies and gather functions
- *(ui)* Move component state and metric type to separate files
- Merge branch 'main' of https://github.com/Daminoup88/ProjetE5
- *(ui)* Each component creates its card & scrollable dashboard
- *(ui)* Shared data between dashboard and charts
- GPU intel usage percent
- GPU power and display
- *(ui)* Simplify element styles
- *(ui)* Enable new styles in view methods
- *(ui)* Change iced features
- *(deps)* Remove unused dependencies
- Fix font size type to match `impl Into<Pixels>`
- Merge with main v2
- Merge with main v1
- *(ui)* Update to iced 0.14 and plotters_iced2
- Update iced requirement from 0.13.1 to 0.14.0
- Update rusqlite requirement from 0.37.0 to 0.38.0 ([#6](https://github.com/Daminoup88/WattSeal/pull/6))
- SensorChart number of series no longer generic
- Y description and labels overlap fix
- Chart hover tooltip
- Formatting
- UI refactoring - The kitchen is burning 🔥
- Theme support for chart, custom app themes
- Small changes
- Dynamic Y range, more precise hover and debug points
- Better hover button distribution
- Hover detection on graph
- Move the build chart utility inside the SensorChart impl
- 5 line types support
- Chart supports N timeseries with a specific color and grid style
- Multiple empty pages with navigation
- Rust automatic formatting (needs to be run with cargo +nightly fmt)
- UI package architecture
- Cargo workspace with collector + UI
