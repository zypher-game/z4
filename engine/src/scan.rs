use anyhow::Result;
use ark_serialize::{CanonicalDeserialize, Compress, Validate};
use ethers::prelude::*;
use std::sync::Arc;
use std::time::Duration;
use tokio::{
    sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender},
    time::timeout,
};

use crate::contracts::{Network, RoomMarket};
use crate::{ChainMessage, PeerId, PublicKey, RoomId};

const TIMEOUT: u64 = 10;
const DELAY: u64 = 3;

#[derive(Clone, Debug, EthEvent)]
struct StartRoom {
    room: U256,
    reward: U256,
    players: Vec<Address>,
    pubkeys: Vec<H256>,
}

#[derive(Clone, Debug, EthEvent)]
struct AcceptRoom {
    room: U256,
    sequencer: Address,
    locked: U256,
}

pub fn chain_channel() -> (
    UnboundedSender<ChainMessage>,
    UnboundedReceiver<ChainMessage>,
) {
    unbounded_channel()
}

pub async fn listen(
    clients: Vec<Arc<Provider<Http>>>,
    network: Network,
    sender: UnboundedSender<ChainMessage>,
) -> Result<()> {
    let market_address = network.address("RoomMarket").unwrap();
    let markets: Vec<_> = clients
        .iter()
        .map(|client| RoomMarket::new(market_address, client.clone()))
        .collect();

    let mut next_index = 0;
    loop {
        if let Ok(start_block) = clients[next_index].get_block_number().await {
            let start_block = start_block.as_u64() - DELAY;

            let _ = running(
                start_block,
                clients.clone(),
                markets.clone(),
                sender.clone(),
            )
            .await;
        };

        // waiting 2s
        tokio::time::sleep(std::time::Duration::from_secs(2)).await;
        next_index += 1;
        if next_index == clients.len() {
            next_index = 0;
        }
        error!("SCAN service failure, next_index: {}", next_index);
    }
}

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
            info!("start {} == {} end", start, end);
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

        let start_room = markets[i]
            .event::<StartRoom>()
            .from_block(from)
            .to_block(to);

        let accept_room = markets[i]
            .event::<AcceptRoom>()
            .from_block(from)
            .to_block(to);

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

        if let Ok(starts) = starts_room {
            for start in starts {
                let StartRoom {
                    room,
                    reward,
                    players,
                    pubkeys,
                } = start;
                info!(
                    "scan start: {} {} {} {}",
                    room,
                    reward,
                    players.len(),
                    pubkeys.len()
                );
                match (parse_room(room), parse_peers(players), parse_pks(pubkeys)) {
                    (Some(rid), pids, pks) => {
                        if pids.len() == pks.len() {
                            sender.send(ChainMessage::StartRoom(rid, pids, pks))?;
                        }
                    }
                    _ => continue,
                }

                //
            }
        }

        if let Ok(accepts) = accepts_room {
            for accept in accepts {
                let AcceptRoom {
                    room,
                    sequencer,
                    locked,
                } = accept;
                info!("scan accept: {} {} {}", room, sequencer, locked);
                match (parse_room(room), parse_peer(sequencer)) {
                    (Some(rid), Some(pid)) => {
                        sender.send(ChainMessage::AcceptRoom(rid, pid))?;
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
fn parse_peers(cpids: Vec<Address>) -> Vec<PeerId> {
    let mut res = vec![];
    for cpid in cpids {
        if let Some(p) = parse_peer(cpid) {
            res.push(p)
        }
    }
    res
}

#[inline]
fn parse_pk(cpk: H256) -> Option<PublicKey> {
    PublicKey::deserialize_with_mode(cpk.as_bytes(), Compress::Yes, Validate::Yes).ok()
}

#[inline]
fn parse_pks(cpks: Vec<H256>) -> Vec<PublicKey> {
    let mut res = vec![];
    for cpk in cpks {
        if let Some(p) = parse_pk(cpk) {
            res.push(p)
        }
    }
    res
}
