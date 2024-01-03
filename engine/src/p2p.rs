use tdn::prelude::{GroupId, RecvType, SendMessage};
use tokio::sync::mpsc::Sender;

use crate::{
    engine::{handle_result, Engine},
    room::ConnectType,
    types::Result,
    Handler,
};

/// handle p2p message
pub async fn handle_p2p<H: Handler>(
    engine: &mut Engine<H>,
    send: &Sender<SendMessage>,
    gid: GroupId,
    msg: RecvType,
) -> Result<()> {
    match msg {
        RecvType::Connect(peer, _data) => {
            println!("receive group peer {} join", peer.id.short_show());
            let handler = engine.get_mut_handler(&gid).unwrap(); // safe
            let res = handler.online(peer.id).await?;

            if engine.online(gid, peer.id, ConnectType::P2p) {
                // TODO send.send().await;
            } else {
                if !engine.has_peer(&peer.id) {
                    // TODO close the connections
                }
            }

            let room = engine.get_room(&gid).unwrap(); // safe
            handle_result(room, res, send, None).await;
        }
        RecvType::Leave(peer) => {
            println!("receive group peer {} leave", peer.id.short_show());
            engine.offline(peer.id);

            let handler = engine.get_mut_handler(&gid).unwrap(); // safe
            let res = handler.offline(peer.id).await?;

            let room = engine.get_room(&gid).unwrap(); // safe
            handle_result(room, res, send, None).await;
        }
        RecvType::Event(peer_id, _data) => {
            println!("receive group event from {}", peer_id.short_show());
            // TODO handle params
            if engine.is_room_peer(&gid, &peer_id) {
                // parse method & params
                let method = "info";
                let params = vec![];

                let handler = engine.get_mut_handler(&gid).unwrap(); // safe
                let res = handler.handle(peer_id, method, params).await?;

                let is_over = res.over;
                let room = engine.get_room(&gid).unwrap(); // safe
                handle_result(room, res, send, None).await;
                if is_over {
                    // TODO
                }
            }
        }
        _ => {}
    }

    Ok(())
}
