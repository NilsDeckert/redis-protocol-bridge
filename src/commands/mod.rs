/// # HELLO
///
/// The `HELLO` command is sent by clients to initialize the connection, set the protocol version,
/// authenticate and set a name for the connection.
///
/// ## Syntax
/// ```
/// HELLO [protover [AUTH username password] [SETNAME clientname]]
/// ```
///
/// ## Redis Documentation
/// [Docs/Commands/HELLO](https://redis.io/docs/latest/commands/hello/)
pub mod hello;

/// GET
pub mod get;

/// Util functions
pub mod parse;

/// SET
pub mod set;

/// COMMAND
pub mod command;

/// INFO
pub mod info;

/// PING
pub mod ping;

/// SELECT
pub mod select;
