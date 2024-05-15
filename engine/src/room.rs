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
    viewable: bool,
    players: Vec<PeerId>,
    viewers: HashMap<PeerId, ConnectType>,
}

impl Room {
    pub fn new(id: RoomId, viewable: bool, peers: &[PeerId]) -> Self {
        let players = peers.to_vec();
        let viewers = if viewable {
            HashMap::new()
        } else {
            peers.iter().map(|p| (*p, ConnectType::None)).collect()
        };

        Self {
            id,
            viewable,
            players,
            viewers,
        }
    }

    pub fn iter(&self) -> Iter<PeerId, ConnectType> {
        self.viewers.iter()
    }

    pub fn is_player(&self, peer: &PeerId) -> bool {
        self.players.contains(peer)
    }

    pub fn get(&self, peer: &PeerId) -> ConnectType {
        self.viewers
            .get(peer)
            .map(|c| *c)
            .unwrap_or(ConnectType::None)
    }

    pub fn online(&mut self, peer: PeerId, ctype: ConnectType) -> bool {
        if self.viewable {
            self.viewers.insert(peer, ctype);
            true
        } else {
            if self.viewers.contains_key(&peer) {
                self.viewers.insert(peer, ctype);
                true
            } else {
                false
            }
        }
    }

    pub fn offline(&mut self, peer: PeerId) {
        self.viewers.insert(peer, ConnectType::None);
    }
}
