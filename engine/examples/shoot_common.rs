use ark_bn254::Fr;
use ark_ed_on_bn254::EdwardsAffine;
use ark_std::rand::{CryptoRng, RngCore, SeedableRng};
use rand_chacha::ChaChaRng;
use std::collections::HashMap;
use tdn::types::primitives::vec_remove_item;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};
use z4_engine::{
    json,
    request::{message_channel, run_p2p_channel, run_ws_channel, ChannelMessage},
    Error, HandleResult, Handler, Param, Peer, PeerId, PeerKey, PublicKey, Result, RoomId,
    SecretKey, Value,
};
use zplonk::{
    gen_params::{load_lagrange_params, load_srs_params, ProverParams, VerifierParams},
    poly_commit::kzg_poly_commitment::KZGCommitmentSchemeBN254,
    turboplonk::{
        constraint_system::{ConstraintSystem, TurboCS, VarIndex},
        indexer::{indexer_with_lagrange, PlonkProof},
        prover::prover_with_lagrange,
        verifier::verifier,
    },
    utils::transcript::Transcript,
};

#[derive(Default)]
pub struct ShootPlayer {
    // Hit Points or Health Points
    hp: u32,
    bullet: u32,
}

#[derive(Default)]
pub struct ShootHandler {
    accounts: HashMap<PeerId, (PublicKey, ShootPlayer)>,
    operations: Vec<ShootOperation>,
}

#[derive(Default)]
pub struct Params(Vec<Value>);

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
                    } else {
                        return Err(Error::Params);
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

                self.operations.push(ShootOperation {
                    from: a_pk,
                    to: b_pk,
                });

                // check game is over
                let mut game_over = true;
                let lives: Vec<PeerId> = self
                    .accounts
                    .iter()
                    .filter_map(|(p, (_, a))| if a.hp > 0 { Some(*p) } else { None })
                    .collect();
                if lives.len() > 1 {
                    for (_, (_, account)) in self.accounts.iter() {
                        if account.bullet != 0 && account.hp != 0 {
                            game_over = false;
                            break;
                        }
                    }
                }

                if game_over {
                    // zkp
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

                    // TODO results serialize to bytes
                    let proof_bytes = bincode::serialize(&proof).unwrap();
                    result.over(vec![], proof_bytes);
                }

                return Ok(result);
            }
        }

        Err(Error::Params)
    }
}

pub async fn mock_player_with_rpc(
    room_id: RoomId,
    player: PeerKey,
    opponents: Vec<PeerId>,
    _sk: SecretKey,
) {
    println!("Player: {:?} with RPC", player.peer_id());
    tokio::time::sleep(std::time::Duration::from_secs(3)).await;

    // create ws channel with message
    let (in_send, in_recv) = message_channel::<Params>();
    let out_recv = run_ws_channel(&player, room_id, in_recv, "ws://127.0.0.1:8000")
        .await
        .unwrap();

    mock_player(room_id, player, opponents, in_send, out_recv).await
}

pub async fn mock_player_with_p2p(
    room_id: RoomId,
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
    let out_recv = run_p2p_channel(&player, room_id, in_recv, server)
        .await
        .unwrap();

    mock_player(room_id, player, opponents, in_send, out_recv).await
}

