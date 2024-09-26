extern crate nixv;
use nixv::nix_commands::nix_build::nix_build_process;
use nixv::nix_commands::nix_build_flake::*;
use nixv::nix_commands::nix_develop_flake::nix_develop_flake_process;
use nixv::nix_commands::nix_shell::nix_shell_process;
use nixv::nix_logs::helpers::log_;
use std::collections::HashMap;
use std::env;
use std::process::{Command, Stdio};
/// This method serves as the entry point for the program. It collects command line arguments, sets up a logging configuration, and handles different commands based on the input arguments.
fn main() {
    let args: Vec<String> = env::args().collect(); // Collect command line arguments
    let mut log_level_map = HashMap::new(); // Create a HashMap for log levels
    log_level_map.insert("error", log::LevelFilter::Error); // Insert log levels into the map
    log_level_map.insert("warn", log::LevelFilter::Warn);
    log_level_map.insert("info", log::LevelFilter::Info);
    log_level_map.insert("debug", log::LevelFilter::Debug);
    log_level_map.insert("trace", log::LevelFilter::Trace);
    let log_level = match env::var("RUST_LOG") { // Set log level based on environment variable
        Ok(v) => log_level_map
            .get(v.as_str())
            .copied()
            .unwrap_or(log::LevelFilter::Info),
        Err(_) => log::LevelFilter::Info,
    };
    env_logger::builder()
        .filter_level(log_level) // Set log level for the logger
        .format(|_buf, record| -> Result<(), std::io::Error> { // Customize log format
            Ok({
                log_(record); // Log the record
            })
        })
        .init(); // Initialize the logger
    let default = &String::from(""); // Create a default string
    match args.split_first() { // Match the first command line argument
        Some((x, xs)) => {
            let command = x.split('/').last().unwrap_or(default); // Extract the command
            match command {
                "nixv" => { // Handle 'nixv' command
                    let (subcommand, xargs) = xs.split_first().unwrap_or((default, &[])); // Extract subcommand and arguments
                    match subcommand.as_str() {
                        "develop" => { // Handle 'develop' subcommand
                            let _ = nix_develop_flake_process(xargs.to_vec().to_owned()); // Call nix develop process
                            let shell = "/bin/bash"; // Set shell
                            let nix_develop_command = format!("nix develop --command {}", shell); // Create nix develop command
                            let mut shell = Command::new("nix-shell"); // Create shell command
                            shell
                                .arg("--command")
                                .arg(&nix_develop_command)
                                .stdin(Stdio::inherit())
                                .stdout(Stdio::inherit())
                                .stderr(Stdio::inherit())
                                .status()
                                .expect("Failed to execute 'nix develop'"); // Execute shell command
                        }
                        "build" => { // Handle 'build' subcommand
                            let _ = nix_build_flake_process(xargs.to_vec().to_owned()); // Call nix build process
                        }
                        _ => println!( // Handle unsupported subcommands
                            "supported commands: [nixv develop , nixv build , nixv-build , nixv-shell]\nlog-level can be set by ENV: RUST_LOG -> [ error , warn , info , debug , trace]\nto dump logs to files set ENV: DUMP_LOGS=true"
                        ),
                    };
                }
                "nixv-build" => { // Handle 'nixv-build' command
                    let _ = nix_build_process(xs.to_vec().to_owned()); // Call nix build process
                }
                "nixv-shell" => { // Handle 'nixv-shell' command
                    let _ = nix_shell_process(xs.to_vec().to_owned()); // Call nix shell process
                    let shell = "/bin/bash"; // Set shell
                    let nix_develop_command = format!("nix-shell --command {}", shell); // Create nix shell command
                    let mut shell = Command::new("nix-shell"); // Create shell command
                    shell
                        .arg("--command")
                        .arg(&nix_develop_command)
                        .stdin(Stdio::inherit())
                        .stdout(Stdio::inherit())
                        .stderr(Stdio::inherit())
                        .status()
                        .expect("Failed to execute 'nix-shell'"); // Execute shell command
                }
                _ => println!( // Handle unsupported commands
                    "supported commands: [nixv develop , nixv build , nixv-build , nixv-shell]\nlog-level can be set by ENV: RUST_LOG -> [ error , warn , info , debug , trace]\nto dump logs to files set ENV: DUMP_LOGS=true"
                ),
            }
        }
        None => println!( // Handle no command line arguments
            "supported commands: [nixv develop , nixv build , nixv-build , nixv-shell]\nlog-level can be set by ENV: RUST_LOG -> [ error , warn , info , debug , trace]\nto dump logs to files set ENV: DUMP_LOGS=true"
        ),
    }
}
