use std::fs;

use chrono::Datelike;
use chrono::Local;
use chrono::NaiveDateTime;
use chrono::TimeDelta;
use regex::Regex;

use super::commands::cmd_name;
use super::model::{Bind, ParseValue};
use super::utils::{ get_arg, get_config, ValueResult, CommandError};


pub fn reading_log() -> ValueResult<()>{
    let bind_name = get_arg(2, "bind_name")?;
    let resolve_name = cmd_name(&bind_name);
    let conf = get_config()?;
    let current_bind = conf.binds.iter().find(|bind| {
        let name = match bind{
            Bind::Interval(bind) => bind.name.clone(),
            Bind::Toggle(bind) => bind.name.clone(),
            Bind::Execute(bind) => bind.name.clone(),
            Bind::RepeatSay(bind) => bind.name.clone()
        };
        cmd_name(&name) == resolve_name
    }).ok_or_else(||
        CommandError::ArgumentError(format!("Invalid bind name:{bind_name}"))
    )?;  // Integrity check
    let re = Regex::new(&format!("{resolve_name}_(?<value>[\\w_]+)")).unwrap();
    let content = fs::read_to_string(conf.cs2_console_path).unwrap_or("".into());
    let mut current_value = String::from("NA");
    let current_year = Local::now().year();

    let now: NaiveDateTime = Local::now().naive_local();
    let before_10_seconds = now - TimeDelta::seconds(10);
    let mut so_far_reads = vec![];
    for line in content.split("\n"){
        let vline: Vec<&str> = line.split(" ").collect();
        let Some(datetime) = vline.get(0..2) else {
            continue;
        };
        let dateformat = datetime.join(" ");
        let Ok(dt) = NaiveDateTime::
            parse_from_str(format!("{current_year}/{dateformat}").as_str(), "%Y/%m/%d %H:%M:%S")
        else {
            continue;
        };
        so_far_reads.push((dt, line));

        let Some(group) = re.captures(&line) else {
            continue;
        };
        let Some(value) = group.name("value") else {
            continue;
        };
        current_value = match current_bind{
            Bind::RepeatSay(b) => {
                let lines = so_far_reads.iter().filter(|(cdt, _)| 
                    before_10_seconds < *cdt && *cdt < now 
                );
                let mut new_value = None;
                for (_, reading) in lines{
                    if reading.contains(&b.user){
                        let split_by = format!("{}: ", "");
                        let splitting: Vec<&str> = reading.split(&split_by).collect();
                        new_value = splitting.get(1).cloned()
                    }
                }
                if let Some(new) = new_value{
                    let fp = format!("{}/{}", b.fullpath, b.filename);
                    let f = fs::read_to_string(fp.clone())
                        .map_err(|_| CommandError::ProcessError(format!("Couldn't read path: {}", &fp)))?;
                    let cmd = format!("say {}", new);
                    if f != cmd{
                        fs::write(fp.clone(), cmd)
                        .map_err(|_| CommandError::ProcessError(format!("Couldn't write path: {}", &fp)))?;
                    }
                    String::from(new)
                }else { continue }
            },
            Bind::Interval(b) => b.console_value(value.as_str())?,
            Bind::Toggle(b) => b.console_value(value.as_str())?,
            Bind::Execute(b) => b.console_value(value.as_str())?,
        };
    }
    println!("{current_value}");
    Ok(())
}