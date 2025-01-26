use futures::{SinkExt, StreamExt};
use tokio::{
    self,
    net::{TcpListener, TcpStream},
    sync::broadcast::{self, Sender},
};
use tokio_util::codec::{FramedRead, FramedWrite, LinesCodec, LinesCodecError};

use tokio::net::tcp::OwnedWriteHalf;
use tokio::sync::broadcast::error::RecvError;

////////////////////////////////////////////

const HELP_MSG: &str = include_str!("help.txt");

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let server = TcpListener::bind("127.0.0.1:42069").await?;
    let (tx, _) = broadcast::channel::<String>(32);

    loop {
        let (tcp, _) = server.accept().await?;
        tokio::spawn(handle_user(tcp, tx.clone()));
    }
}

/////////////////////////////////////////////////////

async fn handle_user(mut tcp: TcpStream, tx: Sender<String>) -> anyhow::Result<()> {
    let (reader, writer) = tcp.into_split();
    let mut stream = FramedRead::new(reader, LinesCodec::new());
    let mut sink = FramedWrite::new(writer, LinesCodec::new());
    let mut rx = tx.subscribe();

    loop {
        tokio::select! {
            user_msg = stream.next() => {
                if process_user_msg(user_msg, &mut sink, &tx).await? {
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
    user_msg: Option<Result<String, LinesCodecError>>,
    sink: &mut FramedWrite<OwnedWriteHalf, LinesCodec>,
    tx: &Sender<String>,
) -> anyhow::Result<bool> {
    match user_msg {
        Some(Ok(msg)) => {
            if msg.starts_with("/help") {
                sink.send(HELP_MSG).await?;
            } else if msg.starts_with("/quit") {
                return Ok(true);
            } else {
                tx.send(msg)?;
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
