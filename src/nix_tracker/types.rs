use std::{
    collections::{HashMap, HashSet},
    time::SystemTime,
};

use serde::{Deserialize, Serialize};

use crate::nix_logs::types::{Activity, ActivityProgress};

#[derive(Debug, Serialize, Clone)]

pub struct ActivityState {
    pub activity: Activity,
    pub start: SystemTime,
    pub end: Option<SystemTime>,
    pub phase: Option<String>,
    pub progress: Option<ActivityProgress>,
    pub package_name: Option<String>,
}

#[derive(Debug, Serialize)]

pub struct CommandState {
    pub activity: HashMap<i64, ActivityState>,
    pub required_derivations: HashSet<String>,
    pub running: HashSet<i64>,
    pub completed: HashSet<i64>,
    pub failed: HashSet<i64>,
    pub start: SystemTime,
    pub end: Option<SystemTime>,
}

#[derive(Debug, Eq, Serialize, Deserialize, PartialEq, Clone)]
pub struct JSONActCopyPath {
    package_name: String,
    store_path: String,
    from: String,
    to: String,
    start: SystemTime,
    end: SystemTime,
}

#[derive(Debug, Eq, Serialize, Deserialize, PartialEq, Clone)]
pub struct JSONActBuild {
    package_name: String,
    store_path: String,
    host: String,
    start: SystemTime,
    end: SystemTime,
}

#[derive(Debug, Eq, Serialize, Deserialize, PartialEq, Clone)]
pub struct JSONActFileTransfer {
    file: String,
    start: SystemTime,
    end: SystemTime,
}

#[derive(Debug, Eq, Serialize, Deserialize, PartialEq, Clone)]
pub struct JSONActOptimiseStore {
    start: SystemTime,
    end: SystemTime,
}

#[derive(Debug, Eq, Serialize, Deserialize, PartialEq, Clone)]
pub struct JSONActUnknown {
    start: SystemTime,
    end: SystemTime,
}

#[derive(Debug, Eq, Serialize, Deserialize, PartialEq, Clone)]
pub struct JSONActVerifyPaths {
    start: SystemTime,
    end: SystemTime,
}

#[derive(Debug, Eq, Serialize, Deserialize, PartialEq, Clone)]
pub struct JSONActRealise {
    start: SystemTime,
    end: SystemTime,
}

#[derive(Debug, Eq, Serialize, Deserialize, PartialEq, Clone)]
pub struct JSONActCopyPaths {
    start: SystemTime,
    end: SystemTime,
}

#[derive(Debug, Eq, Serialize, Deserialize, PartialEq, Clone)]
pub struct JSONActBuilds {
    start: SystemTime,
    end: SystemTime,
}

#[derive(Debug, Eq, Serialize, Deserialize, PartialEq, Clone)]
pub struct JSONActBuildWaiting {
    start: SystemTime,
    end: SystemTime,
}

#[derive(Debug, Eq, Serialize, Deserialize, PartialEq, Clone)]
pub struct JSONActSubstitute {
    package_name: String,
    store_path: String,
    from: String,
    start: SystemTime,
    end: SystemTime,
}

#[derive(Debug, Eq, Serialize, Deserialize, PartialEq, Clone)]
pub struct JSONActQueryPathInfo {
    package_name: String,
    store_path: String,
    from: String,
    start: SystemTime,
    end: SystemTime,
}

#[derive(Debug, Eq, Serialize, Deserialize, PartialEq, Clone)]
pub struct JSONActPostBuildHook {
    store_path: String,
    start: SystemTime,
    end: SystemTime,
}

#[derive(Debug, Eq, Serialize, Deserialize, PartialEq, Clone)]
pub struct JSONCommandState {
    pub act_unknown: Vec<JSONActUnknown>,
    pub act_copy_path: Vec<JSONActCopyPath>,
    pub act_file_transfer: Vec<JSONActFileTransfer>,
    pub act_realise: Vec<JSONActRealise>,
    pub act_copy_paths: Vec<JSONActCopyPaths>,
    pub act_builds: Vec<JSONActBuilds>,
    pub act_build: Vec<JSONActBuild>,
    pub act_optimise_store: Vec<JSONActOptimiseStore>,
    pub act_verify_paths: Vec<JSONActVerifyPaths>,
    pub act_substitute: Vec<JSONActSubstitute>,
    pub act_query_path_info: Vec<JSONActQueryPathInfo>,
    pub act_post_build_hook: Vec<JSONActPostBuildHook>,
    pub act_build_waiting: Vec<JSONActBuildWaiting>,
    pub start: SystemTime,
    pub end: SystemTime,
    pub required_derivations: HashSet<String>,
}

