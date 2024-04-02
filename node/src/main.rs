mod handler;
mod risc0;

use z4_engine::{Config, DefaultParams, Engine, Error, HandleResult};

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();

    let network = std::env::var("NETWORK").unwrap();
    let game = std::env::var("GAME").unwrap();
    let secret_key = std::env::var("SECRET_KEY").unwrap();
    let start_block = std::env::var("START_BLOCK")
        .ok()
        .map(|v| v.parse().unwrap());
    let server = std::env::var("PUBLIC_SERVER").unwrap();
    let http_port = std::env::var("HTTP_PORT")
        .unwrap_or("8080".to_owned())
        .parse()
        .unwrap();
    let ws_port = std::env::var("WS_PORT")
        .unwrap_or("8000".to_owned())
        .parse()
        .unwrap();

    let mut config = Config::default();
    config.http_port = http_port;
    config.ws_port = Some(ws_port);
    config.secret_key = secret_key.to_owned();
    config.chain_network = network;
    config.chain_start_block = start_block;
    config.games = vec![game.to_owned()];
    config.auto_stake = true;
    config.http = server;

    Engine::<handler::Z4Handler<risc0::Risc0>>::init(config)
        .run()
        .await
        .expect("Server down !!!");
}

trait Executor: 'static + Send + Sized {
    /// create an executor
    fn create() -> Self;

    /// execute program with code, storage, and params,
    /// result is new storage and z4 HandleResult.
    fn execute(
        &self,
        code: &[u8],
        storage: &[u8],
        params: &DefaultParams,
    ) -> Result<(Vec<u8>, HandleResult<DefaultParams>), Error>;
}
