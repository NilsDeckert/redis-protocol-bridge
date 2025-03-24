use crate::commands::parse::Request;
use crate::util::convert::map_to_array;
use crate::util::convert::AsFrame;
use crate::util::errors;
use redis_protocol::error::{RedisProtocolError, RedisProtocolErrorKind};
use redis_protocol::resp3::types::OwnedFrame;
use redis_protocol::types::REDIS_CLUSTER_SLOTS;
use std::any::Any;
use std::collections::HashMap;

#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Cluster {
    INFO,
    SHARDS,
    NODES,
    SLOTS,
}

pub fn parse(args: Vec<String>) -> Result<Request, RedisProtocolError> {
    let mut iter = args.iter();
    let subcommand = iter.next();
    if subcommand.is_none() {
        return Err(RedisProtocolError::new(
            RedisProtocolErrorKind::Parse,
            "CLUSTER needs a subcommand",
        ));
    }

    match subcommand.unwrap().to_uppercase().as_str() {
        "SHARDS" => {
            if iter.next().is_some() {
                return Err(errors::error_too_many_arguments("CLUSTER SHARDS"));
            }

            Ok(Request::CLUSTER(Cluster::SHARDS))
        }
        "INFO" => {
            if iter.next().is_some() {
                return Err(errors::error_too_many_arguments("CLUSTER INFO"));
            }
            Ok(Request::CLUSTER(Cluster::INFO))
        }
        "NODES" => {
            if iter.next().is_some() {
                return Err(errors::error_too_many_arguments("CLUSTER NODES"));
            }
            Ok(Request::CLUSTER(Cluster::NODES))
        }
        "SLOTS" => {
            if iter.next().is_some() {
                return Err(errors::error_too_many_arguments("CLUSTER SLOTS"));
            }
            Ok(Request::CLUSTER(Cluster::SLOTS))
        }
        unknown => Err(RedisProtocolError::new(
            RedisProtocolErrorKind::Parse,
            format!("Unsupported command: CLUSTER {unknown}"),
        )),
    }
}

/// Dispatcher for the default handles of the CLUSTER subcommands.
pub fn default_handle(args: Request) -> Result<OwnedFrame, RedisProtocolError> {
    if let Request::CLUSTER(subcommand) = args {
        match subcommand {
            Cluster::SHARDS => default_handle_shards(),
            Cluster::INFO => default_handle_info(),
            Cluster::NODES => default_handle_nodes(),
            Cluster::SLOTS => default_handle_slots(),
        }
    } else {
        panic!("Expected enum variant CLUSTER but got {:?}", args.type_id())
    }
}

///
/// # Returns
///
/// A list of shards with each shard being a list of nodes and
/// a list of HashSlots they serve.
///
/// ```
/// |-----------------------------------------------|
/// | 1)    |-----------------------------------|   |
/// |       |1) "slots"                         |   |
/// |       |2) 1) 0    (hash slot range start) |   |
/// |       |   2) 3000 (hash slot range end)   |   |
/// |       |-----------------------------------|   |
/// |                                               |
/// |       |-----------------------------------|   |
/// |       |3) "nodes"                         |   |
/// |       |4)  1)  1) "id"     (key)          |   |
/// |       |        2) my-id    (value)        |   |
/// |       |-----------------------------------|
/// |-----------------------------------------------|
/// ```
fn default_handle_shards() -> Result<OwnedFrame, RedisProtocolError> {
    let mut slots_and_nodes: Vec<OwnedFrame> = cluster_slots(vec![(0, REDIS_CLUSTER_SLOTS)]);
    slots_and_nodes.push("nodes".as_frame());
    slots_and_nodes.push(default_values_nodes().as_frame());

    let list_of_shards: Vec<OwnedFrame> = vec![{ slots_and_nodes.as_frame() }];

    Ok(list_of_shards.as_frame())
}

/// Returns a list of the cluster slot ranges.
/// # Arguments
/// List of (start, end) tuples of the cluster ranges
///
/// # Returns
/// ```
/// 1) "slots"
/// 2)  1)  (integer) slots.first.0
///     2)  (integer) slots.first.1
///     3)  (integer) slots.second.0
///     4)  (integer) ...
/// ```
pub fn cluster_slots(slots: Vec<(u16, u16)>) -> Vec<OwnedFrame> {
    vec!["slots".as_frame(), slots.as_frame()]
}

fn default_values_nodes() -> Vec<OwnedFrame> {
    let mut all_nodes = vec![];
    let value_map = HashMap::from([
        ("id", "defaultId"),
        ("endpoint", "127.0.0.1"),
        ("ip", "127.0.0.1"),
        ("port", "3769"),
    ]);

    all_nodes.push(map_to_array(value_map));
    all_nodes
}

fn default_handle_nodes() -> Result<OwnedFrame, RedisProtocolError> {
    Ok("myid 127.0.0.1:3769@3769,- myself,master - 0 0 1 connected 0-16384\r\n".as_frame())
}

fn default_handle_slots() -> Result<OwnedFrame, RedisProtocolError> {
    let mut ranges = vec![];
    let this_range = vec![
        0.as_frame(),
        REDIS_CLUSTER_SLOTS.as_frame(),
        vec![
            "127.0.0.1".as_frame(),
            "3769".as_frame(),
            "my-id".as_frame(),
        ]
        .as_frame(),
    ];
    ranges.push(this_range.as_frame());
    Ok(ranges.as_frame())
}

fn default_handle_info() -> Result<OwnedFrame, RedisProtocolError> {
    Ok("Not implemented".as_frame())
}
