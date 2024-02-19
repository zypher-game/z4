use serde_json::Value;
use tdn::prelude::{PeerId, SendMessage};
use tokio::sync::mpsc::{Sender, UnboundedSender};

use crate::{
    engine::{handle_result, Engine},
    room::ConnectType,
    types::{ChainMessage, Error, Result},
    Handler, Param,
};

/// handle rpc message
pub async fn handle_rpc<H: Handler>(
    engine: &Engine<H>,
    send: &Sender<SendMessage>,
    chain_send: &UnboundedSender<ChainMessage>,
    uid: u64,
    mut params: Value,
    is_ws: bool,
) -> Result<()> {
    let gid = params["gid"].as_u64().unwrap_or(0);
    if !engine.has_room(&gid) {
        return Err(Error::NoRoom);
    }

    let method = params["method"].as_str().unwrap_or("").to_owned();
    let peer_id = PeerId::from_hex(params["peer"].as_str().unwrap_or("")).unwrap();
    let params = params["params"].take();

    if &method == "connect" && is_ws {
        if engine.online(gid, peer_id, ConnectType::Rpc(uid)).await {
            let mut hr = engine.get_room(&gid).lock().await;
            let res = hr.handler.online(peer_id).await?;

            let is_rpc = if is_ws { None } else { Some((peer_id, uid)) };
            handle_result(&hr.room, res, send, is_rpc).await;
            drop(hr);
        } else {
            if !engine.has_peer(&peer_id).await {
                // TODO close the connections
            }
        }
    }

    if engine.is_room_peer(&gid, &peer_id).await {
        let params = H::Param::from_value(params)?;
        let mut hr = engine.get_room(&gid).lock().await;
        let mut res = hr.handler.handle(peer_id, &method, params).await?;

        let over = res.replace_over();
        let is_rpc = if is_ws { None } else { Some((peer_id, uid)) };
        handle_result(&hr.room, res, send, is_rpc).await;
        drop(hr);
        if let Some((data, proof)) = over {
            let _ = chain_send.send(ChainMessage::OverRoom(gid, data, proof));
        }
    }
    Ok(())
}
