use std::thread;

use super::{helpers::append_log_to_file, types::*};
use serde_json::{self, Value};

fn get_package_from_drv(store_path: String) -> String {
    match store_path.split_once('-') {
        Some((_, xs)) => xs
            .to_owned()
            .replace(".drv", "")
            .replace("\\\"", "")
            .replace('\"', ""),
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

fn str_to_activity_result(activity_result: i64, fields: Value) -> ActivityResult {
    match activity_result {
        100 => {
            let fields_i64: Vec<i64> = serde_json::from_value(fields).unwrap();
            ActivityResult::FileLinked(fields_i64[0], fields_i64[1])
        }
        101 => {
            let fields_str: Vec<String> = serde_json::from_value(fields).unwrap();
            ActivityResult::BuildLogLine(fields_str[0].to_owned())
        }
        102 => {
            let fields_str: Vec<String> = serde_json::from_value(fields).unwrap();
            ActivityResult::UntrustedPath(fields_str[0].to_owned())
        }
        103 => {
            let fields_str: Vec<String> = serde_json::from_value(fields).unwrap();
            ActivityResult::CorruptedPath(fields_str[0].to_owned())
        }
        104 => {
            let fields_str: Vec<String> = serde_json::from_value(fields).unwrap();
            ActivityResult::SetPhase(fields_str[0].to_owned())
        }
        105 => {
            let fields_i64: Vec<i64> = serde_json::from_value(fields).unwrap();
            ActivityResult::Progress(ActivityProgress {
                done: fields_i64[0],
                expected: fields_i64[1],
                running: fields_i64[2],
                failed: fields_i64[3],
            })
        }
        106 => {
            let fields_i64: Vec<i64> = serde_json::from_value(fields).unwrap();
            ActivityResult::SetExpected(number_to_activity_type(fields_i64[0]), fields_i64[1])
        }
        107 => {
            let fields_str: Vec<String> = serde_json::from_value(fields).unwrap();
            ActivityResult::PostBuildLogLine(fields_str[0].to_owned())
        }
        x => panic!("unable to parse str_to_activity_result: {}", x),
    }
}

fn str_to_activity(activity: i64, fields: Value) -> Activity {
    match activity {
        // actUnknown = 0,
        0 => Activity::ActUnknown,
        // actCopyPath = 100,
        100 => {
            let fields_str: Vec<String> = serde_json::from_value(fields).unwrap();
            let store_path = fields_str[0].to_owned();
            let from = fields_str[1].to_owned();
            let to = fields_str[2].to_owned();
            let package_name = get_package_from_drv(store_path.to_owned());
            Activity::ActCopyPath(package_name, store_path, from, to)
        }
        // actFileTransfer = 101,
        101 => {
            let fields_str: Vec<String> = serde_json::from_value(fields).unwrap();
            let nar = fields_str[0].to_owned();
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
            let fields_str: Vec<Value> = serde_json::from_value(fields).unwrap();
            let path: String = serde_json::from_value(fields_str[0].to_owned()).unwrap();
            let host: String = serde_json::from_value(fields_str[1].to_owned()).unwrap();
            let package_name = get_package_from_drv(path.clone());
            Activity::ActBuild(package_name, path, host, 1, 1)
        }
        // actOptimiseStore = 106,
        106 => Activity::ActOptimiseStore,
        // actVerifyPaths = 107,
        107 => Activity::ActVerifyPaths,
        // actSubstitute = 108,
        108 => {
            let fields_str: Vec<String> = serde_json::from_value(fields).unwrap();
            let path = fields_str[0].to_owned();
            let host = fields_str[1].to_owned();
            let package_name = get_package_from_drv(path.clone());
            Activity::ActSubstitute(package_name, path, host)
        }
        // actQueryPathInfo = 109,
        109 => {
            let fields_str: Vec<String> = serde_json::from_value(fields).unwrap();
            let path = fields_str[0].to_owned();
            let host = fields_str[1].to_owned();
            let package_name = get_package_from_drv(path.clone());
            Activity::ActQueryPathInfo(package_name, path, host)
        }
        // actPostBuildHook = 110,
        110 => {
            let fields_str: Vec<String> = serde_json::from_value(fields).unwrap();
            Activity::ActPostBuildHook(fields_str[0].to_owned())
        }
        // actBuildWaiting = 111,
        111 => Activity::ActBuildWaiting,
        _ => panic!("Invalid Activity"),
    }
}

pub fn parse(line: String) -> (Option<JSONMessage>, i64) {
    let opt_res: Result<serde_json::Value, _> = serde_json::from_str(&line.replace("@nix ", ""));
    match opt_res {
        Ok(res) => {
            let action = res.get("action").unwrap().as_str();
            let d_fields = Value::Array([].to_vec());
            let mut id = -1;
            let msg = match action {
                Some("start") => {
                    id = serde_json::from_value(res.get("id").unwrap().to_owned()).unwrap();
                    let fields = match res.get("fields") {
                        Some(v) => v,
                        None => &d_fields,
                    };
                    let level = str_to_verbosity(
                        serde_json::from_value(res.get("level").unwrap().to_owned()).unwrap(),
                    );
                    let text: String =
                        serde_json::from_value(res.get("text").unwrap().to_owned()).unwrap();
                    let text_log: String = text.clone();
                    let _type =
                        serde_json::from_value(res.get("type").unwrap().to_owned()).unwrap();
                    let activity = str_to_activity(_type, fields.clone());

                    //write to log file
                    thread::spawn(move || {
                        let log_file = "id_".to_owned() + &(id.clone().to_string());
                        append_log_to_file(log_file, text_log.to_owned());
                    });

                    Some(JSONMessage::Start(StartAction {
                        id,
                        level,
                        activity,
                        text: text.to_owned(),
                    }))
                }
                Some("stop") => {
                    id = serde_json::from_value(res.get("id").unwrap().to_owned()).unwrap();
                    thread::spawn(move || {
                        let log_file = "id_".to_owned() + &(id.clone().to_string());
                        append_log_to_file(log_file, String::from("done"));
                    });
                    Some(JSONMessage::Stop(StopAction { id }))
                }
                Some("result") => {
                    id = serde_json::from_value(res.get("id").unwrap().to_owned()).unwrap();
                    let fields = res.get("fields").unwrap();
                    let activity: ActivityResult = str_to_activity_result(
                        serde_json::from_value(res.get("type").unwrap().to_owned()).unwrap(),
                        fields.clone(),
                    );
                    let text: Option<String> = match activity.clone() {
                ActivityResult::BuildLogLine(msg) => Some(msg),
                ActivityResult::UntrustedPath(msg) => Some(msg),
                ActivityResult::CorruptedPath(msg) => Some(msg),
                ActivityResult::SetPhase(msg) => Some(msg),
                _ => None
                // ActivityResult::Progress(msg) => Some(),
                // ActivityResult::FileLinked(_, _) => Some(String::from("")),
                // ActivityResult::SetExpected(_, _) => Some(String::from("")),
                // ActivityResult::PostBuildLogLine(_) => Some(String::from("")),
            };
                    match text {
                        Some(msg) => {
                            thread::spawn(move || {
                                let log_file = "id_".to_owned() + &(id.clone().to_string());
                                append_log_to_file(log_file, msg.clone());
                            });
                        }
                        None => (),
                    }
                    Some(JSONMessage::Result(ResultAction {
                        id,
                        result: activity,
                    }))
                }
                Some("msg") => {
                    let level = str_to_verbosity(
                        serde_json::from_value(res.get("level").unwrap().to_owned()).unwrap(),
                    );
                    let msg: String =
                        serde_json::from_value(res.get("msg").unwrap().clone()).unwrap();
                    let msg_log: String =
                        serde_json::from_value(res.get("msg").unwrap().clone()).unwrap();

                    //write to log file
                    thread::spawn(move || {
                        let log_file = "id_".to_owned() + &(id.clone().to_string());
                        append_log_to_file(log_file, msg_log.to_owned());
                    });

                    Some(JSONMessage::Message(MessageAction {
                        level,
                        msg: msg.to_owned(),
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
        Err(err) => {
            log::error!("Failed to parse: {} -> {}", line, err);
            (None, -1)
        }
    }
}
