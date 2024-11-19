use ethabi::{encode, Address, Token};

/// Helper for generate simple game result, for ranking
pub fn simple_game_result(ranks: &[Address]) -> Vec<u8> {
    encode(&[Token::Array(
        ranks.iter().map(|v| Token::Address(*v)).collect(),
    )])
}
