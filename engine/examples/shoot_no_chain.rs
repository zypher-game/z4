use rand_chacha::{rand_core::SeedableRng, ChaChaRng};
use z4_engine::{chain_channel, ChainMessage, Config, Engine, PeerKey, H160};

mod shoot_common;
use shoot_common::*;

const GAME: &str = "0x0000000000000000000000000000000000000000";
const ROOM: u64 = 1;

/// in engine,
/// - run `cargo run --example shoot_no_chain`
#[tokio::main]
async fn main() {
    std::env::set_var("RUST_LOG", "debug");
    //tracing_subscriber::fmt::init();

    // mock 4 players
    let mut prng = ChaChaRng::from_seed([0u8; 32]);
    let server_key = PeerKey::generate(&mut prng);
    let player1 = PeerKey::generate(&mut prng); // for evm-chain
    let player2 = PeerKey::generate(&mut prng); // for evm-chain
    let player3 = PeerKey::generate(&mut prng); // for evm-chain
    let player4 = PeerKey::generate(&mut prng); // for evm-chain

    let sid = server_key.peer_id();
    let id1 = player1.peer_id();
    let id2 = player2.peer_id();
    let id3 = player3.peer_id();
    let id4 = player4.peer_id();
    let opponent1 = vec![id2, id3, id4];
    let opponent2 = vec![id1, id3, id4];
    let opponent3 = vec![id1, id2, id4];
    let opponent4 = vec![id1, id2, id3];

    // running server
    let mut config = Config::default();
    config.ws_port = Some(8000);
    config.secret_key = hex::encode(server_key.to_db_bytes());
    config.groups = vec![ROOM]; // Add default room to it.
    config.games = vec![GAME.to_owned()];
    let game = GAME.parse().unwrap();

    let mut engine = Engine::<ShootHandler>::init(config);
    engine.create_pending(
        ROOM,
        game,
        false,
        H160(id1.0),
        id1,
        [0u8; 32],
        [0u8; 32],
        [0u8; 32],
    );
    engine.join_pending(ROOM, H160(id2.0), id2, [0u8; 32]);
    engine.join_pending(ROOM, H160(id3.0), id3, [0u8; 32]);
    engine.join_pending(ROOM, H160(id4.0), id4, [0u8; 32]);

    let (chain_send, chain_recv) = chain_channel();
    let chain_send1 = chain_send.clone();
    tokio::spawn(engine.run_with_channel(chain_send, chain_recv));
    tokio::spawn(async move {
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
        chain_send1.send(ChainMessage::AcceptRoom(
            ROOM,
            sid,
            "".to_owned(),
            vec![0u8; 32],
        ))
    });

    let _ = tokio::join! {
        mock_player_with_rpc(ROOM, player1, opponent1),
        mock_player_with_rpc(ROOM, player2, opponent2),
        mock_player_with_p2p(ROOM, player3, opponent3, sid),
        mock_player_with_p2p(ROOM, player4, opponent4, sid),
    };
    println!("GAME OVER");
}
