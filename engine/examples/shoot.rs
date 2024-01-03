use rand_chacha::{
    rand_core::{RngCore, SeedableRng},
    ChaChaRng,
};
use std::collections::HashMap;
use tdn::types::primitives::vec_remove_item;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};
use zroom_engine::{
    json,
    request::{message_channel, run_p2p_channel, run_ws_channel, ChannelMessage},
    Config, Engine, Error, HandleResult, Handler, PeerId, PeerKey, Result, Value,
};

const TOTAL: u32 = 5;
const ROOM: u64 = 1;

#[derive(Default)]
struct ShootPlayer {
    // Hit Points or Health Points
    hp: u32,
    bullet: u32,
}

#[derive(Default)]
struct ShootHandler {
    accounts: HashMap<PeerId, ShootPlayer>,
}

#[async_trait::async_trait]
impl Handler for ShootHandler {
    async fn create(peers: &[PeerId]) -> Self {
        let accounts = peers
            .iter()
            .map(|id| {
                (
                    *id,
                    ShootPlayer {
                        hp: TOTAL,
                        bullet: TOTAL,
                    },
                )
            })
            .collect();

        Self { accounts }
    }

    async fn handle(
        &mut self,
        player: PeerId,
        method: &str,
        mut params: Vec<Value>,
    ) -> Result<HandleResult> {
        // only support shoot method
        if method == "shoot" {
            if let Some(value) = params.pop() {
                let target = PeerId::from_hex(value.as_str().unwrap()).unwrap();

                let mut a_bullet = 0;
                if let Some(account) = self.accounts.get_mut(&player) {
                    if account.bullet > 0 {
                        account.bullet -= 1;
                        a_bullet = account.bullet;
                    } else {
                        return Err(Error::Params);
                    }
                }

                let mut b_hp = 0;
                if let Some(account) = self.accounts.get_mut(&target) {
                    if account.hp > 0 {
                        account.hp -= 1;
                        b_hp = account.hp;
                    }
                }

                println!(
                    "SERVER: {:?}-{} shooting {:?}-{}",
                    player, a_bullet, target, b_hp
                );

                let mut result = HandleResult::default();
                result.add_all(
                    "shoot",
                    vec![player.to_hex().into(), value, a_bullet.into(), b_hp.into()],
                );
                return Ok(result);
            }
        }

        Err(Error::Params)
    }
}

#[tokio::main]
async fn main() {
    // mock 4 players
    let mut prng = ChaChaRng::from_seed([0u8; 32]);
    let server_key = PeerKey::generate(&mut prng);
    let player1 = PeerKey::generate(&mut prng);
    let player2 = PeerKey::generate(&mut prng);
    let player3 = PeerKey::generate(&mut prng);
    let player4 = PeerKey::generate(&mut prng);
    let id1 = player1.peer_id();
    let id2 = player2.peer_id();
    let id3 = player3.peer_id();
    let id4 = player4.peer_id();
    let opponent1 = vec![id2, id3, id4];
    let opponent2 = vec![id1, id3, id4];
    let opponent3 = vec![id1, id2, id4];
    let opponent4 = vec![id1, id2, id3];
    tokio::spawn(mock_player_with_rpc(player1, opponent1));
    tokio::spawn(mock_player_with_rpc(player2, opponent2));
    tokio::spawn(mock_player_with_rpc(player3, opponent3));
    tokio::spawn(mock_player_with_rpc(player4, opponent4));

    // running server
    let mut config = Config::default();
    config.ws_port = Some(8000);
    config.secret_key = hex::encode(server_key.to_db_bytes());
    let mut accounts = HashMap::new();
    for id in vec![id1, id2, id3, id4] {
        accounts.insert(
            id,
            ShootPlayer {
                hp: TOTAL,
                bullet: TOTAL,
            },
        );
    }

    let mut engine = Engine::<ShootHandler>::init(config);
    engine.add_pending(ROOM);
    engine.add_peer(ROOM, id1);
    engine.add_peer(ROOM, id2);
    engine.add_peer(ROOM, id3);
    engine.add_peer(ROOM, id4);
    engine.start_room(ROOM).await;
    let _ = engine.run().await;
}

