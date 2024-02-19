use tdn::prelude::{GroupId, RecvType, SendMessage, SendType};
use tokio::sync::mpsc::{Sender, UnboundedSender};

use crate::{
    engine::{handle_result, Engine},
    room::ConnectType,
    types::{ChainMessage, P2pMessage, Result},
    Handler, Param,
};

/// handle p2p message
pub async fn handle_p2p<H: Handler>(
    engine: &Engine<H>,
    send: &Sender<SendMessage>,
    chain_send: &UnboundedSender<ChainMessage>,
    gid: GroupId,
    msg: RecvType,
) -> Result<()> {
    match msg {
        RecvType::Connect(peer, _data) => {
            let mut hr = engine.get_room(&gid).lock().await;
            let res = hr.handler.online(peer.id).await?;
            drop(hr);

            if engine.online(gid, peer.id, ConnectType::P2p).await {
                let _ = send
                    .send(SendMessage::Group(
                        gid,
                        SendType::Result(0, peer, true, false, vec![]),
                    ))
                    .await;
            } else {
                if !engine.has_peer(&peer.id).await {
                    // close the connections
                    let _ = send
                        .send(SendMessage::Group(
                            gid,
                            SendType::Result(0, peer, false, false, vec![]),
                        ))
                        .await;
                }
            }

            let hr = engine.get_room(&gid).lock().await;
            handle_result(&hr.room, res, send, None).await;
            drop(hr);
        }
        RecvType::Leave(peer) => {
            engine.offline(peer.id).await;

            let mut hr = engine.get_room(&gid).lock().await;
            let res = hr.handler.offline(peer.id).await?;
            handle_result(&hr.room, res, send, None).await;
            drop(hr)
        }
        RecvType::Event(peer_id, data) => {
            if engine.is_room_peer(&gid, &peer_id).await {
                let P2pMessage { method, params } = bincode::deserialize(&data)?;
                let params = H::Param::from_bytes(&params)?;

                let mut hr = engine.get_room(&gid).lock().await;
                let mut res = hr.handler.handle(peer_id, method, params).await?;

                let over = res.replace_over();
                handle_result(&hr.room, res, send, None).await;
                drop(hr);
                if let Some((data, proof)) = over {
                    let _ = chain_send.send(ChainMessage::OverRoom(gid, data, proof));
                }
            }
        }
        _ => {}
    }

    Ok(())
}
