# Power Monitor

![Built with Iced badge]

A system power consumption monitoring application built in Rust.

## Architecture

The project is divided into two main components:

- **Collector**: A background service responsible for querying hardware sensors (CPU, GPU, RAM) and logging power consumption metrics to a database.
- **UI**: A graphical interface built with Iced that visualizes the collected data in real-time.

![Overall Architecture](overall_architecture.svg)

The power consumption has been tested with a Shelly Plug Gen3 S smart plug on various devices.

## Usage

To run the full application (Collector + UI):

```bash
cargo run
```

To run individual components, see:
- [Collector](collector/README.md)
- [UI](ui/README.md)

**Note**: The application requires Administrator privileges on Windows to access hardware sensors.

[Built with Iced badge]: https://img.shields.io/badge/Built%20With%20Iced-3645FF?logo=iced&logoColor=fff