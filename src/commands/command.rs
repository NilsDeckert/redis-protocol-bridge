use crate::commands::command::Command::*;
use crate::commands::parse::Request;
use crate::commands::parse::Request::COMMAND;
use crate::util::convert::AsFrame;
use redis_protocol::error::{RedisProtocolError, RedisProtocolErrorKind};
use redis_protocol::resp3::types::OwnedFrame;
use std::any::Any;
use std::collections::HashMap;

/** Encapsulation for COMMAND subcommands */
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Command {
    /** `COMMAND` */
    CMD,
    /** `COMMAND COUNT` */
    COUNT,
    /** `COMMAND DOCS [command-name [command-name ...]]` */
    DOCS(Vec<String>),
    /** `COMMAND INFO [command-name [command-name ...]]` */
    INFO(Vec<String>),
    /** `COMMAND LIST` */
    LIST,
}

pub fn parse(mut args: Vec<String>) -> Result<Request, RedisProtocolError> {
    let mut iter = args.iter();
    let subcommand = iter.next();
    let ret;
    if let Some(sub) = subcommand {
        ret = match sub.to_uppercase().as_ref() {
            "COUNT" => Ok(COMMAND(COUNT)),
            "DOCS" => Ok(COMMAND(DOCS(args.split_off(1)))),
            "INFO" => Ok(COMMAND(INFO(args.split_off(1)))),
            "LIST" => Ok(COMMAND(LIST)),
            _ => Err(RedisProtocolError::new(
                RedisProtocolErrorKind::Parse,
                format!("Unknown sub command {}", sub),
            )),
        }
    } else {
        ret = Ok(COMMAND(CMD));
    }

    ret
}

pub fn default_handle(args: Request) -> Result<OwnedFrame, RedisProtocolError> {
    let mut command_info: HashMap<String, Vec<(String, String)>> = HashMap::new();
    command_info.insert(
        "GET".into(),
        vec![("summary".into(), "Get the value for a given key".into())],
    );
    command_info.insert(
        "HELLO".into(),
        vec![("summary".into(), "Session init".into())],
    );

    handle(command_info, args)
}

pub fn handle(
    values: HashMap<String, Vec<(String, String)>>,
    args: Request,
) -> Result<OwnedFrame, RedisProtocolError> {
    if let COMMAND(cmd) = args {
        match cmd {
            CMD => handle_cmd(values),
            COUNT => handle_count(values),
            DOCS(args) => handle_docs(values, args),
            INFO(args) => handle_docs(values, args),
            LIST => handle_list(values),
        }
    } else {
        panic!(
            "Expected enum variant COMMAND, but got {:?}",
            args.type_id()
        )
    }
}

/// Return an array with details about every Redis command
fn handle_cmd(
    values: HashMap<String, Vec<(String, String)>>,
) -> Result<OwnedFrame, RedisProtocolError> {
    Ok(values.as_frame())
}

/// Return documentary information about commands
///
/// # Syntax
/// ```
/// COMMAND DOCS [command-name [command-name ...]]
/// ```
fn handle_docs(
    values: HashMap<String, Vec<(String, String)>>,
    args: Vec<String>,
) -> Result<OwnedFrame, RedisProtocolError> {
    /* For COMMAND DOCS, return info for all commands */
    if args.is_empty() {
        return handle_cmd(values);
    }

    let mut intermediate: Vec<OwnedFrame> = Vec::new();
    for command in args {
        intermediate.push(command.as_frame());
        if let Some(v) = values.get(&command) {
            intermediate.push(v.as_frame())
        } // If command is unknown, just don't include it in return
    }

    Ok(OwnedFrame::Array {
        data: intermediate,
        attributes: None,
    })
}

/// Return list of all supported commands
fn handle_list(
    values: HashMap<String, Vec<(String, String)>>,
) -> Result<OwnedFrame, RedisProtocolError> {
    let keys: Vec<String> = values.keys().cloned().collect();
    Ok(keys.as_frame())
}

/// Returns Integer reply of number of total commands in this Redis server
fn handle_count(
    values: HashMap<String, Vec<(String, String)>>,
) -> Result<OwnedFrame, RedisProtocolError> {
    Ok((values.len() as i64).as_frame())
}
