use poker_methods::{POKER_METHOD_ELF, POKER_METHOD_ID};
use risc0_zkvm::{default_prover, ExecutorEnv, Receipt};
use poker_core::{play::{Task,PlayAction,PlayerEnvBuilder,PlayerEnv}, schnorr::KeyPair};
use rand_chacha::{ChaChaRng, rand_core::SeedableRng};

pub fn summation(a: u64) -> (Receipt, u64) {
    let env = ExecutorEnv::builder().write(&a).unwrap().build().unwrap();

    let prover = default_prover();

    let receipt = prover.prove_elf(env, POKER_METHOD_ELF).unwrap();

    let b: u64 = receipt.journal.decode().expect(
        "Journal output should deserialize into the same types (& order) that it was written",
    );

    println!("I know the cumulative sum is {}, and I can prove it!", b);

    (receipt, b)
}

pub fn prove_task(task:&Task) {
    let env = ExecutorEnv::builder()
      .write(&task)
      .unwrap()
      .build()
      .unwrap();
  
      let prover = default_prover();
  
      //    .prove
      let receipt =   prover.prove_elf(env, POKER_METHOD_ELF).unwrap();
  
      let b: u64 = receipt.journal.decode().expect(
          "Journal output should deserialize into the same types (& order) that it was written",
      );
  
      println!("I know the cumulative sum is {}, and I can prove it!", b);
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

    let task = Task {
        room_id: 1,
        game_id: 1,
        num_round: 1,
        players_order: vec![],
        players_env: vec![vec![player]],
        players_hand: HashMap::new(),
    };

      prove_task(&task);

    // let (receipt, _) = summation(10);

    // receipt.verify(POKER_METHOD_ID).expect(
    //     "Code you have proven should successfully verify; did you specify the correct image ID?",
    // );
}
