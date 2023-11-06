use std::{
    collections::{HashMap, HashSet},
    time::SystemTime,
};

use serde::Serialize;

use crate::nix_logs::types::{Activity, ActivityProgress};

#[derive(Debug, Serialize, Clone)]

pub struct ActivityState {
    pub activity: Activity,
    pub start: SystemTime,
    pub end: Option<SystemTime>,
    pub phase: Option<String>,
    pub progress: Option<ActivityProgress>,
    pub package_name: Option<String>
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

impl CommandState {
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
}
