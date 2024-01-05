use ark_serialize::{CanonicalSerialize, Compress};
use rand_chacha::{
    rand_core::{RngCore, SeedableRng},
    ChaChaRng,
};
use std::collections::HashMap;
use tdn::types::primitives::vec_remove_item;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};
use zroom_engine::{
    generate_keypair, json,
    request::{message_channel, run_p2p_channel, run_ws_channel, ChannelMessage},
    Config, Engine, Error, HandleResult, Handler, Param, Peer, PeerId, PeerKey, PublicKey, Result,
    SecretKey, Value,
};

mod shoot_zk;
use shoot_zk::*;

const ROOM: u64 = 1;

#[derive(Default)]
struct ShootPlayer {
    // Hit Points or Health Points
    hp: u32,
    bullet: u32,
}

#[derive(Default)]
struct ShootHandler {
    accounts: HashMap<PeerId, (PublicKey, ShootPlayer)>,
    operations: Vec<ShootOperation>,
}

#[derive(Default)]
struct Params(Vec<Value>);

impl Param for Params {
    fn to_value(self) -> Value {
        Value::Array(self.0)
    }

    fn from_value(value: Value) -> Result<Self> {
        match value {
            Value::Array(p) => Ok(Params(p)),
            o => Ok(Params(vec![o])),
        }
    }

    fn to_bytes(&self) -> Vec<u8> {
        serde_json::to_vec(&self.0).unwrap_or(vec![])
    }

    fn from_bytes(bytes: &[u8]) -> Result<Self> {
        let v: Value = serde_json::from_slice(bytes)?;
        Self::from_value(v)
    }
}

#[async_trait::async_trait]
impl Handler for ShootHandler {
    type Param = Params;

    async fn create(peers: &[(PeerId, PublicKey)]) -> Self {
        let accounts = peers
            .iter()
            .map(|(id, pk)| {
                (
                    *id,
                    (
                        *pk,
                        ShootPlayer {
                            hp: TOTAL,
                            bullet: TOTAL,
                        },
                    ),
                )
            })
            .collect();

        Self {
            accounts,
            operations: vec![],
        }
    }

    async fn handle(
        &mut self,
        player: PeerId,
        method: &str,
        params: Params,
    ) -> Result<HandleResult<Self::Param>> {
        let mut params = params.0;

        // only support shoot method
        if method == "shoot" {
            if let Some(value) = params.pop() {
                let target = PeerId::from_hex(value.as_str().unwrap()).unwrap();

                let mut a_bullet = 0;
                let mut a_pk = PublicKey::default();
                if let Some((pk, account)) = self.accounts.get_mut(&player) {
                    if account.bullet > 0 {
                        account.bullet -= 1;
                        a_bullet = account.bullet;
                        a_pk = *pk;
                    } else {
                        return Err(Error::Params);
                    }
                }

                let mut b_hp = 0;
                let mut b_pk = PublicKey::default();
                if let Some((pk, account)) = self.accounts.get_mut(&target) {
                    if account.hp > 0 {
                        account.hp -= 1;
                        b_hp = account.hp;
                        b_pk = *pk;
                    }
                }

                println!(
                    "SERVER: {:?}-{} shooting {:?}-{}",
                    player, a_bullet, target, b_hp
                );

                let mut result = HandleResult::default();
                result.add_all(
                    "shoot",
                    Params(vec![
                        player.to_hex().into(),
                        value,
                        a_bullet.into(),
                        b_hp.into(),
                    ]),
                );

                // zkp
                self.operations.push(ShootOperation {
                    from: a_pk,
                    to: b_pk,
                });

                let players: Vec<PublicKey> =
                    self.accounts.iter().map(|(_, (pk, _))| *pk).collect();

                let mut prng = ChaChaRng::from_seed([0u8; 32]);
                let prover_params = gen_prover_params(&players, &self.operations).unwrap();
                println!("SERVER: zk key ok, op: {}", self.operations.len());

                let (proof, results) =
                    prove_shoot(&mut prng, &prover_params, &players, &self.operations).unwrap();
                println!("SERVER: zk prove ok, op: {}", self.operations.len());
                let verifier_params = get_verifier_params(prover_params);
                verify_shoot(&verifier_params, &results, &proof).unwrap();
                println!("SERVER: zk verify ok, op: {}", self.operations.len());

                return Ok(result);
            }
        }

        Err(Error::Params)
    }
}

