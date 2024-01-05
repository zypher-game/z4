use tdn::prelude::{GroupId, RecvType, SendMessage, SendType};
use tokio::sync::mpsc::{Sender, UnboundedSender};

use crate::{
    engine::{handle_result, Engine},
    room::ConnectType,
    types::{P2pMessage, PoolMessage, Result},
    Handler, Param,
};

/// handle p2p message
pub async fn handle_p2p<H: Handler>(
    engine: &mut Engine<H>,
    send: &Sender<SendMessage>,
    pool_send: &UnboundedSender<PoolMessage>,
    gid: GroupId,
    msg: RecvType,
) -> Result<()> {
    match msg {
        RecvType::Connect(peer, _data) => {
            let handler = engine.get_mut_handler(&gid).unwrap(); // safe
            let res = handler.online(peer.id).await?;

            if engine.online(gid, peer.id, ConnectType::P2p) {
                let _ = send
                    .send(SendMessage::Group(
                        gid,
                        SendType::Result(0, peer, true, false, vec![]),
                    ))
                    .await;
            } else {
                if !engine.has_peer(&peer.id) {
                    // close the connections
                    let _ = send
                        .send(SendMessage::Group(
                            gid,
                            SendType::Result(0, peer, false, false, vec![]),
                        ))
                        .await;
                }
            }

            let room = engine.get_room(&gid).unwrap(); // safe
            handle_result(room, res, send, None).await;
        }
        RecvType::Leave(peer) => {
            engine.offline(peer.id);

            let handler = engine.get_mut_handler(&gid).unwrap(); // safe
            let res = handler.offline(peer.id).await?;

            let room = engine.get_room(&gid).unwrap(); // safe
            handle_result(room, res, send, None).await;
        }
        RecvType::Event(peer_id, data) => {
            if engine.is_room_peer(&gid, &peer_id) {
                let P2pMessage { method, params } = bincode::deserialize(&data)?;
                let params = H::Param::from_bytes(&params)?;

                let handler = engine.get_mut_handler(&gid).unwrap(); // safe
                let mut res = handler.handle(peer_id, method, params).await?;

                let over = res.replace_over();
                let room = engine.get_room(&gid).unwrap(); // safe
                handle_result(room, res, send, None).await;
                if let Some((data, proof)) = over {
                    let _ = pool_send.send(PoolMessage::OverRoom(gid, data, proof));
                }
            }
        }
        _ => {}
    }

    Ok(())
}
