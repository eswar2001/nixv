use crate::{
    nix_logs::{helpers::dump_state_to_file, parser::parse, process_logs::process_log},
    nix_tracker::types::CommandState,
};
use std::{
    io::{BufRead, BufReader, Error, ErrorKind},
    process::{self as PC, Stdio},
    time::SystemTime,
};
/// Executes a nix-build process with the given arguments and processes the output. The method captures the standard output and error, parses the internal JSON log format, and logs the process state. If the nix-build process fails, the method logs an error and exits the program with a status code of 1.
pub fn nix_build_process(args: Vec<String>) -> Result<(), Error> {
    let mut binding = PC::Command::new("nix-build");
    let cmd = binding
        .arg("-v")
        .arg("--log-format")
        .arg("internal-json")
        .args(args)
        .stderr(Stdio::piped())
        .stdout(Stdio::piped());
    let mut p = cmd.spawn().expect("unable to run the command");
    let _stdout = p
        .stdout
        .as_mut()
        .ok_or_else(|| Error::new(ErrorKind::Other, "Could not capture standard output."))?;

    let stderr = p
        .stderr
        .as_mut()
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
    if !p.wait().expect("command wasn't running").success(){
        log::error!("Nix build failed");
        std::process::exit(1);
    }
    Ok(())
}
