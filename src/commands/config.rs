use crate::commands::parse::Request;
use crate::util::convert::{map_to_array, AsFrame};
use crate::util::errors::{error_too_few_arguments, error_unsupported_command};
use redis_protocol::error::RedisProtocolError;
use redis_protocol::resp3::types::OwnedFrame;
use std::collections::HashMap;

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Config {
    Get(ConfigGet),
}

/// Command `CONFIG GET` can specify multiple configuration parameters.
/// This lists all supported parameters and sets them to true if queried.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ConfigGet {
    pub save: bool,
    pub appendonly: bool,
}

impl ConfigGet {
    pub fn new() -> Self {
        Self {
            save: false,
            appendonly: false,
        }
    }
}

pub fn parse(args: Vec<String>) -> Result<Request, RedisProtocolError> {
    if args.is_empty() {
        return Err(error_too_few_arguments("CONFIG", Some(1)));
    }

    match args.first().unwrap().to_uppercase().as_str() {
        "GET" => parse_config_get(&args[1..]),
        unsupported => Err(error_unsupported_command(unsupported)),
    }
}

fn parse_config_get(args: &[String]) -> Result<Request, RedisProtocolError> {
    if args.is_empty() {
        return Err(error_too_few_arguments("CONFIG GET", Some(1)));
    }

    let mut config = ConfigGet::new();
    for arg in args {
        match arg.to_lowercase().as_ref() {
            "save" => config.save = true,
            "appendonly" => config.appendonly = true,
            unknown => {
                return Err(error_unsupported_command(
                    &*("CONFIG GET ".to_owned() + unknown),
                ))
            }
        }
    }

    Ok(Request::CONFIG(Config::Get(config)))
}

pub fn default_handle(args: Request) -> Result<OwnedFrame, RedisProtocolError> {
    if let Request::CONFIG(subcommand) = args {
        match subcommand {
            Config::Get(get) => default_handle_config_get(get),
        }
    } else {
        panic!("Expected enum variant CONFIG, but got {:?}", args)
    }
}

pub fn default_handle_config_get(args: ConfigGet) -> Result<OwnedFrame, RedisProtocolError> {
    let mut config_map: HashMap<&str, &str> = HashMap::new();
    
    if args.save {
        config_map.insert("save", "");
    }
    
    if args.appendonly {
        config_map.insert("appendonly", "no");
    }
    
    Ok(map_to_array(config_map))
}
