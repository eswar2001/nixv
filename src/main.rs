extern crate nixv;
use clap::{self};
use nixv::nix_commands::nix_build_flake::*;
use nixv::nix_commands::nix_develop_flake::nix_develop_flake_sub_command;
use nixv::nix_logs::helpers::filter_ansi;
use yansi::Paint;

fn main() {
    env_logger::builder()
        .format(|_buf, record| {
            Ok({
                let str = record.args().to_string();
                match record.level() {
                    log::Level::Error => {
                        println!("{}", Paint::red(&filter_ansi(str)),)
                    }
                    log::Level::Warn => {
                        println!("{}", Paint::magenta(&filter_ansi(str)),)
                    }
                    log::Level::Info => {
                        println!("{}", Paint::white(&filter_ansi(str)),)
                    }
                    log::Level::Debug => {
                        println!("{}", Paint::bright_yellow(&filter_ansi(str)),)
                    }
                    log::Level::Trace => {
                        println!("{}", Paint::blue(&filter_ansi(str)),)
                    }
                }
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
            println!("{:#?}", args);
        }
        (subcommand, _) => {
            println!("{} is invalid subcommand", subcommand);
        }
    };
}
