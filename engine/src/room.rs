use std::collections::hash_map::{HashMap, Iter};

use crate::{PeerId, RoomId};

/// The type of player connect to node
#[derive(Clone, Copy, Debug)]
pub enum ConnectType {
    /// use p2p connected
    P2p,
    /// use ws connected
    Rpc(u64),
    /// offline or http
    None,
}

/// The room info
pub struct Room {
    /// the room id
    pub id: RoomId,
    /// room is viewable
    viewable: bool,
    /// room players
    players: Vec<PeerId>,
    /// room viewers
    viewers: HashMap<PeerId, ConnectType>,
}

impl Room {
    /// Create a room
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

    /// Item the room viewers including the player
    pub fn iter(&self) -> Iter<PeerId, ConnectType> {
        self.viewers.iter()
    }

    /// Check peer is player
    pub fn is_player(&self, peer: &PeerId) -> bool {
        self.players.contains(peer)
    }

    /// Get the player/viewer connect type
    pub fn get(&self, peer: &PeerId) -> ConnectType {
        self.viewers
            .get(peer)
            .map(|c| *c)
            .unwrap_or(ConnectType::None)
    }

    /// When player/viewer online/connected
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

    /// When player/viewer offline/disconnected
    pub fn offline(&mut self, peer: PeerId) {
        self.viewers.insert(peer, ConnectType::None);
    }
}
