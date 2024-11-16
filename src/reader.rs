use std::{fs, cmp};
use regex::Regex;
use crate::{model::Bind, utils::{ get_arg, get_config, ValueResult }};

pub fn reading_log() -> ValueResult<()>{
    let mode = get_arg(2, "mode")?;
    let leading_pattern = format!("set_{}_", &mode);
    let re = Regex::new(format!(r"{}(?P<n>\d+)(?P<i>\w)", leading_pattern).as_str()).unwrap();
    let conf = get_config()?;
    let content = fs::read_to_string(conf.cs2_console_path).unwrap_or("".into());
    let mut mode_value_inc = 0;
    let mut mode_value_dec = 0;
    let leadings = conf.binds.iter().filter_map(|bind_data| {
        match bind_data{
            Bind::Interval(bind) => Some(bind.name.clone()),
            Bind::Toggle(bind) => Some(bind.name.clone()),
            Bind::Say(bind) => Some(bind.name.clone()),
            Bind::Unknown(_) => None
        }
    });
    for line in content.split("\n"){
        let is_con = true;
        for lead in leadings{
            if line.contains(&lead){
                is_con = false;
            }
        }
        if leadings.any(|lead| line.contains(&lead)){
            continue;
        }
        

        let (Some(value), Some(value_type)) = (found.name("n"), found.name("i")) else {
            continue;
        };

        let assign_value = value.as_str().parse().unwrap_or_default();
        match value_type.as_str() {
            "i" => mode_value_inc = assign_value,
            "d" => mode_value_dec = assign_value,
            _ => {},
        }
    }
    let current = if mode_value_inc == 200 && mode_value_dec == 190{ 200 } else {
        cmp::max(mode_value_inc - 10, 0)
    };
    let value = if current == 0 { "OFF".into() } else { format!("{}%", current.to_string())};
    println!("{value}");
    Ok(())
}