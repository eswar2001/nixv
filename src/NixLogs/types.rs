use serde::{Deserialize, Serialize};

// #[derive(Debug, Serialize, Deserialize)]
// struct ActivityId {
//     id: i64,
// }

// #[derive(Debug, Serialize, Deserialize)]
// enum NixJSONMessage {
//     Stop(StopAction),
//     Start(StartAction),
//     Result(ResultAction),
//     Message(MessageAction),
//     Plain(Vec<u8>),
//     ParseError(NOMError),
// }

// #[derive(Debug, Serialize, Deserialize)]
// struct StopAction {
//     id: ActivityId,
// }

// #[derive(Debug, Serialize, Deserialize)]
// struct StartAction {
//     id: ActivityId,
//     level: i16,
//     actType: i16,
//     text: String,
// }

// #[derive(Debug, Serialize, Deserialize)]
// struct MessageAction {
//     level: i16,
//     actType: i16,
//     msg: String,
//     raw_msg: String,
// }

// #[derive(Debug, Serialize, Deserialize)]
// struct ResultAction {
//     fields:Vec<u8>,
//     actType:i16,
//     id: ActivityId,
// }

// {"action":"result","fields":[7938048,15416628,0,0],"id":218725504516505,"type":105}

// json["action"] = "msg";
// json["level"] = ei.level;
// json["msg"] = oss.str();
// json["raw_msg"] = ei.msg.str();

// #[derive(Debug, Serialize, Deserialize)]
// enum ActivityType {
//     ActUnknown = 0,
//     ActCopyPath = 100,
//     ActFileTransfer = 101,
//     ActRealise = 102,
//     ActCopyPaths = 103,
//     ActBuilds = 104,
//     ActBuild = 105,
//     ActOptimiseStore = 106,
//     ActVerifyPaths = 107,
//     ActSubstitute = 108,
//     ActQueryPathInfo = 109,
//     ActPostBuildHook = 110,
//     ActBuildWaiting = 111,
// }
