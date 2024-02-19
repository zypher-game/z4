use std::collections::HashMap;
use std::sync::Arc;
use tdn::{
    prelude::{
        start_with_config_and_key, NetworkType, PeerId, ReceiveMessage, SendMessage, SendType,
    },
    types::rpc::rpc_response,
};
use tokio::{
    select,
    sync::mpsc::{channel, Sender, UnboundedSender},
    sync::Mutex,
};

use crate::{
    config::Config,
    p2p::handle_p2p,
    pool::{listen as pool_listen, pool_channel},
    room::{ConnectType, Room},
    rpc::handle_rpc,
    scan::{chain_channel, listen as scan_listen},
    task::{handle_tasks, TaskMessage},
    types::*,
    HandleResult, Handler, Param, PublicKey,
};

pub struct HandlerRoom<H: Handler> {
    pub handler: H,
    pub room: Room,
    tasks: Sender<TaskMessage>,
}

/// Engine
pub struct Engine<H: Handler> {
    /// config of engine and network
    config: Config,
    /// rooms which is running
    rooms: HashMap<RoomId, Arc<Mutex<HandlerRoom<H>>>>,
    /// rooms which is waiting create
    pending: HashMap<RoomId, Vec<(PeerId, PublicKey)>>,
    /// connected peers
    onlines: Arc<Mutex<HashMap<PeerId, Vec<RoomId>>>>,
}

impl<H: Handler> Engine<H> {
    /// init a engine with config
    pub fn init(config: Config) -> Self {
        Self {
            config,
            rooms: HashMap::new(),
            pending: HashMap::new(),
            onlines: Arc::new(Mutex::new(HashMap::new())),
        }
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
    pub async fn start_room(
        &mut self,
        id: RoomId,
        send: Sender<SendMessage>,
        chain_send: UnboundedSender<ChainMessage>,
    ) {
        if let Some(peers) = self.pending.remove(&id) {
            let (handler, tasks) = H::create(&peers).await;
            let ids: Vec<PeerId> = peers.iter().map(|(id, _pk)| *id).collect();
            // running tasks
            let (tx, rx) = channel(1);
            let room = Arc::new(Mutex::new(HandlerRoom {
                handler,
                tasks: tx,
                room: Room::new(id, &ids),
            }));

            tokio::spawn(handle_tasks(id, room.clone(), send, chain_send, rx, tasks));
            self.rooms.insert(id, room);
        }
    }

    pub async fn over_room(&mut self, id: RoomId) {
        if let Some(room) = self.rooms.remove(&id) {
            let _ = room.lock().await.tasks.send(TaskMessage::Close).await;
        }
        self.pending.remove(&id);

        // TODO clear onlines
    }

    pub fn has_room(&self, id: &RoomId) -> bool {
        self.rooms.contains_key(id)
    }

    pub fn get_room(&self, id: &RoomId) -> &Arc<Mutex<HandlerRoom<H>>> {
        self.rooms.get(id).unwrap() // safe before check
    }

    pub async fn is_room_peer(&self, id: &RoomId, peer: &PeerId) -> bool {
        if let Some(hr) = self.rooms.get(id) {
            hr.lock().await.room.contains(peer)
        } else {
            false
        }
    }

    pub async fn has_peer(&self, peer: &PeerId) -> bool {
        if let Some(rooms) = self.onlines.lock().await.get(&peer) {
            !rooms.is_empty()
        } else {
            false
        }
    }

    pub async fn online(&self, id: RoomId, peer: PeerId, ctype: ConnectType) -> bool {
        let is_ok = if let Some(hr) = self.rooms.get(&id) {
            hr.lock().await.room.online(peer, ctype)
        } else {
            false
        };

        if is_ok {
            let mut onlines_lock = self.onlines.lock().await;
            onlines_lock
                .entry(peer)
                .and_modify(|rooms| {
                    if !rooms.contains(&id) {
                        rooms.push(id)
                    }
                })
                .or_insert(vec![id]);
        }

        is_ok
    }

    pub async fn offline(&self, peer: PeerId) {
        let mut onlines_lock = self.onlines.lock().await;
        if let Some(rooms) = onlines_lock.remove(&peer) {
            for rid in rooms {
                if let Some(hr) = self.rooms.get(&rid) {
                    hr.lock().await.room.offline(peer);
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
            let send1 = chain_send.clone();
            let send2 = chain_send.clone();
            tokio::spawn(scan_listen(scan_providers, chain_net, send1));
            tokio::spawn(pool_listen(pool_provider, chain_net, send2, pool_recv));
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
                        let _ = handle_p2p(&mut self, &send, &chain_send, gid, msg).await;
                    }
                    ReceiveMessage::Rpc(uid, params, is_ws) => {
                        let _ = handle_rpc(&mut self, &send, &chain_send, uid, params, is_ws).await;
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
                            self.start_room(rid, send.clone(), chain_send.clone()).await;
                            let _ = send
                                .send(SendMessage::Network(NetworkType::AddGroup(rid)))
                                .await;
                        } else {
                            // if not mine, delete it.
                            self.del_pending(rid);
                        }
                    }
                    ChainMessage::OverRoom(gid, data, proof) => {
                        let _ = pool_send.send(PoolMessage::OverRoom(gid, data, proof));
                        self.over_room(gid).await;
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
