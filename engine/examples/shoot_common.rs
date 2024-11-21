use ark_bn254::Fr;
use ark_ff::PrimeField;
use ark_std::rand::{CryptoRng, RngCore, SeedableRng};
use rand_chacha::ChaChaRng;
use std::collections::HashMap;
use tdn::types::primitives::vec_remove_item;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};
use uzkge::{
    gen_params::{load_lagrange_params, load_srs_params, ProverParams, VerifierParams},
    plonk::{
        constraint_system::{ConstraintSystem, TurboCS, VarIndex},
        indexer::{indexer_with_lagrange, PlonkProof},
        prover::prover_with_lagrange,
        verifier::verifier,
    },
    poly_commit::kzg_poly_commitment::KZGCommitmentSchemeBN254,
    utils::transcript::Transcript,
};
use z4_engine::{
    json,
    request::{message_channel, run_p2p_channel, run_ws_channel, ChannelMessage},
    simple_game_result, Address, Error, HandleResult, Handler, MethodValues, Peer, PeerId, PeerKey,
    Player, Result, RoomId, Tasks,
};

#[allow(dead_code)]
fn main() {
    //
}

#[derive(Default)]
pub struct ShootPlayer {
    // Hit Points or Health Points
    hp: u32,
    bullet: u32,
}

#[derive(Default)]
pub struct ShootHandler {
    accounts: HashMap<PeerId, (Address, ShootPlayer)>,
    operations: Vec<ShootOperation>,
}

#[async_trait::async_trait]
impl Handler for ShootHandler {
    type Param = MethodValues;

    async fn chain_accept(_peers: &[Player]) -> Vec<u8> {
        vec![]
    }

    async fn chain_create(
        peers: &[Player],
        _params: Vec<u8>,
        _rid: RoomId,
        _seed: [u8; 32],
    ) -> Option<(Self, Tasks<Self>)> {
        let accounts = peers
            .iter()
            .map(|p| {
                (
                    p.peer,
                    (
                        p.account,
                        ShootPlayer {
                            hp: TOTAL,
                            bullet: TOTAL,
                        },
                    ),
                )
            })
            .collect();

        Some((
            Self {
                accounts,
                operations: vec![],
            },
            Default::default(),
        ))
    }

