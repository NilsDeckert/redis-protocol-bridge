use std::any::Any;
use std::collections::HashMap;
use redis_protocol::error::{RedisProtocolError, RedisProtocolErrorKind};
use redis_protocol::resp3::types::OwnedFrame;
use crate::commands::parse::Request;
use crate::util::convert::AsFrame;

/// A struct containing all possible parameters for the INFO command
/// 
/// For info on sections and their meanings, see
/// [Redis Docs](https://redis.io/docs/latest/commands/info/).
#[derive(Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Info {
    /** Return only the default set of sections. True if no parameters were passed.
    As this is implementation specific, this is treated like any other parameter**/
    pub default: bool, 
    pub server: bool,
    pub clients: bool,
    pub memory: bool,
    pub persistence: bool,
    pub stats: bool,
    pub replication: bool,
    pub cpu: bool,
    pub commandstats: bool,
    pub latencystats: bool,
    pub sentinel: bool,
    pub cluster: bool,
    pub modules: bool,
    pub keyspace: bool,
    pub errorstats: bool
}

impl Default for Info {
    
    /// Return an [`Info`] instance with only `default` set to true.
    fn default() -> Self {
        Self {
            default: true,
            server: false,
            clients: false,
            memory: false,
            persistence: false,
            stats: false,
            replication: false,
            cpu: false,
            commandstats: false,
            latencystats: false,
            sentinel: false,
            cluster: false,
            modules: false,
            keyspace: false,
            errorstats: false
        }
    }
}

impl Info {
    /// Return an [`Info`] instance with every section except modules set to true.
    fn all() -> Info {
        Info {
            default: true,
            server: true,
            clients: true,
            memory: true,
            persistence: true,
            stats: true,
            replication: true,
            cpu: true,
            commandstats: true,
            latencystats: true,
            sentinel: true,
            cluster: true,
            modules: false, /* This is on purpose! */
            keyspace: true,
            errorstats: true
        }
    }
    
    /// Return an [`Info`] instance with all sections set to true.
    fn everything() -> Info {
        Info {
            default: true,
            server: true,
            clients: true,
            memory: true,
            persistence: true,
            stats: true,
            replication: true,
            cpu: true,
            commandstats: true,
            latencystats: true,
            sentinel: true,
            cluster: true,
            modules: true,
            keyspace: true,
            errorstats: true
        }
    }
}

/// Return an [`Info`] instance with all parameters mentioned in `args` set to true.
/// 
/// # Implementation Details:
/// 
///  * If `args` contains `all` or `everything`, this function will immediately return an Info object
/// with the respective sections set to true, without continuing parsing.
///  * If `args` contains `default`, parsing will continue and allow additional flag to be set.
/// 
/// ## `All` vs `Everything`
/// 
///  * Aligning with redis naming, `all` will set all *but* `modules` to true, while `everything`
/// will set all flags, *including* `modules` to true.
pub fn parse(args: Vec<String>) -> Result<Request, RedisProtocolError> {
    let mut default = Info::default();
    if args.is_empty() {
        Ok(Request::INFO(default))
    } else {
        for arg in args {
            match arg.to_lowercase().as_ref() {
                "all"           => return Ok(Request::INFO(Info::all())),
                "everything"    => return Ok(Request::INFO(Info::everything())),
                "default"       => default.default = true,
                "server"        => default.server = true,
                "clients"       => default.clients = true,
                "memory"        => default.memory = true,
                "persistence"   => default.persistence = true,
                "stats"         => default.stats = true,
                "replication"   => default.replication = true,
                "cpu"           => default.cpu = true,
                "commandstats"  => default.commandstats = true,
                "latencystats"  => default.latencystats = true,
                "sentinel"      => default.sentinel = true,
                "cluster"       => default.cluster = true,
                "modules"       => default.modules = true,
                "keyspace"      => default.keyspace = true,
                "errorstats"    => default.errorstats = true,
                unknown    => {
                    return Err(
                        RedisProtocolError::new(
                            RedisProtocolErrorKind::Parse,
                            format!("Asked for unknown section: {}", unknown)
                        )
                    )
                } 
            }
        }
        Ok(Request::INFO(default))
    }
}

/// Ignore passed args and return an empty HashMap
pub fn default_handle(args: Request) -> Result<OwnedFrame, RedisProtocolError> {
    if let Request::INFO(_) = args {
        let empty: HashMap<String, String> = HashMap::new();
        Ok(empty.as_frame())
    } else {
        panic!("Expected enum variant INFO, but got {:?}", args.type_id())
    }
}