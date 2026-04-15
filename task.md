# WattSeal Enhancement — Headless API + Prometheus + Battery Health

## Goal
Implement three features for WattSeal (https://github.com/Daminoup88/WattSeal). Fork the repo, implement on a branch, open a PR.

## Context

WattSeal is a real-time PC power consumption monitor written in Rust.
- Rust cargo workspace: `wattseal/` (root binary), `collector/` (sensor daemon), `common/` (types + SQLite WAL IPC), `ui/` (Iced GUI)
- Existing crates: sysinfo, battery, iced, plotters, rusqlite
- SQLite WAL mode for concurrent read/write between collector and UI
- License: GPLv3

## Tasks

### 1. Headless HTTP API Server

Add a new `api/` crate to the workspace. Add `--headless` flag to the main binary.

**Config** (add to settings.json or create wattseal.toml):
```json
{
  "api": {
    "enabled": false,
    "port": 8080,
    "host": "127.0.0.1",
    "auth": {
      "type": "none",
      "token": ""
    }
  }
}
```

**Endpoints** (all return JSON except /metrics):
- `GET /api/v1/current` — live power snapshot from SQLite
- `GET /api/v1/history?from=&to=&resolution=1m` — historical data
- `GET /api/v1/processes?sort=cpu_watts&limit=10` — top processes
- `GET /api/v1/health` — service health check
- `GET /metrics` — Prometheus text format

**Auth**: If `auth.type == "token"`, require `Authorization: Bearer <token>` on all `/api/` endpoints.

**Tech**: Use `axum` for the HTTP server (add to Cargo.toml). Make it compile cleanly with the existing workspace.

**Important**: The collector writes to SQLite. The API reads. Do NOT modify the collector's DB writes. Read the existing schema in `common/src/` to understand the table structure.

### 2. Prometheus Exporter

Implement `/metrics` endpoint in Prometheus text format:

```
# HELP wattseal_total_watts Total system power in watts
# TYPE wattseal_total_watts gauge
wattseal_total_watts 247.3

# HELP wattseal_component_watts Component power in watts
# TYPE wattseal_component_watts gauge
wattseal_component_watts{component="cpu",source="rapl"} 45.2
wattseal_component_watts{component="gpu",source="nvml"} 182.1

# HELP wattseal_process_watts Process power in watts
# TYPE wattseal_process_watts gauge
wattseal_process_watts{process_name="chrome",pid="1234"} 12.5

# HELP wattseal_battery_health_percent Battery health percentage
# TYPE wattseal_battery_health_percent gauge
wattseal_battery_health_percent 87.5
```

### 3. Battery Health Tracking

For laptops, track battery health over time.

**New SQLite table**:
```sql
CREATE TABLE IF NOT EXISTS battery_health (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    timestamp INTEGER NOT NULL,
    health_percent REAL NOT NULL,
    discharge_rate_watts REAL,
    cycle_count INTEGER,
    time_since_full_charge_seconds INTEGER
);
```

**Track** (using existing `battery` crate):
- Battery health % (current capacity / design capacity × 100)
- Discharge rate when on battery
- Charge cycle count (if available)
- Time since last full charge

**Collector changes**: Write a battery health record every time the collector does its regular poll.

**UI**: Add a battery health section/page in the Iced UI showing current health % and a trend chart. Use the existing `plotters` crate for the chart.

## Process

1. Fork https://github.com/Daminoup88/WattSeal to your GitHub account
2. Clone the fork: `git clone https://github.com/<your-username>/WattSeal.git`
3. Add `axum` and `tokio` to the root `Cargo.toml` or a new `api/Cargo.toml`
4. Create branch: `git checkout -b feat/headless-api`
5. Implement all three features
6. Build: `cargo build --all` — fix any compilation errors
7. Commit: `git add -A && git commit -m "feat: add headless API server, Prometheus exporter, and battery health tracking"`
8. Push: `git push -u origin feat/headless-api`
9. Open PR against `Daminoup88/WattSeal:main`
10. Print the PR URL

## Rules
- Follow existing code style (check rustfmt.toml)
- Do NOT modify collector's existing DB schema writes
- Keep GPLv3 license
- All dependencies must be added to Cargo.toml files properly
- If you need to understand the DB schema, inspect files in `common/src/` carefully

## Notify when done
Run this command when completely finished:
```
openclaw system event --text "WattSeal PR opened: <PR URL>" --mode now
```

Print the PR URL as your final output.
