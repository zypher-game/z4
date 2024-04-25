#[proc_macro_attribute]
pub fn z4_method(attr: TokenStream, item: TokenStream) -> TokenStream {
    // TODO



    risc0_zkvm::guest::entry!(main);
}

#[proc_macro_attribute]
pub fn z4_task(attr: TokenStream, item: TokenStream) -> TokenStream {
    //
    item
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Serialize, Deseriaze)]
    struct Shoot {
        players: HashMap<PlayerId, Player>
    }

    #[z4_method]
    impl Shoot {
        #[constructor]
        fn constructor() -> Shoot {
            //
        }

        #[online]
        fn online(shoot: &mut Shoot, player: PlayerId) -> Result<HandleResult> {
            //
        }

        #[offline]
        fn offline(shoot: &mut Shoot, player: PlayerId) -> Result<HandleResult> {
            //
        }

        fn shoot(shoot: &mut Shoot, player_a: PlayerId, player_b: PlayerId) -> Result<HandleResult> {
            //
        }

        fn regain(shoot: &mut Shoot, player: PlayerId) -> Result<HandleResult> {
            //
        }
    }

    #[z4_task]
    fn timer1(shoot: &mut Shoot) -> Result<(u32, HandleResult)> {
        //
    }

    #[z4_task]
    fn timer2(shoot: &mut Shoot) -> Result<(u32, HandleResult)> {
        //
    }
}

