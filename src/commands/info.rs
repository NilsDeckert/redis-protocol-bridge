use crate::commands::parse::Request;
use crate::util::convert::AsFrame;
use redis_protocol::error::{RedisProtocolError, RedisProtocolErrorKind};
use redis_protocol::resp3::types::OwnedFrame;
use std::any::Any;
use std::collections::HashMap;
use std::ops::{BitOr, BitOrAssign};

/// A struct containing all possible parameters for the INFO command
///
/// For info on sections and their meanings, see
/// [Redis Docs](https://redis.io/docs/latest/commands/info/).
#[derive(Debug, Clone)]
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
    pub errorstats: bool,
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
            errorstats: false,
        }
    }
}

impl Info {
    /// Construct an instance with all fields set to `false`.
    fn new() -> Info {
        Info {
            default: false,
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
            errorstats: false,
        }
    }

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
            errorstats: true,
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
            errorstats: true,
        }
    }
}

impl BitOr for Info {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self {
        let mut ret = self.clone();
        ret.default = rhs.default;
        ret.server = rhs.server;
        ret.clients = rhs.clients;
        ret.memory = rhs.memory;
        ret.persistence = rhs.persistence;
        ret.stats = rhs.stats;
        ret.replication |= rhs.replication;
        ret.cpu |= rhs.cpu;
        ret.commandstats |= rhs.commandstats;
        ret.latencystats |= rhs.latencystats;
        ret.sentinel |= rhs.sentinel;
        ret.cluster |= rhs.cluster;
        ret.modules |= rhs.modules;
        ret.keyspace |= rhs.keyspace;
        ret.errorstats |= rhs.errorstats;
        ret
    }
}

impl BitOrAssign for Info {
    fn bitor_assign(&mut self, rhs: Self) {
        self.default = rhs.default;
        self.server = rhs.server;
        self.clients = rhs.clients;
        self.memory = rhs.memory;
        self.persistence = rhs.persistence;
        self.stats = rhs.stats;
        self.replication |= rhs.replication;
        self.cpu |= rhs.cpu;
        self.commandstats |= rhs.commandstats;
        self.latencystats |= rhs.latencystats;
        self.sentinel |= rhs.sentinel;
        self.cluster |= rhs.cluster;
        self.modules |= rhs.modules;
        self.keyspace |= rhs.keyspace;
        self.errorstats |= rhs.errorstats;
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
    let mut ret = Info::new();
    if args.is_empty() {
        Ok(Request::INFO(Info::default()))
    } else {
        for arg in args {
            match arg.to_lowercase().as_ref() {
                "all" => ret |= Info::all(),
                "everything" => return Ok(Request::INFO(Info::everything())),
                "default" => ret |= Info::default(),
                "server" => ret.server = true,
                "clients" => ret.clients = true,
                "memory" => ret.memory = true,
                "persistence" => ret.persistence = true,
                "stats" => ret.stats = true,
                "replication" => ret.replication = true,
                "cpu" => ret.cpu = true,
                "commandstats" => ret.commandstats = true,
                "latencystats" => ret.latencystats = true,
                "sentinel" => ret.sentinel = true,
                "cluster" => ret.cluster = true,
                "modules" => ret.modules = true,
                "keyspace" => ret.keyspace = true,
                "errorstats" => ret.errorstats = true,
                unknown => {
                    return Err(RedisProtocolError::new(
                        RedisProtocolErrorKind::Parse,
                        format!("Asked for unknown section: {}", unknown),
                    ))
                }
            }
        }
        Ok(Request::INFO(ret))
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
