use std::path::PathBuf;
use tdn::prelude::{Config as TdnConfig, PeerKey};

#[derive(Default)]
pub struct Config {
    pub secret_key: String,
    pub chain_rpcs: Vec<String>,
    pub ws_port: Option<u16>,
    pub http_port: u16,
}

impl Config {
    pub fn to_tdn(&self) -> (TdnConfig, PeerKey) {
        let mut config = TdnConfig::default();
        config.rpc_ws = match self.ws_port {
            Some(port) => Some(format!("0.0.0.0:{}", port).parse().unwrap()),
            None => None,
        };
        config.rpc_http = Some(format!("0.0.0.0:{}", self.http_port).parse().unwrap());
        // TODO boostrap seed

        let sk_str = self.secret_key.trim_start_matches("0x");
        let sk_bytes = hex::decode(&sk_str).expect("Invalid secret key");
        let key = PeerKey::from_db_bytes(&sk_bytes).expect("Invalid secret key");

        config.db_path = Some(PathBuf::from(&format!("./.tdn/{:?}", key.peer_id())));

        (config, key)
    }
}
