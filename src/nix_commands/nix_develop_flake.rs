use clap::Command;

use crate::nix_args::common_args::{max_jobs, cores};

pub fn nix_develop_flake_sub_command() -> Command {
    Command::new("develop")
        .about("equivalent of nix develop")
        .arg(max_jobs())
        .arg(cores())
}