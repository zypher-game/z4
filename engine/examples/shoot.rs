use zroom_engine::{Player, Handler, HandleResult, Value, Account, Engine, Result, Config};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Default)]
struct EmptyPlayer;

#[derive(Default)]
struct EmptyHandler;

impl Player for EmptyPlayer {
    fn state(&self) -> Value {
        Value::default()
    }
}

impl Handler for EmptyHandler {
    type P = EmptyPlayer;

    fn handle(&mut self, _: Account, _: Vec<Value>, _: Arc<RwLock<HashMap<Account, Self::P>>>) -> Result<HandleResult> {
        Ok(HandleResult::default())
    }
}

#[tokio::main]
async fn main() {
    let config = Config::default();
    let _ = Engine::new(EmptyHandler).run(config).await;
}
