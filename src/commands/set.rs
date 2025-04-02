use crate::commands::parse::Request;
use crate::util::convert::AsFrame;
use redis_protocol::error::{RedisProtocolError, RedisProtocolErrorKind};
use redis_protocol::resp3::types::OwnedFrame;
use std::any::Any;
use std::collections::HashMap;

/// Parse command and write to enum for easy handling
///
/// # Syntax
/// ```
/// SET key value [NX | XX] [GET] [EX seconds | PX milliseconds |
///   EXAT unix-time-seconds | PXAT unix-time-milliseconds | KEEPTTL]
/// ```
pub fn parse(args: Vec<String>) -> Result<Request, RedisProtocolError> {
    let mut iter = args.into_iter();
    let key = iter.next();
    let value = iter.next();
    if key.is_none() || value.is_none() {
        Err(RedisProtocolError::new(
            RedisProtocolErrorKind::Parse,
            "Required arguments: Key, Value",
        ))
    } else {
        Ok(Request::SET {
            key: key.unwrap(),
            value: value.unwrap(),
        })
    }
}

/// Default reply to SET: "Ok"
#[allow(dead_code)]
pub fn default_handle(args: Request) -> Result<OwnedFrame, RedisProtocolError> {
    {
        let mut dummy: HashMap<String, String> = HashMap::new();
        handle(&mut dummy, &args)
    }
}

/// Set the given key to the given value
///
/// # TODO:
///
/// Redis allows to specify whether or not to return the previously stored value - if any.
/// Rusts `.insert()` also returns the previous value, so it would be easy to do.
pub fn handle(
    values: &mut HashMap<String, String>,
    args: &Request,
) -> Result<OwnedFrame, RedisProtocolError> {
    if let Request::SET { key, value } = args {
        values.insert(key.to_owned(), value.to_owned());
        return Ok("Ok".as_frame());
    }

    panic!("Expected enum variant SET, but got {:?}", args.type_id())
}
