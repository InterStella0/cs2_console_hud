use std::cmp::min;
use std::collections::HashMap;
use std::fs;

use super::utils::{get_config, ValueResult, CommandError};
use super::model::Bind;

pub fn cmd_name(name: &str) -> String{
    format!("set_{}", name.trim().replace(" ", "_"))
}
pub fn name_to_cmd(name: &str, suffix: &str) -> String{
    format!("{}_{}", cmd_name(name), suffix)
}

pub fn clean_key(key: &str) -> &str{
    let key_name = key.trim();
    match key_name{
        ";" => "semicolon",
        other => other
    }
}


pub fn process_bind() -> ValueResult<()>{
    let conf = get_config()?;
    let mut config_binds = vec![];
    let mut all_binds = conf.binds;
    all_binds.sort_by_key(|a| match a{
        Bind::Cycle(_) => 1,  // Cycle must be last so it has other binds available.
        _ => -1
    });
    let mut generated_binds = HashMap::new();
    for bind in all_binds{
        match bind{
            Bind::Execute(config) => {
                let cmds = config.commands.join(";");
                let command = format!("bind {} \"{}\"", clean_key(&config.key), cmds);
                generated_binds.insert(config.name, command.clone());
                config_binds.push(command);
            },
            Bind::RepeatSay(config) => {
                let alias_value = name_to_cmd(&config.name, "record");
                let commands: Vec<String> = vec![
                    format!("alias {0} \"echo READ_LAST;bind {1} {0}\"", 
                        alias_value, clean_key(&config.record_key)),
                    format!("bind {} {}", clean_key(&config.record_key), alias_value),
                    format!("bind {} \"exec {}\"", clean_key(&config.send_key), config.filename),
                ];
                generated_binds.insert(config.name, commands.join("\n"));
                config_binds.extend(commands);
            },
            Bind::Toggle(config) => {
                let alias_name_toggle = name_to_cmd(&config.name, "on");
                let alias_name_untoggle = name_to_cmd(&config.name, "off");
                let commands = vec![
                    format!("bind {} {}", clean_key(&config.key), alias_name_toggle),
                    format!("\nalias {} \"{};bind {} {}\"", &alias_name_toggle, &config.console_activate, 
                        clean_key(&config.key), alias_name_untoggle
                    ),
                    format!("\nalias {} \"{};bind {} {}\"", &alias_name_untoggle, 
                        &config.console_deactivate, clean_key(&config.key), alias_name_toggle
                    )
                ];
                generated_binds.insert(config.name, commands.join("\n"));
                config_binds.extend(commands);
            },
            Bind::Interval(config) => {
                // default
                // bind f7 set_music_100
                // bind f8 set_music_100
                // snd_musicvolume 1
                // ongoing
                // alias set_music_90 "snd_musicvolume 0.9; bind f7 set_music_80; bind f8 set_music_100"
                // alias set_music_80 "snd_musicvolume 0.8; bind f7 set_music_70; bind f8 set_music_90"

                let mut commands = vec![];
                let mut value = config.min;
                let mut vec_aliases = vec![];
                let mut lever_exit = false;
                let mut default_alias: Option<String> = None;
                loop{
                    let command_value = format!("{} {}", config.console, value);
                    let value_formatted = format!("{}", value)
                                                    .replace(".", "_");
                    let alias_value = name_to_cmd(&config.name, &value_formatted);
                    
                    if value == config.default{
                        default_alias = Some(alias_value.clone());
                    }

                    vec_aliases.push((alias_value, command_value));
                    let current = value;
                    value += config.step;
                    value = (value * 100.0).round() / 100.0;  // remove floating precision dumbstuff
                    
                    if current < config.default && config.default < value{
                        value = config.default;
                    }
                    if value >= config.max{  // safe guard
                        if lever_exit{
                            break;
                        }
                        value = config.max;
                        lever_exit = true;
                    }
                }
                let max_size = vec_aliases.len().wrapping_sub(1);
                for (i, (alias_value, value)) in vec_aliases.iter().enumerate(){
                    let (next_key, _) = vec_aliases[min(i + 1, max_size)].clone();
                    let (prev_key, _) = vec_aliases[if i == 0 { 0 } else { i - 1 }].clone();
                    commands.push(format!(
                        "alias {} \"{}; bind {} {}; bind {} {}\"", 
                        &alias_value, value, 
                        clean_key(&config.up_key), &next_key,
                        clean_key(&config.down_key), &prev_key
                    ));
                }
                let initial = match default_alias {
                    Some(d) => d,
                    None => vec_aliases.get(0)
                    .ok_or_else(
                        || CommandError::ProcessError("Couldn't resolve default bind.".into())
                    ).and_then(|d  | Ok(d.0.clone()))?
                };
                generated_binds.insert(config.name, initial.clone());
                commands.push(initial);
                config_binds.extend(commands);
            },
            Bind::Cycle(config) => {
                let alias_names: Vec<String> = config.bind_names.iter().enumerate().map(
                    |(i, _)| name_to_cmd(&config.name, &i.to_string())
                ).collect();
                let mut cmds = vec![];
                for (i, name) in config.bind_names.iter().enumerate(){
                    if !generated_binds.contains_key(name){
                        return Err(CommandError::ProcessError(format!("Bind name '{}' does not exist!", name)))
                    }
                    let execute = generated_binds.get(name).expect("Generated bind not found?");
                    let alias_name = &alias_names[i];
                    let next_index = if i + 1 == alias_names.len() { 0 } else { i + 1 };
                    let next_alias = &alias_names[next_index];
                    let cmd = format!("alias {} \"{}; \"bind {} {}\"\"", alias_name, execute, clean_key(&config.key), next_alias);
                    cmds.push(cmd);
                }
                cmds.push(format!("bind {} {}", clean_key(&config.key), alias_names[config.default]));
                config_binds.extend(cmds);
            }
        }
    }

    let writing = config_binds.join("\n");
    let path = "bind_generated.cfg";
    fs::write(path, writing).map_err(
        |e| CommandError::ProcessError(format!("Couldn't write '{path}': {e}"))
    )?;
    println!("File written to {path}");

    Ok(())
}