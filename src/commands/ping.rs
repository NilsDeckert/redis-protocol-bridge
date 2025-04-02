use crate::commands::parse::Request;
use crate::util::convert::AsFrame;
use redis_protocol::error::RedisProtocolError;
use redis_protocol::resp3::types::OwnedFrame;
use std::any::Any;

/// # Implementation
///
/// Merge all strings in args and return [`Request::PING`] with resulting string.
pub fn parse(args: Vec<String>) -> Result<Request, RedisProtocolError> {
    Ok(Request::PING(args.join(" ")))
}

/// Return passed string.
pub fn default_handle(args: &Request) -> Result<OwnedFrame, RedisProtocolError> {
    if let Request::PING(message) = args {
        if message.is_empty() {
            Ok("PONG".as_frame())
        } else {
            Ok(message.as_frame())
        }
    } else {
        panic!("Expected enum variant PING, but got {:?}", args.type_id())
    }
}