impl CommandState {
    /// Creates a new CommandState with default values for activity, required_derivations, running, completed, failed, start, and end.
    pub fn new() -> CommandState {
        CommandState {
            activity: HashMap::new(),
            required_derivations: HashSet::new(),
            running: HashSet::new(),
            completed: HashSet::new(),
            failed: HashSet::new(),
            start: SystemTime::now(),
            end: None, // Initialize end as None by default
        }
    }
    /// Converts the given CommandState into a JSONCommandState by iterating through the activities and converting them into their corresponding JSON representations, then returning a new JSONCommandState with the converted activities and other relevant state information.
    pub fn to_json(state: CommandState) -> JSONCommandState {
        let mut act_unknown = Vec::new();
        let mut act_copy_path = Vec::new();
        let mut act_file_transfer = Vec::new();
        let mut act_realise = Vec::new();
        let mut act_copy_paths = Vec::new();
        let mut act_builds = Vec::new();
        let mut act_build = Vec::new();
        let mut act_optimise_store = Vec::new();
        let mut act_verify_paths = Vec::new();
        let mut act_substitute = Vec::new();
        let mut act_query_path_info = Vec::new();
        let mut act_post_build_hook = Vec::new();
        let mut act_build_waiting = Vec::new();
        for (_, act) in state.activity {
            let start = act.start;
            let end = act.end.unwrap_or(SystemTime::now());
            match act.activity {
                Activity::ActCopyPath(package_name, store_path, from, to) => {
                    act_copy_path.push(JSONActCopyPath {
                        start,
                        end,
                        package_name,
                        store_path,
                        from,
                        to,
                    })
                }
                Activity::ActBuild(package_name, store_path, host, _, _) => {
                    act_build.push(JSONActBuild {
                        start,
                        end,
                        package_name,
                        store_path,
                        host,
                    })
                }
                Activity::ActFileTransfer(file) => {
                    act_file_transfer.push(JSONActFileTransfer { start, end, file })
                }
                Activity::ActSubstitute(package_name, store_path, from) => {
                    act_substitute.push(JSONActSubstitute {
                        start,
                        end,
                        package_name,
                        store_path,
                        from,
                    })
                }
                Activity::ActQueryPathInfo(package_name, store_path, from) => act_query_path_info
                    .push(JSONActQueryPathInfo {
                        start,
                        end,
                        package_name,
                        store_path,
                        from,
                    }),
                Activity::ActPostBuildHook(store_path) => {
                    act_post_build_hook.push(JSONActPostBuildHook {
                        store_path: store_path,
                        start,
                        end,
                    })
                }
                Activity::ActRealise => act_realise.push(JSONActRealise { start, end }),
                Activity::ActCopyPaths => act_copy_paths.push(JSONActCopyPaths { start, end }),
                Activity::ActBuilds => act_builds.push(JSONActBuilds { start, end }),
                Activity::ActUnknown => act_unknown.push(JSONActUnknown { start, end }),
                Activity::ActOptimiseStore => {
                    act_optimise_store.push(JSONActOptimiseStore { start, end })
                }
                Activity::ActVerifyPaths => {
                    act_verify_paths.push(JSONActVerifyPaths { start, end })
                }
                Activity::ActBuildWaiting => {
                    act_build_waiting.push(JSONActBuildWaiting { start, end })
                }
            }
        }
 
        JSONCommandState {
            act_unknown,
            act_copy_path,
            act_file_transfer,
            act_realise,
            act_copy_paths,
            act_builds,
            act_build,
            act_optimise_store,
            act_verify_paths,
            act_substitute,
            act_query_path_info,
            act_post_build_hook,
            act_build_waiting,
            start: state.start,
            end: state.end.unwrap(),
            required_derivations: state.required_derivations,
        }
    }
}
