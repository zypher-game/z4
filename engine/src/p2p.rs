use tdn::prelude::{GroupId, RecvType, SendMessage, SendType};
use tokio::sync::mpsc::Sender;
use z4_types::{Handler, Param, Result, HandleResult};

use crate::{
    engine::Engine,
    room::ConnectType
};

/// Handle p2p message
pub async fn handle_p2p<H: Handler>(
    engine: &mut Engine<H>,
    send: &Sender<SendMessage>,
    gid: GroupId,
    msg: RecvType,
) -> Result<Option<HandleResult<H::Param>>> {
    match msg {
        RecvType::Connect(peer, _data) => {
            let mut handler = engine.get_room(&gid).handler.lock().await;
            let res = handler.online(peer.id).await?;
            drop(handler);

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

            Ok(Some(res))
        }
        RecvType::Leave(peer) => {
            engine.offline(peer.id).await;

            let mut handler = engine.get_room(&gid).handler.lock().await;
            let res = handler.offline(peer.id).await?;
            drop(handler);

            Ok(Some(res))
        }
        RecvType::Event(peer_id, data) => {
            let param = H::Param::from_bytes(data)?;

            if engine.is_room_player(&gid, &peer_id).await {
                let mut handler = engine.get_room(&gid).handler.lock().await;
                let res = handler.handle(peer_id, param).await?;
                drop(handler);

                Ok(Some(res))
            } else {
                Ok(None)
            }
        }
        _ => {
            Ok(None)
        }
    }
}
