use std::collections::HashMap;

#[derive(Clone, Copy)]
pub enum Suite {
    Club,
    Diamond,
    Heart,
    Spade,
}

pub const SUITES: [Suite; 4] = [Suite::Club, Suite::Diamond, Suite::Heart, Suite::Spade];

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Value {
    Two,
    Three,
    Four,
    Five,
    Six,
    Seven,
    Eight,
    Nine,
    Ten,
    Jack,
    Queen,
    King,
    Ace,
}

impl Value {
    pub const VALUES: [Value; 13] = [
        Value::Three,
        Value::Four,
        Value::Five,
        Value::Six,
        Value::Seven,
        Value::Eight,
        Value::Nine,
        Value::Ten,
        Value::Jack,
        Value::Queen,
        Value::King,
        Value::Ace,
        Value::Two,
    ];

    pub fn weight(&self) -> u8 {
        match self {
            Value::Three => 3,
            Value::Four => 4,
            Value::Five => 5,
            Value::Six => 6,
            Value::Seven => 7,
            Value::Eight => 8,
            Value::Nine => 9,
            Value::Ten => 10,
            Value::Jack => 11,
            Value::Queen => 12,
            Value::King => 13,
            Value::Ace => 14,
            Value::Two => 15,
        }
    }
}

impl PartialOrd for Value {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.weight().partial_cmp(&other.weight())
    }
}

#[derive(Clone, Copy)]
pub struct ClassicCard {
    value: Value,
    suite: Suite,
}

impl ClassicCard {
    pub fn new(value: Value, suite: Suite) -> Self {
        Self { value, suite }
    }

    pub fn get_value(&self) -> Value {
        self.value
    }

    pub fn weight(&self) -> u8 {
        self.value.weight()
    }
}

impl std::fmt::Debug for ClassicCard {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let suite = match self.suite {
            Suite::Club => "♣",
            Suite::Diamond => "♦",
            Suite::Heart => "♥",
            Suite::Spade => "♠",
        };

        let val = match self.value {
            Value::Two => "2",
            Value::Three => "3",
            Value::Four => "4",
            Value::Five => "5",
            Value::Six => "6",
            Value::Seven => "7",
            Value::Eight => "8",
            Value::Nine => "9",
            Value::Ten => "10",
            Value::Jack => "J",
            Value::Queen => "Q",
            Value::King => "K",
            Value::Ace => "A",
        };

        write!(f, "{}{}", suite, val)
    }
}

lazy_static! {
  pub static ref ENCODE_CARDS_MAPPING: HashMap<zshuffle::Card, ClassicCard> = {
        let map = HashMap::new();
        // TODO
        map
    };
}
