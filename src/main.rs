mod commands;
mod util;
use crate::commands::*;

use log::{debug, error, info, warn};

use std::net::SocketAddr;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

use crate::commands::config;
use crate::commands::parse::Request;
use redis_protocol::resp3::types::*;
use redis_protocol::resp3::{decode, encode};
use redis_protocol_bridge::parse_owned_frame;
use std::collections::HashMap;
use std::env;
/*##########################################################*/
/*  Everything below is part of the minimal example binary  */
/*##########################################################*/

/// Dispatch command handlers.
///
/// This method is not part of the library and would be implemented by the server itself.
/// For redis documentation on commands see [Commands](https://redis.io/docs/latest/commands/)
fn handle_command(query: Vec<String>, map: &mut HashMap<String, String>) -> Vec<u8> {
    let reply: OwnedFrame = match parse::parse(query) {
        Ok(request) => {
            debug!("{:?}", request);

            let r = match request {
                Request::HELLO { .. } => hello::default_handle(&request),
                Request::GET { .. } => get::handle(map, &request),
                Request::SET { .. } => set::handle(map, &request),
                Request::COMMAND { .. } => command::default_handle(&request),
                Request::INFO { .. } => info::default_handle(&request),
                Request::PING { .. } => ping::default_handle(&request),
                Request::SELECT { .. } => select::default_handle(&request),
                Request::QUIT { .. } => quit::default_handle(&request),
                Request::CLUSTER { .. } => cluster::default_handle(&request),
                Request::CONFIG { .. } => config::default_handle(&request),
            };

            r.unwrap_or_else(|err| OwnedFrame::SimpleError {
                data: err.details().to_string(),
                attributes: None,
            })
        }

        Err(err) => OwnedFrame::SimpleError {
            data: err.details().to_string(),
            attributes: None,
        },
    };

    debug!("Reply: {:#?}", reply);

    let mut buf: Vec<u8> = vec![0u8; reply.encode_len(false)];
    encode::complete::encode(&mut buf, &reply, false).expect("Failed to encode");
    buf
}

async fn handle_client(mut stream: TcpStream, addr: SocketAddr) {
    info!("Incoming connection from: {}", addr);
    let mut map: HashMap<String, String> = HashMap::new();
    loop {
        stream.readable().await.unwrap();
        let mut buf = [0; 512];

        match stream.read(&mut buf).await {
            Ok(0) => {
                warn!("Client closed channel");
                break;
            }
            Ok(_) => {}
            Err(e) => {
                error!("Error reading from socket: {}", e);
                break;
            }
        };

        let res_op = decode::complete::decode(&buf);
        match res_op {
            Ok(Some((frame, _size))) => {
                let query = parse_owned_frame(frame);
                info!("{:?}", query);
                let reply = handle_command(query, &mut map);
                stream
                    .write_all(&reply)
                    .await
                    .expect("Failed to send reply");
                stream.flush().await.unwrap()
            }
            Ok(None) => warn!("Received empty command"),
            Err(e) => error!("Error: {}", e),
        }
    }
}

fn setup_logging() {
    // Set default RUST_LOG level if not already set
    if env::var("RUST_LOG").is_err() {
        let log_level = if cfg!(debug_assertions) {
            "debug" // dev profile
        } else {
            "info" // release profile
        };
        env::set_var("RUST_LOG", log_level);
    }

    // Initialize env_logger
    env_logger::init();
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let listener = TcpListener::bind("0.0.0.0:6379").await?;

    setup_logging();

    loop {
        let (tcp_stream, socket_addr) = listener.accept().await?;
        tokio::spawn(async move {
            handle_client(tcp_stream, socket_addr).await;
        });
    }
}
