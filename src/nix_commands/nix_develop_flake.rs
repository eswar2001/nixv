use crate::{
    nix_logs::{helpers::dump_state_to_file, parser::parse, process_logs::process_log},
    nix_tracker::types::CommandState,
};
use std::{
    io::{BufRead, BufReader, Error, ErrorKind},
    process::{self as PC, Stdio},
    time::SystemTime,
};

pub fn nix_develop_flake_process(args: Vec<String>) -> Result<(), Error> {
    let mut binding = PC::Command::new("nix");
    let cmd: &mut PC::Command = binding
        .arg("develop")
        .arg("-v")
        .arg("--log-format")
        .arg("internal-json")
        .arg("--extra-experimental-features")
        .arg("flakes")
        .arg("--extra-experimental-features")
        .arg("nix-command")
        .args(args)
        .arg("--command")
        .arg("bash")
        .arg("-c")
        .arg("exit")
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
        }
        Err(_) => {}
    });
    state.end = Some(SystemTime::now());
    dump_state_to_file(state);
    Ok(())
}
