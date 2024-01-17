use crate::{
    nix_logs::{helpers::dump_state_to_file, parser::parse, process_logs::process_log},
    nix_tracker::types::CommandState,
};
use std::{
    io::{BufRead, BufReader, Error, ErrorKind},
    process::{self as PC, Stdio},
    time::SystemTime,
};
/// Executes a Nix develop command with the specified arguments and captures the standard output and error.
/// If the command fails, it logs an error and exits with status code 1.
pub fn nix_develop_flake_process(args: Vec<String>) -> Result<(), Error> {
    // Create a new command for the "nix" executable
    let mut binding = PC::Command::new("nix");
    // Set up the command with the necessary arguments and options
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
    // Spawn the command and capture the process
    let mut p = cmd.spawn().expect("unable to run the command");
    // Capture the standard output and handle errors
    let _stdout = p
        .stdout
        .as_mut()
        .ok_or_else(|| Error::new(ErrorKind::Other, "Could not capture standard output."))?;
    // Capture the standard error and handle errors
    let stderr = p
        .stderr
        .as_mut()
        .ok_or_else(|| Error::new(ErrorKind::Other, "Could not capture standard output error."))?;
    // Create a buffer reader for the standard error
    let reader = BufReader::new(stderr);
    // Create a state to store command logs
    let mut state = CommandState::new();
    // Process each line of the standard error and update the state
    reader.lines().for_each(|l| match l {
        Ok(line) => {
            let (res, id) = parse(line.clone());
            process_log(id, res.clone(), &mut state);
        }
        Err(_) => {}
    });
    // Set the end time for the command execution
    state.end = Some(SystemTime::now());
    // Dump the state to a file
    dump_state_to_file(state);
    // Check if the command was successful, log an error and exit if it fails
    if !p.wait().expect("command wasn't running").success(){
        log::error!("Nix build failed");
        std::process::exit(1);
    }
    // Return Ok(()) if the command was successful
    Ok(())
}
