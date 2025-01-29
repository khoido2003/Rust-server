use futures::{io::sink, SinkExt, StreamExt};
use tokio::{
    self,
    net::{TcpListener, TcpStream},
    sync::broadcast::{self, Receiver, Sender},
};
use tokio_util::codec::{FramedRead, FramedWrite, LinesCodec, LinesCodecError};

use std::{
    collections::{HashMap, HashSet},
    sync::{Arc, Mutex, RwLock},
};
use tokio::net::tcp::OwnedWriteHalf;
use tokio::sync::broadcast::error::RecvError;

use crate::shared::random_name;

////////////////////////////////////////////

mod shared;

use shared::b;
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

    fn remove(&self, name: &str) -> bool {
        self.0.lock().unwrap().remove(name)
    }
}

///////////////////////////////////////////

struct Room {
    tx: Sender<String>,
}

impl Room {
    fn new() -> Self {
        let (tx, _) = broadcast::channel(32);
        Self { tx }
    }
}

#[derive(Clone)]
struct Rooms {
    rooms: Arc<RwLock<HashMap<String, Room>>>,
}

impl Rooms {
    fn new() -> Self {
        Self {
            rooms: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    fn join(&self, room_name: &str) -> Sender<String> {
        let read_guard = self.rooms.read().unwrap();

        if let Some(room) = read_guard.get(room_name) {
            return room.tx.clone();
        }

        drop(read_guard);

        let mut write_guard = self.rooms.write().unwrap();
        let room = write_guard
            .entry(room_name.to_owned())
            .or_insert(Room::new());

        room.tx.clone()
    }

    fn lists(&self) -> Vec<(String, usize)> {
        let mut list: Vec<_> = self
            .rooms
            .read()
            .unwrap()
            .iter()
            .map(|(name, room)| (name.to_owned(), room.tx.receiver_count()))
            .collect();

        list.sort_by(|a, b| match b.1.cmp(&a.1) {
            std::cmp::Ordering::Equal => a.0.cmp(&b.0),
            ordering => ordering,
        });

        list
    }

    fn leave(&self, room_name: &str) {
        let read_guard = self.rooms.read().unwrap();
        let mut delete_room = false;
        if let Some(room) = read_guard.get(room_name) {
            delete_room = room.tx.receiver_count() <= 1;
        }

        drop(read_guard);
        if delete_room {
            let mut write_guard = self.rooms.write().unwrap();
            write_guard.remove(room_name);
        }
    }

    fn change(&self, prev_room: &str, next_room: &str) -> Sender<String> {
        self.leave(prev_room);
        self.join(next_room)
    }
}

const MAIN: &str = "main";

////////////////////////////////////////////

const HELP_MSG: &str = include_str!("help.txt");

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let server = TcpListener::bind("127.0.0.1:42069").await?;
    let rooms = Rooms::new();
    let names = Names::new();

    loop {
        let (tcp, _) = server.accept().await?;
        tokio::spawn(handle_user(tcp, names.clone(), rooms.clone()));
    }
}

/////////////////////////////////////////////////////

async fn handle_user(tcp: TcpStream, names: Names, rooms: Rooms) -> anyhow::Result<()> {
    let (reader, writer) = tcp.into_split();
    let mut stream = FramedRead::new(reader, LinesCodec::new());
    let mut sink = FramedWrite::new(writer, LinesCodec::new());

    let mut name = names.get_unique();
    sink.send(format!("You are {name}")).await?;

    // Add User to room
    let mut room_name = MAIN.to_owned();

    let mut room_tx = rooms.join(&room_name);
    let mut room_rx = room_tx.subscribe();

    let _ = room_tx.send(format!("{name} join {room_name}"));

    loop {
        tokio::select! {
            user_msg = stream.next() => {
                if process_user_msg(&mut name,user_msg, &mut sink,  names.clone(), &mut room_tx, &mut room_rx , &mut room_name, rooms.clone()).await? {
                    break;
                }
            },

            peer_msg = room_rx.recv() => {
               process_peer_msg(peer_msg, &mut sink).await?;
            },
        }
    }

    let _ = room_tx.send(format!("{name} left {room_name}"));
    rooms.leave(&room_name);
    Ok(())
}

/////////////////////////////////////////////////////

async fn process_user_msg(
    name: &mut String,
    user_msg: Option<Result<String, LinesCodecError>>,
    sink: &mut FramedWrite<OwnedWriteHalf, LinesCodec>,
    names: Names,
    room_tx: &mut Sender<String>,
    room_rx: &mut Receiver<String>,
    cur_room_name: &mut String,
    rooms: Rooms,
) -> anyhow::Result<bool> {
    match user_msg {
        Some(Ok(msg)) => {
            if msg.starts_with("/help") {
                b!(sink.send(HELP_MSG).await);
            } else if msg.starts_with("/quit") {
                return Ok(true);
            } else if msg.starts_with("/name") {
                let new_name = msg.split_ascii_whitespace().nth(1).unwrap().to_owned();

                let changed_name = names.insert(new_name.clone());
                if changed_name {
                    room_tx.send(format!("{name} is now {new_name}"))?;
                    names.remove(name);
                    *name = new_name;
                }
            } else if msg.starts_with("/join") {
                let new_room = msg.split_ascii_whitespace().nth(1).unwrap().to_owned();

                if new_room == cur_room_name.to_string() {
                    let _ = sink.send(format!("You are in {cur_room_name}")).await;
                } else {
                    let _ = room_tx.send(format!("{name} left {cur_room_name}"));

                    *room_tx = rooms.change(&cur_room_name, &new_room);
                    *room_rx = room_tx.subscribe();
                    *cur_room_name = new_room;

                    let _ = room_tx.send(format!("{name} joined {cur_room_name}"));
                }
            } else if msg.starts_with("/rooms") {
                let room_list = rooms.lists();
                let room_list = room_list
                    .into_iter()
                    .map(|(name, count)| format!("{name}: {count}"))
                    .collect::<Vec<String>>()
                    .join(", ");

                let _ = sink.send(format!("Rooms - {room_list}")).await;
            } else {
                room_tx.send(format!("{name}: {msg}"))?;
            }
        }
        Some(Err(err)) => {
            return Err(err.into());
        }
        None => return Ok(true),
    }
    Ok(false)
}

//////////////////////////////////////////////////

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
