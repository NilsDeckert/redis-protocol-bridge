use redis_protocol::error::{RedisProtocolError, RedisProtocolErrorKind};

/// Shorthand to return default error if user supplied more arguments than expected
pub fn error_too_many_arguments(command: &str) -> RedisProtocolError {
    RedisProtocolError::new(
        RedisProtocolErrorKind::Parse,
        format!("{command} does not take additional arguments"),
    )
}

/// Shorthand to return default error if user supplied fewer arguments than expected
pub fn error_too_few_arguments(command: &str, num: Option<u8>) -> RedisProtocolError {
    let msg = if let Some(num) = num {
        format!("{command} needs at least {num} arguments")
    } else {
        format!("{command} needs additional arguments")
    }
    .to_string();

    RedisProtocolError::new(RedisProtocolErrorKind::Parse, msg)
}

/// Shorthand to return default error if user supplied command that is not supported
pub fn error_unsupported_command(command: &str) -> RedisProtocolError {
    RedisProtocolError::new(
        RedisProtocolErrorKind::Parse,
        format!("Unsupported command: {command}"),
    )
}
