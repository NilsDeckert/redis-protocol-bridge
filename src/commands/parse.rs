use redis_protocol::error::{RedisProtocolError, RedisProtocolErrorKind};
use crate::commands::command::Command;
use crate::commands::{*};
use crate::commands::info::Info;

/// Wrapper for supported commands
#[derive(Debug)]
#[allow(dead_code)]
pub enum Request {
    HELLO{ version: Option<String>, clientname: Option<String>, auth: Option<(String, String)> },
    GET{ key: String },
    SET{ key: String, value: String },
    COMMAND(Command),
    INFO(Info),
    PING(String),
    SELECT(usize)
}

/// Parse incoming commands
/// 
/// # Returns
/// Result<[`Request`], [`RedisProtocolError`]>
pub fn parse(mut query: Vec<String>) -> Result<Request, RedisProtocolError> {
    let args = query.split_off(1);
    if let Some(command) = query.get(0) {
        match command.to_uppercase().as_ref() {
            "HELLO" => hello::parse(args),
            "GET" => get::parse(args),
            "SET" => set::parse(args),
            "COMMAND" => command::parse(args),
            "INFO" => info::parse(args),
            "PING" => ping::parse(args),
            "SELECT" => select::parse(args),
            
            _ => Err(RedisProtocolError::new(
                RedisProtocolErrorKind::Parse,
                format!("Unsupported command: {}", command)
            ))
        }
    } else {
        Err(RedisProtocolError::new(
            RedisProtocolErrorKind::Parse,
            "Empty query"
        ))
    }
}