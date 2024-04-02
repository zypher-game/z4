use std::collections::HashMap;
use z4_engine::{
    Address, DefaultParams, Error, HandleResult, Handler, PeerId, Result, RoomId, SubGame, Tasks,
};

use crate::Executor;

pub struct Z4Handler<E: Executor> {
    exector: E,
    game: Vec<u8>,
    room: RoomId,
    accounts: HashMap<PeerId, Address>,
    storage: Vec<u8>,
    operations: Vec<DefaultParams>,
}

impl<E: Executor> Z4Handler<E> {
    fn prove(&self) {
        todo!()
    }
}

#[async_trait::async_trait]
impl<E: Executor> Handler for Z4Handler<E> {
    type Param = DefaultParams;

    async fn accept(subgame: &SubGame, peers: &[(Address, PeerId, [u8; 32])]) -> Vec<u8> {
        todo!()
    }

    async fn create(
        room_id: RoomId,
        subgame: &SubGame,
        peers: &[(Address, PeerId, [u8; 32])],
        callback: Vec<u8>,
    ) -> (Self, Tasks<Self>) {
        // TODO use room id to to fetch game logic
        todo!()
    }

    /// when player connected to server, will send remain cards
    async fn online(&mut self, player: PeerId) -> Result<HandleResult<Self::Param>> {
        todo!()
    }

    /// when player offline, tell other players, then do some change in game UI
    async fn offline(&mut self, player: PeerId) -> Result<HandleResult<Self::Param>> {
        todo!()
    }

    async fn handle(
        &mut self,
        player: PeerId,
        method: &str,
        params: DefaultParams,
    ) -> Result<HandleResult<Self::Param>> {
        let public_key = self.accounts.get(&player).ok_or(Error::NoPlayer)?;
        let params = params.0;

        let mut results = HandleResult::default();

        Ok(results)
    }
}
