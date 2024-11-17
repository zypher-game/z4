#[macro_use]
extern crate tracing;

use ethereum_types::Address;
use futures_util::{SinkExt, StreamExt};
use pozk_utils::{convert_task_to_connect_api, BinaryMessage, TextMessage};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::{
    net::TcpStream,
    select,
    sync::{mpsc::unbounded_channel, Mutex},
};
use tokio_tungstenite::{
    connect_async,
    tungstenite::{client::IntoClientRequest, protocol::Message},
    MaybeTlsStream, WebSocketStream,
};
use z4_types::{
    address_to_peer, handle_tasks, peer_to_address, Error, HandleResult, Handler, Param, Player,
    Result, TaskMessage, PLAYER_BYTES_LEN,
};

/// Store the room info
pub struct Engine<H: Handler> {
    /// Game logic handler
    pub handler: Arc<Mutex<H>>,
    /// Game players
    pub players: HashMap<Address, bool>,
}

enum FutureMessage<H: Handler> {
    Ws(Option<Message>),
    Task(TaskMessage<H>),
}

impl<H: Handler> Engine<H> {
    /// Run the engine with game logic
    pub async fn run() -> Result<()> {
        // 1. read inputs & publics
        let input_path = std::env::var("INPUT").expect("env INPUT missing");
        let bytes = reqwest::get(&input_path)
            .await
            .unwrap()
            .bytes()
            .await
            .unwrap();

        // parse inputs and publics
        let mut input_len_bytes = [0u8; 4];
        input_len_bytes.copy_from_slice(&bytes[0..4]);
        let input_len = u32::from_be_bytes(input_len_bytes) as usize;
        let input_bytes = &bytes[4..input_len + 4];
        let publics_bytes = &bytes[input_len + 4..];
        let player = Player::from_bytes(input_bytes).expect("Invalid created player");

        // 2. connect to pozk server
        let (url, id) = convert_task_to_connect_api(&input_path);
        let rid = id.parse::<u64>().expect("Invalid INPUT");
        let viewable = H::viewable().to_string();

        let mut request = url.into_client_request().unwrap();
        request
            .headers_mut()
            .insert("X-VIEWABLE", viewable.parse().unwrap());
        let (mut ws_stream, _) = connect_async(request).await.expect("Failed to connect");

        // 3. build the handler
        let mut players = HashMap::new();
        players.insert(peer_to_address(player.peer), false);
        let (raw_handler, tasks) = H::pozk_create(player, publics_bytes.to_vec(), rid)
            .await
            .unwrap();
        let handler = Arc::new(Mutex::new(raw_handler));
        let (task_sender, mut task_receiver) = unbounded_channel();
        if !tasks.is_empty() {
            handle_tasks(rid, tasks, handler.clone(), task_sender);
        }
        let engine = Engine { handler, players };

        let mut is_over = false;
        loop {
            let work = select! {
                w = async {
                    ws_stream.next().await.map(|res| FutureMessage::Ws(res.ok()))
                } => w,
                w = async {
                    task_receiver.recv().await.map(FutureMessage::Task)
                } => w,
            };

            match work {
                Some(FutureMessage::Task(message)) => match message {
                    TaskMessage::Result(_rid, res) => {
                        if let Ok(true) = handle_res::<H>(Ok(res), false, &mut ws_stream).await {
                            is_over = true;
                            break;
                        }
                    }
                },
                Some(FutureMessage::Ws(message)) => match message {
                    Some(Message::Text(text)) => {
                        match TextMessage::decode(text) {
                            TextMessage::ConnectPlayer(peer, text) => {
                                if engine.players.contains_key(&peer) {
                                    // Online
                                    let mut handler = engine.handler.lock().await;
                                    let res = handler.online(address_to_peer(peer)).await;
                                    drop(handler);

                                    if let Ok(true) =
                                        handle_res::<H>(res, false, &mut ws_stream).await
                                    {
                                        is_over = true;
                                        break;
                                    }
                                } else {
                                    let mut data = hex::decode(text.trim_start_matches("0x"))
                                        .unwrap_or(vec![]);
                                    if let Ok(player) = Player::from_bytes(&data) {
                                        // Join
                                        let params = data.split_off(PLAYER_BYTES_LEN);
                                        let mut handler = engine.handler.lock().await;
                                        let res = handler.pozk_join(player, params).await;
                                        drop(handler);

                                        if let Ok(true) =
                                            handle_res::<H>(res, false, &mut ws_stream).await
                                        {
                                            is_over = true;
                                            break;
                                        }
                                    } else {
                                        // close
                                        let msg = TextMessage::ClosePlayer(peer).encode();
                                        let _ = ws_stream.send(Message::Text(msg));
                                    }
                                }
                            }
                            TextMessage::ConnectViewer(peer) => {
                                let mut handler = engine.handler.lock().await;
                                let res = handler.viewer_online(address_to_peer(peer)).await;
                                drop(handler);

                                if let Ok(true) = handle_res::<H>(res, false, &mut ws_stream).await
                                {
                                    is_over = true;
                                    break;
                                }
                            }
                            TextMessage::ClosePlayer(peer) => {
                                if engine.players.contains_key(&peer) {
                                    // Offline
                                    let mut handler = engine.handler.lock().await;
                                    let res = handler.offline(address_to_peer(peer)).await;
                                    drop(handler);

                                    if let Ok(true) =
                                        handle_res::<H>(res, false, &mut ws_stream).await
                                    {
                                        is_over = true;
                                        break;
                                    }
                                }
                            }
                            TextMessage::CloseViewer(peer) => {
                                let mut handler = engine.handler.lock().await;
                                let res = handler.viewer_offline(address_to_peer(peer)).await;
                                drop(handler);

                                if let Ok(true) = handle_res::<H>(res, false, &mut ws_stream).await
                                {
                                    is_over = true;
                                    break;
                                }
                            }
                            TextMessage::Player(peer, text) => {
                                let param = match H::Param::from_string(text) {
                                    Ok(p) => p,
                                    Err(_) => continue,
                                };

                                let mut handler = engine.handler.lock().await;
                                let res = handler.handle(address_to_peer(peer), param).await;
                                drop(handler);

                                if let Ok(true) = handle_res::<H>(res, false, &mut ws_stream).await
                                {
                                    is_over = true;
                                    break;
                                }
                            }
                            TextMessage::Started
                            | TextMessage::Over
                            | TextMessage::Broadcast(_) => {
                                warn!("Started/Over/Broadcast must sent to miner");
                            }
                        }
                    }
                    Some(Message::Binary(data)) => match BinaryMessage::decode(data) {
                        BinaryMessage::Player(peer, data) => {
                            let param = match H::Param::from_bytes(data) {
                                Ok(p) => p,
                                Err(_) => continue,
                            };

                            let mut handler = engine.handler.lock().await;
                            let res = handler.handle(address_to_peer(peer), param).await;
                            drop(handler);

                            if let Ok(true) = handle_res::<H>(res, true, &mut ws_stream).await {
                                is_over = true;
                                break;
                            }
                        }
                        BinaryMessage::Broadcast(_) => {
                            warn!("Broadcast must sent to miner");
                        }
                    },
                    Some(Message::Close(_)) => {
                        break;
                    }
                    Some(_) => {}
                    None => {
                        if is_over {
                            break;
                        } else {
                            // TODO try connect again
                        }
                    }
                },
                _ => {}
            }
        }

        if is_over {
            let mut handler = engine.handler.lock().await;
            if let Ok((mut data, proof)) = handler.prove().await {
                drop(handler);
                data.extend(proof);
                let client = reqwest::Client::new();
                client.post(&input_path).body(data).send().await.unwrap();
                let _ = ws_stream.send(Message::Close(None));
            }
        }

        Ok(())
    }
}

