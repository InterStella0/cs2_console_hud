use std::{env, fs};
use super::model::Config;

pub enum CommandError{
    ProcessError(String),
    ExpectedArgument(String, usize),
    ArgumentError(String),
    ConfigError(String)
}

pub fn get_arg(pos: usize, argument_name: &str) -> Result<String, CommandError>{
    match env::args().nth(pos){ 
        Some(s) => Ok(s),
        None => Err(CommandError::ExpectedArgument(argument_name.to_string(), pos))
    }
}
pub fn get_config() -> Result<Config, CommandError>{
    let file = fs::read_to_string("config.json").map_err(
        |e| CommandError::ConfigError(format!("Couldn't process config: {e}").to_string())
    )?;
    let conf: Config = serde_json::from_str(&file).map_err(
        |e| CommandError::ProcessError(format!("Couldn't open config {e}"))
    )?;
    Ok(conf)
}
pub type ValueResult<T> = Result<T, CommandError>;
