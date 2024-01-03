use futures_util::{SinkExt, StreamExt};
use serde_json::Value;
use tdn::{prelude::PeerKey, types::rpc::rpc_request};
use tokio::{
    net::TcpStream,
    sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender},
};
use tokio_tungstenite::{
    connect_async,
    tungstenite::{client::IntoClientRequest, protocol::Message},
    MaybeTlsStream, WebSocketStream,
};

use crate::types::*;

pub type ChannelMessage = (RoomId, String, Vec<Value>);

#[inline]
pub fn message_channel() -> (
    UnboundedSender<ChannelMessage>,
    UnboundedReceiver<ChannelMessage>,
) {
    unbounded_channel()
}

pub async fn run_ws_channel(
    peer: &PeerKey,
    room: RoomId,
    in_recv: UnboundedReceiver<ChannelMessage>,
    url: impl IntoClientRequest + Unpin,
) -> Result<UnboundedReceiver<ChannelMessage>> {
    let (out_send, out_recv) = unbounded_channel();
    let (ws_stream, _) = connect_async(url).await.expect("Failed to connect"); // TODO

    let peer = PeerKey::from_db_bytes(&peer.to_db_bytes()).unwrap(); // safe
    tokio::spawn(ws_listen(peer, room, out_send, in_recv, ws_stream));
    Ok(out_recv)
}

pub async fn run_p2p_channel(
    peer: &PeerKey,
    room: RoomId,
    recv: UnboundedReceiver<ChannelMessage>,
) -> Result<UnboundedReceiver<ChannelMessage>> {
    let (sender, receiver) = unbounded_channel();

    //

    // tokio::spawn(listen(recv));

    Ok(receiver)
}

enum FutureResult {
    Out(ChannelMessage),
    Stream(Message),
}

async fn ws_listen(
    peer: PeerKey,
    room: RoomId,
    send: UnboundedSender<ChannelMessage>,
    mut in_recv: UnboundedReceiver<ChannelMessage>,
    ws_stream: WebSocketStream<MaybeTlsStream<TcpStream>>,
) {
    let (mut writer, mut reader) = ws_stream.split();

    // send connect
    let request = rpc_request(0, "connect", vec![peer.peer_id().to_hex().into()], room);
    let s = Message::from(serde_json::to_string(&request).unwrap());
    let _ = writer.send(s).await;

    loop {
        let res = tokio::select! {
            v = async { in_recv.recv().await.map(|msg| FutureResult::Out(msg)) } => v,
            v = async {
                reader
                    .next()
                    .await
                    .map(|msg| msg.map(|msg| FutureResult::Stream(msg)).ok())
                    .flatten()
            } => v,
        };

        match res {
            Some(FutureResult::Out((room, method, params))) => {
                let request = rpc_request(0, &method, params, room);
                let s = Message::from(serde_json::to_string(&request).unwrap());
                let _ = writer.send(s).await;
            }
            Some(FutureResult::Stream(msg)) => {
                let msg = msg.to_text().unwrap();
                match serde_json::from_str::<Value>(&msg) {
                    Ok(mut values) => {
                        let gid = values["gid"].as_u64().unwrap();
                        let method = values["method"].as_str().unwrap().to_owned();
                        let params = values["result"].as_array().unwrap().to_vec();

                        let _ = send.send((gid, method, params));
                    }
                    Err(_e) => {}
                }
            }
            None => break,
        }
    }
}
