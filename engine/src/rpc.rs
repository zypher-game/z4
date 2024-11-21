use ethers::prelude::Address;
use serde_json::{json, Value};
use tdn::{
    prelude::{PeerId, SendMessage},
    types::rpc::rpc_response,
};
use tokio::sync::mpsc::Sender;
use z4_types::{
    address_hex, Error, HandleResult, Handler, Param, Result, RoomId, Z4_ROOM_MARKET_GROUP,
};

use crate::{engine::Engine, room::ConnectType};

/// Handle rpc message
pub async fn handle_rpc<H: Handler>(
    engine: &mut Engine<H>,
    send: &Sender<SendMessage>,
    uid: u64,
    mut params: Value,
    is_ws: bool,
) -> Result<Option<(HandleResult<H::Param>, RoomId, Option<(PeerId, u64)>, u64)>> {
    let id = params["id"].as_u64().unwrap_or(0);
    let gid = params["gid"].as_u64().unwrap_or(0);
    let method = params["method"].as_str().unwrap_or("").to_owned();
    let peer_id = PeerId::from_hex(params["peer"].as_str().unwrap_or(""))?;

    // inner rpc method for query all pending room for a game
    if &method == "room_market" && gid == Z4_ROOM_MARKET_GROUP {
        let values = params["params"].take();
        let p = values.as_array().ok_or(Error::Params)?;

        if p.is_empty() {
            return Err(Error::NoRoom);
        }
        let game: Address = p[0]
            .as_str()
            .ok_or(Error::Params)?
            .parse()
            .map_err(|_| Error::Params)?;
        let mut pendings = vec![];
        if let Some(rooms) = engine.games.get(&game) {
            for room in rooms {
                if let Some(proom) = engine.pending.get(room) {
                    let players: Vec<String> = proom
                        .players
                        .iter()
                        .map(|p| address_hex(&p.account))
                        .collect();
                    if let Some((seq, ws)) = &proom.sequencer {
                        pendings.push(json!({
                            "room": room,
                            "players": players,
                            "sequencer": seq.to_hex(),
                            "websocket": ws
                        }));
                    } else {
                        pendings.push(json!({
                            "room": room,
                            "players": players
                        }));
                    }
                }
            }
        } else {
            return Err(Error::NoGame);
        }

        let rpc_msg = rpc_response(id, &method, json!(pendings), gid);
        let _ = send.send(SendMessage::Rpc(uid, rpc_msg, is_ws)).await;

        return Ok(None);
    }

    if !engine.has_room(&gid) {
        return Err(Error::NoRoom);
    }

    if &method == "connect" && is_ws {
        if engine.online(gid, peer_id, ConnectType::Rpc(uid)).await {
            let mut handler = engine.get_room(&gid).handler.lock().await;
            let res = handler.online(peer_id).await?;
            drop(handler);

            let is_rpc = if is_ws { None } else { Some((peer_id, uid)) };
            return Ok(Some((res, gid, is_rpc, id)));
        } else {
            if !engine.has_peer(&peer_id).await {
                // TODO close the connections
            }
        }
        return Ok(None);
    }

    let param = H::Param::from_value(params)?;

    if engine.is_room_player(&gid, &peer_id).await {
        let mut handler = engine.get_room(&gid).handler.lock().await;
        let res = handler.handle(peer_id, param).await?;
        drop(handler);

        let is_rpc = if is_ws { None } else { Some((peer_id, uid)) };
        return Ok(Some((res, gid, is_rpc, id)));
    }

    Ok(None)
}
