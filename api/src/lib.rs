use std::{net::SocketAddr, sync::Arc, time::SystemTime};

use axum::{
    Router,
    extract::{Query, State},
    http::{HeaderMap, StatusCode},
    middleware::{self, Next},
    response::{IntoResponse, Response},
    routing::get,
};
use common::{Database, DatabaseEntry, SensorData, TotalData, config::ApiConfig};
use serde::{Deserialize, Serialize};

struct AppState {
    config: ApiConfig,
}

/// Starts the headless API server. Blocks until the server shuts down.
pub async fn run_server(config: ApiConfig) {
    let addr: SocketAddr = format!("{}:{}", config.host, config.port)
        .parse()
        .unwrap_or_else(|_| SocketAddr::from(([127, 0, 0, 1], 8080)));

    let state = Arc::new(AppState {
        config: config.clone(),
    });

    let api_routes = Router::new()
        .route("/api/v1/current", get(handle_current))
        .route("/api/v1/history", get(handle_history))
        .route("/api/v1/processes", get(handle_processes))
        .route("/api/v1/health", get(handle_health))
        .layer(middleware::from_fn_with_state(state.clone(), auth_middleware));

    let app = Router::new()
        .merge(api_routes)
        .route("/metrics", get(handle_metrics))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    println!("WattSeal API server listening on {}", addr);
    axum::serve(listener, app).await.unwrap();
}

async fn auth_middleware(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    request: axum::extract::Request,
    next: Next,
) -> Response {
    if state.config.auth.auth_type == "token" && !state.config.auth.token.is_empty() {
        let expected = format!("Bearer {}", state.config.auth.token);
        match headers.get("authorization").and_then(|v| v.to_str().ok()) {
            Some(value) if value == expected => {}
            _ => return StatusCode::UNAUTHORIZED.into_response(),
        }
    }
    next.run(request).await
}

#[derive(Serialize)]
struct CurrentResponse {
    timestamp_ms: i64,
    components: Vec<ComponentSnapshot>,
    total_watts: f64,
}

#[derive(Serialize)]
struct ComponentSnapshot {
    component: String,
    power_watts: Option<f64>,
}

async fn handle_current() -> Response {
    let mut db = match Database::new() {
        Ok(db) => db,
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, format!("DB error: {e}")).into_response(),
    };

    let data = match db.select_last_n_records(1) {
        Ok(d) => d,
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, format!("Query error: {e}")).into_response(),
    };

    let now_ms = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as i64;

    let mut total_watts = 0.0;
    let mut components = Vec::new();

    for (_, sensor_data) in &data {
        match sensor_data {
            SensorData::Total(t) => total_watts = t.total_power_watts,
            SensorData::Process(_) => {}
            other => {
                components.push(ComponentSnapshot {
                    component: other.sensor_type().to_string(),
                    power_watts: other.total_power_watts(),
                });
            }
        }
    }

    let resp = CurrentResponse {
        timestamp_ms: now_ms,
        components,
        total_watts,
    };
    axum::Json(resp).into_response()
}

#[derive(Deserialize)]
struct HistoryParams {
    from: Option<i64>,
    to: Option<i64>,
    resolution: Option<String>,
}

#[derive(Serialize)]
struct HistoryPoint {
    timestamp_ms: i64,
    total_watts: f64,
}

async fn handle_history(Query(params): Query<HistoryParams>) -> Response {
    let mut db = match Database::new() {
        Ok(db) => db,
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, format!("DB error: {e}")).into_response(),
    };

    let now_ms = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as i64;

    let from_ms = params.from.unwrap_or(now_ms - 3600 * 1000);
    let to_ms = params.to.unwrap_or(now_ms);

    let from_time = SystemTime::UNIX_EPOCH + std::time::Duration::from_millis(from_ms as u64);
    let to_time = SystemTime::UNIX_EPOCH + std::time::Duration::from_millis(to_ms as u64);

    let table_name = TotalData::table_name_static();
    let data = match db.select_data_in_time_range(table_name, from_time, to_time) {
        Ok(d) => d,
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, format!("Query error: {e}")).into_response(),
    };

    let _resolution = params.resolution.unwrap_or_else(|| "1m".to_string());

    let points: Vec<HistoryPoint> = data
        .into_iter()
        .filter_map(|(ts, sd)| {
            if let SensorData::Total(t) = sd {
                let ms = ts
                    .duration_since(SystemTime::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_millis() as i64;
                Some(HistoryPoint {
                    timestamp_ms: ms,
                    total_watts: t.total_power_watts,
                })
            } else {
                None
            }
        })
        .collect();

    axum::Json(points).into_response()
}

