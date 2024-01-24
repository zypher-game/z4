use poker_core::{
    cards::{ClassicCard, Suite::Club, Value::Ace},
    combination::ClassicCardCombination,
    play::PlayAction,
    schnorr::PublicKey,
    task::{Task, TaskCommit},
};
use risc0_zkvm::guest::env;

pub fn main() {
    let task: String = env::read();

    let task: Task = serde_json::from_str(&task).unwrap();

    let input_hand = task.players_hand.clone();

    let Task {
        room_id,
        game_id,
        num_round,
        players_order,
        players_env,
        mut players_hand,
    } = task;

    assert_eq!(num_round, players_env.len());

    let n = players_order.len();
    let mut first_player_id = 0;
    let mut round_max_cards = ClassicCardCombination::Single(ClassicCard::new(Ace, Club));
    let mut winner = PublicKey::default();

    // check winner

    for (round_id, round_env) in players_env.iter().enumerate() {
        let mut round_first_player_id = 0;

        for (i, player) in round_env.iter().enumerate() {
            let id = (first_player_id + i) % n;
            let pk = &players_order[id];
            assert!(player
                .verify_sign_with_params(&pk, room_id, game_id, round_id)
                .is_ok());

            if i == 0 {
                assert_eq!(player.action, PlayAction::PLAY);
                let reveals = player.verify_and_get_reveals().unwrap();
                let encoding = player
                    .play_cards
                    .as_ref()
                    .and_then(|x| Some(x.morph_to_encoding(&reveals)))
                    .unwrap();
                let classic = encoding.morph_to_classic().unwrap();
                assert!(classic.sanity_check());

                let hand = players_hand.get_mut(pk).unwrap();
                assert!(reveals.iter().all(|x| hand.contains(x)));
                hand.retain(|x| !reveals.contains(x));

                if hand.len() == 0 && winner == PublicKey::default() {
                    winner = pk.clone()
                }

                round_max_cards = classic;
                round_first_player_id = id;
            } else {
                if let PlayAction::PLAY = player.action {
                    let reveals = player.verify_and_get_reveals().unwrap();
                    let encoding = player
                        .play_cards
                        .as_ref()
                        .and_then(|x| Some(x.morph_to_encoding(&reveals)))
                        .unwrap();
                    let classic = encoding.morph_to_classic().unwrap();
                    assert!(classic.sanity_check());
                    assert!(classic > round_max_cards);

                    let hand = players_hand.get_mut(pk).unwrap();
                    assert!(reveals.iter().all(|x| hand.contains(x)));
                    hand.retain(|x| !reveals.contains(x));

                    if hand.len() == 0 && winner == PublicKey::default() {
                        winner = pk.clone()
                    }

                    round_max_cards = classic;
                    round_first_player_id = id;
                }
            }
        }

        first_player_id = round_first_player_id;
    }

    env::commit(&TaskCommit {
        room_id,
        game_id,
        players_order,
        players_hand: input_hand,
        winner,
    });
}
