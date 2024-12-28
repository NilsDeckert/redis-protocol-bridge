use std::any::Any;
use std::collections::HashMap;
use log::{debug, error};
use redis_protocol::error::{RedisProtocolError, RedisProtocolErrorKind};
use redis_protocol::resp3::types::OwnedFrame;

use crate::util;
use util::convert::{*};
use crate::commands::parse::Request;

/// Valid property types returned by HELLO.
pub enum Property {
    String(String),
    Integer(i64),
    Array(Vec<Property>)
}

impl AsFrame for Property {
    fn as_frame(&self) -> OwnedFrame {
        match self {
            Property::String(string) => string.as_frame(),
            Property::Integer(num) => num.as_frame(),
            Property::Array(v) => v.as_frame()
        }
    }
}

/// Handle `HELLO` requests with a default set of properties.
/// See [`handle`]
pub fn default_handle(args: Request) -> Result<OwnedFrame, RedisProtocolError> {
    debug!("Providing default properties to handle HELLO");
    let default_values = HashMap::from([
        ("server".into(),   Property::String("RRedis".into())),
        ("proto".into(),    Property::Integer(3)),
        ("modules".into(),  Property::Array(Vec::new()))
    ]);

    handle(&default_values, args)
}

/// Parse arguments for HELLO command
///
/// # Returns
/// [`Request`]
pub fn parse(args: Vec<String>) -> Result<Request, RedisProtocolError> {
    let mut iter = args.into_iter();

    /* Optional arguments */
    let version:        Option<String>              = iter.next();
    let mut auth:       Option<(String, String)>    =  None;
    let mut clientname: Option<String>              =  None;

    /* Parse arguments */
    while let Some(arg) = iter.next() {
        match arg.to_uppercase().as_str() {
            "AUTH" => {
                if let Some(username) = iter.next() {
                    if let Some(password) = iter.next() {
                        debug!("Received auth: ({}, {})", username, password);
                        auth = Some((username, password));
                    }
                }
            },
            "SETNAME" => {
                if let Some(name) = iter.next() {
                    debug!("Client set name: {}", name);
                    clientname = Some(name);
                }
            },
            _ => {
                error!("Unknown argument passed to HELLO: {}", arg);
                return Err(RedisProtocolError::new(
                    RedisProtocolErrorKind::Parse,
                    format!("Unknown argument: {}", arg)
                ));
            }
        }
    }

    Ok(Request::HELLO{version, clientname, auth})
}

/// Handle redis HELLO requests.
/// 
/// # Arguments
/// 
///  * `values` - A HashMap, mapping property name to property value. E.g. `server`, `proto`
///  * `args`   - The list of arguments supplied to `HELLO`. See [Redis docs](https://redis.io/docs/latest/commands/hello/)
/// 
/// # Returns
/// 
/// The map of properties in `values` as a [`OwnedFrame::Map`] or a [`RedisProtocolError`].
/// 
/// # TODO
/// 
/// Currently, the return type of the function is a Result<>, but errors are returned as
/// `Ok(RedisProtocolError)`. We might want to return actual errors and map them to [`RedisProtocolError`]
/// elsewhere.
/// 
pub fn handle(values: &HashMap<String, Property>, args: Request) -> Result<OwnedFrame, RedisProtocolError> {
    debug!("Handling HELLO with provided properties");
    let default_version = String::from("3");
    match args {
        Request::HELLO {version, ..} => {
            // TODO: Use other parsed args
            let v: String = version.unwrap_or(default_version);
            if "3".eq(&v) {
                Ok(values.as_frame())
            } else {
                error!("Client asked for unsupported protocol version {}", v);
                Err(RedisProtocolError::new(
                    RedisProtocolErrorKind::Parse,
                    format!("Unknown protocol version: {}", v)
                ))
            }
        },
        _ => panic!("Expected enum variant HELLO, but got {:?}", args.type_id())
    }
}
