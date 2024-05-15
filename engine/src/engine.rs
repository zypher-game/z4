use ethers::prelude::Address;
use std::collections::HashMap;
use std::sync::Arc;
use tdn::{
    prelude::{
        start_with_config_and_key, NetworkType, PeerId, ReceiveMessage, SendMessage, SendType,
    },
    types::{
        primitives::vec_remove_item,
        rpc::{rpc_response, RpcError},
    },
};
use tokio::{
    select,
    sync::mpsc::{channel, Sender, UnboundedReceiver, UnboundedSender},
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
    HandleResult, Handler, Param,
};

pub struct HandlerRoom<H: Handler> {
    pub handler: H,
    pub game: GameId,
    pub room: Room,
    tasks: Sender<TaskMessage>,
}

/// Pending room
pub struct PendingRoom {
    game: GameId,
    viewable: bool,
    /// player params: account, peer, pubkey
    pub players: Vec<(Address, PeerId, [u8; 32])>,
    /// sequencer params: peer, websocket
    pub sequencer: Option<(PeerId, String)>,
}

/// Engine
pub struct Engine<H: Handler> {
    /// config of engine and network
    config: Config,
    /// rooms which is running
    rooms: HashMap<RoomId, Arc<Mutex<HandlerRoom<H>>>>,
    /// rooms which is waiting create, room => (game, players, sequencer)
    pub pending: HashMap<RoomId, PendingRoom>,
    /// supported games and game's pending rooms
    pub games: HashMap<GameId, Vec<RoomId>>,
    /// connected peers
    onlines: Arc<Mutex<HashMap<PeerId, Vec<RoomId>>>>,
}

