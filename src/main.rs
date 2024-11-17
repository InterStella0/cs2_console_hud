mod core;
use core::{reader, utils, commands};

use reader::reading_log;
use utils::{get_arg, CommandError};
use commands::process_bind;



fn main(){
    let Ok(mode) = get_arg(1, "reading") else {
        println!("Must provide 'reading' argument!");
        return 
    };
    match mode.as_str(){
        "read" => reading_log(),
        "generate-bind" => process_bind(),
        _ => return
    }.unwrap_or_else(|e| {
        let message = match e {
            CommandError::ExpectedArgument(s, pos) => format!(
                "Expected argument {s} at position {pos}!"
            ),
            CommandError::ProcessError(message) => format!(
                "Something went wrong processing {mode}: {message}"
            ),
            CommandError::ConfigError(message) => message,
            CommandError::ArgumentError(message) => format!("Argument error: {message}"),
        };
        println!("{message}")
    });
    return;
}