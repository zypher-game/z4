use std::collections::HashMap;
use tdn::{
    prelude::{
        start_with_config_and_key, NetworkType, PeerId, ReceiveMessage, SendMessage, SendType,
    },
    types::rpc::rpc_response,
};
use tokio::{select, sync::mpsc::Sender};

use crate::{
    config::Config,
    p2p::handle_p2p,
    pool::{listen as pool_listen, pool_channel},
    room::{ConnectType, Room},
    rpc::handle_rpc,
    scan::{chain_channel, listen as scan_listen},
    types::*,
    HandleResult, Handler, Param, PublicKey, Task,
};

struct HandlerRoom<H: Handler> {
    handler: H,
    room: Room,
}

/// Engine
pub struct Engine<H: Handler> {
    /// config of engine and network
    config: Config,
    /// tasks need run in every room when created
    tasks: Vec<Box<dyn Task<H = H>>>,
    /// rooms which is running
    rooms: HashMap<RoomId, HandlerRoom<H>>,
    /// rooms which is waiting create
    pending: HashMap<RoomId, Vec<(PeerId, PublicKey)>>,
    /// connected peers
    onlines: HashMap<PeerId, Vec<RoomId>>,
}

impl<H: Handler> Engine<H> {
    /// init a engine with config
    pub fn init(config: Config) -> Self {
        Self {
            config,
            tasks: vec![],
            rooms: HashMap::new(),
            pending: HashMap::new(),
            onlines: HashMap::new(),
        }
    }

    /// add task to all running room
    pub fn add_task(mut self, task: impl Task<H = H> + 'static) -> Self {
        self.tasks.push(Box::new(task));
        self
    }

    /// create a pending room when scan from chain
    pub fn add_pending(&mut self, id: RoomId, pids: Vec<PeerId>, pks: Vec<PublicKey>) {
        if !self.pending.contains_key(&id) {
            let mut peers = vec![];
            for (pid, pk) in pids.iter().zip(pks) {
                peers.push((*pid, pk));
            }
            self.pending.insert(id, peers);
        }
    }

    /// create a pending room when scan from chain
    pub fn del_pending(&mut self, id: RoomId) {
        self.pending.remove(&id);
    }

    /// create a room when scan from chain
    pub async fn start_room(&mut self, id: RoomId) {
        if let Some(peers) = self.pending.remove(&id) {
            let handler = H::create(&peers).await;
            let ids: Vec<PeerId> = peers.iter().map(|(id, _pk)| *id).collect();
            self.rooms.insert(
                id,
                HandlerRoom {
                    handler,
                    room: Room::new(id, &ids),
                },
            );
        }
    }

    pub fn get_room(&self, id: &RoomId) -> Option<&Room> {
        self.rooms.get(id).map(|h| &h.room)
    }

    pub fn get_mut_handler(&mut self, id: &RoomId) -> Option<&mut H> {
        self.rooms.get_mut(id).map(|h| &mut h.handler)
    }

    pub fn has_peer(&self, peer: &PeerId) -> bool {
        if let Some(rooms) = self.onlines.get(&peer) {
            !rooms.is_empty()
        } else {
            false
        }
    }

    pub fn has_room(&self, id: &RoomId) -> bool {
        self.rooms.contains_key(id)
    }

    pub fn is_room_peer(&self, id: &RoomId, peer: &PeerId) -> bool {
        if let Some(hr) = self.rooms.get(id) {
            hr.room.contains(peer)
        } else {
            false
        }
    }

    pub fn online(&mut self, id: RoomId, peer: PeerId, ctype: ConnectType) -> bool {
        let is_ok = if let Some(hr) = self.rooms.get_mut(&id) {
            hr.room.online(peer, ctype)
        } else {
            false
        };

        if is_ok {
            self.onlines
                .entry(peer)
                .and_modify(|rooms| {
                    if !rooms.contains(&id) {
                        rooms.push(id);
                    }
                })
                .or_insert(vec![id]);
        }

        is_ok
    }

    pub fn offline(&mut self, peer: PeerId) {
        if let Some(rooms) = self.onlines.remove(&peer) {
            for rid in rooms {
                if let Some(hr) = self.rooms.get_mut(&rid) {
                    hr.room.offline(peer);
                }
            }
        }
    }

