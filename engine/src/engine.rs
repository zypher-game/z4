use ethers::prelude::Address;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::Arc;
use tdn::{
    prelude::{
        start_with_config_and_key, NetworkType, PeerId, ReceiveMessage, SendMessage, SendType,
    },
    types::{primitives::vec_remove_item, rpc::RpcError},
};
use tokio::{
    select,
    sync::mpsc::{unbounded_channel, Sender, UnboundedReceiver, UnboundedSender},
    sync::Mutex,
};
use z4_types::{
    handle_tasks, GameId, HandleResult, Handler, MethodValues, Param, Player, Result, RoomId,
    TaskMessage,
};

use crate::{
    config::Config,
    p2p::handle_p2p,
    pool::{listen as pool_listen, pool_channel},
    room::{ConnectType, Room},
    rpc::handle_rpc,
    scan::{chain_channel, listen as scan_listen},
    ChainMessage, PoolMessage,
};

/// Store the room info
pub struct HandlerRoom<H: Handler> {
    /// Game logic handler
    pub handler: Arc<Mutex<H>>,
    /// Game id/address
    pub game: GameId,
    /// Room info
    pub room: Room,
}

/// Pending room
pub struct PendingRoom {
    /// Game id/address
    game: GameId,
    /// The room is viewable for others
    viewable: bool,
    /// The salt for seed by first player
    salt: [u8; 32],
    /// The block info for seed on chain
    block: [u8; 32],
    /// Player params: account, peer, pubkey
    pub players: Vec<Player>,
    /// Sequencer params: peer, websocket
    pub sequencer: Option<(PeerId, String)>,
}

/// Engine
pub struct Engine<H: Handler> {
    /// Config of engine and network
    config: Config,
    /// Rooms which is running
    rooms: HashMap<RoomId, HandlerRoom<H>>,
    /// Rooms which is waiting create, room => (game, players, sequencer)
    pub pending: HashMap<RoomId, PendingRoom>,
    /// Supported games and game's pending rooms
    pub games: HashMap<GameId, Vec<RoomId>>,
    /// Connected peers
    onlines: Arc<Mutex<HashMap<PeerId, Vec<RoomId>>>>,
}

impl<H: Handler> Engine<H> {
    /// Init a engine with config
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

    /// Create a pending room when scan from chain
    pub fn create_pending(
        &mut self,
        id: RoomId,
        game: GameId,
        viewable: bool,
        player: Player,
        salt: [u8; 32],
        block: [u8; 32],
    ) {
        if let Some(games) = self.games.get_mut(&game) {
            if !self.pending.contains_key(&id) {
                self.pending.insert(
                    id,
                    PendingRoom {
                        game,
                        viewable,
                        salt,
                        block,
                        players: vec![player],
                        sequencer: None,
                    },
                );
                games.push(id);
            }
        }
    }

    /// Join new player to the room
    pub fn join_pending(&mut self, id: RoomId, player: Player) {
        if let Some(proom) = self.pending.get_mut(&id) {
            proom.players.push(player);
        }
    }

    /// Create a pending room when scan from chain
    pub fn del_pending(&mut self, id: RoomId) {
        if let Some(proom) = self.pending.remove(&id) {
            self.games
                .get_mut(&proom.game)
                .map(|v| vec_remove_item(v, &id));
        }
    }

    /// Check if contains pending room
    pub fn contains_pending(&self, id: &RoomId) -> bool {
        self.pending.contains_key(id)
    }

    /// Create a room when scan from chain
    pub async fn start_room(
        &mut self,
        id: RoomId,
        sequencer: (PeerId, String),
        params: Vec<u8>,
        is_self: bool,
        task_sender: UnboundedSender<TaskMessage<H>>,
    ) {
        if let Some(proom) = self.pending.get_mut(&id) {
            proom.sequencer = Some(sequencer);

            if is_self {
                let seed: [u8; 32] = proom
                    .salt
                    .iter()
                    .zip(proom.block.iter())
                    .map(|(&x1, &x2)| x1 ^ x2)
                    .collect::<Vec<u8>>()
                    .try_into()
                    .unwrap_or([0u8; 32]);
                if let Some((raw_handler, tasks)) =
                    H::chain_create(&proom.players, params, id, seed).await
                {
                    let handler = Arc::new(Mutex::new(raw_handler));
                    let ids: Vec<PeerId> = proom.players.iter().map(|p| p.peer).collect();

                    // running tasks
                    if !tasks.is_empty() {
                        handle_tasks(id, tasks, handler.clone(), task_sender);
                    }

                    let room = HandlerRoom {
                        handler: handler,
                        game: proom.game,
                        room: Room::new(id, proom.viewable, &ids),
                    };

                    self.rooms.insert(id, room);
                }
            }
        }
    }

