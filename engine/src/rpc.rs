use ethers::prelude::Address;
use serde_json::{json, Value};
use tdn::{
    prelude::{PeerId, SendMessage},
    types::rpc::rpc_response,
};
use tokio::sync::mpsc::{Sender, UnboundedSender};

use crate::{
    engine::{handle_result, Engine},
    room::ConnectType,
    types::{address_hex, ChainMessage, Error, Result, Z4_ROOM_MARKET_GROUP},
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
    let id = params["id"].as_u64().unwrap_or(0);
    let gid = params["gid"].as_u64().unwrap_or(0);
    let method = params["method"].as_str().unwrap_or("").to_owned();
    let peer_id = PeerId::from_hex(params["peer"].as_str().unwrap_or(""))?;
    let params = params["params"].take();

    // inner rpc method for query all pending room for a game
    if &method == "room_market" && gid == Z4_ROOM_MARKET_GROUP {
        let p = params.as_array().ok_or(Error::Params)?;

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
                        .map(|(p, _, _)| address_hex(p))
                        .collect();
                    if let Some((seq, http)) = &proom.sequencer {
                        pendings.push(json!({
                            "room": room,
                            "players": players,
                            "sequencer": seq.to_hex(),
                            "http": http
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

        return Ok(());
    }

    if !engine.has_room(&gid) {
        return Err(Error::NoRoom);
    }

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
        return Ok(());
    }

    if engine.is_room_player(&gid, &peer_id).await {
        let params = H::Param::from_value(params)?;
        let mut hr = engine.get_room(&gid).lock().await;
        let mut res = hr.handler.handle(peer_id, &method, params).await?;

        let over = res.replace_over();
        let is_rpc = if is_ws { None } else { Some((peer_id, uid)) };
        handle_result(&hr.room, res, send, is_rpc).await;
        drop(hr);
        if let Some((data, proof)) = over {
            let _ = chain_send.send(ChainMessage::GameOverRoom(gid, data, proof));
        }
    }
    Ok(())
}
