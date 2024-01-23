use risc0_zkvm::guest::env;
risc0_zkvm::guest::entry!(main);


pub fn main() {
    let task: Task = env::read();

    println!("{}",task);

     let mut b = 0;
     for i in 1..10 {
         b+=i
     } 

    env::commit(&b);
}