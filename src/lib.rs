/*! # Example
 
Open a tcp port and wait for connections:

```rust
#[tokio::main]
async fn main() -> std::io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:6380").await?;
    Builder::new()
        .filter(None, LevelFilter::Debug)
        .init();

    loop {
        let (tcp_stream, socket_addr) = listener.accept().await?;
        tokio::spawn(async move {
            handle_client(tcp_stream, socket_addr).await;
        });
    }
}
 ```

Read the incoming data, pass it to [`parse_owned_frame`] and then handle the command yourself
using `handle_command`: 

```rust
async fn handle_client(mut stream: TcpStream, addr: SocketAddr) {
    info!("Incoming connection from: {}", addr);
    let mut map: HashMap<String, String> = HashMap::new();
    loop {
        stream.readable().await.unwrap();
        let mut buf = [0; 512];

        match stream.read(&mut buf).await {
            /* [...] */
        };

        let res_op = decode::complete::decode(&mut buf);
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
```

Your `handle_command` could look like this: 

```rust
fn handle_command(query: Vec<String>, map: &mut HashMap<String, String>) -> Vec<u8> {
    let reply: OwnedFrame;

    if let Ok(request) = parse::parse(query) {
        let r = match request {
            Request::HELLO   { .. } => hello::default_handle(request),
            Request::GET     { .. } => get::handle(map, request),
            Request::SET     { .. } => set::handle(map, request),
            Request::COMMAND { .. } => command::default_handle(request),
            Request::INFO    { .. } => info::default_handle(request),
            Request::PING    { .. } => ping::default_handle(request),
            Request::SELECT  { .. } => select::default_handle(request)
        };

        reply = r.unwrap_or_else(|err| 
            OwnedFrame::SimpleError {
                data: err.details().to_string(),
                attributes: None 
            }
        )
    } else {
        reply = OwnedFrame::SimpleError {
            data: "Unsupported Command".to_string(),
            attributes: None
        }
    }

    debug!("Reply: {:#?}", reply);

    let mut buf: Vec<u8> = vec![0u8; reply.encode_len(false)];
    encode::complete::encode(
        &mut buf,
        &reply,
        false).expect("Failed to encode");
    buf
}
```

**/

pub mod commands;
pub mod util;

use redis_protocol::resp3::types::Resp3Frame;
use redis_protocol::resp3::types::OwnedFrame;

/// Turn an incoming, potentially nested  [`OwnedFrame`] into a list of strings
pub fn parse_owned_frame(frame: OwnedFrame) -> Vec<String> {
    // Maybe return command type here
    let mut ret = Vec::new();
    match frame {
        // Recursively parse
        OwnedFrame::Array { data, .. } => {
            for o in data {
                ret.append(&mut parse_owned_frame(o));
            }
        }

        OwnedFrame::BlobString { data: _, attributes: _ }  => {
            match frame.to_string() {
                Some(content) => ret.push(content),
                None => ret.push("ERROR".to_string())
            }
        },
        _ => ret.push("No Idea".to_string()),
    }
    ret
}
