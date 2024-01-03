use std::collections::hash_map::{HashMap, Iter};

use crate::config::Config;
use crate::types::*;
use crate::PeerId;

pub(crate) struct Manager {
    onlines: HashMap<PeerId, Vec<RoomId>>,
    rooms: HashMap<RoomId, Room>,
    pending: HashMap<RoomId, Room>,
}

impl Manager {
    pub fn from_config(config: Config) -> Manager {
        let mut rooms = HashMap::new();
        rooms.insert(
            1,
            Room::new(&vec![
                PeerId::from_hex("0xb3E5f88cB7849A60647Eb3D3A074Ca1F182C038B").unwrap(),
                PeerId::from_hex("0x24142Dbbd86b59599d6263c70c5CB18C0CF00E72").unwrap(),
                PeerId::from_hex("0x63B7F4dd36908472970451d103D7cBd5dc52B07e").unwrap(),
                PeerId::from_hex("0x33B43F422Ad644D131ebe86F840fE58206a72198").unwrap(),
            ]),
        );

        Manager {
            onlines: HashMap::new(),
            rooms,
            pending: HashMap::new(),
        }
    }

    pub fn get_room(&self, id: &RoomId) -> Option<&Room> {
        self.rooms.get(id)
    }

    pub fn has_peer(&self, peer: &PeerId) -> bool {
        if let Some(rooms) = self.onlines.get(&peer) {
            !rooms.is_empty()
        } else {
            false
        }
    }

    pub fn has_room(&self, id: &RoomId) -> bool {
        self.rooms.contains_key(id)
    }

    pub fn is_room_peer(&self, id: &RoomId, peer: &PeerId) -> bool {
        // TODO
        true
    }

    pub fn online(&mut self, id: RoomId, peer: PeerId, ctype: ConnectType) -> bool {
        let is_ok = if let Some(room) = self.rooms.get_mut(&id) {
            room.online(peer, ctype)
        } else {
            false
        };

        if is_ok {
            self.onlines
                .entry(peer)
                .and_modify(|rooms| {
                    if !rooms.contains(&id) {
                        rooms.push(id);
                    }
                })
                .or_insert(vec![id]);
        }

        is_ok
    }

    pub fn offline(&mut self, peer: PeerId) {
        if let Some(rooms) = self.onlines.remove(&peer) {
            for rid in rooms {
                if let Some(room) = self.rooms.get_mut(&rid) {
                    room.offline(peer);
                }
            }
        }
    }
}

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
    players: HashMap<PeerId, ConnectType>,
}

impl Room {
    pub fn new(peers: &[PeerId]) -> Self {
        let players = peers.iter().map(|p| (*p, ConnectType::None)).collect();
        Self { players }
    }

    pub fn iter(&self) -> Iter<PeerId, ConnectType> {
        self.players.iter()
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
