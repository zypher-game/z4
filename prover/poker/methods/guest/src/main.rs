use risc0_zkvm::guest::env;
risc0_zkvm::guest::entry!(main);


pub fn main() {
    let a: u64 = env::read();

     let mut b = 0;
     for i in 1..a {
         b+=i
     } 

    env::commit(&b);
}