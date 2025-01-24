use futures::{SinkExt, StreamExt};
use tokio::{
    self,
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpListener,
};
use tokio_util::codec::{FramedRead, FramedWrite, LinesCodec};

////////////////////////////////////////////

const HELP_MSG: &str = include_str!("help.txt");

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let server = TcpListener::bind("127.0.0.1:42069").await?;

    loop {
        let (mut tcp, _) = server.accept().await?;
        let (reader, writer) = tcp.split();

        //let mut buffer = [0u8; 16];

        let mut stream = FramedRead::new(reader, LinesCodec::new());
        let mut sink = FramedWrite::new(writer, LinesCodec::new());

        //loop {
        //    match tcp.read(&mut buffer).await {
        //        Ok(0) => {
        //            break;
        //        }
        //        Ok(n) => {
        //            let mut line = String::from_utf8(buffer[..n].to_vec())?;
        //
        //            // Remove \n char
        //            line.pop();
        //            line.push_str("\nLove you");
        //
        //            if let Err(e) = tcp.write_all(line.as_bytes()).await {
        //                eprint!("Failed to write to socket; err = {:?}", e);
        //                break;
        //            }
        //        }
        //
        //        Err(e) => {
        //            eprintln!("Failed to read from socket; err = {:?}", e);
        //            break;
        //        }
        //    }
        //}

        while let Some(result) = stream.next().await {
            match result {
                Ok(mut msg) => {
                    if msg.starts_with("/help") {
                        sink.send(HELP_MSG).await?;
                    } else if msg.starts_with("/quit") {
                        break;
                    } else {
                        msg.push_str(" Love you");
                        sink.send(msg).await?;
                    }
                }
                Err(e) => {
                    eprintln!("Failed to read message: {:?}", e);
                    break;
                }
            }
        }
    }
}
