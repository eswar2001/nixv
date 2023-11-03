
use std::{
    io::{BufRead, BufReader, Error, ErrorKind},
    process::{self as PC, Stdio},
};
use clap::{Command, ArgMatches};
use crate::{nix_args::common_args::*, nix_logs::parser::parse};


pub fn nix_build_flake_sub_command() -> Command {
    Command::new("build")
        .about("equivalent of nix build")
        .arg(max_jobs())
        .arg(cores())
}

pub fn nix_build_flake_process(_args: &ArgMatches) -> Result<(), Error> {
    let mut binding = PC::Command::new("nix");
    let cmd = binding
        .arg("build")
        .arg("-v")
        .arg("--log-format")
        .arg("internal-json")
        .stderr(Stdio::piped())
        .stdout(Stdio::piped());
    let p = cmd.spawn().expect("unable to run the command");
    println!("{}", p.id());
    let stdout = p
        .stdout
        .ok_or_else(|| Error::new(ErrorKind::Other, "Could not capture standard output."))?;

    let stderr = p
        .stderr
        .ok_or_else(|| Error::new(ErrorKind::Other, "Could not capture standard output error."))?;

    let reader = BufReader::new(stderr);

    reader.lines().filter_map(|line| line.ok()).for_each(|l| {
        println!("{:#?}", parse(l));
    });
    Ok(())
}