#[derive(Deserialize)]
struct ProcessParams {
    #[allow(dead_code)]
    sort: Option<String>,
    limit: Option<usize>,
}

#[derive(Serialize)]
struct ProcessSnapshot {
    name: String,
    pid: Option<String>,
    power_watts: f64,
    cpu_percent: f64,
    gpu_percent: Option<f64>,
    mem_percent: f64,
}

async fn handle_processes(Query(params): Query<ProcessParams>) -> Response {
    let db = match Database::new() {
        Ok(db) => db,
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, format!("DB error: {e}")).into_response(),
    };

    let limit = params.limit.unwrap_or(10);

    let data = match db.select_top_processes_average(60, limit, false) {
        Ok(d) => d,
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, format!("Query error: {e}")).into_response(),
    };

    let mut processes = Vec::new();
    for (_, sd) in data {
        if let SensorData::Process(procs) = sd {
            for p in procs {
                processes.push(ProcessSnapshot {
                    name: p.app_name.clone(),
                    pid: None,
                    power_watts: p.process_power_watts,
                    cpu_percent: p.process_cpu_usage,
                    gpu_percent: p.process_gpu_usage,
                    mem_percent: p.process_mem_usage,
                });
            }
        }
    }

    axum::Json(processes).into_response()
}

#[derive(Serialize)]
struct HealthResponse {
    status: String,
    uptime_seconds: u64,
    database_ok: bool,
}

async fn handle_health() -> Response {
    let db_ok = Database::new().is_ok();
    let resp = HealthResponse {
        status: if db_ok { "ok".to_string() } else { "degraded".to_string() },
        uptime_seconds: 0,
        database_ok: db_ok,
    };
    axum::Json(resp).into_response()
}

async fn handle_metrics() -> Response {
    let mut output = String::new();

    // Read latest sensor data
    if let Ok(mut db) = Database::new() {
        if let Ok(data) = db.select_last_n_records(1) {
            let mut total_watts = 0.0;
            for (_, sensor_data) in &data {
                match sensor_data {
                    SensorData::Total(t) => total_watts = t.total_power_watts,
                    _ => {}
                }
            }

            output.push_str("# HELP wattseal_total_watts Total system power in watts\n");
            output.push_str("# TYPE wattseal_total_watts gauge\n");
            output.push_str(&format!("wattseal_total_watts {:.1}\n\n", total_watts));

            output.push_str("# HELP wattseal_component_watts Component power in watts\n");
            output.push_str("# TYPE wattseal_component_watts gauge\n");
            for (_, sensor_data) in &data {
                match sensor_data {
                    SensorData::Total(_) | SensorData::Process(_) => {}
                    other => {
                        if let Some(power) = other.total_power_watts() {
                            let component = other.sensor_type().to_lowercase();
                            let source = match other {
                                SensorData::CPU(_) => "rapl",
                                SensorData::GPU(_) => "nvml",
                                _ => "estimate",
                            };
                            output.push_str(&format!(
                                "wattseal_component_watts{{component=\"{}\",source=\"{}\"}} {:.1}\n",
                                component, source, power
                            ));
                        }
                    }
                }
            }
            output.push('\n');

            output.push_str("# HELP wattseal_process_watts Process power in watts\n");
            output.push_str("# TYPE wattseal_process_watts gauge\n");
            for (_, sensor_data) in &data {
                if let SensorData::Process(procs) = sensor_data {
                    for p in procs {
                        output.push_str(&format!(
                            "wattseal_process_watts{{process_name=\"{}\"}} {:.1}\n",
                            p.app_name.replace('"', "\\\""),
                            p.process_power_watts
                        ));
                    }
                }
            }
            output.push('\n');
        }

        // Battery health
        if let Ok(Some(bh)) = db.select_latest_battery_health() {
            output.push_str("# HELP wattseal_battery_health_percent Battery health percentage\n");
            output.push_str("# TYPE wattseal_battery_health_percent gauge\n");
            output.push_str(&format!("wattseal_battery_health_percent {:.1}\n", bh.health_percent));
        }
    }

    (
        StatusCode::OK,
        [("content-type", "text/plain; version=0.0.4; charset=utf-8")],
        output,
    )
        .into_response()
}
