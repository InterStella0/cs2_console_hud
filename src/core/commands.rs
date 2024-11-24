use std::cmp::min;
use std::fs;

use super::utils::{get_config, ValueResult, CommandError};
use super::model::Bind;

pub fn cmd_name(name: &str) -> String{
    format!("set_{}", name.trim().replace(" ", "_"))
}
pub fn name_to_cmd(name: &str, suffix: &str) -> String{
    format!("{}_{}", cmd_name(name), suffix)
}


pub fn process_bind() -> ValueResult<()>{
    let conf = get_config()?;
    let mut config_binds = vec![];
    for bind in conf.binds{
        match bind{
            Bind::Execute(config) => {
                let cmds = config.commands.join(";");
                let command = format!("bind {} \"{}\"", config.key, cmds);
                config_binds.push(command);
            },
            Bind::RepeatSay(config) => {
                let alias_value = name_to_cmd(&config.name, "record");
                let commands = vec![
                    format!("alias {0} \"echo READ_LAST;bind {1} {0}\"", 
                        alias_value, config.record_key),
                    format!("bind {} {}", config.record_key, alias_value),
                    format!("bind {} \"exec {}\"", config.send_key, config.filename),
                ];
                config_binds.extend(commands);
            },
            Bind::Toggle(config) => {
                let alias_name_toggle = name_to_cmd(&config.name, "on");
                let alias_name_untoggle = name_to_cmd(&config.name, "off");
                let mut command = format!("bind {} {}", config.key, alias_name_toggle);
                command += format!(
                    "\nalias {} \"{};bind {} {}\"", &alias_name_toggle, &config.console_activate, 
                    &config.key, alias_name_untoggle
                ).as_str();
                command += format!(
                    "\nalias {} \"{};bind {} {}\"", &alias_name_untoggle, 
                    &config.console_deactivate, &config.key, alias_name_toggle
                ).as_str();
                config_binds.push(command);
            },
            Bind::Interval(config) => {
                // default
                // bind f7 set_music_100
                // bind f8 set_music_100
                // snd_musicvolume 1
                // ongoing
                // alias set_music_90 "snd_musicvolume 0.9; bind f7 set_music_80; bind f8 set_music_100"
                // alias set_music_90 "snd_musicvolume 0.9; bind f7 set_music_80; bind f8 set_music_100"

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
                        &config.up_key, &next_key,
                        &config.down_key, &prev_key
                    ));
                }
                let initial = match default_alias {
                    Some(d) => d,
                    None => vec_aliases.get(0)
                    .ok_or_else(
                        || CommandError::ProcessError("Couldn't resolve default bind.".into())
                    ).and_then(|d  | Ok(d.0.clone()))?
                };
                commands.push(initial);
                config_binds.extend(commands);
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