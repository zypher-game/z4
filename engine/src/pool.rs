use anyhow::Result;
use ethers::prelude::*;
use std::collections::HashMap;
use std::{mem::take, sync::Arc};
use tokio::sync::mpsc::{UnboundedSender, UnboundedReceiver};

use crate::contracts::{RoomMarket, Network};
use crate::{PoolMessage, ChainMessage};

const GAS_PRICE: u64 = 20_000_000_000; // 20 GWEI
const EXTRA_GAS: u64 = 10; // extra 10%

pub async fn listen(
    client: Arc<SignerMiddleware<Provider<Http>, LocalWallet>>,
    network: Network,
    sender: UnboundedSender<ChainMessage>,
    receiver: UnboundedReceiver<PoolMessage>
) -> Result<UnboundedSender<PoolMessage>> {
    let market = RoomMarket::new(network.address("RoomMarket").unwrap(), client.clone());
    let mut games: HashMap<U256, Vec<u8>> = HashMap::new();

    while let Some(msg) = receiver.next().await {
        match msg {
            PoolMessage::Submitted(id) => {
                games.remove(&id);
            }
            PoolMessage::Submit(id, winner, proof) => {
                games.insert(id, proof);

                let gas_price = market
                    .client_ref()
                    .get_gas_price()
                    .await
                    .unwrap_or(GAS_PRICE.into());
                let extra_gas = gas_price + gas_price / U256::from(EXTRA_GAS);

                match market.over_room(id, winner, proof).gas_price(extra_gas).send().await {
                    Ok(pending) => {
                        if let Ok(receipt) = pending.await {
                            info!(
                                "Confirmed, Gas used: {:?}",
                                receipt
                                    .expect("Failed to claim receipt")
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
                        let _ = sender.send(ChainMessage::Reprove);
                    }
                }
            }
        }
    }

}
