use poker_methods::{POKER_METHOD_ELF, POKER_METHOD_ID};
use risc0_zkvm::{default_prover, ExecutorEnv, Receipt};

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

fn main() {
    let (receipt, _) = summation(10);

    receipt.verify(POKER_METHOD_ID).expect(
        "Code you have proven should successfully verify; did you specify the correct image ID?",
    );
}