async fn mock_player(
    room_id: RoomId,
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
                tokio::time::sleep(std::time::Duration::from_secs(i+1)).await;
                Some(Work::Shoot(opponents[i as usize]))
            } => w,
            w = async {
                out_recv.recv().await.map(Work::Shooted)
            } => w,
        };

        match work {
            Some(Work::Shoot(someone)) => {
                let params = Params(vec![json!(my_id.to_hex()), json!(someone.to_hex())]);
                in_send.send((room_id, "shoot".to_owned(), params)).unwrap();
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

//////// ------- ZK -------- ////

#[derive(Clone)]
pub struct ShootOperation {
    pub from: EdwardsAffine,
    pub to: EdwardsAffine,
}

pub struct ShootResult {
    pub player: EdwardsAffine,
    pub hp: u32,
    pub bullet: u32,
}

pub struct ShootResultVar {
    pub p_x: VarIndex,
    pub p_y: VarIndex,
    pub hp: VarIndex,
    pub bullet: VarIndex,
    pub r_hp: u32,
    pub r_bullet: u32,
}

pub const TOTAL: u32 = 5;
const PLONK_PROOF_TRANSCRIPT: &[u8] = b"Plonk shoot Proof";
const N_TRANSCRIPT: &[u8] = b"Number of players";

pub(crate) fn build_cs(
    players: &[EdwardsAffine],
    inputs: &[ShootOperation],
) -> (TurboCS<Fr>, Vec<ShootResult>) {
    let mut cs = TurboCS::new();

    let mut indexs: HashMap<EdwardsAffine, usize> = HashMap::new();
    let mut results: Vec<ShootResultVar> = vec![];
    let hp = cs.new_variable(Fr::from(TOTAL));
    let bullet = cs.new_variable(Fr::from(TOTAL));

    for p in players {
        let p_x = cs.new_variable(p.x);
        let p_y = cs.new_variable(p.y);
        let r = ShootResultVar {
            p_x,
            p_y,
            hp,
            bullet,
            r_hp: TOTAL,
            r_bullet: TOTAL,
        };
        let index = results.len();
        indexs.insert(*p, index);
        results.push(r);
    }

    for input in inputs {
        // TODO sub cs for all players
        let index_from = *indexs.get(&input.from).unwrap();
        let index_to = *indexs.get(&input.to).unwrap();

        let bullet_var = results[index_from].bullet;
        let hp_var = results[index_to].hp;

        let new_bullet = cs.sub(bullet_var, cs.one_var());
        let new_hp = cs.sub(hp_var, cs.one_var());

        results[index_from].bullet = new_bullet;
        results[index_to].hp = new_hp;

        results[index_from].r_bullet -= 1;
        results[index_to].r_hp -= 1;
    }

    let mut publics = vec![];
    for r in results {
        cs.prepare_pi_variable(r.p_x);
        cs.prepare_pi_variable(r.p_y);
        cs.prepare_pi_variable(r.bullet);
        cs.prepare_pi_variable(r.hp);

        publics.push(ShootResult {
            player: EdwardsAffine::new(cs.witness[r.p_x], cs.witness[r.p_y]),
            bullet: r.r_bullet,
            hp: r.r_hp,
        });
    }

    cs.pad();

    (cs, publics)
}

pub fn gen_prover_params(
    players: &[EdwardsAffine],
    operations: &[ShootOperation],
) -> Result<ProverParams> {
    let (cs, _) = build_cs(players, operations);

    let cs_size = cs.size();
    let pcs = load_srs_params(cs_size)?;
    let lagrange_pcs = load_lagrange_params(cs_size);

    let prover_params = indexer_with_lagrange(&cs, &pcs, lagrange_pcs.as_ref(), None).unwrap();

    Ok(ProverParams {
        pcs,
        lagrange_pcs,
        cs,
        prover_params,
    })
}

pub fn get_verifier_params(prover_params: ProverParams) -> VerifierParams {
    VerifierParams::from(prover_params)
}

pub fn prove_shoot<R: CryptoRng + RngCore>(
    prng: &mut R,
    prover_params: &ProverParams,
    players: &[EdwardsAffine],
    inputs: &[ShootOperation],
) -> Result<(PlonkProof<KZGCommitmentSchemeBN254>, Vec<ShootResult>)> {
    let n = players.len();

    let (mut cs, publics) = build_cs(players, inputs);
    let witness = cs.get_and_clear_witness();

    let mut transcript = Transcript::new(PLONK_PROOF_TRANSCRIPT);
    transcript.append_u64(N_TRANSCRIPT, n as u64);

    let proof = prover_with_lagrange(
        prng,
        &mut transcript,
        &prover_params.pcs,
        prover_params.lagrange_pcs.as_ref(),
        &cs,
        &prover_params.prover_params,
        &witness,
    )?;

    Ok((proof, publics))
}

pub fn verify_shoot(
    verifier_params: &VerifierParams,
    publics: &[ShootResult],
    proof: &PlonkProof<KZGCommitmentSchemeBN254>,
) -> Result<()> {
    let n = publics.len();

    let mut transcript = Transcript::new(PLONK_PROOF_TRANSCRIPT);
    transcript.append_u64(N_TRANSCRIPT, n as u64);

    let mut online_inputs = vec![];

    for p in publics.iter() {
        online_inputs.push(p.player.x);
        online_inputs.push(p.player.y);
        online_inputs.push(Fr::from(p.bullet));
        online_inputs.push(Fr::from(p.hp));
    }

    Ok(verifier(
        &mut transcript,
        &verifier_params.shrunk_vk,
        &verifier_params.shrunk_cs,
        &verifier_params.verifier_params,
        &online_inputs,
        proof,
    )?)
}
