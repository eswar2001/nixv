use crate::{
    nix_args::common_args::*,
    nix_logs::{parser::parse, process_logs::process_log, types::JSONMessage},
    nix_tracker::types::CommandState,
};
use chrono::{DateTime, Utc};
use clap::{ArgMatches, Command};
use std::{
    fs::{File, OpenOptions},
    io::{BufRead, BufReader, Error, ErrorKind, Write},
    process::{self as PC, Stdio},
    thread,
    time::SystemTime,
};

pub fn nix_build_flake_sub_command() -> Command {
    Command::new("build")
        .about("equivalent of nix build")
        .arg(max_jobs())
        .arg(cores())
}

pub fn nix_build_flake_process(_args: &ArgMatches) -> Result<(), Error> {
    // let mut hm: HashMap<i64, Vec<JSONMessage>> = HashMap::new();
    let mut binding = PC::Command::new("nix");
    let cmd = binding
        .arg("build")
        .arg("-v")
        .arg("--log-format")
        .arg("internal-json")
        .stderr(Stdio::piped())
        .stdout(Stdio::piped());
    let p = cmd.spawn().expect("unable to run the command");
    let _stdout = p
        .stdout
        .ok_or_else(|| Error::new(ErrorKind::Other, "Could not capture standard output."))?;

    let stderr = p
        .stderr
        .ok_or_else(|| Error::new(ErrorKind::Other, "Could not capture standard output error."))?;

    let reader = BufReader::new(stderr);
    let mut state = CommandState::new();
    reader.lines().for_each(|l| match l {
        Ok(line) => {
            let (res, id) = parse(line.clone());
            process_log(id, res.clone(), &mut state);
            thread::spawn(move || {
                let log_file = "id_".to_owned() + &(id.clone().to_string());
                append_log_to_file(log_file, res.clone());
            });
        }
        Err(_) => {}
    });
    state.end = Some(SystemTime::now());
    dump_state_to_file(state);
    Ok(())
}

fn append_log_to_file(file_name: String, msg: Option<JSONMessage>) {
    let mut file = OpenOptions::new()
        .write(true)
        .append(true)
        .create(true)
        .open(file_name + ".log")
        .unwrap();

    if let Err(e) = writeln!(file, "{:?}", serde_json::to_string(&msg).unwrap()) {
        eprintln!("Couldn't write to file: {}", e);
    }
}

fn dump_state_to_file(state: CommandState) {
    let now: DateTime<Utc> = Utc::now();
    println!("UTC now is: {}", now);
    let mut file = File::create("command_state_".to_owned() + &now.to_rfc3339() + ".json").unwrap();
    let json_dump = serde_json::to_string_pretty(&state).unwrap();
    let _ = file.write_all(json_dump.as_bytes());
}
