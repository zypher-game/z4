use std::collections::HashMap;

use poker_core::{
    play::{PlayAction, PlayerEnv, PlayerEnvBuilder},
    schnorr::{KeyPair, PublicKey},
    task::Task,
};
use poker_methods::{POKER_METHOD_ELF, POKER_METHOD_ID};
use rand_chacha::{rand_core::SeedableRng, ChaChaRng};
use risc0_zkvm::{default_prover, ExecutorEnv, Receipt};

pub fn prove_task(task: &Task) {
    // todo
    let serialized = serde_json::to_string(&task).unwrap();

    let env = ExecutorEnv::builder()
        .write(&serialized)
        .unwrap()
        .build()
        .unwrap();

    let prover = default_prover();

    //    .prove
    let receipt = prover.prove_elf(env, POKER_METHOD_ELF).unwrap();

    println!("I can prove it!");
}

fn main() {
    let mut prng = ChaChaRng::from_seed([0u8; 32]);
    let key_pair = KeyPair::sample(&mut prng);
    let player = PlayerEnvBuilder::new()
        .room_id(1)
        .game_id(1)
        .round_id(1)
        .action(PlayAction::PAAS)
        .build_and_sign(&key_pair, &mut prng)
        .unwrap();

    //let pk = PublicKey::rand(&mut prng);

    let task = Task {
        room_id: 1,
        game_id: 1,
        num_round: 0,
        players_order: vec![],
        players_env: vec![],
        players_hand: HashMap::new(),
    };

    prove_task(&task);
}
