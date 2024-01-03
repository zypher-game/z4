mod config;
mod room;
mod types;

#[cfg(feature = "request")]
pub mod request;

use room::{ConnectType, Manager};
use tdn::prelude::{start_with_config_and_key, ReceiveMessage, RecvType, SendMessage};
use tokio::sync::mpsc::Sender;

pub use config::Config;
pub use serde_json::{json, Value};
pub use tdn::{
    prelude::{GroupId, PeerId, PeerKey, SendType},
    types::rpc::rpc_response,
};
pub use types::*;

/// The result when after handling the message or task.
#[derive(Default)]
pub struct HandleResult {
    /// need broadcast the msg in the room
    all: Vec<(String, Vec<Value>)>,
    /// need send to someone msg
    one: Vec<(PeerId, String, Vec<Value>)>,
    /// when game over, need prove the operations & states
    over: bool,
}

impl HandleResult {
    pub fn add_all(&mut self, method: &str, params: Vec<Value>) {
        self.all.push((method.to_owned(), params));
    }

    pub fn add_one(&mut self, account: PeerId, method: &str, params: Vec<Value>) {
        self.one.push((account, method.to_owned(), params));
    }
}

pub trait Handler {
    async fn online(&mut self, _player: PeerId) -> Result<HandleResult> {
        Ok(HandleResult::default())
    }

    async fn offline(&mut self, _player: PeerId) -> Result<HandleResult> {
        Ok(HandleResult::default())
    }

    async fn handle(
        &mut self,
        player: PeerId,
        method: &str,
        params: Vec<Value>,
    ) -> Result<HandleResult>;
}

pub trait Task {
    type H: Handler;

    fn timer(&self) -> u64;

    fn run(&mut self, states: &mut Self::H) -> Result<HandleResult>;
}

pub struct Engine<H: Handler> {
    tasks: Vec<Box<dyn Task<H = H>>>,
    handler: H,
}

impl<H: Handler> Engine<H> {
    pub fn new(handler: H) -> Self {
        Self {
            handler,
            tasks: vec![],
        }
    }

    pub fn add_task(mut self, task: impl Task<H = H> + 'static) -> Self {
        self.tasks.push(Box::new(task));
        self
    }

    pub async fn run(mut self, config: Config) -> Result<()> {
        let (tdn_config, key) = config.to_tdn();
        let mut manager = Manager::from_config(config);

        let (peer_addr, send, mut out_recv) =
            start_with_config_and_key(tdn_config, key).await.unwrap();
        println!("Example: peer id: {:?}", peer_addr);

        while let Some(message) = out_recv.recv().await {
            match message {
                ReceiveMessage::Group(gid, msg) => {
                    if !manager.has_room(&gid) {
                        continue;
                    }

                    match msg {
                        RecvType::Connect(peer, _data) => {
                            println!("receive group peer {} join", peer.id.short_show());
                            if manager.online(gid, peer.id, ConnectType::P2p) {
                                // TODO send.send().await;

                                if let Ok(res) = self.handler.online(peer.id).await {
                                    handle_result(gid, &manager, res, &send, None).await;
                                }
                            } else {
                                if !manager.has_peer(&peer.id) {
                                    // TODO close the connections
                                }
                            }
                        }
                        RecvType::Leave(peer) => {
                            println!("receive group peer {} leave", peer.id.short_show());
                            manager.offline(peer.id);
                            if let Ok(res) = self.handler.offline(peer.id).await {
                                handle_result(gid, &manager, res, &send, None).await;
                            }
                        }
                        RecvType::Event(peer_id, _data) => {
                            println!("receive group event from {}", peer_id.short_show());
                            // TODO handle params
                            if manager.is_room_peer(&gid, &peer_id) {
                                // parse method & params
                                let method = "info";
                                let params = vec![];

                                if let Ok(res) = self.handler.handle(peer_id, method, params).await
                                {
                                    let is_over = res.over;
                                    handle_result(gid, &manager, res, &send, None).await;
                                    if is_over {
                                        // TODO
                                    }
                                }
                            }
                        }
                        _ => {}
                    }
                }
                ReceiveMessage::Rpc(uid, mut params, is_ws) => {
                    let gid = params["gid"].as_u64().unwrap_or(0);
                    if !manager.has_room(&gid) {
                        continue;
                    }

                    let method = params["method"].as_str().unwrap_or("").to_owned();
                    let mut params = match params["params"].take() {
                        Value::Array(params) => params,
                        _ => vec![],
                    };
                    let peer_id = if params.is_empty() {
                        continue;
                    } else {
                        PeerId::from_hex(params.remove(0).as_str().unwrap_or("")).unwrap()
                    };

                    if &method == "connect" && is_ws {
                        if manager.online(gid, peer_id, ConnectType::Rpc(uid)) {
                            if let Ok(res) = self.handler.online(peer_id).await {
                                let is_rpc = if is_ws { None } else { Some((peer_id, uid)) };
                                handle_result(gid, &manager, res, &send, is_rpc).await;
                            }
                        } else {
                            if !manager.has_peer(&peer_id) {
                                // TODO close the connections
                            }
                        }
                    }

                    if manager.is_room_peer(&gid, &peer_id) {
                        if let Ok(res) = self.handler.handle(peer_id, &method, params).await {
                            let is_over = res.over;
                            let is_rpc = if is_ws { None } else { Some((peer_id, uid)) };
                            handle_result(gid, &manager, res, &send, is_rpc).await;
                            if is_over {
                                // TODO
                            }
                        }
                    }
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

async fn handle_result(
    gid: GroupId,
    manager: &Manager,
    result: HandleResult,
    send: &Sender<SendMessage>,
    rpc: Option<(PeerId, u64)>,
) {
    let HandleResult {
        mut all,
        mut one,
        over,
    } = result;
    let room = manager.get_room(&gid).unwrap();

    loop {
        if !one.is_empty() {
            let (peer, method, params) = one.remove(0);
            let msg = rpc_response(0, &method, json!(params), gid);
            let msg_bytes = vec![]; // TODO
            match room.get(&peer) {
                ConnectType::P2p => send
                    .send(SendMessage::Group(gid, SendType::Event(0, peer, msg_bytes)))
                    .await
                    .expect("TDN channel closed"),
                ConnectType::Rpc(uid) => send
                    .send(SendMessage::Rpc(uid, msg, true))
                    .await
                    .expect("TDN channel closed"),
                ConnectType::None => {
                    if let Some((p, uid)) = rpc {
                        if p == peer {
                            send.send(SendMessage::Rpc(uid, msg, false))
                                .await
                                .expect("TDN channel closed");
                        }
                    }
                }
            }
        } else {
            break;
        }
    }

    loop {
        if !all.is_empty() {
            let (method, params) = all.remove(0);
            let msg = rpc_response(0, &method, json!(params), gid);
            let msg_bytes = vec![]; // TODO
            for (peer, c) in room.iter() {
                match c {
                    ConnectType::P2p => send
                        .send(SendMessage::Group(
                            gid,
                            SendType::Event(0, *peer, msg_bytes.clone()),
                        ))
                        .await
                        .expect("TDN channel closed"),
                    ConnectType::Rpc(uid) => {
                        send.send(SendMessage::Rpc(*uid, msg.clone(), true))
                            .await
                            .expect("TDN channel closed");
                    }
                    ConnectType::None => {
                        if let Some((p, uid)) = rpc {
                            if p == *peer {
                                send.send(SendMessage::Rpc(uid, msg.clone(), false))
                                    .await
                                    .expect("TDN channel closed");
                            }
                        }
                    }
                }
            }
            // TODO
        } else {
            break;
        }
    }
}
