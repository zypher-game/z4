use anyhow::{anyhow, Result};
use ethers::{
    abi::{decode, ParamType, Token},
    prelude::*,
    utils::keccak256,
};
use serde::Deserialize;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::{sync::mpsc::UnboundedSender, time::timeout};

use crate::contracts::{RoomMarket, Network};
use crate::ChainMessage;

const TIMEOUT: u64 = 10;
const DELAY: u64 = 3;

#[allow(non_snake_case)]
#[derive(Clone, Debug, EthEvent)]
struct CreateRoom {
    roomId: U256,
    player: Address
}

#[allow(non_snake_case)]
#[derive(Clone, Debug, EthEvent)]
struct JoinRoom {
    roomId: U256,
    player: Address
}

#[allow(non_snake_case)]
#[derive(Clone, Debug, EthEvent)]
struct StartRoom {
    roomId: U256,
    sequencer: Address
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
            let mut start_block = start_block.as_u64() - DELAY;

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
        info!("Scan {} from {} to {}", i, start, end);

        let (from, to) = if start > end {
            (end, start)
        } else {
            (start + 1, end)
        };

        let create_room = markets[i]
            .event::<CreateRoom>()
            .from_block(from)
            .to_block(to);

        let join_room = markets[i]
            .event::<JoinRoom>()
            .from_block(from)
            .to_block(to);

        let start_room = markets[i]
            .event::<StartRoom>()
            .from_block(from)
            .to_block(to);

        let creates = if let Ok(res) = timeout(Duration::from_secs(TIMEOUT), create_room.query()).await {
            res
        } else {
            warn!("Timeout: {}", i);
            continue;
        };
        let joins =
            if let Ok(res) = timeout(Duration::from_secs(TIMEOUT), join_room.query()).await {
                res
            } else {
                warn!("Timeout: {}", i);
                continue;
            };
        let starts =
            if let Ok(res) = timeout(Duration::from_secs(TIMEOUT), start_room.query()).await {
                res
            } else {
                warn!("Timeout: {}", i);
                continue;
            };

        if let Ok(creates) = creates {
            for create in creates {
                info!("scan create: {} {:?}", create.roomId, create.player);
                sender.send(ChainMessage::CreateRoom(create.roomId, create.player))?;
            }
        }

        if let Ok(joins) = joins {
            for join in joins {
                info!("scan join: {} {:?}", join.roomId, join.player);
                sender.send(ChainMessage::JoinRoom(join.roomId, join.player))?;
            }
        }

        if let Ok(starts) = starts {
            for start in starts {
                info!("scan start: {} {:?}", start.roomId, start.sequencer);
                sender.send(ChainMessage::StartRoom(start.roomId, start.sequencer))?;
            }
        }

        starts[i] = end;

        // waiting 1s
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
    }
}
