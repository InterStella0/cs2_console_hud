use crate::utils::{ValueResult, CommandError};

use serde::{de::{DeserializeOwned, Error as _}, Deserialize, Deserializer, Serialize};
use serde_json::Value;

#[derive(Debug)]
pub enum Bind{
    Say(SayBind),
    Toggle(ToggleBind),
    Interval(IntervalBind),
    Unknown(String)
}


#[derive(Deserialize, Debug)]
pub struct Config{
    pub cs2_console_path: String,
    #[serde[deserialize_with="from_json"]]
    pub binds: Vec<Bind>
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SayBind{
    pub name: String,
    pub console: String,
    pub key: String
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ToggleBind{
    pub name: String,
    pub console_activate: String,
    pub console_deactivate: String,
    pub key: String,
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
            .ok_or_else(|| D::Error::custom("Couldn't find \"name\" key in config"))?;
        let bind_type = bind.get("type")
            .and_then(Value::as_str)
            .ok_or_else(|| D::Error::custom(
                format!("Couldn't find \"type\" key in config for {bind_name}")
            ))?;

        let bind_owned = bind.to_owned();
        let parsed: Bind = match bind_type {
            "repeat-say" => Bind::Say(create_bind::<D, SayBind>(bind_owned, bind_name)?),
            "toggle" =>  Bind::Toggle(create_bind::<D, ToggleBind>(bind_owned, bind_name)?),
            "interval" => Bind::Interval(create_bind::<D, IntervalBind>(bind_owned, bind_name)?),
            _ => Bind::Unknown(format!("{bind}")),
            };
        result.push(parsed);
        }
    Ok(result)

}
