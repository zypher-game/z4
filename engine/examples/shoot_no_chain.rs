use rand_chacha::{rand_core::SeedableRng, ChaChaRng};
use zroom_engine::{generate_keypair, Config, Engine, PeerKey};

mod shoot_common;
use shoot_common::*;

const ROOM: u64 = 1;

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
    let (sk1, pk1) = generate_keypair(&mut prng); // for player
    let (sk2, pk2) = generate_keypair(&mut prng); // for player
    let (sk3, pk3) = generate_keypair(&mut prng); // for player
    let (sk4, pk4) = generate_keypair(&mut prng); // for player
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

    let mut engine = Engine::<ShootHandler>::init(config);
    engine.add_pending(ROOM, vec![id1, id2, id3, id4], vec![pk1, pk2, pk3, pk4]);
    engine.start_room(ROOM).await;

    tokio::spawn(engine.run());
    let _ = tokio::join! {
        mock_player_with_rpc(ROOM, player1, opponent1, sk1),
        mock_player_with_rpc(ROOM, player2, opponent2, sk2),
        mock_player_with_p2p(ROOM, player3, opponent3, sid, sk3),
        mock_player_with_p2p(ROOM, player4, opponent4, sid, sk4),
    };
    println!("GAME OVER");
}