impl<H: Handler> Engine<H> {
    /// init a engine with config
    pub fn init(config: Config) -> Self {
        let mut games = HashMap::new();
        for game in config.games.iter() {
            if let Ok(addr) = game.parse::<Address>() {
                games.insert(addr, vec![]);
            }
        }
        Self {
            config,
            games,
            rooms: HashMap::new(),
            pending: HashMap::new(),
            onlines: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// create a pending room when scan from chain
    pub fn create_pending(
        &mut self,
        id: RoomId,
        game: GameId,
        viewable: bool,
        account: Address,
        peer: PeerId,
        pubkey: [u8; 32],
    ) {
        if let Some(games) = self.games.get_mut(&game) {
            if !self.pending.contains_key(&id) {
                self.pending.insert(
                    id,
                    PendingRoom {
                        game,
                        viewable,
                        players: vec![(account, peer, pubkey)],
                        sequencer: None,
                    },
                );
                games.push(id);
            }
        }
    }

    /// join new player to the room
    pub fn join_pending(&mut self, id: RoomId, account: Address, peer: PeerId, pubkey: [u8; 32]) {
        if let Some(proom) = self.pending.get_mut(&id) {
            proom.players.push((account, peer, pubkey));
        }
    }

    /// create a pending room when scan from chain
    pub fn del_pending(&mut self, id: RoomId) {
        if let Some(proom) = self.pending.remove(&id) {
            self.games
                .get_mut(&proom.game)
                .map(|v| vec_remove_item(v, &id));
        }
    }

    /// check if contains pending room
    pub fn contains_pending(&self, id: &RoomId) -> bool {
        self.pending.contains_key(id)
    }

    /// create a room when scan from chain
    pub async fn start_room(
        &mut self,
        id: RoomId,
        sequencer: (PeerId, String),
        params: Vec<u8>,
        is_self: bool,
        send: Sender<SendMessage>,
        chain_send: UnboundedSender<ChainMessage>,
    ) {
        if let Some(proom) = self.pending.get_mut(&id) {
            proom.sequencer = Some(sequencer);

            if is_self {
                let (handler, tasks) = H::create(&proom.players, params, id).await;
                let ids: Vec<PeerId> = proom.players.iter().map(|(_aid, pid, _pk)| *pid).collect();
                // running tasks
                let (tx, rx) = channel(1);
                let room = Arc::new(Mutex::new(HandlerRoom {
                    handler,
                    game: proom.game,
                    tasks: tx,
                    room: Room::new(id, proom.viewable, &ids),
                }));

                tokio::spawn(handle_tasks(id, room.clone(), send, chain_send, rx, tasks));
                self.rooms.insert(id, room);
            }
        }
    }

    pub async fn over_room(&mut self, id: RoomId) {
        if let Some(room) = self.rooms.remove(&id) {
            let _ = room.lock().await.tasks.send(TaskMessage::Close).await;
        }

        // TODO clear onlines
    }

    pub fn has_room(&self, id: &RoomId) -> bool {
        self.rooms.contains_key(id)
    }

    pub fn get_room(&self, id: &RoomId) -> &Arc<Mutex<HandlerRoom<H>>> {
        self.rooms.get(id).unwrap() // safe before check
    }

    pub async fn is_room_player(&self, id: &RoomId, peer: &PeerId) -> bool {
        if let Some(hr) = self.rooms.get(id) {
            hr.lock().await.room.is_player(peer)
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

    pub async fn run(self) -> Result<()> {
        let (chain_send, chain_recv) = chain_channel();
        self.run_with_channel(chain_send, chain_recv).await
    }

    pub async fn run_with_channel(
        mut self,
        chain_send: UnboundedSender<ChainMessage>,
        mut chain_recv: UnboundedReceiver<ChainMessage>,
    ) -> Result<()> {
        let (tdn_config, key) = self.config.to_tdn();
        let chain_option = self.config.to_chain().await;

        let (peer_addr, send, mut out_recv) = start_with_config_and_key(tdn_config, key).await?;
        println!("SERVER: peer id: {:?}", peer_addr);
        println!("HTTP  : http://0.0.0.0:{}", self.config.http_port);
        if let Some(p) = self.config.ws_port {
            println!("WS    : ws://0.0.0.0:{}", p);
        }

        let (pool_send, pool_recv) = pool_channel();
        if let Some((scan_providers, pool_provider, chain_net, start_block)) = chain_option {
            let send1 = chain_send.clone();
            let send2 = chain_send.clone();
            tokio::spawn(scan_listen(scan_providers, chain_net, send1, start_block));
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
                        if let Err(err) =
                            handle_rpc(&mut self, &send, &chain_send, uid, params, is_ws).await
                        {
                            let msg = RpcError::Custom(format!("{:?}", err)).json(0);
                            let _ = send.send(SendMessage::Rpc(uid, msg, is_ws)).await;
                        }
                    }
                    ReceiveMessage::NetworkLost => {
                        debug!("No network connections");
                    }
                    ReceiveMessage::Own(..) => {}
                },
                Some(FutureMessage::Chain(message)) => match message {
                    ChainMessage::CreateRoom(rid, game, viewable, player, peer, pk) => {
                        info!("Engine: chain new room created !");
                        self.create_pending(rid, game, viewable, player, peer, pk);
                    }
                    ChainMessage::JoinRoom(rid, player, peer, pk) => {
                        info!("Engine: chain new player joined !");
                        self.join_pending(rid, player, peer, pk);
                    }
                    ChainMessage::StartRoom(rid, game) => {
                        // send accept operation to chain
                        // check room is exist
                        if let Some(proom) = self.pending.get(&rid) {
                            let params = H::accept(&proom.players).await;
                            let _ = pool_send.send(PoolMessage::AcceptRoom(rid, params));
                        } else if self.games.contains_key(&game) {
                            // TODO fetch room from chain.
                        }
                    }
                    ChainMessage::AcceptRoom(rid, sequencer, http, params) => {
                        info!("Engine: start new room: {}", rid);
                        // if mine, create room
                        let is_own = sequencer == peer_addr;
                        self.start_room(
                            rid,
                            (sequencer, http),
                            params,
                            is_own,
                            send.clone(),
                            chain_send.clone(),
                        )
                        .await;

                        if is_own {
                            let _ = send
                                .send(SendMessage::Network(NetworkType::AddGroup(rid)))
                                .await;
                        }
                    }
                    ChainMessage::GameOverRoom(gid, data, proof) => {
                        let _ = pool_send.send(PoolMessage::OverRoom(gid, data, proof));
                        self.over_room(gid).await;
                    }
                    ChainMessage::ChainOverRoom(gid) => {
                        self.del_pending(gid);
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
    id: u64,
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
            let rpc_msg = rpc_response(id, &method, params.to_value(), room.id);
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
            let rpc_msg = rpc_response(id, &method, params.to_value(), room.id);
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
        let rpc_msg = rpc_response(id, "over", Default::default(), room.id);
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
