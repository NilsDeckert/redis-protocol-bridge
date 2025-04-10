use crate::commands::parse::Request;
use crate::util::convert::AsFrame;
use redis_protocol::error::{RedisProtocolError, RedisProtocolErrorKind};
use redis_protocol::resp3::types::OwnedFrame;
use std::any::Any;

/// # Implementation
///
/// Ensure args is empty, then return [`Request::QUIT`]
pub fn parse(args: Vec<String>) -> Result<Request, RedisProtocolError> {
    if !args.is_empty() {
        Err(RedisProtocolError::new(
            RedisProtocolErrorKind::Parse,
            "This command does not accept any arguments",
        ))
    } else {
        Ok(Request::QUIT)
    }
}

/// Return "OK"
pub fn default_handle(args: &Request) -> Result<OwnedFrame, RedisProtocolError> {
    if let Request::QUIT = args {
        Ok("OK".as_frame())
    } else {
        panic!("Expected enum variant QUIT, but got {:?}", args.type_id())
    }
}
