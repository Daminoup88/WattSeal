# Roadmap

## Collector

- [ ] Improve accuracy of total power usage on several devices by adding more sensors and refining estimation algorithms ([#17](https://github.com/Daminoup88/WattSeal/issues/17))
- [ ] Add tests
- [x] Run as a service ([#18](https://github.com/Daminoup88/WattSeal/issues/18))
- [ ] Configurable sensors polling frequency with no regression on purge / UI averages ([46](https://github.com/Daminoup88/WattSeal/issues/46))
- [ ] Improved estimation of energy consumption on machines with Apple Silicon processors ([46](https://github.com/Daminoup88/WattSeal/issues/46))
- [x] Add the possibility to run only sensors by implementing a headless mode ([52](https://github.com/Daminoup88/WattSeal/issues/52))

### Security

- [x] Remove WinRing0 driver dependency on Windows (see [Security](SECURITY.md#winring0-kernel-driver-windows) section for details) ([#19](https://github.com/Daminoup88/WattSeal/issues/19))

## Data integration

- [x] The ability to send data via MQTT to a broker ([#55](https://github.com/Daminoup88/WattSeal/issues/55))
- [x] Change from power metrics to energy metrics (at least at the sensor level) ([#58](https://github.com/Daminoup88/WattSeal/issues/58))

## UI / UX

- [ ] Top process in tooltip for each component and in the total chart ([#20](https://github.com/Daminoup88/WattSeal/issues/20))
- [ ] Select each component in the total chart ([#21](https://github.com/Daminoup88/WattSeal/issues/21))
- [ ] Notification thresholds – total and per process ([#12](https://github.com/Daminoup88/WattSeal/issues/12))
- [ ] Differentiate apps and background processes ([#22](https://github.com/Daminoup88/WattSeal/issues/22))

## Network & emissions

- [ ] Indirect network power usage and emissions calculation ([#23](https://github.com/Daminoup88/WattSeal/issues/23))
- [ ] Indirect network power usage by domain ([#23](https://github.com/Daminoup88/WattSeal/issues/23))
- [ ] Power usage breakdown by browser tab ([#24](https://github.com/Daminoup88/WattSeal/issues/24))
- [ ] Auto-update electricity prices and carbon emissions on build ([#25](https://github.com/Daminoup88/WattSeal/issues/25))

