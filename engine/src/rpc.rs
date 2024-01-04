use serde_json::Value;
use tdn::prelude::{PeerId, SendMessage};
use tokio::sync::mpsc::Sender;

use crate::{
    engine::{handle_result, Engine},
    room::ConnectType,
    types::{Error, Result},
    Handler, Param,
};

/// handle rpc message
pub async fn handle_rpc<H: Handler>(
    engine: &mut Engine<H>,
    send: &Sender<SendMessage>,
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
        if engine.online(gid, peer_id, ConnectType::Rpc(uid)) {
            let handler = engine.get_mut_handler(&gid).unwrap(); // safe
            let res = handler.online(peer_id).await?;

            let is_rpc = if is_ws { None } else { Some((peer_id, uid)) };
            let room = engine.get_room(&gid).unwrap(); // safe
            handle_result(room, res, send, is_rpc).await;
        } else {
            if !engine.has_peer(&peer_id) {
                // TODO close the connections
            }
        }
    }

    if engine.is_room_peer(&gid, &peer_id) {
        let params = H::Param::from_value(params)?;
        let handler = engine.get_mut_handler(&gid).unwrap(); // safe
        let res = handler.handle(peer_id, &method, params).await?;

        let is_over = res.over;
        let is_rpc = if is_ws { None } else { Some((peer_id, uid)) };
        let room = engine.get_room(&gid).unwrap(); // safe
        handle_result(room, res, send, is_rpc).await;
        if is_over {
            // TODO
        }
    }
    Ok(())
}
