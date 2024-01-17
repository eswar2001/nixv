use crate::nix_tracker::types::CommandState;
use chrono::Utc;
use std::{
    env,
    fs::{File, OpenOptions},
    io::{self, Write},
    path::Path,
};
use yansi::{Paint, Painted};
/// Removes ANSI escape sequences from the given UTF-8 string.
pub fn filter_ansi(mut utf8_string: String) -> Painted<std::string::String> {
    // List of ANSI escape sequences to filter from the string
    let filter_from_string = [
        "\\u001b[39m",
        "\\u001b[30m",
        "\\u001b[31m",
        "\\u001b[32m",
        "\\u001b[33m",
        "\\u001b[34m",
        "\\u001b[35m",
        "\\u001b[36m",
        "\\u001b[37m",
        "\\u001b[49m",
        "\\u001b[40m",
        "\\u001b[41m",
        "\\u001b[42m",
        "\\u001b[43m",
        "\\u001b[44m",
        "\\u001b[45m",
        "\\u001b[46m",
        "\\u001b[47m",
        "\\u001b[90m",
        "\\u001b[91m",
        "\\u001b[92m",
        "\\u001b[93m",
        "\\u001b[94m",
        "\\u001b[95m",
        "\\u001b[96m",
        "\\u001b[97m",
        "\\u001b[100m",
        "\\u001b[101m",
        "\\u001b[102m",
        "\\u001b[103m",
        "\\u001b[104m",
        "\\u001b[105m",
        "\\u001b[106m",
        "\\u001b[107m",
        "\\u001b[38;5;0m",
        "\\u001b[38;5;1m",
        "\\u001b[38;5;2m",
        "\\u001b[38;5;3m",
        "\\u001b[38;5;4m",
        "\\u001b[38;5;5m",
        "\\u001b[38;5;6m",
        "\\u001b[38;5;7m",
        "\\u001b[38;5;8m",
        "\\u001b[38;5;9m",
        "\\u001b[38;5;10m",
        "\\u001b[38;5;11m",
        "\\u001b[38;5;12m",
        "\\u001b[38;5;13m",
        "\\u001b[38;5;14m",
        "\\u001b[38;5;15m",
        "\\u001b[38;5;16m",
        "\\u001b[38;5;17m",
        "\\u001b[38;5;18m",
        "\\u001b[38;5;19m",
        "\\u001b[38;5;20m",
        "\\u001b[38;5;21m",
        "\\u001b[38;5;22m",
        "\\u001b[38;5;23m",
        "\\u001b[38;5;24m",
        "\\u001b[38;5;25m",
        "\\u001b[38;5;26m",
        "\\u001b[38;5;27m",
        "\\u001b[38;5;28m",
        "\\u001b[38;5;29m",
        "\\u001b[38;5;30m",
        "\\u001b[38;5;31m",
        "\\u001b[38;5;32m",
        "\\u001b[38;5;33m",
        "\\u001b[38;5;34m",
        "\\u001b[38;5;35m",
        "\\u001b[38;5;36m",
        "\\u001b[38;5;37m",
        "\\u001b[38;5;38m",
        "\\u001b[38;5;39m",
        "\\u001b[38;5;40m",
        "\\u001b[38;5;41m",
        "\\u001b[38;5;42m",
        "\\u001b[38;5;43m",
        "\\u001b[38;5;44m",
        "\\u001b[38;5;45m",
        "\\u001b[38;5;46m",
        "\\u001b[38;5;47m",
        "\\u001b[38;5;48m",
        "\\u001b[38;5;49m",
        "\\u001b[38;5;50m",
        "\\u001b[38;5;51m",
        "\\u001b[38;5;52m",
        "\\u001b[38;5;53m",
        "\\u001b[38;5;54m",
        "\\u001b[38;5;55m",
        "\\u001b[38;5;56m",
        "\\u001b[38;5;57m",
        "\\u001b[38;5;58m",
        "\\u001b[38;5;59m",
        "\\u001b[38;5;60m",
        "\\u001b[38;5;61m",
        "\\u001b[38;5;62m",
        "\\u001b[38;5;63m",
        "\\u001b[38;5;64m",
        "\\u001b[38;5;65m",
        "\\u001b[38;5;66m",
        "\\u001b[38;5;67m",
        "\\u001b[38;5;68m",
        "\\u001b[38;5;69m",
        "\\u001b[38;5;70m",
        "\\u001b[38;5;71m",
        "\\u001b[38;5;72m",
        "\\u001b[38;5;73m",
        "\\u001b[38;5;74m",
        "\\u001b[38;5;75m",
        "\\u001b[38;5;76m",
        "\\u001b[38;5;77m",
        "\\u001b[38;5;78m",
        "\\u001b[38;5;79m",
        "\\u001b[38;5;80m",
        "\\u001b[38;5;81m",
        "\\u001b[38;5;82m",
        "\\u001b[38;5;83m",
        "\\u001b[38;5;84m",
        "\\u001b[38;5;85m",
        "\\u001b[38;5;86m",
        "\\u001b[38;5;87m",
        "\\u001b[38;5;88m",
        "\\u001b[38;5;89m",
        "\\u001b[38;5;90m",
        "\\u001b[38;5;91m",
        "\\u001b[38;5;92m",
        "\\u001b[38;5;93m",
        "\\u001b[38;5;94m",
        "\\u001b[38;5;95m",
        "\\u001b[38;5;96m",
        "\\u001b[38;5;97m",
        "\\u001b[38;5;98m",
        "\\u001b[38;5;99m",
        "\\u001b[38;5;100m",
        "\\u001b[38;5;101m",
        "\\u001b[38;5;102m",
        "\\u001b[38;5;103m",
        "\\u001b[38;5;104m",
        "\\u001b[38;5;105m",
        "\\u001b[38;5;106m",
        "\\u001b[38;5;107m",
        "\\u001b[38;5;108m",
        "\\u001b[38;5;109m",
        "\\u001b[38;5;110m",
        "\\u001b[38;5;111m",
        "\\u001b[38;5;112m",
        "\\u001b[38;5;113m",
        "\\u001b[38;5;114m",
        "\\u001b[38;5;115m",
        "\\u001b[38;5;116m",
        "\\u001b[38;5;117m",
        "\\u001b[38;5;118m",
        "\\u001b[38;5;119m",
        "\\u001b[38;5;120m",
        "\\u001b[38;5;121m",
        "\\u001b[38;5;122m",
        "\\u001b[38;5;123m",
        "\\u001b[38;5;124m",
        "\\u001b[38;5;125m",
        "\\u001b[38;5;126m",
        "\\u001b[38;5;127m",
        "\\u001b[38;5;128m",
        "\\u001b[38;5;129m",
        "\\u001b[38;5;130m",
        "\\u001b[38;5;131m",
        "\\u001b[38;5;132m",
        "\\u001b[38;5;133m",
        "\\u001b[38;5;134m",
        "\\u001b[38;5;135m",
        "\\u001b[38;5;136m",
        "\\u001b[38;5;137m",
        "\\u001b[38;5;138m",
        "\\u001b[38;5;139m",
        "\\u001b[38;5;140m",
        "\\u001b[38;5;141m",
        "\\u001b[38;5;142m",
        "\\u001b[38;5;143m",
        "\\u001b[38;5;144m",
        "\\u001b[38;5;145m",
        "\\u001b[38;5;146m",
        "\\u001b[38;5;147m",
        "\\u001b[38;5;148m",
        "\\u001b[38;5;149m",
        "\\u001b[38;5;150m",
        "\\u001b[38;5;151m",
        "\\u001b[38;5;152m",
        "\\u001b[38;5;153m",
        "\\u001b[38;5;154m",
        "\\u001b[38;5;155m",
        "\\u001b[38;5;156m",
        "\\u001b[38;5;157m",
        "\\u001b[38;5;158m",
        "\\u001b[38;5;159m",
        "\\u001b[38;5;160m",
        "\\u001b[38;5;161m",
        "\\u001b[38;5;162m",
        "\\u001b[38;5;163m",
        "\\u001b[38;5;164m",
        "\\u001b[38;5;165m",
        "\\u001b[38;5;166m",
        "\\u001b[38;5;167m",
        "\\u001b[38;5;168m",
        "\\u001b[38;5;169m",
        "\\u001b[38;5;170m",
        "\\u001b[38;5;171m",
        "\\u001b[38;5;172m",
        "\\u001b[38;5;173m",
        "\\u001b[38;5;174m",
        "\\u001b[38;5;175m",
        "\\u001b[38;5;176m",
        "\\u001b[38;5;177m",
        "\\u001b[38;5;178m",
        "\\u001b[38;5;179m",
        "\\u001b[38;5;180m",
        "\\u001b[38;5;181m",
        "\\u001b[38;5;182m",
        "\\u001b[38;5;183m",
        "\\u001b[38;5;184m",
        "\\u001b[38;5;185m",
        "\\u001b[38;5;186m",
        "\\u001b[38;5;187m",
        "\\u001b[38;5;188m",
        "\\u001b[38;5;189m",
        "\\u001b[38;5;190m",
        "\\u001b[38;5;191m",
        "\\u001b[38;5;192m",
        "\\u001b[38;5;193m",
        "\\u001b[38;5;194m",
        "\\u001b[38;5;195m",
        "\\u001b[38;5;196m",
        "\\u001b[38;5;197m",
        "\\u001b[38;5;198m",
        "\\u001b[38;5;199m",
        "\\u001b[38;5;200m",
        "\\u001b[38;5;201m",
        "\\u001b[38;5;202m",
        "\\u001b[38;5;203m",
        "\\u001b[38;5;204m",
        "\\u001b[38;5;205m",
        "\\u001b[38;5;206m",
        "\\u001b[38;5;207m",
        "\\u001b[38;5;208m",
        "\\u001b[38;5;209m",
        "\\u001b[38;5;210m",
        "\\u001b[38;5;211m",
        "\\u001b[38;5;212m",
        "\\u001b[38;5;213m",
        "\\u001b[38;5;214m",
        "\\u001b[38;5;215m",
        "\\u001b[38;5;216m",
        "\\u001b[38;5;217m",
        "\\u001b[38;5;218m",
        "\\u001b[38;5;219m",
        "\\u001b[38;5;220m",
        "\\u001b[38;5;221m",
        "\\u001b[38;5;222m",
        "\\u001b[38;5;223m",
        "\\u001b[38;5;224m",
        "\\u001b[38;5;225m",
        "\\u001b[38;5;226m",
        "\\u001b[38;5;227m",
        "\\u001b[38;5;228m",
        "\\u001b[38;5;229m",
        "\\u001b[38;5;230m",
        "\\u001b[38;5;231m",
        "\\u001b[38;5;232m",
        "\\u001b[38;5;233m",
        "\\u001b[38;5;234m",
        "\\u001b[38;5;235m",
        "\\u001b[38;5;236m",
        "\\u001b[38;5;237m",
        "\\u001b[38;5;238m",
        "\\u001b[38;5;239m",
        "\\u001b[38;5;240m",
        "\\u001b[38;5;241m",
        "\\u001b[38;5;242m",
        "\\u001b[38;5;243m",
        "\\u001b[38;5;244m",
        "\\u001b[38;5;245m",
        "\\u001b[38;5;246m",
        "\\u001b[38;5;247m",
        "\\u001b[38;5;248m",
        "\\u001b[38;5;249m",
        "\\u001b[38;5;250m",
        "\\u001b[38;5;251m",
        "\\u001b[38;5;252m",
        "\\u001b[38;5;253m",
        "\\u001b[38;5;254m",
        "\\u001b[38;5;255m",
        "\\u001b[48;5;0m",
        "\\u001b[48;5;1m",
        "\\u001b[48;5;2m",
        "\\u001b[48;5;3m",
        "\\u001b[48;5;4m",
        "\\u001b[48;5;5m",
        "\\u001b[48;5;6m",
        "\\u001b[48;5;7m",
        "\\u001b[48;5;8m",
        "\\u001b[48;5;9m",
        "\\u001b[48;5;10m",
        "\\u001b[48;5;11m",
        "\\u001b[48;5;12m",
        "\\u001b[48;5;13m",
        "\\u001b[48;5;14m",
        "\\u001b[48;5;15m",
        "\\u001b[48;5;16m",
        "\\u001b[48;5;17m",
        "\\u001b[48;5;18m",
        "\\u001b[48;5;19m",
        "\\u001b[48;5;20m",
        "\\u001b[48;5;21m",
        "\\u001b[48;5;22m",
        "\\u001b[48;5;23m",
        "\\u001b[48;5;24m",
        "\\u001b[48;5;25m",
        "\\u001b[48;5;26m",
        "\\u001b[48;5;27m",
        "\\u001b[48;5;28m",
        "\\u001b[48;5;29m",
        "\\u001b[48;5;30m",
        "\\u001b[48;5;31m",
        "\\u001b[48;5;32m",
        "\\u001b[48;5;33m",
        "\\u001b[48;5;34m",
        "\\u001b[48;5;35m",
        "\\u001b[48;5;36m",
        "\\u001b[48;5;37m",
        "\\u001b[48;5;38m",
        "\\u001b[48;5;39m",
        "\\u001b[48;5;40m",
        "\\u001b[48;5;41m",
        "\\u001b[48;5;42m",
        "\\u001b[48;5;43m",
        "\\u001b[48;5;44m",
        "\\u001b[48;5;45m",
        "\\u001b[48;5;46m",
        "\\u001b[48;5;47m",
        "\\u001b[48;5;48m",
        "\\u001b[48;5;49m",
        "\\u001b[48;5;50m",
        "\\u001b[48;5;51m",
        "\\u001b[48;5;52m",
        "\\u001b[48;5;53m",
        "\\u001b[48;5;54m",
        "\\u001b[48;5;55m",
        "\\u001b[48;5;56m",
        "\\u001b[48;5;57m",
        "\\u001b[48;5;58m",
        "\\u001b[48;5;59m",
        "\\u001b[48;5;60m",
        "\\u001b[48;5;61m",
        "\\u001b[48;5;62m",
        "\\u001b[48;5;63m",
        "\\u001b[48;5;64m",
        "\\u001b[48;5;65m",
        "\\u001b[48;5;66m",
        "\\u001b[48;5;67m",
        "\\u001b[48;5;68m",
        "\\u001b[48;5;69m",
        "\\u001b[48;5;70m",
        "\\u001b[48;5;71m",
        "\\u001b[48;5;72m",
        "\\u001b[48;5;73m",
        "\\u001b[48;5;74m",
        "\\u001b[48;5;75m",
        "\\u001b[48;5;76m",
        "\\u001b[48;5;77m",
        "\\u001b[48;5;78m",
        "\\u001b[48;5;79m",
        "\\u001b[48;5;80m",
        "\\u001b[48;5;81m",
        "\\u001b[48;5;82m",
        "\\u001b[48;5;83m",
        "\\u001b[48;5;84m",
        "\\u001b[48;5;85m",
        "\\u001b[48;5;86m",
        "\\u001b[48;5;87m",
        "\\u001b[48;5;88m",
        "\\u001b[48;5;89m",
        "\\u001b[48;5;90m",
        "\\u001b[48;5;91m",
        "\\u001b[48;5;92m",
        "\\u001b[48;5;93m",
        "\\u001b[48;5;94m",
        "\\u001b[48;5;95m",
        "\\u001b[48;5;96m",
        "\\u001b[48;5;97m",
        "\\u001b[48;5;98m",
        "\\u001b[48;5;99m",
        "\\u001b[48;5;100m",
        "\\u001b[48;5;101m",
        "\\u001b[48;5;102m",
        "\\u001b[48;5;103m",
        "\\u001b[48;5;104m",
        "\\u001b[48;5;105m",
        "\\u001b[48;5;106m",
        "\\u001b[48;5;107m",
        "\\u001b[48;5;108m",
        "\\u001b[48;5;109m",
        "\\u001b[48;5;110m",
        "\\u001b[48;5;111m",
        "\\u001b[48;5;112m",
        "\\u001b[48;5;113m",
        "\\u001b[48;5;114m",
        "\\u001b[48;5;115m",
        "\\u001b[48;5;116m",
        "\\u001b[48;5;117m",
        "\\u001b[48;5;118m",
        "\\u001b[48;5;119m",
        "\\u001b[48;5;120m",
        "\\u001b[48;5;121m",
        "\\u001b[48;5;122m",
        "\\u001b[48;5;123m",
        "\\u001b[48;5;124m",
        "\\u001b[48;5;125m",
        "\\u001b[48;5;126m",
        "\\u001b[48;5;127m",
        "\\u001b[48;5;128m",
        "\\u001b[48;5;129m",
        "\\u001b[48;5;130m",
        "\\u001b[48;5;131m",
        "\\u001b[48;5;132m",
        "\\u001b[48;5;133m",
        "\\u001b[48;5;134m",
        "\\u001b[48;5;135m",
        "\\u001b[48;5;136m",
        "\\u001b[48;5;137m",
        "\\u001b[48;5;138m",
        "\\u001b[48;5;139m",
        "\\u001b[48;5;140m",
        "\\u001b[48;5;141m",
        "\\u001b[48;5;142m",
        "\\u001b[48;5;143m",
        "\\u001b[48;5;144m",
        "\\u001b[48;5;145m",
        "\\u001b[48;5;146m",
        "\\u001b[48;5;147m",
        "\\u001b[48;5;148m",
        "\\u001b[48;5;149m",
        "\\u001b[48;5;150m",
        "\\u001b[48;5;151m",
        "\\u001b[48;5;152m",
        "\\u001b[48;5;153m",
        "\\u001b[48;5;154m",
        "\\u001b[48;5;155m",
        "\\u001b[48;5;156m",
        "\\u001b[48;5;157m",
        "\\u001b[48;5;158m",
        "\\u001b[48;5;159m",
        "\\u001b[48;5;160m",
        "\\u001b[48;5;161m",
        "\\u001b[48;5;162m",
        "\\u001b[48;5;163m",
        "\\u001b[48;5;164m",
        "\\u001b[48;5;165m",
        "\\u001b[48;5;166m",
        "\\u001b[48;5;167m",
        "\\u001b[48;5;168m",
        "\\u001b[48;5;169m",
        "\\u001b[48;5;170m",
        "\\u001b[48;5;171m",
        "\\u001b[48;5;172m",
        "\\u001b[48;5;173m",
        "\\u001b[48;5;174m",
        "\\u001b[48;5;175m",
        "\\u001b[48;5;176m",
        "\\u001b[48;5;177m",
        "\\u001b[48;5;178m",
        "\\u001b[48;5;179m",
        "\\u001b[48;5;180m",
        "\\u001b[48;5;181m",
        "\\u001b[48;5;182m",
        "\\u001b[48;5;183m",
        "\\u001b[48;5;184m",
        "\\u001b[48;5;185m",
        "\\u001b[48;5;186m",
        "\\u001b[48;5;187m",
        "\\u001b[48;5;188m",
        "\\u001b[48;5;189m",
        "\\u001b[48;5;190m",
        "\\u001b[48;5;191m",
        "\\u001b[48;5;192m",
        "\\u001b[48;5;193m",
        "\\u001b[48;5;194m",
        "\\u001b[48;5;195m",
        "\\u001b[48;5;196m",
        "\\u001b[48;5;197m",
        "\\u001b[48;5;198m",
        "\\u001b[48;5;199m",
        "\\u001b[48;5;200m",
        "\\u001b[48;5;201m",
        "\\u001b[48;5;202m",
        "\\u001b[48;5;203m",
        "\\u001b[48;5;204m",
        "\\u001b[48;5;205m",
        "\\u001b[48;5;206m",
        "\\u001b[48;5;207m",
        "\\u001b[48;5;208m",
        "\\u001b[48;5;209m",
        "\\u001b[48;5;210m",
        "\\u001b[48;5;211m",
        "\\u001b[48;5;212m",
        "\\u001b[48;5;213m",
        "\\u001b[48;5;214m",
        "\\u001b[48;5;215m",
        "\\u001b[48;5;216m",
        "\\u001b[48;5;217m",
        "\\u001b[48;5;218m",
        "\\u001b[48;5;219m",
        "\\u001b[48;5;220m",
        "\\u001b[48;5;221m",
        "\\u001b[48;5;222m",
        "\\u001b[48;5;223m",
        "\\u001b[48;5;224m",
        "\\u001b[48;5;225m",
        "\\u001b[48;5;226m",
        "\\u001b[48;5;227m",
        "\\u001b[48;5;228m",
        "\\u001b[48;5;229m",
        "\\u001b[48;5;230m",
        "\\u001b[48;5;231m",
        "\\u001b[48;5;232m",
        "\\u001b[48;5;233m",
        "\\u001b[48;5;234m",
        "\\u001b[48;5;235m",
        "\\u001b[48;5;236m",
        "\\u001b[48;5;237m",
        "\\u001b[48;5;238m",
        "\\u001b[48;5;239m",
        "\\u001b[48;5;240m",
        "\\u001b[48;5;241m",
        "\\u001b[48;5;242m",
        "\\u001b[48;5;243m",
        "\\u001b[48;5;244m",
        "\\u001b[48;5;245m",
        "\\u001b[48;5;246m",
        "\\u001b[48;5;247m",
        "\\u001b[48;5;248m",
        "\\u001b[48;5;249m",
        "\\u001b[48;5;250m",
        "\\u001b[48;5;251m",
        "\\u001b[48;5;252m",
        "\\u001b[48;5;253m",
        "\\u001b[48;5;254m",
        "\\u001b[48;5;255m",
        "\\u001b[0m",
        "\\u001b[1m",
        "\\u001b[2m",
        "\\u001b[4m",
        "\\u001b[5m",
        "\\u001b[7m",
        "\\u001b[8m",
        "\\u001b[21m",
        "\\u001b[22m",
        "\\u001b[24m",
        "\\u001b[25m",
        "\\u001b[27m",
        "\\u001b[28m",
        "\\u001b",
        "B[m",
        "\\\"",
        "\"",
        "[;1m",
        "[;0m",
        "[;2m",
    ];
    for i in filter_from_string {
        utf8_string = utf8_string.replace(i, "");
    }
    Painted::new(utf8_string)
}
/// Appends the given message to a log file if the environment variable DUMP_LOGS is set to true.
pub fn append_log_to_file(file_name: String, msg: String) {
    let append = match env::var("DUMP_LOGS") {
        Ok(value) => value.parse().unwrap_or_default(),
        Err(_) => false,
    };
    if append {
        let mut file = OpenOptions::new()
            .write(true)
            .append(true)
            .create(true)
            .open(file_name + ".log")
            .unwrap();

        if let Err(e) = writeln!(file, "{}", msg) {
            eprintln!("Couldn't write to file: {}", e);
        }
    }
}
/// Dump the given CommandState to a JSON file, including the time taken to run the command. If a file with the same name already exists, the new file will have a timestamp appended to its name.
pub fn dump_state_to_file(state: CommandState) {
    println!(
        "time taken to run the command: {:?}",
        state
            .end
            .unwrap()
            .duration_since(state.start)
            .expect("Clock may have gone backwards")
    );
    let t = Utc::now().to_rfc3339().to_string();
    let mut file = match Path::new("command_state.json").exists() {
        true => File::create("command_state".to_owned() + "_" + &t + ".json").unwrap(),
        false => File::create("command_state.json").unwrap(),
    };
    let json_dump = serde_json::to_string_pretty(&CommandState::to_json(state)).unwrap();
    let _ = file.write_all(json_dump.as_bytes());
}
/// This method takes a log record and prints it to the console with optional ANSI color formatting based on the log level.
pub fn log_(record: &log::Record<'_>) {
    let str = record.args().to_string();
    let ansi = match env::var("ANSI").unwrap_or("true".to_owned()).as_str() {
        "false" => false,
        _ => true,
    };
    match record.level() {
        log::Level::Error => {
            if ansi {
                println!("{}", Paint::red(&filter_ansi(str)));
            } else {
                println!("[Error]{}", filter_ansi(str))
            }
        }
        log::Level::Warn => {
            if ansi {
                println!("{}", Paint::magenta(&filter_ansi(str)));
            } else {
                println!("[Warn] {}", filter_ansi(str))
            }
        }
        log::Level::Info => {
            if ansi {
                println!("{}", Paint::white(&filter_ansi(str)));
            } else {
                println!("[Info] {}", filter_ansi(str))
            }
        }
        log::Level::Debug => {
            if ansi {
                println!("{}", Paint::bright_yellow(&filter_ansi(str)));
            } else {
                println!("[Debug]{}", filter_ansi(str))
            }
        }
        log::Level::Trace => {
            if ansi {
                println!("{}", Paint::blue(&filter_ansi(str)));
            } else {
                println!("[Trace]{}", filter_ansi(str))
            }
        }
    }
    io::stdout().flush().unwrap();
}
