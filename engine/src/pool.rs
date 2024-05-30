use ethers::prelude::*;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};

use crate::contracts::{ RoomMarket};
use crate::{ChainMessage, PoolMessage, Result, RoomId};

const GAS_PRICE: u64 = 20_000_000_000; // 20 GWEI
const EXTRA_GAS: u64 = 10; // extra 10%

pub fn pool_channel() -> (UnboundedSender<PoolMessage>, UnboundedReceiver<PoolMessage>) {
    unbounded_channel()
}

pub async fn listen(
    client: Arc<SignerMiddleware<Arc<Provider<Http>>, LocalWallet>>,
    market_address: Address,
    sender: UnboundedSender<ChainMessage>,
    mut receiver: UnboundedReceiver<PoolMessage>,
) -> Result<()> {
    let market = RoomMarket::new(market_address, client.clone());
    let mut games: HashMap<RoomId, (Vec<u8>, Vec<u8>)> = HashMap::new();

    while let Some(msg) = receiver.recv().await {
        match msg {
            PoolMessage::AcceptRoom(id, params) => {
                let gas_price = market
                    .client_ref()
                    .get_gas_price()
                    .await
                    .unwrap_or(GAS_PRICE.into());
                let extra_gas = gas_price + gas_price / U256::from(EXTRA_GAS);

                match market
                    .accept_room(U256::from(id), params.into())
                    .gas_price(extra_gas)
                    .send()
                    .await
                {
                    Ok(pending) => {
                        if let Ok(receipt) = pending.await {
                            info!(
                                "Accepted, Gas used: {:?}",
                                receipt
                                    .expect("Failed to accept receipt")
                                    .cumulative_gas_used
                            );
                        } else {
                            error!("Failed to accept event");
                        }
                    }
                    Err(err) => {
                        if let Some(rcode) = err.decode_revert::<String>() {
                            error!("{}", rcode);
                        } else {
                            error!("{}", err);
                        }
                    }
                }
            }
            PoolMessage::OverRoom(id, result, proof) => {
                games.insert(id, (result.clone(), proof.clone()));

                let gas_price = market
                    .client_ref()
                    .get_gas_price()
                    .await
                    .unwrap_or(GAS_PRICE.into());
                let extra_gas = gas_price + gas_price / U256::from(EXTRA_GAS);

                match market
                    .over_room_with_zk(U256::from(id), result.into(), proof.into())
                    .gas_price(extra_gas)
                    .send()
                    .await
                {
                    Ok(pending) => {
                        if let Ok(receipt) = pending.await {
                            info!(
                                "Game over sent, Gas used: {:?}",
                                receipt
                                    .expect("Failed to claim receipt")
                                    .cumulative_gas_used
                            );
                        } else {
                            error!("Failed to sent event");
                        }
                    }
                    Err(err) => {
                        if let Some(rcode) = err.decode_revert::<String>() {
                            error!("{}", rcode);
                        } else {
                            error!("{}", err);
                        }
                        let _ = sender.send(ChainMessage::Reprove);
                    }
                }
            }
            PoolMessage::Submitted(id) => {
                games.remove(&id);
            }
        }
    }

    Ok(())
}
