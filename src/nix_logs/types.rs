
#[derive(Debug, Eq, PartialEq)]
pub struct StopAction {
    pub(crate) id: i64,
}

#[derive(Debug, Eq, PartialEq)]
pub enum Verbosity {
    Error = 0,
    Warn,
    Notice,
    Info,
    Talkative,
    Chatty,
    Debug,
    Vomit,
}

#[derive(Debug, Eq, PartialEq)]
pub struct StorePath {
    pub(crate) path: String,
}

#[derive(Debug, Eq, PartialEq)]
pub enum ActivityType {
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

#[derive(Debug, Eq, PartialEq)]
pub struct ActivityProgress {
    pub(crate) done: i64,
    pub(crate) expected: i64,
    pub(crate) running: i64,
    pub(crate) failed: i64,
}

#[derive(Debug, Eq, PartialEq)]
pub struct StartAction {
    pub(crate) id: i64,
    pub(crate) level: Verbosity,
    pub(crate) text: String,
    pub(crate) activity: Activity,
}
#[derive(Debug, Eq, PartialEq)]
pub struct ResultAction {
    pub(crate) id: i64,
    pub(crate) result: ActivityResult,
}
#[derive(Debug, Eq, PartialEq)]
pub struct MessageAction {
    pub(crate) level: Verbosity,
    pub(crate) msg: String,
}
#[derive(Debug, Eq, PartialEq)]
pub enum JSONMessage {
    Stop(StopAction),
    Start(StartAction),
    Result(ResultAction),
    Message(MessageAction),
}
#[derive(Debug, Eq, PartialEq)]
pub enum ActivityResult {
    FileLinked(i64, i64),
    BuildLogLine(String),
    UntrustedPath(StorePath),
    CorruptedPath(StorePath),
    SetPhase(String),
    Progress(ActivityProgress),
    SetExpected(ActivityType, i64),
    PostBuildLogLine(String),
}
#[derive(Debug, Eq, PartialEq)]
pub enum Activity {
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
