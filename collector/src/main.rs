#![allow(dead_code, unused_imports)]
use std::{time::Duration};
use sensors::Sensor;
use tray_icon::menu::MenuItem;
use std::time::{UNIX_EPOCH};
use std::sync::mpsc;
use std::thread;
use rusqlite::{Connection, Result, ToSql};
use std::time::SystemTime;
use chrono::{DateTime, Utc};
use database::Database;
use tray_icon::{TrayIconBuilder, Icon, menu::Menu, menu::MenuItemKind, TrayIconEvent, menu::MenuEvent};

mod core;
mod database;
mod sensors;

pub fn main() {
    // let conn = Connection::open("test.db").unwrap();
    // conn.execute(
    //     "CREATE TABLE IF NOT EXISTS cpu_data (
    //               id             INTEGER PRIMARY KEY,
    //               timestamp      TEXT NOT NULL,
    //               power_watts    REAL NOT NULL,
    //               usage_percent  REAL NOT NULL
    //               )",
    //     [],
    // ).unwrap();
    // let sensor = sensors::cpu::get_cpu_power_sensor().unwrap();
    // loop {
    //     let power = sensor.read_power_watts().unwrap();
    //     let usage = sensor.read_usage_percent().unwrap();
    //     let mut stmt = conn.prepare("INSERT INTO cpu_data (timestamp, power_watts, usage_percent) VALUES (?1, ?2, ?3)").unwrap();
    //     stmt.execute((DateTime::<Utc>::from(power.time()), power.data(), usage.data())).unwrap();
    //     println!("Inserted data: time {:?}, power {:.3} W, usage {:.2} %", power.time().duration_since(UNIX_EPOCH).unwrap().as_secs_f64().to_string(), power.data(), usage.data());
    //     println!("DateTime from SystemTime: {}", DateTime::<Utc>::from(power.time()));
    //     let mut stmt = conn.prepare("SELECT timestamp FROM cpu_data ORDER BY id DESC LIMIT 1").unwrap();
    //     let date_from_db: DateTime::<Utc> = stmt.query_row([], |row| {
    //         row.get(0)
    //     }).unwrap();
    //     println!("DateTime from DB: {}", date_from_db);
    //     thread::sleep(Duration::from_secs(1));
    // }

    let database = Database::new("test.db").unwrap();
    database.create_tables_if_not_exists().unwrap();
    let sensor = sensors::cpu::get_cpu_power_sensor().unwrap();
    let gpu_sensor = sensors::gpu::get_gpu_power_sensor().unwrap();

    let icon = Icon::from_rgba(vec![0, 0, 0, 0], 1, 1).expect("Failed to create icon");
    let menu = Menu::new();
    let item1 = MenuItem::new("item1", true, None);
    menu.append(&item1).unwrap();

    let tray_icon = TrayIconBuilder::new()
        .with_tooltip("PC Power Collector")
        .with_icon(icon)
        .with_menu(Box::new(menu))
        .build()
        .expect("Failed to create tray icon");

    loop {
        let event = match sensor.read_full_data() {
            Ok(event) => event,
            Err(e) => {
                eprintln!("Error reading CPU data: {:?}", e);
                continue;
            }
        };
        database.insert_cpu_data(&event).unwrap();
        println!("Read CPU data: power {:.3} W, usage {:.2} %", event.data().total_power_watts, event.data().usage_percent);
        tray_icon.set_tooltip(Some(format!("CPU Power: {:.3} W, Usage: {:.2} %", event.data().total_power_watts, event.data().usage_percent))).unwrap();

        let gpu_event = gpu_sensor.read_full_data().unwrap();
        println!("GPU power {:.3} W, CPU power {:.3} W, 2 RAMS : 10 W, total {:.3} W", gpu_event.data().total_power_watts, event.data().total_power_watts, gpu_event.data().total_power_watts + event.data().total_power_watts + 10.0);
        thread::sleep(Duration::from_millis(1000));
    }
    // let (tx, rx) = mpsc::channel();
    // let tx2 = tx.clone();
    // thread::spawn(move || {
    //     let vals = vec![
    //         String::from("hi"),
    //         String::from("from"),
    //         String::from("the"),
    //         String::from("thread"),
    //     ];
    //     for val in vals {
    //         tx.send(val).unwrap();
    //         thread::sleep(Duration::from_secs(1));
    //     }
    // });

    // thread::spawn(move || {
    //     let vals = vec![
    //         String::from("more"),
    //         String::from("messages"),
    //         String::from("for"),
    //         String::from("you"),
    //     ];
    //     for val in vals {
    //         tx2.send(val).unwrap();
    //         thread::sleep(Duration::from_secs(1));
    //     }
    // });

    // for received in rx {
    //     println!("Got: {}", received);
    // }

    // check_permissions();
    // let sensor = sensors::cpu::get_cpu_power_sensor().unwrap();
    // for _ in 0..5 {
    //     println!("Using sensor: {}", sensor.name());
    //     let power = sensor.read_power_watts().unwrap();
    //     println!("CPU Power: {:.3} W at time {:?}", power.value(), power.time().duration_since(UNIX_EPOCH).unwrap());
    //     let usage = sensor.read_usage_percent().unwrap();
    //     println!("CPU Usage: {:.2} % at time {:?}", usage.value(), usage.time().duration_since(UNIX_EPOCH).unwrap());
    //     thread::sleep(Duration::from_secs(1));
    // }
}

fn check_permissions() {
    #[cfg(target_os = "windows")]
    {
        if !is_admin::is_admin() {
            eprintln!("This program requires Administrator privileges on Windows.");
            eprintln!("Please run this program as Administrator.");
            std::process::exit(1);
        }
    }

    #[cfg(target_os = "linux")]
    {
        if !is_root() {
            eprintln!("This program requires root privileges on Linux.");
            eprintln!(
                "Please run with: sudo {}",
                std::env::current_exe().unwrap().display()
            );
            std::process::exit(1);
        }
    }
}
