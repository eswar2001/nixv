use super::types::*;
use serde_json::{self, Value};

fn get_package_from_drv(store_path: String) -> String {
    match store_path.split_once('-') {
        Some((_, xs)) => xs
            .to_owned()
            .replace(".drv", "")
            .replace("\\\"", "")
            .replace("\"", ""),
        None => store_path,
    }
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
        102 => ActivityResult::UntrustedPath(fields[0].to_string()),
        103 => ActivityResult::CorruptedPath(fields[0].to_string()),
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

fn str_to_activity(activity: i64, fields: Vec<Value>) -> Activity {
    match activity {
        // actUnknown = 0,
        0 => Activity::ActUnknown,
        // actCopyPath = 100,
        100 => {
            let store_path = fields[0].as_str().unwrap();
            let from = fields[1].to_string();
            let to = fields[2].to_string();
            let package_name = get_package_from_drv(store_path.to_owned());
            Activity::ActCopyPath(package_name, store_path.to_string(), from, to)
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
            let package_name = get_package_from_drv(path.clone());
            Activity::ActBuild(package_name, path, host, 1, 1)
        }
        // actOptimiseStore = 106,
        106 => Activity::ActOptimiseStore,
        // actVerifyPaths = 107,
        107 => Activity::ActVerifyPaths,
        // actSubstitute = 108,
        108 => {
            let path = fields[0].to_string();
            let host = fields[1].to_string();
            let package_name = get_package_from_drv(path.clone());
            Activity::ActSubstitute(package_name, path, host)
        }
        // actQueryPathInfo = 109,
        109 => {
            let path = fields[0].to_string();
            let host = fields[1].to_string();
            let package_name = get_package_from_drv(path.clone());
            Activity::ActQueryPathInfo(package_name, path, host)
        }
        // actPostBuildHook = 110,
        110 => Activity::ActPostBuildHook(fields[0].to_string()),
        // actBuildWaiting = 111,
        111 => Activity::ActBuildWaiting,
        _ => panic!("Invalid Activity"),
    }
}

pub fn parse(line: String) -> (Option<JSONMessage>, i64) {
    let res: serde_json::Value = serde_json::from_str(&line.replace("@nix ", "")).unwrap();
    let action = res.get("action").unwrap().as_str();
    let mut id = -1;
    let msg = match action {
        Some("start") => {
            id = res.get("id").unwrap().as_i64().unwrap();
            let fields = match res.get("fields") {
                Some(v) => v.as_array().unwrap().clone(),
                None => Vec::new(),
            };
            let level = str_to_verbosity(res.get("level").unwrap().as_i64().unwrap());
            let text = serde_json::from_value(res.get("text").unwrap().clone()).unwrap();
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
            id = res.get("id").unwrap().as_i64().unwrap();
            Some(JSONMessage::Stop(StopAction { id: id }))
        }
        Some("result") => {
            id = res.get("id").unwrap().as_i64().unwrap();
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
            let msg = serde_json::from_value(res.get("msg").unwrap().clone()).unwrap();
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
    (msg, id)
}
