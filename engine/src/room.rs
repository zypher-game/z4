use std::collections::hash_map::{HashMap, Iter};

use crate::{PeerId, RoomId};

#[derive(Clone, Copy, Debug)]
pub enum ConnectType {
    /// use p2p connected
    P2p,
    /// use ws connected
    Rpc(u64),
    /// offline or http
    None,
}

pub struct Room {
    pub id: RoomId,
    players: HashMap<PeerId, ConnectType>,
}

impl Room {
    pub fn new(id: RoomId, peers: &[PeerId]) -> Self {
        let players = peers.iter().map(|p| (*p, ConnectType::None)).collect();
        Self { id, players }
    }

    pub fn iter(&self) -> Iter<PeerId, ConnectType> {
        self.players.iter()
    }

    pub fn contains(&self, peer: &PeerId) -> bool {
        self.players.contains_key(peer)
    }

    pub fn get(&self, peer: &PeerId) -> ConnectType {
        self.players
            .get(peer)
            .map(|c| *c)
            .unwrap_or(ConnectType::None)
    }

    pub fn online(&mut self, peer: PeerId, ctype: ConnectType) -> bool {
        if self.players.contains_key(&peer) {
            self.players.insert(peer, ctype);
            true
        } else {
            false
        }
    }

    pub fn offline(&mut self, peer: PeerId) {
        self.players.insert(peer, ConnectType::None);
    }
}
