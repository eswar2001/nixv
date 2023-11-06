extern crate nixv;
use clap::{self};
use nixv::nix_commands::nix_build_flake::*;
use nixv::nix_commands::nix_develop_flake::{
    nix_develop_flake_process, nix_develop_flake_sub_command,
};
use nixv::nix_logs::helpers::log_async;
use std::collections::HashMap;
use std::env;
use std::process::{self as PC, Stdio};

fn main() {
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
                log_async(record);
            })
        })
        .init();
    let cmd = clap::Command::new("nixv")
        .bin_name("nixv")
        .subcommand_required(true)
        .subcommand(nix_build_flake_sub_command())
        .subcommand(nix_develop_flake_sub_command());
    match cmd.get_matches().clone().subcommand().unwrap() {
        ("build", args) => {
            let _ = nix_build_flake_process(&args);
        }
        ("develop", args) => {
            let _ = nix_develop_flake_process(&args);
            // To get into interactive shell
            let shell = "/bin/bash";
            let nix_develop_command = format!("nix develop --command {}", shell);
            let mut shell = PC::Command::new("nix-shell");
            shell
                .arg("--command")
                .arg(&nix_develop_command)
                .stdin(Stdio::inherit())
                .stdout(Stdio::inherit())
                .stderr(Stdio::inherit())
                .status()
                .expect("Failed to execute 'nix develop'");
        }
        (subcommand, _) => {
            println!("{} is invalid subcommand", subcommand);
        }
    };
}
