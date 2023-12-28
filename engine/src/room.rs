use std::collections::HashMap;

use crate::types::*;
use crate::config::Config;

pub(crate) struct Manager {
    account: Account,
    signers: Vec<Account>,
    rooms: HashMap<RoomId, Room>,
    pending: HashMap<RoomId, Room>
}

impl Manager {
    pub fn from_config(config: Config) -> Manager {
        Manager {
            account: config.account,
            signers: config.signers,
            rooms: HashMap::new(),
            pending: HashMap::new()
        }
    }
}

pub struct Room {
    
}
