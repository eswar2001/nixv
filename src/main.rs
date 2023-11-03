use clap::{self, Arg, ArgAction, ArgMatches, Command};
use serde_json::{self, Value};
use std::{
    io::{BufRead, BufReader, Error, ErrorKind},
    panic,
    process::{self as PC, Stdio},
};

fn max_jobs() -> Arg {
    Arg::new("max-jobs")
        .short('j')
        .long("max-jobs")
        .help("The maximum number of parallel builds.")
        .action(ArgAction::Set)
        .default_value("auto")
}

fn cores() -> Arg {
    Arg::new("cores")
        .long("cores")
        .help("The maximum number of cores allowed.")
        .action(ArgAction::Set)
        .default_value("0")
}

fn nix_build_flake_sub_command() -> Command {
    Command::new("build")
        .about("equivalent of nix build")
        .arg(max_jobs())
        .arg(cores())
}

fn nix_build_flake_process(_args: &ArgMatches) -> Result<(), Error> {
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

struct StopAction {
    id: i64,
}

#[derive(Debug)]
enum Verbosity {
    Error = 0,
    Warn,
    Notice,
    Info,
    Talkative,
    Chatty,
    Debug,
    Vomit,
}

struct StorePath {
    path: String,
}

enum ActivityType {
    ActUnknownType = 0,
    ActCopyPathType = 100,
    ActFileTransferType = 101,
    ActRealiseType = 102,
    ActCopyPathsType = 103,
    ActBuildsType = 104,
    ActBuildType = 105,
    ActOptimiseStoreType = 106,
    ActVerifyPathsType = 107,
    ActSubstituteType = 108,
    ActQueryPathInfoType = 109,
    ActPostBuildHookType = 110,
    ActBuildWaitingType = 111,
}

struct ActivityProgress {
    done: i64,
    expected: i64,
    running: i64,
    failed: i64,
}

struct StartAction {
    id: i64,
    level: Verbosity,
    text: String,
    activity: Activity,
}

struct ResultAction {
    id: i64,
    result: ActivityResult,
}

struct MessageAction {
    level: Verbosity,
    msg: String,
}

enum JSONMessage {
    Stop(StopAction),
    Start(StartAction),
    Result(ResultAction),
    Message(MessageAction),
}

fn str_to_verbosity(lvl: i64) -> Verbosity {
    match lvl {
        0 => Verbosity::Error,
        1 => Verbosity::Warn,
        2 => Verbosity::Notice,
        3 => Verbosity::Info,
        4 => Verbosity::Talkative,
        5 => Verbosity::Chatty,
        6 => Verbosity::Debug,
        7 => Verbosity::Vomit,
        _ => Verbosity::Info,
    }
}

enum ActivityResult {
    FileLinked(i64, i64),
    BuildLogLine(String),
    UntrustedPath(StorePath),
    CorruptedPath(StorePath),
    SetPhase(String),
    Progress(ActivityProgress),
    SetExpected(ActivityType, i64),
    PostBuildLogLine(String),
}

fn number_to_activity_type(number: i64) -> ActivityType {
    match number {
        0 => ActivityType::ActUnknownType,
        100 => ActivityType::ActCopyPathType,
        101 => ActivityType::ActFileTransferType,
        102 => ActivityType::ActRealiseType,
        103 => ActivityType::ActCopyPathsType,
        104 => ActivityType::ActBuildsType,
        105 => ActivityType::ActBuildType,
        106 => ActivityType::ActOptimiseStoreType,
        107 => ActivityType::ActVerifyPathsType,
        108 => ActivityType::ActSubstituteType,
        109 => ActivityType::ActQueryPathInfoType,
        110 => ActivityType::ActPostBuildHookType,
        111 => ActivityType::ActBuildWaitingType,
        _ => ActivityType::ActUnknownType, // Default case for unknown values
    }
}

fn str_to_activity_result(activity_result: i64, fields: Vec<Value>) -> ActivityResult {
    match activity_result {
        100 => ActivityResult::FileLinked(fields[0].as_i64().unwrap(), fields[1].as_i64().unwrap()),
        101 => ActivityResult::BuildLogLine(fields[0].to_string()),
        102 => ActivityResult::UntrustedPath(StorePath {
            path: fields[0].to_string(),
        }),
        103 => ActivityResult::CorruptedPath(StorePath {
            path: fields[0].to_string(),
        }),
        104 => ActivityResult::SetPhase(fields[0].to_string()),
        105 => ActivityResult::Progress(ActivityProgress {
            done: fields[0].as_i64().unwrap(),
            expected: fields[1].as_i64().unwrap(),
            running: fields[2].as_i64().unwrap(),
            failed: fields[3].as_i64().unwrap(),
        }),
        106 => ActivityResult::SetExpected(
            number_to_activity_type(fields[0].as_i64().unwrap()),
            fields[1].as_i64().unwrap(),
        ),
        107 => ActivityResult::PostBuildLogLine(fields[0].to_string()),
        x => panic!("unable to parse str_to_activity_result: {}", x),
    }
}

enum Activity {
    ActUnknown,
    ActCopyPath(StorePath, String, String),
    ActFileTransfer(String),
    ActRealise,
    ActCopyPaths,
    ActBuilds,
    ActBuild(String, String, i16, i16),
    ActOptimiseStore,
    ActVerifyPaths,
    ActSubstitute(StorePath, String),
    ActQueryPathInfo(StorePath, String),
    ActPostBuildHook(StorePath),
    ActBuildWaiting,
}

fn str_to_activity(activity: i64, fields: Vec<Value>) -> Activity {
    match activity {
        // actUnknown = 0,
        0 => Activity::ActUnknown,
        // actCopyPath = 100,
        100 => {
            let store_path = StorePath {
                path: fields[0].to_string(),
            };
            let from = fields[1].to_string();
            let to = fields[2].to_string();
            Activity::ActCopyPath(store_path, from, to)
        }
        // actFileTransfer = 101,
        101 => {
            let nar = fields[0].to_string();
            Activity::ActFileTransfer(nar)
        }
        // actRealise = 102,
        102 => Activity::ActRealise,
        // actCopyPaths = 103,
        103 => Activity::ActCopyPaths,
        // actBuilds = 104,
        104 => Activity::ActBuilds,
        // actBuild = 105,
        105 => {
            let path = fields[0].to_string();
            let host = fields[1].to_string();
            Activity::ActBuild(path, host, 1, 1)
        }
        // actOptimiseStore = 106,
        106 => Activity::ActOptimiseStore,
        // actVerifyPaths = 107,
        107 => Activity::ActVerifyPaths,
        // actSubstitute = 108,
        108 => {
            let path = fields[0].to_string();
            let host = fields[1].to_string();
            Activity::ActSubstitute(StorePath { path: path }, host)
        }
        // actQueryPathInfo = 109,
        109 => {
            let path = fields[0].to_string();
            let host = fields[1].to_string();
            Activity::ActQueryPathInfo(StorePath { path: path }, host)
        }
        // actPostBuildHook = 110,
        110 => Activity::ActPostBuildHook(StorePath {
            path: fields[0].to_string(),
        }),
        // actBuildWaiting = 111,
        111 => Activity::ActBuildWaiting,
        _ => panic!("Invalid Activity"),
    }
}

fn parse(line: String) -> () {
    println!("{:#?}", line);
    let res: serde_json::Value = serde_json::from_str(&line.replace("@nix ", "")).unwrap();
    println!("{:#?}", res);
    let action = res.get("action").unwrap().as_str();
    let _msg = match action {
        Some("start") => {
            let id = res.get("id").unwrap().as_i64().unwrap();
            let fields = match res.get("fields") {
                Some(v) => v.as_array().unwrap().clone(),
                None => Vec::new(),
            };
            let level = str_to_verbosity(res.get("level").unwrap().as_i64().unwrap());
            let text = res.get("text").unwrap().to_string();
            let _type = res.get("type").unwrap().as_i64().unwrap();
            let activity = str_to_activity(_type, fields);
            Some(JSONMessage::Start(StartAction {
                id: id,
                level: level,
                activity: activity,
                text: text,
            }))
        }
        Some("stop") => {
            let id = res.get("id").unwrap().as_i64().unwrap();
            Some(JSONMessage::Stop(StopAction { id: id }))
        }
        Some("result") => {
            let id = res.get("id").unwrap().as_i64().unwrap();
            let fields = res.get("fields").unwrap().as_array().unwrap().clone();
            let activity =
                str_to_activity_result(res.get("type").unwrap().as_i64().unwrap(), fields);
            Some(JSONMessage::Result(ResultAction {
                id: id,
                result: activity,
            }))
        }
        Some("msg") => {
            let level = str_to_verbosity(res.get("level").unwrap().as_i64().unwrap());
            let msg = res.get("msg").unwrap().to_string();
            Some(JSONMessage::Message(MessageAction {
                level: level,
                msg: msg,
            }))
        }
        Some(l) => {
            println!("Missed to handle: {:#?} , json: {:#?}", l, res);
            None
        }
        None => None,
    };
}

fn nix_develop_flake_sub_command() -> Command {
    Command::new("develop")
        .about("equivalent of nix develop")
        .arg(max_jobs())
        .arg(cores())
}

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
