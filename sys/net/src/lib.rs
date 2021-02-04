//! Demon Network Module - sys/net
//!
//! To support the creation of new clients, this crate provides a
//! generic UDP transport bound to a single socket. It is designed for
//! communication between Peers in a user-readable way.
//!
//! On the surface, what this crate provides is a line-based UDP
//! protocol with some default functionality. It does not provide a
//! parser for complex arguments - that is done upon implementation of
//! this crate.
use serde::Deserialize;
use tokio::net::UdpSocket;
use tokio::sync::{mpsc, Mutex};
use tokio::io;
use tokio_util::udp::UdpFramed;
use tokio_util::codec::BytesCodec;
use bytes::Bytes;
use futures::{Sink, SinkExt, Stream, StreamExt};
use std::net::SocketAddr;
use std::sync::Arc;
use std::collections::HashMap;
use std::error::Error;

mod codec;

type Tx = mpsc::UnboundedSender<String>;

struct Shared {
    peers: HashMap<SocketAddr, Tx>,
}

pub struct Peer {
		server_config: Option<Server>,		
		client_config: Client,
		db: Option<Database>,
}

impl Peer {
		pub fn new(client_config: &Client, server_config: &Server) -> Self {
				Self {
						server_config: None,
						client_config: Client::default(),
						db: None,
				}
		}
}

pub async fn connect(
    addr: &SocketAddr,
    stdin: impl Stream<Item = Result<Bytes, io::Error>> + Unpin,
    stdout: impl Sink<Bytes, Error = io::Error> + Unpin,
) -> Result<(), Box<dyn Error>> {
    // We'll bind our UDP socket to a local IP/port, but for now we
    // basically let the OS pick both of those.
    let bind_addr = if addr.ip().is_ipv4() {
        "0.0.0.0:0"
    } else {
        "[::]:0"
    };

    let socket = UdpSocket::bind(&bind_addr).await?;
    socket.connect(addr).await?;

    tokio::try_join!(send(stdin, &socket), recv(stdout, &socket))?;

    Ok(())
}

async fn send(
    mut stdin: impl Stream<Item = Result<Bytes, io::Error>> + Unpin,
    writer: &UdpSocket,
) -> Result<(), io::Error> {
    while let Some(item) = stdin.next().await {
        let buf = item?;
        writer.send(&buf[..]).await?;
    }

    Ok(())
}

async fn recv(
    mut stdout: impl Sink<Bytes, Error = io::Error> + Unpin,
    reader: &UdpSocket,
) -> Result<(), io::Error> {
    loop {
        let mut buf = vec![0; 1024];
        let n = reader.recv(&mut buf[..]).await?;

        if n > 0 {
            stdout.send(Bytes::from(buf)).await?;
        }
    }
}

struct Server {
    socket: UdpSocket,
    buf: Vec<u8>,
    to_send: Option<(usize, SocketAddr)>,
}

impl Server {
    async fn run(self) -> Result<(), io::Error> {
        let Server {
            socket,
            mut buf,
            mut to_send,
        } = self;

        loop {
            // First we check to see if there's a message we need to echo back.
            // If so then we try to send it back to the original source, waiting
            // until it's writable and we're able to do so.
            if let Some((size, peer)) = to_send {
                let amt = socket.send_to(&buf[..size], &peer).await?;

                println!("Echoed {}/{} bytes to {}", amt, size, peer);
            }

            // If we're here then `to_send` is `None`, so we take a look for the
            // next message we're going to echo back.
            to_send = Some(socket.recv_from(&mut buf).await?);
        }
    }
}

pub struct Client;

#[derive(Deserialize, Debug)]
struct User {
    id: u32,
    name: String,
}

struct Database {
		map: Mutex<HashMap<String, String>>,
}

/// Possible requests
enum Request {
    Get { key: String },
    Set { key: String, value: String },
		Help,
}

impl Request {
    fn parse(input: &str) -> Result<Request, String> {
        let mut parts = input.splitn(3, ' ');
        match parts.next() {
						Some("HELP") => {
								Ok(Request::Get {
										key: "help".to_string(),
								})
						}
            Some("GET") => {
                let key = parts.next().ok_or("GET must be followed by a key")?;
                if parts.next().is_some() {
                    return Err("GET's key must not be followed by anything".into());
                }
                Ok(Request::Get {
                    key: key.to_string(),
                })
            }
            Some("SET") => {
                let key = match parts.next() {
                    Some(key) => key,
                    None => return Err("SET must be followed by a key".into()),
                };
                let value = match parts.next() {
                    Some(value) => value,
                    None => return Err("SET needs a value".into()),
                };
                Ok(Request::Set {
                    key: key.to_string(),
                    value: value.to_string(),
                })
            }
            Some(cmd) => Err(format!("unknown command: {}", cmd)),
            None => Err("empty input".into()),
        }
    }
}

/// Responses to the `Request` commands above
enum Response {
    Value {
        key: String,
        value: String,
    },
    Set {
        key: String,
        value: String,
        previous: Option<String>,
    },
    Error {
        msg: String,
    },
}

impl Response {
    fn serialize(&self) -> String {
        match *self {
            Response::Value { ref key, ref value } => format!("{} = {}", key, value),
            Response::Set {
                ref key,
                ref value,
                ref previous,
            } => format!("set {} = `{}`, previous: {:?}", key, value, previous),
            Response::Error { ref msg } => format!("error: {}", msg),
        }
    }
}

const MIN_MTU: u16 = 1232;
