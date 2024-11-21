use futures_util::{SinkExt, StreamExt};
use std::path::PathBuf;
use tdn::prelude::{
    start_with_config_and_key, Config as TdnConfig, NetworkType, Peer, PeerKey, ReceiveMessage,
    RecvType, SendMessage, SendType,
};
use tokio::{
    net::TcpStream,
    sync::mpsc::{unbounded_channel, Receiver, Sender, UnboundedReceiver, UnboundedSender},
};
use tokio_tungstenite::{
    connect_async,
    tungstenite::{client::IntoClientRequest, protocol::Message},
    MaybeTlsStream, WebSocketStream,
};
use z4_types::{json, merge_json, Param, Result, RoomId, Value};

/// Channel message
pub type ChannelMessage<P> = (RoomId, P);

/// Create a channel
#[inline]
pub fn message_channel<P: Param>() -> (
    UnboundedSender<ChannelMessage<P>>,
    UnboundedReceiver<ChannelMessage<P>>,
) {
    unbounded_channel()
}

/// Running a ws channel
pub async fn run_ws_channel<P: 'static + Param>(
    peer: &PeerKey,
    room: RoomId,
    in_recv: UnboundedReceiver<ChannelMessage<P>>,
    url: impl IntoClientRequest + Unpin,
) -> Result<UnboundedReceiver<ChannelMessage<P>>> {
    let (out_send, out_recv) = unbounded_channel();
    let (ws_stream, _) = connect_async(url).await.expect("Failed to connect"); // TODO

    let peer = PeerKey::from_db_bytes(&peer.to_db_bytes()).unwrap(); // safe
    tokio::spawn(ws_listen(peer, room, out_send, in_recv, ws_stream));
    Ok(out_recv)
}

/// Running a p2p channel
pub async fn run_p2p_channel<P: 'static + Param>(
    peer: &PeerKey,
    room: RoomId,
    in_recv: UnboundedReceiver<ChannelMessage<P>>,
    server: Peer,
) -> Result<UnboundedReceiver<ChannelMessage<P>>> {
    let (out_send, out_recv) = unbounded_channel();
    let peer = PeerKey::from_db_bytes(&peer.to_db_bytes()).unwrap(); // safe

    // Running P2P network
    let mut config = TdnConfig::default();
    config.db_path = Some(PathBuf::from(&format!("./.tdn/{:?}", peer.peer_id())));
    config.rpc_ws = None;
    config.rpc_http = None;
    config.p2p_peer = Peer::socket("0.0.0.0:0".parse().unwrap()); // safe
    let (_, p2p_send, p2p_recv) = start_with_config_and_key(config, peer).await?;

    tokio::spawn(p2p_listen(
        server, room, out_send, in_recv, p2p_send, p2p_recv,
    ));
    Ok(out_recv)
}

enum WsResult<P: Param> {
    Out(ChannelMessage<P>),
    Stream(Message),
}

#[inline]
fn build_request(params: Value, room: RoomId, peer: &PeerKey) -> Value {
    let mut request = json!({
        "jsonrpc": "2.0",
        "id": 0,
        "gid": room,
        "peer": peer.peer_id().to_hex(),
    });

    merge_json(&mut request, &params);
    request
}

async fn ws_listen<P: Param>(
    peer: PeerKey,
    room: RoomId,
    send: UnboundedSender<ChannelMessage<P>>,
    mut in_recv: UnboundedReceiver<ChannelMessage<P>>,
    ws_stream: WebSocketStream<MaybeTlsStream<TcpStream>>,
) {
    let (mut writer, mut reader) = ws_stream.split();

    // send connect
    let request = build_request(
        json!({
            "method": "connect",
            "params": [],
        }),
        room,
        &peer,
    );
    let s = Message::from(serde_json::to_string(&request).unwrap_or("".to_owned()));
    let _ = writer.send(s).await;

    loop {
        let res = tokio::select! {
            v = async { in_recv.recv().await.map(|msg| WsResult::Out(msg)) } => v,
            v = async {
                reader
                    .next()
                    .await
                    .map(|msg| msg.map(|msg| WsResult::Stream(msg)).ok())
                    .flatten()
            } => v,
        };

        match res {
            Some(WsResult::Out((room, params))) => {
                let request = build_request(params.to_value(), room, &peer);
                let s = Message::from(serde_json::to_string(&request).unwrap_or("".to_owned()));
                let _ = writer.send(s).await;
            }
            Some(WsResult::Stream(msg)) => {
                let msg = msg.to_text().unwrap_or("");
                match serde_json::from_str::<Value>(&msg) {
                    Ok(mut values) => {
                        let gid = values["gid"].as_u64().unwrap_or(0);
                        let method = values["method"].as_str().unwrap_or("").to_owned();
                        let mut params = values["result"].take();
                        merge_json(
                            &mut params,
                            &json!({
                                "method": method
                            }),
                        );

                        match P::from_value(params) {
                            Ok(p) => {
                                let _ = send.send((gid, p));
                            }
                            _ => {}
                        }
                    }
                    Err(_e) => {}
                }
            }
            None => break,
        }
    }
}

enum P2pResult<P: Param> {
    Out(ChannelMessage<P>),
    Stream(ReceiveMessage),
}

async fn p2p_listen<P: Param>(
    server: Peer,
    room: RoomId,
    send: UnboundedSender<ChannelMessage<P>>,
    mut in_recv: UnboundedReceiver<ChannelMessage<P>>,
    p2p_send: Sender<SendMessage>,
    mut p2p_recv: Receiver<ReceiveMessage>,
) {
    let server_id = server.id;
    // add room to network
    let _ = p2p_send
        .send(SendMessage::Network(NetworkType::AddGroup(room)))
        .await;
    // create connection to peer socket
    let _ = p2p_send
        .send(SendMessage::Network(NetworkType::Connect(Peer::socket(
            server.socket,
        ))))
        .await;
    // create stable connection to peer
    tokio::time::sleep(std::time::Duration::from_secs(2)).await;
    let _ = p2p_send
        .send(SendMessage::Group(
            room,
            SendType::Connect(0, Peer::peer(server.id), vec![]),
        ))
        .await;

    loop {
        let res = tokio::select! {
            v = async { in_recv.recv().await.map(|msg| P2pResult::Out(msg)) } => v,
            v = async {
                p2p_recv
                    .recv()
                    .await
                    .map(|msg| P2pResult::Stream(msg))
            } => v,
        };

        match res {
            Some(P2pResult::Out((room, params))) => {
                let _ = p2p_send
                    .send(SendMessage::Group(
                        room,
                        SendType::Event(0, server_id, params.to_bytes()),
                    ))
                    .await;
            }
            Some(P2pResult::Stream(message)) => match message {
                ReceiveMessage::Group(gid, msg) => match msg {
                    RecvType::Event(peer, msg) => {
                        if peer == server_id {
                            match Param::from_bytes(msg) {
                                Ok(p) => {
                                    let _ = send.send((gid, p));
                                }
                                _ => {}
                            }
                        }
                    }
                    _ => {}
                },
                _ => {}
            },
            None => break,
        }
    }
}
