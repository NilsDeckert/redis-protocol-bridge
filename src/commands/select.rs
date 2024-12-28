use std::any::Any;
use redis_protocol::error::{RedisProtocolError, RedisProtocolErrorKind};
use redis_protocol::resp3::types::OwnedFrame;
use crate::commands::parse::Request;
use crate::util::convert::AsFrame;

pub fn parse(args: Vec<String>) -> Result<Request, RedisProtocolError>{
    if args.len() != 1 {
        return Err(RedisProtocolError::new(
            RedisProtocolErrorKind::Parse,
            format!("Expected 1 argument, got {}", args.len())
        ))
    }
    
    if let Some(arg) = args.get(0) {
        if let Ok(number) = arg.parse::<usize>() {
            return Ok(Request::SELECT(number))
        }
    }
    
    Err(RedisProtocolError::new(
        RedisProtocolErrorKind::Parse,
        "Failed to parse SELECT argument"
    ))
}

/// Return `Ok` if requested database index is 0. Return Error otherwise.
pub fn default_handle(args: Request) -> Result<OwnedFrame, RedisProtocolError>{
    if let Request::SELECT( db ) = args {
        return if db == 0 {
            Ok("Ok".as_frame())
        } else {
            Err(RedisProtocolError::new(
                RedisProtocolErrorKind::Parse,
                format!("Invalid index: {}", db)
            ))
        }
    }

    panic!("Expected enum variant SELECT, but got {:?}", args.type_id())
}