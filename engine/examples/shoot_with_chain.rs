use ethers::prelude::*;
use std::sync::Arc;
use z4_engine::{
    hex_address, Config, Engine, Network, NetworkConfig, PeerKey, RoomId, RoomMarket, SimpleGame,
};

mod shoot_common;
use shoot_common::*;

const GAME: &str = "0xe7f1725E7734CE288F8367e1Bb143E90bb3F0512";

/// in contracts/solidity,
/// - 1. run `npx hardhat node`
/// - 2. run `npm run deploy`
/// in engine,
/// - run `cargo run --example shoot_with_chain`
#[tokio::main]
async fn main() {
    //std::env::set_var("RUST_LOG", "info");
    tracing_subscriber::fmt::init();

    // SAFE: hardhat default sk
    let s_str = "ac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80";
    let p1_str = "59c6995e998f97a5a0044966f0945389dc9e86dae88c7a8412f4603b6b78690d";
    let p2_str = "5de4111afa1a4b94908f83103eb1f1706367c2e68ca870fc3fb9a804cdab365a";
    let p3_str = "7c852118294e51e653712a81e05800f419141751be58f605c371e15141b007a6";
    let p4_str = "47e179ec197488593b187f80a00eb0da91f1b9d0b13f8733639f19c30a34926a";

    // mock 4 players
    let server_key = PeerKey::from_db_bytes(&hex::decode(s_str).unwrap()).unwrap();
    let player1 = PeerKey::from_db_bytes(&hex::decode(p1_str).unwrap()).unwrap();
    let player2 = PeerKey::from_db_bytes(&hex::decode(p2_str).unwrap()).unwrap();
    let player3 = PeerKey::from_db_bytes(&hex::decode(p3_str).unwrap()).unwrap();
    let player4 = PeerKey::from_db_bytes(&hex::decode(p4_str).unwrap()).unwrap();

    let sid = server_key.peer_id();
    let id1 = player1.peer_id();
    let id2 = player2.peer_id();
    let id3 = player3.peer_id();
    let id4 = player4.peer_id();
    let opponent1 = vec![id2, id3, id4];
    let opponent2 = vec![id1, id3, id4];
    let opponent3 = vec![id1, id2, id4];
    let opponent4 = vec![id1, id2, id3];

    let network = Network::Localhost;
    let nc = NetworkConfig::from(network);
    let provider = Provider::<Http>::try_from(&nc.rpc_urls[0]).unwrap();

    let server_signer: LocalWallet = s_str.parse().unwrap();
    let p1_signer: LocalWallet = p1_str.parse().unwrap();
    let p2_signer: LocalWallet = p2_str.parse().unwrap();
    let p3_signer: LocalWallet = p3_str.parse().unwrap();
    let p4_signer: LocalWallet = p4_str.parse().unwrap();

    let server_client = Arc::new(
        SignerMiddleware::new_with_provider_chain(provider.clone(), server_signer)
            .await
            .unwrap(),
    );
    let p1_client = Arc::new(
        SignerMiddleware::new_with_provider_chain(provider.clone(), p1_signer)
            .await
            .unwrap(),
    );
    let p2_client = Arc::new(
        SignerMiddleware::new_with_provider_chain(provider.clone(), p2_signer)
            .await
            .unwrap(),
    );
    let p3_client = Arc::new(
        SignerMiddleware::new_with_provider_chain(provider.clone(), p3_signer)
            .await
            .unwrap(),
    );
    let p4_client = Arc::new(
        SignerMiddleware::new_with_provider_chain(provider.clone(), p4_signer)
            .await
            .unwrap(),
    );
    println!("init account ok");

    // on-chain
    let game = hex_address(GAME).unwrap();

    let mut config = Config::default();
    config.p2p_port = 7364;
    config.ws_port = Some(8000);
    config.secret_key = hex::encode(server_key.to_db_bytes());
    config.chain_network = network.to_str().to_owned();
    config.games = vec![GAME.to_owned()];
    config.auto_stake = true;
    config.url_websocket = "ws://127.0.0.1:8000".to_owned();
    config.chain_start_block = Some(1);
    tokio::spawn(Engine::<ShootHandler>::init(config).run());
    println!("running engine ok");

    let room_id = create_room(game, p1_client).await;
    println!("p1 create room: {} ok", room_id);
    join_room(room_id, game, p2_client).await;
    println!("p2 join room {} ok", room_id);
    join_room(room_id, game, p3_client).await;
    println!("p3 join room {} ok", room_id);
    join_room(room_id, game, p4_client).await;
    println!("p4 join room {} ok", room_id);

    check_room_status(room_id, game, server_client.clone()).await;
    println!("check room status ok");

    println!("waiting room is accepted...");
    tokio::time::sleep(std::time::Duration::from_secs(20)).await;
    println!("maybe accepted, so started, if not work, waiting more time");
    let _ = tokio::join! {
        mock_player_with_rpc(room_id, player1, opponent1),
        mock_player_with_rpc(room_id, player2, opponent2),
        mock_player_with_p2p(room_id, player3, opponent3, sid),
        mock_player_with_p2p(room_id, player4, opponent4, sid),
    };
    println!("GAME OVER, waiting send result to chain");
    tokio::time::sleep(std::time::Duration::from_secs(10)).await;
}

async fn create_room(
    game: Address,
    client: Arc<SignerMiddleware<Provider<Http>, LocalWallet>>,
) -> RoomId {
    let market = RoomMarket::new(game, client.clone());
    let next_room = market.next_room_id().await.unwrap();

    let addr = client.address();
    let game = SimpleGame::new(game, client);
    game.create_room(U256::zero(), false, addr, [0u8; 32], [0u8; 32])
        .send()
        .await
        .unwrap()
        .await
        .unwrap();

    next_room.as_u64()
}

async fn join_room(
    room: RoomId,
    game: Address,
    client: Arc<SignerMiddleware<Provider<Http>, LocalWallet>>,
) {
    let addr = client.address();
    let game = SimpleGame::new(game, client);
    game.join_room(U256::from(room), addr, [0u8; 32])
        .send()
        .await
        .unwrap()
        .await
        .unwrap();
}

async fn check_room_status(
    room: RoomId,
    game: Address,
    client: Arc<SignerMiddleware<Provider<Http>, LocalWallet>>,
) {
    let addr = client.address();
    let market = RoomMarket::new(game, client);
    let result1 = market.sequencers(addr).await.unwrap();
    let result2 = market.rooms(U256::from(room)).await.unwrap();
    println!("Chain sequencer Status: {:?}", result1);
    println!("Chain room      Status: {:?}", result2);
}
