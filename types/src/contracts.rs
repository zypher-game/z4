use serde_json::Value;

pub const ROOM_MARKET_ABI: &str = include_str!("../../public/ABI/RoomMarket.json");
pub const TOKEN_ABI: &str = include_str!("../../public/ABI/Token.json");
pub const SIMPLE_GAME_ABI: &str = include_str!("../../public/ABI/SimpleGame.json");

const LOCALHOST_ADDRESS: &str = include_str!("../../public/localhost.json");
const HOLESKY_ADDRESS: &str = include_str!("../../public/localhost.json");
const SEPOLIA_ADDRESS: &str = include_str!("../../public/sepolia.json");
const OPBNBTESTNET_ADDRESS: &str = include_str!("../../public/opbnbtestnet.json");

/// Default network
pub const DEFAULT_NETWORK: Network = Network::Localhost;

/// Network types
#[derive(Hash, Eq, PartialEq, Copy, Clone, Debug)]
pub enum Network {
    Localhost,
    Holesky,
    Sepolia,
    OpBNBTestnet,
}

impl Network {
    /// Get contract name from network
    pub fn address(&self, name: &str) -> Option<[u8; 20]> {
        let addresses = match self {
            Network::Localhost => LOCALHOST_ADDRESS,
            Network::Holesky => HOLESKY_ADDRESS,
            Network::Sepolia => SEPOLIA_ADDRESS,
            Network::OpBNBTestnet => OPBNBTESTNET_ADDRESS,
        };

        if let Ok(address) = serde_json::from_str::<Value>(addresses) {
            address[name].as_str().and_then(|v| {
                if let Ok(v) = hex::decode(v.trim_start_matches("0x")) {
                    let mut bytes = [0u8; 20];
                    bytes.copy_from_slice(&v);
                    Some(bytes)
                } else {
                    None
                }
            })
        } else {
            None
        }
    }

    /// Get the network from lower-case string
    pub fn from_str(s: &str) -> Self {
        match s {
            "localhost" => Network::Localhost,
            "holesky" => Network::Holesky,
            "sepolia" => Network::Sepolia,
            "opbnbtestnet" => Network::OpBNBTestnet,
            _ => DEFAULT_NETWORK,
        }
    }

    pub fn to_str<'a>(&self) -> &'a str {
        match self {
            Network::Localhost => "localhost",
            Network::Holesky => "holesky",
            Network::Sepolia => "sepolia",
            Network::OpBNBTestnet => "opbnbtestnet",
        }
    }

    pub fn from_chain_id(chain_id: u64) -> Self {
        match chain_id {
            17000 => Network::Holesky,
            11155111 => Network::Sepolia,
            5611 => Network::OpBNBTestnet,
            _ => DEFAULT_NETWORK,
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
            Network::Holesky => NetworkConfig {
                chain_id: 17000,
                chain_name: "Holesky".to_owned(),
                rpc_urls: vec!["https://1rpc.io/holesky".to_owned()],
                icon_urls: vec!["https://icons.llamao.fi/icons/chains/rsz_ethereum.jpg".to_owned()],
                block_explorer_urls: vec!["https://holesky.beaconcha.in/".to_owned()],
                native_currency: NetworkCurrency {
                    name: "Holesky ETH".to_owned(),
                    symbol: "ETH".to_owned(),
                    decimals: 18,
                },
            },
            Network::Sepolia => NetworkConfig {
                chain_id: 11155111,
                chain_name: "Sepolia".to_owned(),
                rpc_urls: vec!["https://rpc.sepolia.org".to_owned()],
                icon_urls: vec!["https://icons.llamao.fi/icons/chains/rsz_ethereum.jpg".to_owned()],
                block_explorer_urls: vec!["https://sepolia.etherscan.io/".to_owned()],
                native_currency: NetworkCurrency {
                    name: "Sepolia ETH".to_owned(),
                    symbol: "ETH".to_owned(),
                    decimals: 18,
                },
            },
            Network::OpBNBTestnet => NetworkConfig {
                chain_id: 5611,
                chain_name: "opBNB Testnet".to_owned(),
                rpc_urls: vec!["https://opbnb-testnet-rpc.bnbchain.org".to_owned()],
                icon_urls: vec!["https://icons.llamao.fi/icons/chains/rsz_opbnb.jpg".to_owned()],
                block_explorer_urls: vec!["http://testnet.opbnbscan.com/".to_owned()],
                native_currency: NetworkCurrency {
                    name: "opBNB ETH".to_owned(),
                    symbol: "tBNB".to_owned(),
                    decimals: 18,
                },
            },
        }
    }
}
