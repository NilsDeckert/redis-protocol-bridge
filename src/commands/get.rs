use crate::commands::parse::Request;
use crate::util::convert::AsFrame;
use log::debug;
use redis_protocol::error::{RedisProtocolError, RedisProtocolErrorKind};
use redis_protocol::resp3::types::OwnedFrame;
use std::any::Any;
use std::collections::HashMap;

pub fn parse(args: Vec<String>) -> Result<Request, RedisProtocolError> {
    if args.len() != 1 {
        return Err(RedisProtocolError::new(
            RedisProtocolErrorKind::Parse,
            format!("Received {} arguments, expected 1", args.len()),
        ));
    }

    let key = args
        .first()
        .expect("Failed to fetch first element of array");
    Ok(Request::GET {
        key: String::from(key),
    })
}

/// Handle `GET` requests with a default set of key/value-pairs.
/// See [`handle`]
#[allow(dead_code)]
pub fn default_handle(args: Request) -> Result<OwnedFrame, RedisProtocolError> {
    debug!("Providing default values to handle GET");

    let default_values: HashMap<String, String> =
        HashMap::from([("mykey".into(), "myvalue".into())]);

    handle(&default_values, args)
}

/// Handle redis GET requests.
///
/// # Arguments
///
///  * `values` - [`HashMap`] storing keys and values
///  * `args` - The requested key
///
/// # Returns
///  * [`OwnedFrame`] containing the value for the given key or [`OwnedFrame::Null`] if the key is
/// not inside `values`
pub fn handle<V: AsFrame + Clone>(
    values: &HashMap<String, V>,
    args: Request,
) -> Result<OwnedFrame, RedisProtocolError> {
    if let Request::GET { ref key } = args {
        if let Some(value) = values.get(key) {
            return Ok(value.as_frame());
        } else {
            return Ok(OwnedFrame::Null);
        }
    } else {
        panic!("Expected enum variant GET, but got {:?}", args.type_id())
    }
}
