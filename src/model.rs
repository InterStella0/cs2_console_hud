use crate::utils::{ValueResult, CommandError};

use serde::{de::{DeserializeOwned, Error as _}, Deserialize, Deserializer, Serialize};
use serde_json::Value;

#[derive(Debug)]
pub enum Bind{
    RepeatSay(RepeatSayBind),
    Execute(ExecuteBind),
    Toggle(ToggleBind),
    Interval(IntervalBind)
}

pub trait ParseValue{
    fn console_value(&self, value: &str) -> ValueResult<String>;
}

#[derive(Deserialize, Debug)]
pub struct Config{
    pub cs2_console_path: String,
    #[serde[deserialize_with="from_json"]]
    pub binds: Vec<Bind>
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ExecuteBind{
    pub name: String,
    pub commands: Vec<String>,
    pub key: String
}
impl ParseValue for ExecuteBind{
    fn console_value(&self, value: &str) -> ValueResult<String> {
        Ok(String::from(value))
    }
}
#[derive(Serialize, Deserialize, Debug)]
pub struct RepeatSayBind{
    pub name: String,
    pub user: String,
    pub filename: String,
    pub fullpath: String,
    pub record_key: String,
    pub send_key: String,
}
impl ParseValue for RepeatSayBind{
    fn console_value(&self, value: &str) -> ValueResult<String> {
        Ok(String::from(value))
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ToggleBind{
    pub name: String,
    pub console_activate: String,
    pub console_deactivate: String,
    pub key: String,
}
impl ParseValue for ToggleBind{
    fn console_value(&self, value: &str) -> ValueResult<String> {
        let val = if value.contains("on"){
            "ON"
        }else if value.contains("off"){
            "OFF"
        }else{
            "NA"
        };
        Ok(String::from(val))
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct IntervalBind{
    pub name: String,
    pub step: f64,
    pub up_key: String,
    pub down_key: String, 
    pub min: f64,
    pub max: f64,
    pub console: String,
    pub default: f64,
}
impl ParseValue for IntervalBind{
    fn console_value(&self, value: &str) -> ValueResult<String> {
        let real = value.replace("_", ".");
        let parsed = real.parse::<f64>().map_err(
            |_| CommandError::ProcessError(format!("Couldn't parse {real} to float."))
        )?;
        let display = parsed * 100.;
        Ok(format!("{:.1$}%", display, 1))
    }
}


fn from_json<'de, D>(deserializer: D) -> Result<Vec<Bind>, D::Error>
where
    D: Deserializer<'de>,
{
    let map: Value = Value::deserialize(deserializer)?;
    let binds = map.as_array().ok_or_else(
        || D::Error::custom("unable to construct NaiveDateTime from i64")
    )?;

    let mut result = vec![];
    fn create_bind<'de, U, T>(data: Value, bind_name: &str) -> Result<T, U::Error>
    where 
        U: Deserializer<'de>, 
        T: DeserializeOwned
    {
        serde_json::from_value(data)
        .map_err(|e| 
            U::Error::custom(format!("Error parsing {bind_name}: {e}"))
        )
    }

    for bind in binds{
        let bind_name = bind.get("type")
            .and_then(Value::as_str)
            .ok_or_else(|| D::Error::custom(
                format!("Couldn't find \"name\" key for: {bind:?}")
            ))?;
        let bind_type = bind.get("type")
            .and_then(Value::as_str)
            .ok_or_else(|| D::Error::custom(
                format!("Couldn't find \"type\" key in config for {bind_name}")
            ))?;

        let bind_owned = bind.to_owned();
        let parsed: Bind = match bind_type {
            "execute" => Bind::Execute(create_bind::<D, ExecuteBind>(bind_owned, bind_name)?),
            "repeat_say" => Bind::RepeatSay(create_bind::<D, RepeatSayBind>(bind_owned, bind_name)?),
            "toggle" =>  Bind::Toggle(create_bind::<D, ToggleBind>(bind_owned, bind_name)?),
            "interval" => Bind::Interval(create_bind::<D, IntervalBind>(bind_owned, bind_name)?),
            _ => return Err(D::Error::custom(format!("Unknown bind type: {bind_type}"))),
            };
        result.push(parsed);
        }
    Ok(result)

}
