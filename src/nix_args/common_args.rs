use clap::{Arg, ArgAction};

pub fn max_jobs() -> Arg {
    Arg::new("max-jobs")
        .short('j')
        .long("max-jobs")
        .help("The maximum number of parallel builds.")
        .action(ArgAction::Set)
        .default_value("auto")
}

pub fn cores() -> Arg {
    Arg::new("cores")
        .long("cores")
        .help("The maximum number of cores allowed.")
        .action(ArgAction::Set)
        .default_value("0")
}