use anyhow::Result;
use ark_serialize::{CanonicalDeserialize, Compress, Validate};
use ethers::prelude::*;
use std::sync::Arc;
use std::time::Duration;
use tokio::{
    sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender},
    time::timeout,
};
use z4_types::{PeerId, PublicKey, RoomId};

use crate::contracts::RoomMarket;
use crate::ChainMessage;

const TIMEOUT: u64 = 10;
const DELAY: u64 = 1;

#[derive(Clone, Debug, EthEvent)]
struct CreateRoom {
    room: U256,
    game: Address,
    reward: U256,
    viewable: bool,
    player: Address,
    peer: Address,
    pk: H256,
    salt: H256,
    block: H256,
}

#[derive(Clone, Debug, EthEvent)]
struct JoinRoom {
    room: U256,
    player: Address,
    peer: Address,
    pk: H256,
}

#[derive(Clone, Debug, EthEvent)]
struct StartRoom {
    room: U256,
    game: Address,
}

#[derive(Clone, Debug, EthEvent)]
struct AcceptRoom {
    room: U256,
    sequencer: Address,
    websocket: String,
    locked: U256,
    params: Bytes,
}

#[derive(Clone, Debug, EthEvent)]
struct OverRoom {
    room: U256,
}

/// Create scan channel
pub fn chain_channel() -> (
    UnboundedSender<ChainMessage>,
    UnboundedReceiver<ChainMessage>,
) {
    unbounded_channel()
}

/// Listen scan task
pub async fn listen(
    clients: Vec<Arc<Provider<Http>>>,
    market_address: Address,
    sender: UnboundedSender<ChainMessage>,
    start: Option<u64>,
) -> Result<()> {
    let markets: Vec<_> = clients
        .iter()
        .map(|client| RoomMarket::new(market_address, client.clone()))
        .collect();

    let mut next_index = 0;
    loop {
        let start_block = if start.is_some() {
            start
        } else {
            if let Ok(start_block) = clients[next_index].get_block_number().await {
                Some(start_block.as_u64() - DELAY)
            } else {
                None
            }
        };

        if let Some(start_block) = start_block {
            let _ = running(
                start_block,
                clients.clone(),
                markets.clone(),
                sender.clone(),
            )
            .await;
        }

        // waiting 2s
        tokio::time::sleep(std::time::Duration::from_secs(2)).await;
        next_index += 1;
        if next_index == clients.len() {
            next_index = 0;
        }
        error!("SCAN service failure, next_index: {}", next_index);
    }
}

