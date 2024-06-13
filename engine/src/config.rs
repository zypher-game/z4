use ethers::prelude::{Address, Http, LocalWallet, Provider, SignerMiddleware, U256};
use std::{path::PathBuf, sync::Arc};
use tdn::prelude::{Config as TdnConfig, PeerKey};

use crate::{
    contracts::{Network, NetworkConfig, RoomMarket, Token},
    types::{env_value, env_values, hex_address, Result, Z4_ROOM_MARKET_GROUP},
};

/// config of engine
#[derive(Default)]
pub struct Config {
    /// default groups
    pub groups: Vec<u64>,
    /// supported games
    pub games: Vec<String>,
    /// main room market
    pub room_market: String,
    /// the server secret key (SECP256K1)
    pub secret_key: String,
    /// the server websocket port
    pub ws_port: Option<u16>,
    /// the server rpc port
    pub http_port: u16,
    /// the p2p port
    pub p2p_port: u16,
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
    /// Get config from env
    pub fn from_env() -> Result<Self> {
        dotenv::dotenv().ok();

        let network = env_value("NETWORK", None)?;
        let games: Vec<String> = env_values("GAMES", None)?;
        let secret_key = env_value("SECRET_KEY", None)?;
        let start_block = env_value("START_BLOCK", None).ok();

        let chain_rpcs = env_values("RPC_ENDPOINTS", Some(vec![]))?;
        let room_market = env_value("ROOM_MARKET", Some(games[0].clone()))?;
        let url_http = env_value("URL_HTTP", Some("".to_owned()))?;
        let url_websocket = env_value("URL_WEBSOCKET", Some("".to_owned()))?;
        let http_port = env_value("HTTP_PORT", Some(8080))?;
        let ws_port = env_value("WS_PORT", Some(8000))?;
        let p2p_port = env_value("P2P_PORT", Some(7364))?;
        let auto_stake = env_value("AUTO_STAKE", Some(false))?;

        let mut config = Config::default();
        config.http_port = http_port;
        config.ws_port = Some(ws_port);
        config.p2p_port = p2p_port;
        config.secret_key = secret_key;
        config.chain_network = network;
        config.chain_rpcs = chain_rpcs;
        config.chain_start_block = start_block;
        config.games = games;
        config.room_market = room_market;
        config.auto_stake = auto_stake;
        config.url_http = url_http;
        config.url_websocket = url_websocket;

        Ok(config)
    }

    /// Convert config to TDN config
    pub fn to_tdn(&self) -> (TdnConfig, PeerKey) {
        let rpc_addr = format!("0.0.0.0:{}", self.http_port).parse().unwrap();
        let p2p_addr = format!("0.0.0.0:{}", self.p2p_port).parse().unwrap();
        let mut config = TdnConfig::with_addr(p2p_addr, rpc_addr);
        config.rpc_ws = match self.ws_port {
            Some(port) => Some(format!("0.0.0.0:{}", port).parse().unwrap()),
            None => None,
        };
        config.group_ids = self.groups.clone();
        config.group_ids.push(Z4_ROOM_MARKET_GROUP);

        // TODO boostrap seed

        let sk_str = self.secret_key.trim_start_matches("0x");
        let sk_bytes = hex::decode(&sk_str).expect("Invalid secret key");
        let key = PeerKey::from_db_bytes(&sk_bytes).expect("Invalid secret key");

        config.db_path = Some(PathBuf::from(&format!("./.tdn/{:?}", key.peer_id())));

        (config, key)
    }

    /// Convert config to chain params
    pub async fn to_chain(
        &self,
    ) -> Option<(
        Vec<Arc<Provider<Http>>>,
        Arc<SignerMiddleware<Arc<Provider<Http>>, LocalWallet>>,
        Address,
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

        let room_market = if self.room_market.is_empty() {
            &self.games[0]
        } else {
            &self.room_market
        };
        let market_address = hex_address(room_market).expect("Invalid room market address");
        if self.auto_stake && (!self.url_http.is_empty() || !self.url_websocket.is_empty()) {
            // check & register sequencer
            let contract = RoomMarket::new(market_address, signer_provider.clone());
            let token_address = contract.token().await.unwrap();
            let token = Token::new(token_address, signer_provider.clone());

            // TODO check staking is enough

            let amount = contract.min_staking().await.unwrap() * U256::from(100);
            info!("Auto staking: {}", amount);
            match token.approve(market_address, amount).send().await {
                Ok(pending) => {
                    if let Ok(_receipt) = pending.await {
                        let _ = contract
                            .stake_sequencer(
                                self.url_http.clone(),
                                self.url_websocket.clone(),
                                amount,
                            )
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

        Some((
            providers,
            signer_provider,
            market_address,
            self.chain_start_block,
        ))
    }
}