    async fn handle(
        &mut self,
        player: PeerId,
        param: MethodValues,
    ) -> Result<HandleResult<Self::Param>> {
        let MethodValues { method, mut params } = param;

        // only support shoot method
        let method = method.as_str();
        if method == "shoot" {
            if let Some(value) = params.pop() {
                let target = PeerId::from_hex(value.as_str().unwrap()).unwrap();

                let mut a_bullet = 0;
                let mut a_account = Address::default();
                if let Some((aid, account)) = self.accounts.get_mut(&player) {
                    if account.bullet > 0 {
                        account.bullet -= 1;
                        a_bullet = account.bullet;
                        a_account = *aid;
                    } else {
                        return Err(Error::Params);
                    }
                }

                let mut b_hp = 0;
                let mut b_account = Address::default();
                if let Some((aid, account)) = self.accounts.get_mut(&target) {
                    if account.hp > 0 {
                        account.hp -= 1;
                        b_hp = account.hp;
                        b_account = *aid;
                    } else {
                        return Err(Error::Params);
                    }
                }

                println!(
                    "SERVER: {:?}-{} shooting {:?}-{}",
                    player, a_bullet, target, b_hp
                );

                let mut result = HandleResult::default();
                result.add_all(MethodValues {
                    method: "shoot".to_owned(),
                    params: vec![player.to_hex().into(), value, a_bullet.into(), b_hp.into()],
                });

                self.operations.push(ShootOperation {
                    from: a_account,
                    to: b_account,
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
                    println!("Game over, will proving");
                    result.over();
                }

                return Ok(result);
            }
        }

        Err(Error::Params)
    }

    async fn prove(&mut self) -> Result<(Vec<u8>, Vec<u8>)> {
        // zkp
        let players: Vec<Address> = self
            .accounts
            .iter()
            .map(|(_, (account, _))| *account)
            .collect();

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
        let rank = simple_game_result(&players);
        Ok((rank, proof_bytes))
    }
}

pub async fn mock_player_with_rpc(room_id: RoomId, player: PeerKey, opponents: Vec<PeerId>) {
    println!("Player: {:?} with RPC", player.peer_id());
    tokio::time::sleep(std::time::Duration::from_secs(3)).await;

    // create ws channel with message
    let (in_send, in_recv) = message_channel::<MethodValues>();
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
) {
    println!("Player: {:?} with P2P", player.peer_id());
    tokio::time::sleep(std::time::Duration::from_secs(3)).await;

    // create p2p channel with message
    let (in_send, in_recv) = message_channel::<MethodValues>();
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
    in_send: UnboundedSender<ChannelMessage<MethodValues>>,
    mut out_recv: UnboundedReceiver<ChannelMessage<MethodValues>>,
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
        Shooted(ChannelMessage<MethodValues>),
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
                let params = MethodValues {
                    method: "shoot".to_owned(),
                    params: vec![json!(my_id.to_hex()), json!(someone.to_hex())],
                };
                in_send.send((room_id, params)).unwrap();
            }
            Some(Work::Shooted((_room, params))) => match params.method.as_str() {
                "shoot" => {
                    let params = params.params;
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
    pub from: Address,
    pub to: Address,
}

pub struct ShootResult {
    pub player: Fr,
    pub hp: u32,
    pub bullet: u32,
}

pub struct ShootResultVar {
    pub p: VarIndex,
    pub hp: VarIndex,
    pub bullet: VarIndex,
    pub r_hp: u32,
    pub r_bullet: u32,
}

pub const TOTAL: u32 = 5;
const PLONK_PROOF_TRANSCRIPT: &[u8] = b"Plonk shoot Proof";
const N_TRANSCRIPT: &[u8] = b"Number of players";

pub(crate) fn build_cs(
    players: &[Address],
    inputs: &[ShootOperation],
) -> (TurboCS<Fr>, Vec<ShootResult>) {
    let mut cs = TurboCS::new();

    let mut indexs: HashMap<Address, usize> = HashMap::new();
    let mut results: Vec<ShootResultVar> = vec![];
    let hp = cs.new_variable(Fr::from(TOTAL));
    let bullet = cs.new_variable(Fr::from(TOTAL));

    for player in players {
        let mut bytes = [0u8; 32];
        bytes[1..21].copy_from_slice(player.as_bytes());
        let p = cs.new_variable(Fr::from_be_bytes_mod_order(&bytes));
        let r = ShootResultVar {
            p,
            hp,
            bullet,
            r_hp: TOTAL,
            r_bullet: TOTAL,
        };
        let index = results.len();
        indexs.insert(*player, index);
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
        cs.prepare_pi_variable(r.p);
        cs.prepare_pi_variable(r.bullet);
        cs.prepare_pi_variable(r.hp);

        publics.push(ShootResult {
            player: cs.witness[r.p],
            bullet: r.r_bullet,
            hp: r.r_hp,
        });
    }

    cs.pad();

    (cs, publics)
}

pub fn gen_prover_params(
    players: &[Address],
    operations: &[ShootOperation],
) -> Result<ProverParams> {
    let (cs, _) = build_cs(players, operations);

    let cs_size = cs.size();
    let pcs = load_srs_params(cs_size).map_err(|err| Error::Zk(err.to_string()))?;
    let lagrange_pcs = load_lagrange_params(cs_size);

    let prover_params =
        indexer_with_lagrange(&cs, &pcs, lagrange_pcs.as_ref(), None, None).unwrap();

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
    players: &[Address],
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
    )
    .map_err(|err| Error::Zk(err.to_string()))?;

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
        online_inputs.push(p.player);
        online_inputs.push(Fr::from(p.bullet));
        online_inputs.push(Fr::from(p.hp));
    }

    verifier(
        &mut transcript,
        &verifier_params.shrunk_vk,
        &verifier_params.shrunk_cs,
        &verifier_params.verifier_params,
        &online_inputs,
        proof,
    )
    .map_err(|err| Error::Zk(err.to_string()))
}