/// Loop running scan task
pub async fn running(
    start_block: u64,
    clients: Vec<Arc<Provider<Http>>>,
    markets: Vec<RoomMarket<Provider<Http>>>,
    sender: UnboundedSender<ChainMessage>,
) -> Result<()> {
    let clients_len = clients.len();

    let mut starts: Vec<_> = vec![start_block; clients_len];
    let mut i = 0;
    loop {
        i += 1;
        i = if i < clients_len { i } else { 0 };

        let start = starts[i];
        let end_res = if let Ok(res) =
            timeout(Duration::from_secs(TIMEOUT), clients[i].get_block_number()).await
        {
            res
        } else {
            warn!("Timeout: {}", i);
            continue;
        };
        if let Err(err) = end_res {
            error!("{}", err);
            continue;
        }
        let mut end = end_res.unwrap().as_u64() - DELAY; // safe
        if start == end {
            debug!("start {} == {} end", start, end);
            tokio::time::sleep(std::time::Duration::from_secs(1)).await;
            continue;
        }
        if end > start && end - start > 200 {
            end = start + 200;
        }
        debug!("Scan {} from {} to {}", i, start, end);

        let (from, to) = if start > end {
            (end, start)
        } else {
            (start + 1, end)
        };

        let create_room = markets[i]
            .event::<CreateRoom>()
            .from_block(from)
            .to_block(to);

        let join_room = markets[i].event::<JoinRoom>().from_block(from).to_block(to);

        let start_room = markets[i]
            .event::<StartRoom>()
            .from_block(from)
            .to_block(to);

        let accept_room = markets[i]
            .event::<AcceptRoom>()
            .from_block(from)
            .to_block(to);

        let over_room = markets[i].event::<OverRoom>().from_block(from).to_block(to);

        let creates_room =
            if let Ok(res) = timeout(Duration::from_secs(TIMEOUT), create_room.query()).await {
                res
            } else {
                warn!("Timeout: {}", i);
                continue;
            };

        let joins_room =
            if let Ok(res) = timeout(Duration::from_secs(TIMEOUT), join_room.query()).await {
                res
            } else {
                warn!("Timeout: {}", i);
                continue;
            };

        let starts_room =
            if let Ok(res) = timeout(Duration::from_secs(TIMEOUT), start_room.query()).await {
                res
            } else {
                warn!("Timeout: {}", i);
                continue;
            };

        let accepts_room =
            if let Ok(res) = timeout(Duration::from_secs(TIMEOUT), accept_room.query()).await {
                res
            } else {
                warn!("Timeout: {}", i);
                continue;
            };

        let overs_room =
            if let Ok(res) = timeout(Duration::from_secs(TIMEOUT), over_room.query()).await {
                res
            } else {
                warn!("Timeout: {}", i);
                continue;
            };

        if let Ok(creates) = creates_room {
            for create in creates {
                let CreateRoom {
                    room,
                    game,
                    reward,
                    viewable,
                    player,
                    peer,
                    pk,
                    salt,
                    block,
                } = create;
                info!(
                    "scan create: {} {} {} {} {}",
                    room, game, reward, viewable, player
                );

                match (parse_room(room), parse_peer(peer)) {
                    (Some(rid), Some(peer)) => {
                        sender.send(ChainMessage::CreateRoom(
                            rid,
                            game,
                            viewable,
                            player,
                            peer,
                            pk.to_fixed_bytes(),
                            salt.to_fixed_bytes(),
                            block.to_fixed_bytes(),
                        ))?;
                    }
                    _ => continue,
                }
            }
        }

        if let Ok(joins) = joins_room {
            for join in joins {
                let JoinRoom {
                    room,
                    player,
                    peer,
                    pk,
                } = join;
                info!("scan join: {} {}", room, player);

                match (parse_room(room), parse_peer(peer)) {
                    (Some(rid), Some(peer)) => {
                        sender.send(ChainMessage::JoinRoom(
                            rid,
                            player,
                            peer,
                            pk.to_fixed_bytes(),
                        ))?;
                    }
                    _ => continue,
                }
            }
        }

        if let Ok(starts) = starts_room {
            for start in starts {
                let StartRoom { room, game } = start;
                info!("scan start: {} {} ", room, game);

                match parse_room(room) {
                    Some(rid) => {
                        sender.send(ChainMessage::StartRoom(rid, game))?;
                    }
                    _ => continue,
                }
            }
        }

        if let Ok(accepts) = accepts_room {
            for accept in accepts {
                let AcceptRoom {
                    room,
                    sequencer,
                    websocket,
                    locked,
                    params,
                } = accept;
                info!(
                    "scan accept: {} {} {} {}",
                    room, sequencer, websocket, locked
                );
                match (parse_room(room), parse_peer(sequencer)) {
                    (Some(rid), Some(pid)) => {
                        sender.send(ChainMessage::AcceptRoom(
                            rid,
                            pid,
                            websocket,
                            params.to_vec(),
                        ))?;
                    }
                    _ => continue,
                }
            }
        }

        if let Ok(overs) = overs_room {
            for over in overs {
                let OverRoom { room } = over;
                info!("scan over: {}", room);
                match parse_room(room) {
                    Some(rid) => {
                        sender.send(ChainMessage::ChainOverRoom(rid))?;
                    }
                    _ => continue,
                }
            }
        }

        starts[i] = end;

        // waiting 1s
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
    }
}

#[inline]
fn parse_room(cid: U256) -> Option<RoomId> {
    if cid > U256::from(RoomId::MAX) {
        None
    } else {
        Some(cid.as_u64())
    }
}

#[inline]
fn parse_peer(cpid: Address) -> Option<PeerId> {
    PeerId::from_bytes(cpid.as_bytes()).ok()
}

#[inline]
fn _parse_peers(cpids: Vec<Address>) -> Vec<PeerId> {
    let mut res = vec![];
    for cpid in cpids {
        if let Some(p) = parse_peer(cpid) {
            res.push(p)
        }
    }
    res
}

#[inline]
fn _parse_pk(cpk: H256) -> Option<PublicKey> {
    PublicKey::deserialize_with_mode(cpk.as_bytes(), Compress::Yes, Validate::Yes).ok()
}

#[inline]
fn _parse_pks(cpks: Vec<H256>) -> Vec<PublicKey> {
    let mut res = vec![];
    for cpk in cpks {
        if let Some(p) = _parse_pk(cpk) {
            res.push(p)
        }
    }
    res
}
