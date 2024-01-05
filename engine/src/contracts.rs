use ethers::prelude::*;
use serde_json::Value;

abigen!(RoomMarket, "public/ABI/RoomMarket.json");

const LOCALHOST_ADDRESS: &str = include_str!("../../public/localhost.json");
const MUMBAI_ADDRESS: &str = include_str!("../../public/localhost.json");
const FUJI_ADDRESS: &str = include_str!("../../public/localhost.json");
const ARBITRUMGOERLI_ADDRESS: &str = include_str!("../../public/localhost.json");

/// Default network
pub const DEFAULT_NETWORK: Network = Network::Localhost;

/// Network types
#[derive(Hash, Eq, PartialEq, Copy, Clone, Debug)]
pub enum Network {
    Localhost,
    Mumbai,
    Fuji,
    ArbitrumGoerli,
}

impl Network {
    /// Get contract name from network
    pub fn address(&self, name: &str) -> Option<Address> {
        let addresses = match self {
            Network::Localhost => LOCALHOST_ADDRESS,
            Network::Mumbai => MUMBAI_ADDRESS,
            Network::Fuji => FUJI_ADDRESS,
            Network::ArbitrumGoerli => ARBITRUMGOERLI_ADDRESS,
        };

        if let Ok(address) = serde_json::from_str::<Value>(addresses) {
            address[name].as_str().and_then(|v| v.parse().ok())
        } else {
            None
        }
    }

    /// Get the network from lower-case string
    pub fn from_str(s: &str) -> Self {
        match s {
            "localhost" => Network::Localhost,
            "mumbai" => Network::Mumbai,
            "fuji" => Network::Fuji,
            "arbitrumgoerli" => Network::ArbitrumGoerli,
            _ => DEFAULT_NETWORK,
        }
    }

    pub fn to_str<'a>(&self) -> &'a str {
        match self {
            Network::Localhost => "localhost",
            Network::Mumbai => "mumbai",
            Network::Fuji => "fuji",
            Network::ArbitrumGoerli => "arbitrumgoerli",
        }
    }
}

/// Network native currency config
#[derive(Default)]
pub struct NetworkCurrency {
    /// Currency name
    pub name: String,
    /// Currency symbol
    pub symbol: String,
    /// Currency decimals
    pub decimals: i32,
}

/// Network config informato, use EIP-3085
#[derive(Default)]
pub struct NetworkConfig {
    /// Chain id
    pub chain_id: i32,
    /// Chain name
    pub chain_name: String,
    /// List of endpoints
    pub rpc_urls: Vec<String>,
    /// list of block explorer urls
    pub block_explorer_urls: Vec<String>,
    /// List of chain icon urls
    pub icon_urls: Vec<String>,
    /// Native currency config
    pub native_currency: NetworkCurrency,
}

impl NetworkConfig {
    /// Get the network config
    pub fn from(network: Network) -> NetworkConfig {
        match network {
            Network::Localhost => {
                let mut nc = NetworkConfig::default();
                nc.rpc_urls = vec!["http://127.0.0.1:8545".to_owned()];
                nc
            }
            Network::Mumbai => NetworkConfig {
                chain_id: 80001,
                chain_name: "Mumbai".to_owned(),
                rpc_urls: vec!["https://rpc.ankr.com/polygon_mumbai".to_owned()],
                icon_urls: vec!["https://icons.llamao.fi/icons/chains/rsz_polygon.jpg".to_owned()],
                block_explorer_urls: vec!["https://mumbai.polygonscan.com".to_owned()],
                native_currency: NetworkCurrency {
                    name: "Matic Token".to_owned(),
                    symbol: "MATIC".to_owned(),
                    decimals: 18,
                },
            },
            Network::Fuji => NetworkConfig {
                chain_id: 43113,
                chain_name: "Avalanche Fuji".to_owned(),
                rpc_urls: vec!["https://api.avax-test.network/ext/bc/C/rpc".to_owned()],
                icon_urls: vec!["https://icons.llamao.fi/icons/chains/rsz_avalanche.jpg".to_owned()],
                block_explorer_urls: vec!["https://cchain.explorer.avax-test.network".to_owned()],
                native_currency: NetworkCurrency {
                    name: "Avalanche Token".to_owned(),
                    symbol: "AVAX".to_owned(),
                    decimals: 18,
                },
            },
            Network::ArbitrumGoerli => NetworkConfig {
                chain_id: 421613,
                chain_name: "Arbitrum Goerli".to_owned(),
                rpc_urls: vec!["https://goerli-rollup.arbitrum.io/rpc".to_owned()],
                icon_urls: vec!["https://icons.llamao.fi/icons/chains/rsz_arbitrum.jpg".to_owned()],
                block_explorer_urls: vec!["https://goerli-rollup-explorer.arbitrum.io".to_owned()],
                native_currency: NetworkCurrency {
                    name: "Arbitrum Goerli Token".to_owned(),
                    symbol: "AGOR".to_owned(),
                    decimals: 18,
                },
            },
        }
    }
}
