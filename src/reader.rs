use std::fs;

use regex::Regex;

use crate::commands::cmd_name;
use crate::model::Bind;
use crate::model::ParseValue;
use crate::utils::{ get_arg, get_config, ValueResult, CommandError};

pub fn reading_log() -> ValueResult<()>{
    let bind_name = get_arg(2, "bind_name")?;
    let resolve_name = cmd_name(&bind_name);
    let conf = get_config()?;
    let current_bind = conf.binds.iter().find(|bind| {
        let name = match bind{
            Bind::Interval(bind) => bind.name.clone(),
            Bind::Toggle(bind) => bind.name.clone(),
            Bind::Say(bind) => bind.name.clone()
        };
        cmd_name(&name) == resolve_name
    }).ok_or_else(||
        CommandError::ArgumentError(format!("Invalid bind name:{bind_name}"))
    )?;  // Integrity check
    let re = Regex::new(&format!("{resolve_name}_(?<value>[\\w_]+)")).unwrap();
    let content = fs::read_to_string(conf.cs2_console_path).unwrap_or("".into());
    let mut current_value = String::from("NA");
    for line in content.split("\n"){
        let Some(group) = re.captures(&line) else {
            continue;
        };
        let Some(value) = group.name("value") else {
            continue;
        };
        current_value = match current_bind{
            Bind::Interval(b) => b.console_value(value.as_str())?,
            Bind::Toggle(b) => b.console_value(value.as_str())?,
            Bind::Say(b) => b.console_value(value.as_str())?,
        };
    }
    println!("{current_value}");
    Ok(())
}