    /// Over a room
    pub async fn over_room(&mut self, id: RoomId) {
        if let Some(_room) = self.rooms.remove(&id) {
            // TODO clear onlines
        }
    }

    /// Check room exists
    pub fn has_room(&self, id: &RoomId) -> bool {
        self.rooms.contains_key(id)
    }

    /// Get room info
    pub fn get_room(&self, id: &RoomId) -> &HandlerRoom<H> {
        self.rooms.get(id).unwrap() // safe before check
    }

    /// Check the player is in the room
    pub async fn is_room_player(&self, id: &RoomId, peer: &PeerId) -> bool {
        if let Some(hr) = self.rooms.get(id) {
            hr.room.is_player(peer)
        } else {
            false
        }
    }

    /// Check the player is in some rooms that hold by this node
    pub async fn has_peer(&self, peer: &PeerId) -> bool {
        if let Some(rooms) = self.onlines.lock().await.get(&peer) {
            !rooms.is_empty()
        } else {
            false
        }
    }

    /// When a player online/connected
    pub async fn online(&mut self, id: RoomId, peer: PeerId, ctype: ConnectType) -> bool {
        let is_ok = if let Some(hr) = self.rooms.get_mut(&id) {
            hr.room.online(peer, ctype)
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

    /// When a player offline/disconnected
    pub async fn offline(&mut self, peer: PeerId) {
        let mut onlines_lock = self.onlines.lock().await;
        if let Some(rooms) = onlines_lock.remove(&peer) {
            for rid in rooms {
                if let Some(hr) = self.rooms.get_mut(&rid) {
                    hr.room.offline(peer);
                }
            }
        }
    }

    /// Run the engine with game logic
    pub async fn run(self) -> Result<()> {
        let (chain_send, chain_recv) = chain_channel();
        self.run_with_channel(chain_send, chain_recv).await
    }

    /// Run the engine with game logic and channel
    pub async fn run_with_channel(
        mut self,
        chain_send: UnboundedSender<ChainMessage>,
        mut chain_recv: UnboundedReceiver<ChainMessage>,
    ) -> Result<()> {
        let (tdn_config, key) = self.config.to_tdn();
        let chain_option = self.config.to_chain().await;

        let (peer_addr, send, mut out_recv) = start_with_config_and_key(tdn_config, key).await?;
        println!("SERVER: peer id: {:?}", peer_addr);
        println!("P2P   : http://0.0.0.0:{}", self.config.p2p_port);
        println!("HTTP  : http://0.0.0.0:{}", self.config.http_port);
        if let Some(p) = self.config.ws_port {
            println!("WS    : ws://0.0.0.0:{}", p);
        }

        let (pool_send, pool_recv) = pool_channel();
        if let Some((scan_providers, pool_provider, market_address, start_block)) = chain_option {
            let send1 = chain_send.clone();
            let send2 = chain_send.clone();
            tokio::spawn(scan_listen(
                scan_providers,
                market_address,
                send1,
                start_block,
            ));
            tokio::spawn(pool_listen(pool_provider, market_address, send2, pool_recv));
        }

        let (task_sender, mut task_receiver) = unbounded_channel();
        loop {
            let work = select! {
                w = async {
                    chain_recv.recv().await.map(FutureMessage::Chain)
                } => w,
                w = async {
                    out_recv.recv().await.map(FutureMessage::Network)
                } => w,
                w = async {
                    task_receiver.recv().await.map(FutureMessage::Task)
                } => w,
            };

            match work {
                Some(FutureMessage::Task(message)) => match message {
                    TaskMessage::Result(rid, res) => {
                        let is_over = res.over;
                        handle_result(&self.get_room(&rid).room, res, &send, None, 0).await;
                        if is_over {
                            handle_over(
                                rid,
                                self.get_room(&rid).handler.clone(),
                                chain_send.clone(),
                            );
                        }
                    }
                },
                Some(FutureMessage::Network(message)) => match message {
                    ReceiveMessage::Group(rid, msg) => {
                        if !self.has_room(&rid) {
                            continue;
                        }
                        if let Ok(Some(res)) = handle_p2p(&mut self, &send, rid, msg).await {
                            let is_over = res.over;
                            handle_result(&self.get_room(&rid).room, res, &send, None, 0).await;
                            if is_over {
                                handle_over(
                                    rid,
                                    self.get_room(&rid).handler.clone(),
                                    chain_send.clone(),
                                );
                            }
                        }
                    }
                    ReceiveMessage::Rpc(uid, params, is_ws) => {
                        match handle_rpc(&mut self, &send, uid, params, is_ws).await {
                            Ok(Some((res, rid, is_rpc, id))) => {
                                let is_over = res.over;
                                handle_result(&self.get_room(&rid).room, res, &send, is_rpc, id)
                                    .await;
                                if is_over {
                                    handle_over(
                                        rid,
                                        self.get_room(&rid).handler.clone(),
                                        chain_send.clone(),
                                    );
                                }
                            }
                            Ok(None) => {
                                let msg = RpcError::Custom("None".to_owned()).json(0);
                                let _ = send.send(SendMessage::Rpc(uid, msg, is_ws)).await;
                            }
                            Err(err) => {
                                let msg = RpcError::Custom(format!("{:?}", err)).json(0);
                                let _ = send.send(SendMessage::Rpc(uid, msg, is_ws)).await;
                            }
                        }
                    }
                    ReceiveMessage::NetworkLost => {
                        debug!("No network connections");
                    }
                    ReceiveMessage::Own(..) => {}
                },
                Some(FutureMessage::Chain(message)) => match message {
                    ChainMessage::CreateRoom(
                        rid,
                        game,
                        viewable,
                        account,
                        peer,
                        signer,
                        salt,
                        block,
                    ) => {
                        info!("Engine: chain new room created !");
                        self.create_pending(
                            rid,
                            game,
                            viewable,
                            Player {
                                account,
                                peer,
                                signer,
                            },
                            salt,
                            block,
                        );
                    }
                    ChainMessage::JoinRoom(rid, account, peer, signer) => {
                        info!("Engine: chain new player joined !");
                        self.join_pending(
                            rid,
                            Player {
                                account,
                                peer,
                                signer,
                            },
                        );
                    }
                    ChainMessage::StartRoom(rid, game) => {
                        // send accept operation to chain
                        // check room is exist
                        if let Some(proom) = self.pending.get(&rid) {
                            let params = H::chain_accept(&proom.players).await;
                            let _ = pool_send.send(PoolMessage::AcceptRoom(rid, params));
                        } else if self.games.contains_key(&game) {
                            // TODO fetch room from chain.
                        }
                    }
                    ChainMessage::AcceptRoom(rid, sequencer, ws, params) => {
                        info!("Engine: start new room: {}", rid);
                        // if mine, create room
                        let is_own = sequencer == peer_addr;
                        self.start_room(rid, (sequencer, ws), params, is_own, task_sender.clone())
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

enum FutureMessage<H: Handler> {
    Network(ReceiveMessage),
    Chain(ChainMessage),
    Task(TaskMessage<H>),
}

/// Handle result
async fn handle_result<P: Param>(
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
        started: _,
    } = result;

    loop {
        if !one.is_empty() {
            let (peer, params) = one.remove(0);
            let p2p_bytes = params.to_bytes();
            let rpc_msg = build_rpc_response(id, room.id, params.to_value());
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
            let params = all.remove(0);
            let p2p_bytes = params.to_bytes();
            let rpc_msg = build_rpc_response(id, room.id, params.to_value());
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

    if over {
        let params = MethodValues {
            method: "over".to_owned(),
            params: vec![],
        };
        let p2p_bytes = params.to_bytes();
        let rpc_msg = build_rpc_response(id, room.id, params.to_value());
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

fn handle_over<H: Handler>(
    rid: RoomId,
    handler: Arc<Mutex<H>>,
    chain_send: UnboundedSender<ChainMessage>,
) {
    tokio::spawn(async move {
        let mut lock = handler.lock().await;
        if let Ok((data, proof)) = lock.prove().await {
            let _ = chain_send.send(ChainMessage::GameOverRoom(rid, data, proof));
        }
    });
}

fn build_rpc_response(id: u64, gid: RoomId, params: Value) -> Value {
    match (
        params.get("method"),
        params.get("result"),
        params.get("params"),
    ) {
        (Some(method), Some(result), _) => {
            json!({
                "jsonrpc": "2.0",
                "id": id,
                "gid": gid,
                "method": method,
                "result": result,
            })
        }
        (Some(method), None, Some(result)) => {
            json!({
                "jsonrpc": "2.0",
                "id": id,
                "gid": gid,
                "method": method,
                "result": result,
            })
        }
        _ => {
            json!({
                "jsonrpc": "2.0",
                "id": id,
                "gid": gid,
                "result": params
            })
        }
    }
}