    pub async fn run(mut self) -> Result<()> {
        let (tdn_config, key) = self.config.to_tdn();
        let chain_option = self.config.to_chain().await;

        let (peer_addr, send, mut out_recv) = start_with_config_and_key(tdn_config, key).await?;
        println!("SERVER: peer id: {:?}", peer_addr);

        let (chain_send, mut chain_recv) = chain_channel();
        let (pool_send, pool_recv) = pool_channel();
        if let Some((scan_providers, pool_provider, chain_net)) = chain_option {
            tokio::spawn(scan_listen(scan_providers, chain_net, chain_send.clone()));
            tokio::spawn(pool_listen(pool_provider, chain_net, chain_send, pool_recv));
        }

        loop {
            let work = select! {
                w = async {
                    chain_recv.recv().await.map(FutureMessage::Chain)
                } => w,
                w = async {
                    out_recv.recv().await.map(FutureMessage::Network)
                } => w,
            };

            match work {
                Some(FutureMessage::Network(message)) => match message {
                    ReceiveMessage::Group(gid, msg) => {
                        if !self.has_room(&gid) {
                            continue;
                        }
                        let _ = handle_p2p(&mut self, &send, &pool_send, gid, msg).await;
                    }
                    ReceiveMessage::Rpc(uid, params, is_ws) => {
                        let _ = handle_rpc(&mut self, &send, &pool_send, uid, params, is_ws).await;
                    }
                    ReceiveMessage::NetworkLost => {
                        println!("No network connections");
                    }
                    ReceiveMessage::Own(..) => {}
                },
                Some(FutureMessage::Chain(message)) => match message {
                    ChainMessage::StartRoom(rid, players, pubkeys) => {
                        println!("Engine: accept new room: {}", rid);
                        // send accept operation to chain
                        let _ = pool_send.send(PoolMessage::AcceptRoom(rid));
                        self.add_pending(rid, players, pubkeys);
                    }
                    ChainMessage::AcceptRoom(rid, sequencer) => {
                        if sequencer == peer_addr {
                            println!("Engine: start new room: {}", rid);
                            // if mine, create room
                            self.start_room(rid).await;
                            let _ = send
                                .send(SendMessage::Network(NetworkType::AddGroup(rid)))
                                .await;
                        } else {
                            // if not mine, delete it.
                            self.del_pending(rid);
                        }
                    }
                    ChainMessage::Reprove => {
                        // TODO logic
                    }
                },
                None => break,
            }
        }

        Ok(())
    }
}

enum FutureMessage {
    Network(ReceiveMessage),
    Chain(ChainMessage),
}

pub async fn handle_result<P: Param>(
    room: &Room,
    result: HandleResult<P>,
    send: &Sender<SendMessage>,
    rpc: Option<(PeerId, u64)>,
) {
    let HandleResult {
        mut all,
        mut one,
        over,
    } = result;

    loop {
        if !one.is_empty() {
            let (peer, method, params) = one.remove(0);
            let p2p_msg = P2pMessage {
                method: &method,
                params: params.to_bytes(),
            };
            let p2p_bytes = bincode::serialize(&p2p_msg).unwrap(); // safe
            let rpc_msg = rpc_response(0, &method, params.to_value(), room.id);
            match room.get(&peer) {
                ConnectType::P2p => send
                    .send(SendMessage::Group(
                        room.id,
                        SendType::Event(0, peer, p2p_bytes),
                    ))
                    .await
                    .expect("TDN channel closed"),
                ConnectType::Rpc(uid) => send
                    .send(SendMessage::Rpc(uid, rpc_msg, true))
                    .await
                    .expect("TDN channel closed"),
                ConnectType::None => {
                    if let Some((p, uid)) = rpc {
                        if p == peer {
                            send.send(SendMessage::Rpc(uid, rpc_msg, false))
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
            let p2p_msg = P2pMessage {
                method: &method,
                params: params.to_bytes(),
            };
            let p2p_bytes = bincode::serialize(&p2p_msg).unwrap(); // safe
            let rpc_msg = rpc_response(0, &method, params.to_value(), room.id);
            for (peer, c) in room.iter() {
                match c {
                    ConnectType::P2p => {
                        send.send(SendMessage::Group(
                            room.id,
                            SendType::Event(0, *peer, p2p_bytes.clone()),
                        ))
                        .await
                        .expect("TDN channel closed");
                    }
                    ConnectType::Rpc(uid) => {
                        send.send(SendMessage::Rpc(*uid, rpc_msg.clone(), true))
                            .await
                            .expect("TDN channel closed");
                    }
                    ConnectType::None => {
                        if let Some((p, uid)) = rpc {
                            if p == *peer {
                                send.send(SendMessage::Rpc(uid, rpc_msg.clone(), false))
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

    if over.is_some() {
        let p2p_msg = P2pMessage {
            method: "over",
            params: vec![],
        };
        let p2p_bytes = bincode::serialize(&p2p_msg).unwrap(); // safe
        let rpc_msg = rpc_response(0, "over", Default::default(), room.id);
        for (peer, c) in room.iter() {
            match c {
                ConnectType::P2p => send
                    .send(SendMessage::Group(
                        room.id,
                        SendType::Event(0, *peer, p2p_bytes.clone()),
                    ))
                    .await
                    .expect("TDN channel closed"),
                ConnectType::Rpc(uid) => {
                    send.send(SendMessage::Rpc(*uid, rpc_msg.clone(), true))
                        .await
                        .expect("TDN channel closed");
                }
                ConnectType::None => {
                    if let Some((p, uid)) = rpc {
                        if p == *peer {
                            send.send(SendMessage::Rpc(uid, rpc_msg.clone(), false))
                                .await
                                .expect("TDN channel closed");
                        }
                    }
                }
            }
        }
    }
}
