use ethabi::{encode, Token};
use ethereum_types::{Address, H160};
use serde_json::Value;
use tdn_types::primitives::PeerId;

use crate::{Error, Result};

/// Room id = u64
pub type RoomId = u64;

/// Game id = Address
pub type GameId = Address;

/// Z4 init room id/tdn group id
pub const Z4_ROOM_MARKET_GROUP: RoomId = 4;

/// convert address to peer
#[inline]
pub fn address_to_peer(addr: Address) -> PeerId {
    PeerId(addr.0)
}

/// convert peer to address
#[inline]
pub fn peer_to_address(peer: PeerId) -> Address {
    H160(peer.0)
}

/// Helper for generate simple game result, for ranking
pub fn simple_game_result(ranks: &[Address]) -> Vec<u8> {
    encode(&[Token::Array(
        ranks.iter().map(|v| Token::Address(*v)).collect(),
    )])
}

/// Convert address to hex string
pub fn address_hex(a: &Address) -> String {
    PeerId(a.to_fixed_bytes()).to_hex()
}

/// Convert hex string to address
pub fn hex_address(v: &str) -> Result<Address> {
    if let Ok(v) = hex::decode(v.trim_start_matches("0x")) {
        if v.len() != 20 {
            return Err(Error::Anyhow("address invalid".to_owned()));
        }
        let mut bytes = [0u8; 20];
        bytes.copy_from_slice(&v);
        Ok(H160(bytes))
    } else {
        Err(Error::Anyhow("address invalid".to_owned()))
    }
}

/// Get value from env and parse it to T
pub fn env_value<T: std::str::FromStr>(key: &str, default: Option<T>) -> Result<T> {
    match (std::env::var(key), default) {
        (Ok(v), _) => v
            .parse()
            .map_err(|_| Error::Anyhow(key.to_owned() + " env invalid")),
        (Err(_), Some(v)) => Ok(v),
        (Err(_), None) => return Err(Error::Anyhow(key.to_owned() + " env missing")),
    }
}

/// Get value array from env and parse it to T
pub fn env_values<T: std::str::FromStr>(key: &str, default: Option<Vec<T>>) -> Result<Vec<T>> {
    match (std::env::var(key), default) {
        (Ok(v), default) => {
            let mut items = vec![];
            for item in v.split(",") {
                items.push(
                    item.parse()
                        .map_err(|_| Error::Anyhow(key.to_owned() + " env invalid"))?,
                );
            }
            if items.is_empty() {
                if let Some(d) = default {
                    Ok(d)
                } else {
                    Err(Error::Anyhow(key.to_owned() + " env invalid"))
                }
            } else {
                Ok(items)
            }
        }
        (Err(_), Some(v)) => Ok(v),
        (Err(_), None) => return Err(Error::Anyhow(key.to_owned() + " env missing")),
    }
}

/// Merge two values
pub fn merge_json(a: &mut Value, b: &Value) {
    match (a, b) {
        (&mut Value::Object(ref mut a), Value::Object(b)) => {
            for (k, v) in b {
                merge_json(a.entry(k.clone()).or_insert(Value::Null), v);
            }
        }
        (a, b) => {
            *a = b.clone();
        }
    }
}
