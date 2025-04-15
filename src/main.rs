extern crate nixv;
use chrono::Utc;
use nixv::nix_commands::nix_build::nix_build_process;
use nixv::nix_commands::nix_build_flake::*;
use nixv::nix_commands::nix_develop_flake::nix_develop_flake_process;
use nixv::nix_commands::nix_shell::nix_shell_process;
use nixv::nix_logs::helpers::log_;
use std::collections::HashMap;
use std::env;
use std::fs::OpenOptions;
use std::io::Write;
use std::process::{Command, Stdio};
use std::thread;
use std::time::Duration;
use sysinfo::System;

fn log_usage(cpu: f32, memory: u64) {
    let now = Utc::now().to_rfc3339();
    let log_line = format!("{}, {:.2}, {}\n", now, cpu, memory);

    if let Ok(mut file) = OpenOptions::new()
        .create(true)
        .append(true)
        .open("cpu_mem_usage.csv")
    {
        let _ = file.write_all(log_line.as_bytes());
    }
}

fn main() {
    let sleep_microsecs: u64 = env::var("SLEEP_DURATION_MICRO_SECS")
        .ok()
        .and_then(|val| val.parse().ok())
        .unwrap_or(1);
    let args: Vec<String> = env::args().collect();
    let mut log_level_map = HashMap::new();
    log_level_map.insert("error", log::LevelFilter::Error);
    log_level_map.insert("warn", log::LevelFilter::Warn);
    log_level_map.insert("info", log::LevelFilter::Info);
    log_level_map.insert("debug", log::LevelFilter::Debug);
    log_level_map.insert("trace", log::LevelFilter::Trace);
    let log_level = match env::var("RUST_LOG") {
        Ok(v) => log_level_map
            .get(v.as_str())
            .copied()
            .unwrap_or(log::LevelFilter::Info),
        Err(_) => log::LevelFilter::Info,
    };
    env_logger::builder()
        .filter_level(log_level)
        .format(|_buf, record| -> Result<(), std::io::Error> {
            Ok({
                log_(record);
            })
        })
        .init();
    thread::spawn(move || {
        let mut sys = System::new_all();
        loop {
            sys.refresh_all();
            sys.refresh_all();
            let cpu = sys.global_cpu_usage();
            let memory = sys.used_memory();
            log_usage(cpu, memory);
            thread::sleep(Duration::from_micros(sleep_microsecs));
        }
    });
    let default = &String::from("");
    match args.split_first() {
        Some((x, xs)) => {
            let command = x.split('/').last().unwrap_or(default);
            match command {
                "nixv" => {
                    let (subcommand, xargs) = xs.split_first().unwrap_or((default, &[]));
                    match subcommand.as_str() {
                        "develop" => {
                            let _ = nix_develop_flake_process(xargs.to_vec().to_owned());
                            let shell = "/bin/bash";
                            let nix_develop_command = format!("nix develop --command {}", shell);
                            let mut shell = Command::new("nix-shell");
                            shell
                                .arg("--command")
                                .arg(&nix_develop_command)
                                .stdin(Stdio::inherit())
                                .stdout(Stdio::inherit())
                                .stderr(Stdio::inherit())
                                .status()
                                .expect("Failed to execute 'nix develop'");
                        }
                        "build" => {
                            let _ = nix_build_flake_process(xargs.to_vec().to_owned());
                        }
                        _ => println!(
                            "supported commands: [nixv develop , nixv build , nixv-build , nixv-shell]\nlog-level can be set by ENV: RUST_LOG -> [ error , warn , info , debug , trace]\nto dump logs to files set ENV: DUMP_LOGS=true"
                        ),
                    };
                }
                "nixv-build" => {
                    let _ = nix_build_process(xs.to_vec().to_owned());
                }
                "nixv-shell" => {
                    let _ = nix_shell_process(xs.to_vec().to_owned());
                    let shell = "/bin/bash";
                    let nix_develop_command = format!("nix-shell --command {}", shell);
                    let mut shell = Command::new("nix-shell");
                    shell
                        .arg("--command")
                        .arg(&nix_develop_command)
                        .stdin(Stdio::inherit())
                        .stdout(Stdio::inherit())
                        .stderr(Stdio::inherit())
                        .status()
                        .expect("Failed to execute 'nix-shell'");
                }
                _ => println!(
                    "supported commands: [nixv develop , nixv build , nixv-build , nixv-shell]\nlog-level can be set by ENV: RUST_LOG -> [ error , warn , info , debug , trace]\nto dump logs to files set ENV: DUMP_LOGS=true"
                ),
            }
        }
        None => println!(
            "supported commands: [nixv develop , nixv build , nixv-build , nixv-shell]\nlog-level can be set by ENV: RUST_LOG -> [ error , warn , info , debug , trace]\nto dump logs to files set ENV: DUMP_LOGS=true"
        ),
    }
}