#[tokio::main]
async fn main() {
    std::env::set_var("RUST_LOG", "warn");
    tracing_subscriber::fmt::init();

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
    let mut pk1_bytes = vec![];
    let mut pk2_bytes = vec![];
    let mut pk3_bytes = vec![];
    let mut pk4_bytes = vec![];
    let _ = pk1.serialize_with_mode(&mut pk1_bytes, Compress::Yes);
    let _ = pk2.serialize_with_mode(&mut pk2_bytes, Compress::Yes);
    let _ = pk3.serialize_with_mode(&mut pk3_bytes, Compress::Yes);
    let _ = pk4.serialize_with_mode(&mut pk4_bytes, Compress::Yes);
    let sid = server_key.peer_id();
    let id1 = player1.peer_id();
    let id2 = player2.peer_id();
    let id3 = player3.peer_id();
    let id4 = player4.peer_id();
    let opponent1 = vec![id2, id3, id4];
    let opponent2 = vec![id1, id3, id4];
    let opponent3 = vec![id1, id2, id4];
    let opponent4 = vec![id1, id2, id3];
    tokio::spawn(mock_player_with_rpc(player1, opponent1, sk1));
    tokio::spawn(mock_player_with_rpc(player2, opponent2, sk2));
    tokio::spawn(mock_player_with_p2p(player3, opponent3, sid, sk3));
    tokio::spawn(mock_player_with_p2p(player4, opponent4, sid, sk4));

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
    engine.add_peer(ROOM, (id1, pk1));
    engine.add_peer(ROOM, (id2, pk2));
    engine.add_peer(ROOM, (id3, pk3));
    engine.add_peer(ROOM, (id4, pk4));
    engine.start_room(ROOM).await;
    let _ = engine.run().await;
}

async fn mock_player_with_rpc(player: PeerKey, opponents: Vec<PeerId>, _sk: SecretKey) {
    println!("Player: {:?} with RPC", player.peer_id());
    tokio::time::sleep(std::time::Duration::from_secs(3)).await;

    // create ws channel with message
    let (in_send, in_recv) = message_channel::<Params>();
    let out_recv = run_ws_channel(&player, ROOM, in_recv, "ws://127.0.0.1:8000")
        .await
        .unwrap();

    mock_player(player, opponents, in_send, out_recv).await
}

async fn mock_player_with_p2p(
    player: PeerKey,
    opponents: Vec<PeerId>,
    sid: PeerId,
    _sk: SecretKey,
) {
    println!("Player: {:?} with P2P", player.peer_id());
    tokio::time::sleep(std::time::Duration::from_secs(3)).await;

    // create p2p channel with message
    let (in_send, in_recv) = message_channel::<Params>();
    let mut server = Peer::peer(sid);
    server.socket = "127.0.0.1:7364".parse().unwrap();
    let out_recv = run_p2p_channel(&player, ROOM, in_recv, server)
        .await
        .unwrap();

    mock_player(player, opponents, in_send, out_recv).await
}

async fn mock_player(
    player: PeerKey,
    mut opponents: Vec<PeerId>,
    in_send: UnboundedSender<ChannelMessage<Params>>,
    mut out_recv: UnboundedReceiver<ChannelMessage<Params>>,
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
        Shooted(ChannelMessage<Params>),
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
                let params = Params(vec![json!(my_id.to_hex()), json!(someone.to_hex())]);
                in_send.send((ROOM, "shoot".to_owned(), params)).unwrap();
            }
            Some(Work::Shooted((_room, method, params))) => match method.as_str() {
                "shoot" => {
                    let params = params.0;
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
