use ethers::prelude::{Http, LocalWallet, Provider, SignerMiddleware, H160, U256};
use std::{path::PathBuf, sync::Arc};
use tdn::prelude::{Config as TdnConfig, PeerKey};

use crate::{
    contracts::{Network, NetworkConfig, RoomMarket, Token},
    types::Z4_ROOM_MARKET_GROUP,
};

/// config of engine
#[derive(Default)]
pub struct Config {
    /// default groups
    pub groups: Vec<u64>,
    /// supported games
    pub games: Vec<String>,
    /// the server secret key (SECP256K1)
    pub secret_key: String,
    /// the server websocket port
    pub ws_port: Option<u16>,
    /// the server rpc port
    pub http_port: u16,
    /// the chain network name
    pub chain_network: String,
    /// the chain rpcs
    pub chain_rpcs: Vec<String>,
    /// scan start block
    pub chain_start_block: Option<u64>,
    /// auto stake to sequencer
    pub auto_stake: bool,
    /// http url for this service
    pub url_http: String,
    /// http url for this service
    pub url_websocket: String,
}

impl Config {
    pub fn to_tdn(&self) -> (TdnConfig, PeerKey) {
        let mut config = TdnConfig::default();
        config.rpc_ws = match self.ws_port {
            Some(port) => Some(format!("0.0.0.0:{}", port).parse().unwrap()),
            None => None,
        };
        config.rpc_http = Some(format!("0.0.0.0:{}", self.http_port).parse().unwrap());
        config.group_ids = self.groups.clone();
        config.group_ids.push(Z4_ROOM_MARKET_GROUP);

        // TODO boostrap seed

        let sk_str = self.secret_key.trim_start_matches("0x");
        let sk_bytes = hex::decode(&sk_str).expect("Invalid secret key");
        let key = PeerKey::from_db_bytes(&sk_bytes).expect("Invalid secret key");

        config.db_path = Some(PathBuf::from(&format!("./.tdn/{:?}", key.peer_id())));

        (config, key)
    }

    pub async fn to_chain(
        &self,
    ) -> Option<(
        Vec<Arc<Provider<Http>>>,
        Arc<SignerMiddleware<Arc<Provider<Http>>, LocalWallet>>,
        Network,
        Option<u64>,
    )> {
        if self.chain_network.is_empty() {
            return None;
        }

        let network = Network::from_str(&self.chain_network);
        let nc = NetworkConfig::from(network);
        let rpcs = if self.chain_rpcs.is_empty() {
            &nc.rpc_urls
        } else {
            &self.chain_rpcs
        };
        let providers: Vec<_> = rpcs
            .iter()
            .map(|rpc| Arc::new(Provider::<Http>::try_from(rpc).unwrap()))
            .collect();
        if providers.is_empty() {
            panic!("NO RPCS");
        }

        let sk_str = self.secret_key.trim_start_matches("0x");
        let signer = LocalWallet::from_bytes(&hex::decode(&sk_str).unwrap()).unwrap();
        let signer_provider = Arc::new(
            SignerMiddleware::new_with_provider_chain(providers[0].clone(), signer)
                .await
                .unwrap(),
        );

        if self.auto_stake && (!self.url_http.is_empty() || !self.websocket.is_empty()) {
            // check & register sequencer
            let market_address = H160(network.address("RoomMarket").unwrap());
            let token_address = H160(network.address("Token").unwrap());
            let contract = RoomMarket::new(market_address, signer_provider.clone());
            let token = Token::new(token_address, signer_provider.clone());

            // TODO check staking is enough

            let amount = contract.min_staking().await.unwrap() * U256::from(1000000);
            match token.approve(market_address, amount).send().await {
                Ok(pending) => {
                    if let Ok(_receipt) = pending.await {
                        let _ = contract
                            .stake_sequencer(self.http.clone(), self.websocket.clone(), amount)
                            .send()
                            .await;
                    } else {
                        error!("Failed to approve token");
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

        Some((providers, signer_provider, network, self.chain_start_block))
    }
}
