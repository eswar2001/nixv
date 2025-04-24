use chrono::{DateTime, Utc};
use log::{debug, error, info};
use nixv::nix_commands::nix_build::nix_build_process;
use nixv::nix_commands::nix_build_flake::*;
use nixv::nix_commands::nix_develop_flake::nix_develop_flake_process;
use nixv::nix_commands::nix_shell::nix_shell_process;
use nixv::nix_logs::helpers::log_;
use serde::Serialize;
use std::collections::{HashMap, HashSet, VecDeque};
use std::env;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::thread;
use std::time::Duration;
use sysinfo::*;

#[derive(Serialize, Debug, Clone)]
struct ProcessStats {
    pid: u32,
    name: String,
    cpu_usage: f32,
    memory_usage_bytes: u64,
}

#[derive(Serialize, Debug)]
struct MonitoringOutput {
    #[serde(with = "chrono::serde::ts_milliseconds")]
    timestamp: DateTime<Utc>,
    target_pid: u32,
    main_process: Option<ProcessStats>,
    child_processes: HashMap<u32, ProcessStats>,
}

fn find_all_descendant_pids(start_pid: Pid, sys: &System) -> HashSet<Pid> {
    let mut descendants = HashSet::new();
    let mut queue = VecDeque::new();

    for (pid, process) in sys.processes() {
        if process.parent() == Some(start_pid) {
            if descendants.insert(*pid) {
                queue.push_back(*pid);
            }
        }
    }

    while let Some(current_pid) = queue.pop_front() {
        for (pid, process) in sys.processes() {
            if process.parent() == Some(current_pid) {
                if descendants.insert(*pid) {
                    queue.push_back(*pid);
                }
            }
        }
    }
    descendants
}

fn spawn_pid_monitor(target_pid_u32: u32, interval_ms: u64) -> thread::JoinHandle<()> {
    let monitor_interval = Duration::from_millis(interval_ms);
    let target_pid = Pid::from_u32(target_pid_u32);
    let log_file_path = PathBuf::from("cpu_mem_usage.csv");

    debug!(
        "[Monitor PID {}] Determined log file path: {}",
        target_pid_u32,
        log_file_path.display()
    );

    thread::spawn(move || {
        info!("[Monitor PID {}] Thread started.", target_pid_u32);
        debug!(
            "[Monitor PID {}] Initializing sysinfo::System...",
            target_pid_u32
        );
        let mut sys = System::new_all();
        debug!(
            "[Monitor PID {}] sysinfo::System initialized.",
            target_pid_u32
        );

        info!(
            "[Monitor PID {}] Attempting to open/create log file: {}",
            target_pid_u32,
            log_file_path.display()
        );
        let mut log_file = match OpenOptions::new()
            .create(true)
            .append(true)
            .open(&log_file_path)
        {
            Ok(file) => {
                info!(
                    "[Monitor PID {}] Successfully opened/created log file.",
                    target_pid_u32
                );
                Some(file)
            }
            Err(e) => {
                error!(
                    "[Monitor PID {}] Failed to open log file {}: {}. Monitor thread exiting.",
                    target_pid_u32,
                    log_file_path.display(),
                    e
                );
                None
            }
        };

        let mut loop_count = 0;
        while let Some(ref mut file) = log_file {
            loop_count += 1;
            debug!(
                "[Monitor PID {}] Loop iteration {}",
                target_pid_u32, loop_count
            );

            let timestamp = Utc::now();
            sys.refresh_all();

            let mut main_process_stats: Option<ProcessStats> = None;
            let mut child_process_stats: HashMap<u32, ProcessStats> = HashMap::new();
            let mut process_found = false;

            if let Some(process) = sys.process(target_pid) {
                process_found = true;
                main_process_stats = Some(ProcessStats {
                    pid: process.pid().as_u32(),
                    name: process.name().to_str().unwrap().to_owned(),
                    cpu_usage: process.cpu_usage(),
                    memory_usage_bytes: process.memory(),
                });
            } else {
                debug!(
                    "[Monitor PID {}] Target process (own PID?) not found this cycle.",
                    target_pid_u32
                );
            }

            let descendant_pids = find_all_descendant_pids(target_pid, &sys);

            for child_pid in descendant_pids {
                if let Some(process) = sys.process(child_pid) {
                    let stats = ProcessStats {
                        pid: process.pid().as_u32(),
                        name: process.name().to_str().unwrap().to_owned(),
                        cpu_usage: process.cpu_usage(),
                        memory_usage_bytes: process.memory(),
                    };
                    child_process_stats.insert(stats.pid, stats);
                }
            }
            if loop_count % 20 == 0 {
                debug!(
                    "[Monitor PID {}] Found {} descendants.",
                    target_pid_u32,
                    child_process_stats.len()
                );
            }

            let output = MonitoringOutput {
                timestamp,
                target_pid: target_pid_u32,
                main_process: main_process_stats,
                child_processes: child_process_stats,
            };

            match serde_json::to_string(&output) {
                Ok(mut json_string) => {
                    json_string.push('\n');
                    if let Err(e) = file.write_all(json_string.as_bytes()) {
                        error!(
                            "[Monitor PID {}] Failed to write to log file: {}. Stopping monitor.",
                            target_pid_u32, e
                        );
                        break;
                    }
                }
                Err(e) => {
                    error!(
                        "[Monitor PID {}] Error serializing data to JSON: {}",
                        target_pid_u32, e
                    );
                }
            }

            if !process_found {
                sys.refresh_all();
                if sys.process(target_pid).is_none() {
                    info!("[Monitor PID {}] Target process (main application?) not found. Stopping monitor.", target_pid_u32);
                    break;
                }
            }

            thread::sleep(monitor_interval);
        }

        if log_file.is_none() && loop_count == 0 {
            error!(
                "[Monitor PID {}] Exited before monitoring loop due to file open failure.",
                target_pid_u32
            );
        } else {
            info!(
                "[Monitor PID {}] Monitor thread finished after {} loop iterations.",
                target_pid_u32, loop_count
            );
        }
    })
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let pid_monitor_interval_ms: u64 = env::var("PID_MONITOR_INTERVAL_MS")
        .map_err(|e| {
            eprintln!(
                "Warning: Invalid PID_MONITOR_INTERVAL_MS, using default 500ms. Error: {}",
                e
            );
            e
        })
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

    let main_pid = get_current_pid().map_err(|e| {
        error!("Fatal: Failed to get current process PID: {}", e);
        Box::<dyn std::error::Error>::from(format!("Failed to get PID: {}", e))
    })?;
    let main_pid_u32 = main_pid.as_u32();
    let _persistent_monitor_handle = spawn_pid_monitor(main_pid_u32, pid_monitor_interval_ms);
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
                        _ => print_help(),
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
                _ => print_help(),
            }
        }
        None => print_help(),
    }

    Ok(())
}

fn print_help() {
    let pid_str = get_current_pid()
        .map(|p| p.as_u32().to_string())
        .unwrap_or_else(|_| "?".to_string());
    println!(
        "supported commands: [nixv develop , nixv build , nixv-build , nixv-shell]\n\
         log-level can be set by ENV: RUST_LOG -> [ error , warn , info , debug , trace]\n\
         PID monitor interval (ms) set by ENV: PID_MONITOR_INTERVAL_MS (default: 500)\n\
         Monitor output for main process (PID {}) and children goes to nix_pid_{}_usage.jsonl\n\
         to dump general logs to files set ENV: DUMP_LOGS=true (handled by nixv::nix_logs)",
        pid_str, pid_str
    );
}
