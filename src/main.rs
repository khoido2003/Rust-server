use futures::{SinkExt, StreamExt};
use tokio::{
    self,
    net::{TcpListener, TcpStream},
    sync::broadcast::{self, Sender},
};
use tokio_util::codec::{FramedRead, FramedWrite, LinesCodec, LinesCodecError};

use std::{
    collections::HashSet,
    sync::{Arc, Mutex},
};
use tokio::net::tcp::OwnedWriteHalf;
use tokio::sync::broadcast::error::RecvError;

use crate::shared::random_name;

////////////////////////////////////////////

mod shared;

/////////////////////////////////////////////

#[derive(Clone)]
struct Names(Arc<Mutex<HashSet<String>>>);

impl Names {
    fn new() -> Self {
        Self(Arc::new(Mutex::new(HashSet::new())))
    }

    fn insert(&self, name: String) -> bool {
        self.0.lock().unwrap().insert(name)
    }

    fn get_unique(&self) -> String {
        let mut name = random_name();
        let mut guard = self.0.lock().unwrap();
        while !guard.insert(name.clone()) {
            name = random_name();
        }
        name
    }
}

////////////////////////////////////////////

const HELP_MSG: &str = include_str!("help.txt");

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let server = TcpListener::bind("127.0.0.1:42069").await?;
    let (tx, _) = broadcast::channel::<String>(32);
    let names = Names::new();
    loop {
        let (tcp, _) = server.accept().await?;
        tokio::spawn(handle_user(tcp, tx.clone(), names.clone()));
    }
}

/////////////////////////////////////////////////////

async fn handle_user(tcp: TcpStream, tx: Sender<String>, names: Names) -> anyhow::Result<()> {
    let (reader, writer) = tcp.into_split();
    let mut stream = FramedRead::new(reader, LinesCodec::new());
    let mut sink = FramedWrite::new(writer, LinesCodec::new());
    let mut rx = tx.subscribe();

    let mut name = names.get_unique();
    sink.send(format!("You are {name}")).await?;

    loop {
        tokio::select! {
            user_msg = stream.next() => {
                if process_user_msg(&mut name,user_msg, &mut sink, &tx, names.clone()).await? {
                    break;
                }
            },

            peer_msg = rx.recv() => {
               process_peer_msg(peer_msg, &mut sink).await?;
            },
        }
    }
    Ok(())
}

async fn process_user_msg(
    name: &mut String,
    user_msg: Option<Result<String, LinesCodecError>>,
    sink: &mut FramedWrite<OwnedWriteHalf, LinesCodec>,
    tx: &Sender<String>,
    names: Names,
) -> anyhow::Result<bool> {
    match user_msg {
        Some(Ok(msg)) => {
            if msg.starts_with("/help") {
                sink.send(HELP_MSG).await?;
            } else if msg.starts_with("/quit") {
                return Ok(true);
            } else if msg.starts_with("/name") {
                let new_name = msg.split_ascii_whitespace().nth(1).unwrap().to_owned();

                let changed_name = names.insert(new_name.clone());
                if changed_name {
                    tx.send(format!("{name} is now {new_name}"))?;

                    *name = new_name;
                }
            } else {
                tx.send(format!("{name}: {msg}"))?;
            }
        }
        Some(Err(err)) => {
            return Err(err.into());
        }
        None => return Ok(true),
    }
    Ok(false)
}

async fn process_peer_msg(
    peer_msg: Result<String, RecvError>,
    sink: &mut FramedWrite<tokio::net::tcp::OwnedWriteHalf, LinesCodec>,
) -> anyhow::Result<()> {
    match peer_msg {
        Ok(msg) => {
            sink.send(msg).await?;
        }
        Err(RecvError::Lagged(_)) => {}

        Err(_) => {
            return Ok(());
        }
    }

    Ok(())
}
