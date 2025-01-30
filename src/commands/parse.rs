use redis_protocol::error::{RedisProtocolError, RedisProtocolErrorKind};
use crate::commands::command::Command;
use crate::commands::{*};
use crate::commands::info::Info;

/// Wrapper for supported commands
#[derive(Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[allow(dead_code)]
pub enum Request {
    HELLO{ version: Option<String>, clientname: Option<String>, auth: Option<(String, String)> },
    GET{ key: String },
    SET{ key: String, value: String },
    COMMAND(Command),
    INFO(Info),
    PING(String),
    SELECT(u64)
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

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(feature = "serde")]
    #[test]
    fn serialize_hello() {
        let hello = Request::HELLO {
            version:    Some(String::from("3")),
            clientname: Some(String::from("my_client")),
            auth:       None
        };
        let serialized = bincode::serialize(&hello).unwrap();
        let deserialized = bincode::deserialize(&serialized).unwrap();

        if let Request::HELLO { version, clientname, auth } = deserialized {
            assert_eq!(version, Some(String::from("3")));
            assert_eq!(clientname, Some(String::from("my_client")));
            assert_eq!(auth, None);
        } else {
            panic!("deserialized wrong variant")
        }
    }
    
    #[cfg(feature = "serde")]
    #[test]
    fn serialize_command_docs() {
        let cmd = Command::DOCS(vec![String::from("HELLO")]);
        let req = Request::COMMAND(cmd);
        
        let serialized   = bincode::serialize(&req).expect("Serialization failed");
        let deserialized = bincode::deserialize(&serialized).expect("Deserialization failed");

        if let Request::COMMAND(cmd) = deserialized {
            if let Command::DOCS(vec) = cmd {
                assert_eq!(vec[0], String::from("HELLO"));
            } else {
                panic!("wrong variant")
            }
        } else {
            panic!("deserialized wrong variant")
        }
    }
    
}