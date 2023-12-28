use tdn::prelude::PeerId;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tdn::prelude::{start, ReceiveMessage, RecvType};

pub use serde_json::Value;

mod config;
mod types;
mod room;

pub use types::*;
pub use config::Config;

use room::Manager;

/// The result when after handling the message or task.
#[derive(Default)]
pub struct HandleResult {
    /// need broadcast the msg in the room
    all: Vec<Value>,
    /// need send to someone msg
    one: Vec<(Account, Value)>,
    /// when game over, need prove the operations & states
    over: bool,
}

impl HandleResult {
    pub fn add_all(&mut self, msg: Value) {
        self.all.push(msg);
    }

    pub fn add_one(&mut self, account: Account, msg: Value) {
        self.one.push((account, msg));
    }
}

pub trait Player {
    fn state(&self) -> Value;
}

pub trait Handler {
    type P: Player;

    fn handle(&mut self, player: Account, params: Vec<Value>, states: Arc<RwLock<HashMap<Account, Self::P>>>) -> Result<HandleResult>;
}


pub trait Task {
    type P: Player;

    fn timer(&self) -> u64;

    fn run(&mut self, states: Arc<RwLock<HashMap<Account, Self::P>>>) -> Result<HandleResult>;
}

pub struct Engine<P: Player, H: Handler<P = P>> {
    tasks: Vec<Box<dyn Task<P = P>>>,
    handler: H
}

impl<P: Player, H: Handler<P = P>> Engine<P, H> {
    pub fn new(handler: H) -> Self {
        Self {
            handler,
            tasks: vec![]
        }
    }

    pub fn add_task(mut self, task: impl Task<P = P> + 'static) -> Self {
        self.tasks.push(Box::new(task));
        self
    }

    pub async fn run(mut self, config: Config) -> Result<()> {
        let mut manager = Manager::from_config(config);


        let (peer_addr, send, mut out_recv) = start().await.unwrap();
        println!("Example: peer id: {:?}", peer_addr);

        while let Some(message) = out_recv.recv().await {
            match message {
                ReceiveMessage::Group(gid, msg) => match msg {
                    RecvType::Connect(peer, _data) => {
                        println!("receive group peer {} join", peer.id.short_show());
                    }
                    RecvType::Leave(peer) => {
                        println!("receive group peer {} leave", peer.id.short_show());
                    }
                    RecvType::Event(peer_id, _data) => {
                        println!("receive group event from {}", peer_id.short_show());
                        // TODO handle params
                    }
                    _ => {}
                },
                ReceiveMessage::Rpc(uid, params, is_ws) => {
                    // TODO handle params
                }
                ReceiveMessage::NetworkLost => {
                    println!("No network connections");
                }
                ReceiveMessage::Own(..) => {}
            }
        }

        Ok(())
    }
}
