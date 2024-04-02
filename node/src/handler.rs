use std::collections::HashMap;
use z4_engine::{
    Address, DefaultParams, Error, HandleResult, Handler, PeerId, Result, RoomId, SubGame, Tasks,
};

use crate::game::{contains, load, GameLogic};
use crate::Executor;

pub struct Z4Handler<E: Executor> {
    room: RoomId,
    accounts: HashMap<PeerId, (Address, [u8; 32])>,
    executor: E,
    game: GameLogic,
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

    async fn accept(subgame: &SubGame, peers: &[(Address, PeerId, [u8; 32])]) -> Result<Vec<u8>> {
        if contains(subgame).await {
            // xx
        }

        todo!()
    }

    async fn create(
        room: RoomId,
        subgame: &SubGame,
        peers: &[(Address, PeerId, [u8; 32])],
        callback: Vec<u8>,
    ) -> Result<(Self, Tasks<Self>)> {
        // use subgame to to fetch game logic
        let game = load(subgame).await?;

        // call game constructor
        let executor = E::create();
        let (storage, _) = executor.execute(&game.constructor, &[], &DefaultParams::default())?;

        // TODO inject game tasks

        // accounts, TODO more about sign account
        let accounts = peers
            .iter()
            .map(|(aid, pid, pk)| (*pid, (*aid, *pk)))
            .collect();

        Ok((
            Z4Handler {
                room,
                accounts,
                executor,
                game,
                storage,
                operations: vec![],
            },
            Tasks::default(),
        ))
    }

    /// when player connected to server, will send remain cards
    async fn online(&mut self, player: PeerId) -> Result<HandleResult<Self::Param>> {
        let params = DefaultParams(vec!["online".into(), player.to_hex().into()]);
        let (storage, results) =
            self.executor
                .execute(&self.game.methods, &self.storage, &params)?;
        self.storage = storage;
        self.operations.push(params);
        Ok(results)
    }

    /// when player offline, tell other players, then do some change in game UI
    async fn offline(&mut self, player: PeerId) -> Result<HandleResult<Self::Param>> {
        let params = DefaultParams(vec!["offline".into(), player.to_hex().into()]);
        let (storage, results) =
            self.executor
                .execute(&self.game.methods, &self.storage, &params)?;
        self.storage = storage;
        self.operations.push(params);
        Ok(results)
    }

    async fn handle(
        &mut self,
        player: PeerId,
        method: &str,
        params: DefaultParams,
    ) -> Result<HandleResult<Self::Param>> {
        if !self.accounts.contains_key(&player) {
            return Err(Error::NoPlayer);
        }

        let mut next_params = params.0;
        next_params.insert(0, method.into());
        let new_params = DefaultParams(next_params);
        let (storage, results) =
            self.executor
                .execute(&self.game.methods, &self.storage, &new_params)?;
        self.storage = storage;
        self.operations.push(new_params);
        Ok(results)
    }
}
