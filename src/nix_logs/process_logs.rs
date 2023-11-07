use yansi::Paint;

use crate::nix_tracker::types::{ActivityState, CommandState};

use super::types::{JSONMessage, Verbosity};
use std::time::SystemTime;

pub fn process_log(
    id: i64,
    opt_msg: Option<JSONMessage>,
    state: &mut CommandState,
) -> &mut CommandState {
    match opt_msg {
        Some(JSONMessage::Start(msg)) => {
            let (id, _level, _text, activity) = (msg.id, msg.level, msg.text, msg.activity);
            match activity {
                super::types::Activity::ActCopyPath(package_name, store_path, from, to) => {
                    let now = SystemTime::now();
                    log::trace!(
                        "id: {} -> activity: {} -> time {:?}",
                        id,
                        "ActCopyPath",
                        now
                    );
                    let new_activity_state = ActivityState {
                        activity: super::types::Activity::ActCopyPath(
                            package_name.clone(),
                            store_path.clone(),
                            from,
                            to,
                        ),
                        start: now,
                        end: None,
                        phase: None,
                        progress: None,
                        package_name: Some(package_name),
                    };
                    state.activity.insert(id, new_activity_state);
                    state.required_derivations.insert(store_path.clone());
                }
                super::types::Activity::ActFileTransfer(url) => {
                    let now = SystemTime::now();
                    log::trace!(
                        "id: {} -> activity: {} -> time {:?}",
                        id,
                        "ActFileTransfer",
                        now
                    );
                    let new_activity_state = ActivityState {
                        activity: super::types::Activity::ActFileTransfer(url.clone()),
                        start: now,
                        end: None,
                        phase: None,
                        progress: None,
                        package_name: None,
                    };
                    state.activity.insert(id, new_activity_state);
                }
                super::types::Activity::ActSubstitute(package_name, store_path, from) => {
                    let now = SystemTime::now();
                    log::trace!(
                        "id: {} -> activity: {} -> time {:?}",
                        id,
                        "ActSubstitute",
                        now
                    );
                    let new_activity_state = ActivityState {
                        activity: super::types::Activity::ActSubstitute(
                            package_name.clone(),
                            store_path.clone(),
                            from,
                        ),
                        start: now,
                        end: None,
                        phase: None,
                        progress: None,
                        package_name: Some(package_name),
                    };
                    state.activity.insert(id, new_activity_state);
                    state.required_derivations.insert(store_path.clone());
                }
                super::types::Activity::ActQueryPathInfo(package_name, store_path, from) => {
                    let now = SystemTime::now();
                    log::trace!(
                        "id: {} -> activity: {} -> time {:?}",
                        id,
                        "ActQueryPathInfo",
                        now
                    );
                    let new_activity_state = ActivityState {
                        activity: super::types::Activity::ActSubstitute(
                            package_name.clone(),
                            store_path.clone(),
                            from,
                        ),
                        start: now,
                        end: None,
                        phase: None,
                        progress: None,
                        package_name: Some(package_name),
                    };
                    state.activity.insert(id, new_activity_state);
                    state.required_derivations.insert(store_path.clone());
                }
                super::types::Activity::ActPostBuildHook(store_path) => {
                    let now = SystemTime::now();
                    log::trace!(
                        "id: {} -> activity: {} -> time {:?}",
                        id,
                        "ActPostBuildHook",
                        now
                    );
                    let new_activity_state = ActivityState {
                        activity: super::types::Activity::ActPostBuildHook(store_path.clone()),
                        start: now,
                        end: None,
                        phase: None,
                        progress: None,
                        package_name: None,
                    };
                    state.activity.insert(id, new_activity_state);
                    state.required_derivations.insert(store_path.clone());
                }
                super::types::Activity::ActBuild(package_name, store_path, host, _, _) => {
                    let now = SystemTime::now();
                    log::trace!("id: {} -> activity: {} -> time {:?}", id, "ActBuild", now);
                    let new_activity_state = ActivityState {
                        activity: super::types::Activity::ActBuild(
                            package_name.clone(),
                            store_path.clone(),
                            host,
                            1,
                            1,
                        ),
                        start: now,
                        end: None,
                        phase: None,
                        progress: None,
                        package_name: Some(package_name),
                    };
                    state.activity.insert(id, new_activity_state);
                    state.required_derivations.insert(store_path.clone());
                }
                act => {
                    let now = SystemTime::now();
                    let new_activity_state = ActivityState {
                        activity: act,
                        start: now,
                        end: None,
                        phase: None,
                        progress: None,
                        package_name: None,
                    };
                    state.activity.insert(id, new_activity_state);
                } // super::types::Activity::ActCopyPaths => todo!()
                  // super::types::Activity::ActBuilds => => todo!()
                  // super::types::Activity::ActOptimiseStore => todo!(),
                  // super::types::Activity::ActVerifyPaths => todo!(),
                  // super::types::Activity::ActBuildWaiting => todo!(),
                  // super::types::Activity::ActUnknown => todo!()
            }
        }
        Some(JSONMessage::Stop(act)) => {
            let end = SystemTime::now();
            let id = &act.id;
            match state.activity.get(id) {
                Some(v) => {
                    let v_updated = ActivityState {
                        activity: v.activity.clone(),
                        start: v.start,
                        end: Some(end),
                        phase: v.phase.clone(),
                        progress: v.progress,
                        package_name: v.package_name.clone(),
                    };
                    state.activity.insert(*id, v_updated);
                }
                None => {
                    log::trace!("id not found in the HM: {} -> Stop", id);
                }
            }
        }
        Some(JSONMessage::Result(act)) => match act.result {
            super::types::ActivityResult::SetPhase(phase) => {
                let id = &act.id;
                match state.activity.get(id) {
                    Some(v) => {
                        let v_updated = ActivityState {
                            activity: v.activity.clone(),
                            start: v.start,
                            end: v.end,
                            phase: Some(phase),
                            progress: v.progress,
                            package_name: v.package_name.clone(),
                        };
                        state.activity.insert(*id, v_updated);
                    }
                    None => {
                        log::trace!("id not found in the HM Result: {} -> {}", id, phase);
                    }
                }
            }
            super::types::ActivityResult::Progress(progress) => {
                let id = &act.id;
                match state.activity.get(id) {
                    Some(v) => {
                        let v_updated = ActivityState {
                            activity: v.activity.clone(),
                            start: v.start,
                            end: v.end,
                            phase: v.phase.to_owned(),
                            progress: Some(progress),
                            package_name: v.package_name.clone(),
                        };
                        state.activity.insert(*id, v_updated);
                    }
                    None => {
                        log::trace!("id not found in the HM Progress: {} -> {:#?}", id, progress);
                    }
                }
            }
            super::types::ActivityResult::BuildLogLine(log) => {
                let data_about_build = &state.activity.get(&id).unwrap().package_name;
                let utf8_string = strip_ansi_escapes::strip_str(log);
                let mut pkg_name = match data_about_build {
                    Some(p) => p.to_string(),
                    None => "".to_string(),
                };
                if !pkg_name.is_empty() {
                    pkg_name.push('>');
                }
                if utf8_string.contains("warning") {
                    log::warn!("{} {}", Paint::green(&pkg_name), utf8_string);
                } else if utf8_string.contains("error") {
                    log::error!("{} {}", Paint::green(&pkg_name), utf8_string);
                } else {
                    log::info!("{} {}", Paint::green(&pkg_name), utf8_string);
                }
            }
            super::types::ActivityResult::PostBuildLogLine(log) => {
                log::trace!("PostBuildLogLine: {}", log);
            }
            super::types::ActivityResult::UntrustedPath(log) => {
                log::warn!("PostBuildLogLine: {}", log);
            }
            super::types::ActivityResult::CorruptedPath(log) => {
                log::error!("CorruptedPath: {}", log);
            }
            _ => {} // super::types::ActivityResult::FileLinked(a, b) => {
                    //     log::error!("FileLinked: {} && {}", a, b);
                    // }
                    // super::types::ActivityResult::SetExpected(activity, i) => {
                    //     log::trace!("SetExpected: {:?} && {}", activity, i);
                    // }
        },
        Some(JSONMessage::Message(act)) => {
            let no_package_name = &"".to_string();
            let pkg_name: &mut String = &mut match state.activity.get(&id) {
                Some(v) => match &v.package_name {
                    Some(pkg_name) => pkg_name.to_string(),
                    None => no_package_name.to_string(),
                },
                None => no_package_name.to_string(),
            };
            let (lvl, log) = (act.level.to_owned(), act.msg.to_owned());
            let utf8_string = strip_ansi_escapes::strip_str(log);
            if pkg_name != no_package_name {
                pkg_name.push('>');
                match lvl {
                    Verbosity::Error => log::error!("{} {}", Paint::green(pkg_name), utf8_string),
                    Verbosity::Warn => log::warn!("{} {}", Paint::green(pkg_name), utf8_string),
                    Verbosity::Notice => log::warn!("{} {}", Paint::green(pkg_name), utf8_string),
                    Verbosity::Info => log::info!("{} {}", Paint::green(pkg_name), utf8_string),
                    _ => log::trace!("{} {}", Paint::green(pkg_name), utf8_string),
                };
            } else {
                match lvl {
                    Verbosity::Error => log::error!("{}", utf8_string),
                    Verbosity::Warn => log::warn!("{}", utf8_string),
                    Verbosity::Notice => log::warn!("{}", utf8_string),
                    Verbosity::Info => log::info!("{}", utf8_string),
                    _ => log::trace!("{}", utf8_string),
                };
            }
        }
        None => {}
    }
    state
}
