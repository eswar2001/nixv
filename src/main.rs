extern crate nixv;
use clap::{self, Arg, ArgAction, ArgMatches, Command};
use nixv::nix_commands::nix_build_flake::{nix_build_flake_sub_command, nix_build_flake_process};
use nixv::nix_commands::nix_develop_flake::{nix_develop_flake_sub_command};

fn main() {
    let cmd = clap::Command::new("nixv")
        .bin_name("nixv")
        .subcommand_required(true)
        .subcommand(nix_build_flake_sub_command())
        .subcommand(nix_develop_flake_sub_command());
    match cmd.get_matches().clone().subcommand().unwrap() {
        ("build", args) => {
            println!("{:#?}", args);
            let _ = nix_build_flake_process(&args);
        }
        ("develop", args) => {
            println!("{:#?}", args);
        }
        (subcommand, _) => {
            println!("{} is invalid subcommand", subcommand);
        }
    };
}