async fn mock_player_with_rpc(player: PeerKey, opponents: Vec<PeerId>) {
    println!("Player: {:?} with RPC", player.peer_id());
    tokio::time::sleep(std::time::Duration::from_secs(3)).await;

    // create ws channel with message
    let (in_send, in_recv) = message_channel();
    let out_recv = run_ws_channel(&player, ROOM, in_recv, "ws://127.0.0.1:8000")
        .await
        .unwrap();

    mock_player(player, opponents, in_send, out_recv).await
}

async fn _mock_player_with_p2p(player: PeerKey, opponents: Vec<PeerId>) {
    println!("Player: {:?} with P2P", player.peer_id());
    tokio::time::sleep(std::time::Duration::from_secs(3)).await;

    // create p2p channel with message
    let (in_send, in_recv) = message_channel();
    let out_recv = run_p2p_channel(&player, ROOM, in_recv).await.unwrap();

    mock_player(player, opponents, in_send, out_recv).await
}

async fn mock_player(
    player: PeerKey,
    mut opponents: Vec<PeerId>,
    in_send: UnboundedSender<ChannelMessage>,
    mut out_recv: UnboundedReceiver<ChannelMessage>,
) {
    let my_id = player.peer_id();
    let mut seed = [0u8; 32];
    seed[0..20].copy_from_slice(&my_id.to_bytes());
    let mut prng = ChaChaRng::from_seed(seed);

    let mut my_state = ShootPlayer {
        hp: TOTAL,
        bullet: TOTAL,
    };
    let mut others: HashMap<PeerId, ShootPlayer> = HashMap::new();

    enum Work {
        Shoot(PeerId),
        Shooted(ChannelMessage),
    }

    loop {
        let work = tokio::select! {
            w = async {
                let i = prng.next_u64() % opponents.len() as u64;
                tokio::time::sleep(std::time::Duration::from_secs(i + 1)).await;
                Some(Work::Shoot(opponents[i as usize]))
            } => w,
            w = async {
                out_recv.recv().await.map(Work::Shooted)
            } => w,
        };

        match work {
            Some(Work::Shoot(someone)) => {
                let params = vec![json!(my_id.to_hex()), json!(someone.to_hex())];
                in_send.send((ROOM, "shoot".to_owned(), params)).unwrap();
            }
            Some(Work::Shooted((_room, method, params))) => match method.as_str() {
                "shoot" => {
                    let a = PeerId::from_hex(params[0].as_str().unwrap()).unwrap();
                    let b = PeerId::from_hex(params[1].as_str().unwrap()).unwrap();
                    let a_bullet = params[2].as_i64().unwrap() as u32;
                    let b_hp = params[3].as_i64().unwrap() as u32;

                    if a == my_id {
                        my_state.bullet = a_bullet;
                    }

                    if b == my_id {
                        my_state.hp = b_hp;
                    }

                    others
                        .entry(a)
                        .and_modify(|info| info.bullet = a_bullet)
                        .or_insert(ShootPlayer {
                            hp: TOTAL,
                            bullet: a_bullet,
                        });
                    others
                        .entry(b)
                        .and_modify(|info| info.hp = b_hp)
                        .or_insert(ShootPlayer {
                            hp: b_hp,
                            bullet: TOTAL,
                        });

                    if others.get(&b).unwrap().hp == 0 {
                        vec_remove_item(&mut opponents, &b);
                    }

                    if opponents.is_empty() || my_state.bullet == 0 || my_state.hp == 0 {
                        break;
                    }
                }
                "over" => {
                    break;
                }
                _ => {}
            },
            None => break,
        }
    }

    println!(
        "PLAYER: {:?} game over!, HP: {}, BULLET: {}",
        my_id, my_state.hp, my_state.bullet
    );
}
