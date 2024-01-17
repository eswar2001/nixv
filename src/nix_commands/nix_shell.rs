use crate::{
    nix_logs::{helpers::dump_state_to_file, parser::parse, process_logs::process_log},
    nix_tracker::types::CommandState,
};
use std::{
    io::{BufRead, BufReader, Error, ErrorKind},
    process::{self as PC, Stdio},
    time::SystemTime,
};
/// Executes a nix-shell process with the given arguments and captures its standard output and error.
/// # Arguments
/// * `args` - A vector of strings representing the arguments to pass to the nix-shell command
/// # Returns
/// * `Result<(), Error>` - An empty result if the process execution is successful, or an error if it fails
pub fn nix_shell_process(args: Vec<String>) -> Result<(), Error> {
    // Create a new command for nix-shell
    let mut binding = PC::Command::new("nix-shell");
    // Add arguments and set up I/O redirection for the command
    let cmd: &mut PC::Command = binding
        .arg("-v")
        .arg("--log-format")
        .arg("internal-json")
        .args(args)
        .args(["--command", "bash -c exit"])
        .stderr(Stdio::piped())
        .stdout(Stdio::piped());
    // Spawn the command and capture its standard output and error
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
    // Process the lines from standard error and update the command state
    reader.lines().for_each(|l| match l {
        Ok(line) => {
            let (res, id) = parse(line.clone());
            process_log(id, res.clone(), &mut state);
        }
        Err(_) => {}
    });
    state.end = Some(SystemTime::now());
    // Dump the final state to a file
    dump_state_to_file(state);
    // Check if the command was successful and handle errors
    if !p.wait().expect("command wasn't running").success(){
        log::error!("Nix build failed");
        std::process::exit(1);
    }
    Ok(())
}