async fn handle_res<H: Handler>(
    res: Result<HandleResult<H::Param>>,
    is_binary: bool,
    ws_stream: &mut WebSocketStream<MaybeTlsStream<TcpStream>>,
) -> Result<bool> {
    let hres = match res {
        Ok(hres) => hres,
        Err(err) => {
            error!("Handle error: {:?}", err);
            return Err(Error::Params);
        }
    };
    let HandleResult {
        all,
        one,
        over,
        started,
    } = hres;

    if started {
        // send started
        let msg = TextMessage::Started.encode();
        let _ = ws_stream.send(Message::Text(msg));
    }

    for value in all {
        if is_binary {
            let data = BinaryMessage::Broadcast(value.to_bytes());
            let _ = ws_stream.send(Message::Binary(data.encode()));
        } else {
            let msg = TextMessage::Broadcast(value.to_string());
            let _ = ws_stream.send(Message::Text(msg.encode()));
        }
    }

    for (peer, value) in one {
        let peer = peer_to_address(peer);
        if is_binary {
            let data = BinaryMessage::Player(peer, value.to_bytes());
            let _ = ws_stream.send(Message::Binary(data.encode()));
        } else {
            let msg = TextMessage::Player(peer, value.to_string());
            let _ = ws_stream.send(Message::Text(msg.encode()));
        }
    }

    if over {
        let msg = TextMessage::Over.encode();
        let _ = ws_stream.send(Message::Text(msg));
    }

    Ok(over)
}
