use crate::{
    cards::{ClassicCard, Value, ENCODING_CARDS_MAPPING},
    combination::Combination::*,
    errors::{PokerError, Result},
};
use ark_ec::{AffineRepr, CurveGroup};

#[derive(Debug, Clone)]
/// Different card play combinations
pub enum Combination<T> {
    // Single card
    Single(T),

    // Pair of cards
    Pair(T, T),

    // Three cards of the same rank
    ThreeOfAKind(T, T, T),

    // Three cards of the same rank with one single card
    ThreeWithOne(T, T, T, T),

    // Three cards of the same rank with one pair
    ThreeWithPair(T, T, T, T, T),

    // Five or more consecutive single cards
    Straight(Vec<T>),

    // Three or more consecutive pairs
    DoubleStraight(Vec<(T, T)>),

    // Two or more consecutive three of a kind
    TripleStraight(Vec<(T, T, T)>),

    // Triple straight with one single card
    TripleStraightWithOne(Vec<(T, T, T, T)>),

    // Triple straight with one pair
    TripleStraightWithPair(Vec<(T, T, T, T, T)>),

    // Four cards of the same rank with two single cards
    FourWithTwoSingle(T, T, T, T, T, T),

    // Four cards of the same rank with two pairs
    FourWithTwoPairs(T, T, T, T, T, T, T, T),

    // Four cards of the same rank
    Bomb(T, T, T, T),

    // Both Jokers in a standard deck
    Rocket(T, T),
    // TODO Add more combinations //
}

impl<T> Combination<T> {
    pub fn weight(&self) -> u8 {
        match self {
            Single(_) => 1,
            Pair(_, _) => 1,
            ThreeOfAKind(_, _, _) => 1,
            ThreeWithOne(_, _, _, _) => 1,
            ThreeWithPair(_, _, _, _, _) => 1,
            Straight(_) => 1,
            DoubleStraight(_) => 1,
            TripleStraight(_) => 1,
            TripleStraightWithOne(_) => 1,
            TripleStraightWithPair(_) => 1,
            FourWithTwoSingle(_, _, _, _, _, _) => 1,
            FourWithTwoPairs(_, _, _, _, _, _, _, _) => 1,
            Bomb(_, _, _, _) => 2,
            Rocket(_, _) => 3,
        }
    }

    pub fn len(&self) -> usize {
        match self {
            Single(_) => 1,
            Pair(_, _) => 2,
            ThreeOfAKind(_, _, _) => 3,
            ThreeWithOne(_, _, _, _) => 4,
            ThreeWithPair(_, _, _, _, _) => 5,
            Straight(x) => x.len(),
            DoubleStraight(x) => 2 * x.len(),
            TripleStraight(x) => 3 * x.len(),
            TripleStraightWithOne(x) => 4 * x.len(),
            TripleStraightWithPair(x) => 5 * x.len(),
            FourWithTwoSingle(_, _, _, _, _, _) => 6,
            FourWithTwoPairs(_, _, _, _, _, _, _, _) => 8,
            Bomb(_, _, _, _) => 4,
            Rocket(_, _) => 2,
        }
    }
}

pub type ClassicCardCombination = Combination<ClassicCard>;

impl PartialEq for ClassicCardCombination {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Single(x), Single(y)) => x.get_value().eq(&&y.get_value()),

            (Pair(x, _), Pair(y, _)) => x.get_value().eq(&y.get_value()),

            (ThreeOfAKind(x, _, _), ThreeOfAKind(y, _, _)) => x.get_value().eq(&&y.get_value()),

            (ThreeWithOne(x, _, _, _), ThreeWithOne(y, _, _, _)) => {
                x.get_value().eq(&&y.get_value())
            }

            (ThreeWithPair(x, _, _, _, _), ThreeWithPair(y, _, _, _, _)) => {
                x.get_value().eq(&&y.get_value())
            }

            (Straight(x), Straight(y)) => {
                assert_eq!(x.len(), y.len()); // todo if x.len() ！= y.len() return false
                x.last()
                    .unwrap()
                    .get_value()
                    .eq(&&&y.last().unwrap().get_value())
            }

            (DoubleStraight(x), DoubleStraight(y)) => {
                assert_eq!(x.len(), y.len());
                x.last()
                    .unwrap()
                    .0
                    .get_value()
                    .eq(&y.last().unwrap().0.get_value())
            }

            (TripleStraight(x), TripleStraight(y)) => {
                assert_eq!(x.len(), y.len());
                x.last()
                    .unwrap()
                    .0
                    .get_value()
                    .eq(&y.last().unwrap().0.get_value())
            }

            (TripleStraightWithOne(x), TripleStraightWithOne(y)) => {
                assert_eq!(x.len(), y.len());
                x.last()
                    .unwrap()
                    .0
                    .get_value()
                    .eq(&y.last().unwrap().0.get_value())
            }

            (TripleStraightWithPair(x), TripleStraightWithPair(y)) => {
                assert_eq!(x.len(), y.len());
                x.last()
                    .unwrap()
                    .0
                    .get_value()
                    .eq(&y.last().unwrap().0.get_value())
            }

            (FourWithTwoSingle(x, _, _, _, _, _), FourWithTwoSingle(y, _, _, _, _, _)) => {
                x.get_value().eq(&&y.get_value())
            }

            (
                FourWithTwoPairs(x, _, _, _, _, _, _, _),
                FourWithTwoPairs(y, _, _, _, _, _, _, _),
            ) => x.get_value().eq(&y.get_value()),

            (Bomb(x, _, _, _), Bomb(y, _, _, _)) => x.get_value().eq(&y.get_value()),
            // (
            //     Rocket(x, _ ),
            //     Rocket(y, _),
            // ) => x.eq(y),
            _ => false,
        }
    }
}

impl PartialOrd for ClassicCardCombination {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        if self.weight() == other.weight() {
            match (self, other) {
                (Single(x), Single(y)) => x.weight().partial_cmp(&y.weight()),

                (Pair(x, y), Pair(_, _)) => x.weight().partial_cmp(&y.weight()),

                (ThreeOfAKind(x, _, _), ThreeOfAKind(y, _, _)) => {
                    x.weight().partial_cmp(&y.weight())
                }

                (ThreeWithOne(x, _, _, _), ThreeWithOne(y, _, _, _)) => {
                    x.weight().partial_cmp(&y.weight())
                }

                (ThreeWithPair(x, _, _, _, _), ThreeWithPair(y, _, _, _, _)) => {
                    x.weight().partial_cmp(&y.weight())
                }

                (Straight(x), Straight(y)) => {
                    assert_eq!(x.len(), y.len());
                    x.last()
                        .unwrap()
                        .weight()
                        .partial_cmp(&y.last().unwrap().weight())
                }

                (DoubleStraight(x), DoubleStraight(y)) => {
                    assert_eq!(x.len(), y.len());
                    x.last()
                        .unwrap()
                        .0
                        .weight()
                        .partial_cmp(&y.last().unwrap().0.weight())
                }

                (TripleStraight(x), TripleStraight(y)) => {
                    assert_eq!(x.len(), y.len());
                    x.last()
                        .unwrap()
                        .0
                        .weight()
                        .partial_cmp(&y.last().unwrap().0.weight())
                }

                (TripleStraightWithOne(x), TripleStraightWithOne(y)) => {
                    assert_eq!(x.len(), y.len());
                    x.last()
                        .unwrap()
                        .0
                        .weight()
                        .partial_cmp(&y.last().unwrap().0.weight())
                }

                (TripleStraightWithPair(x), TripleStraightWithPair(y)) => {
                    assert_eq!(x.len(), y.len());
                    x.last()
                        .unwrap()
                        .0
                        .weight()
                        .partial_cmp(&y.last().unwrap().0.weight())
                }

                (FourWithTwoSingle(x, _, _, _, _, _), FourWithTwoSingle(y, _, _, _, _, _)) => {
                    x.weight().partial_cmp(&y.weight())
                }

                (
                    FourWithTwoPairs(x, _, _, _, _, _, _, _),
                    FourWithTwoPairs(y, _, _, _, _, _, _, _),
                ) => x.weight().partial_cmp(&y.weight()),

                (Bomb(x, _, _, _), Bomb(y, _, _, _)) => x.weight().partial_cmp(&y.weight()),
                //  (Rocket(_, _), Rocket(_, _)) => todo!(),
                _ => unimplemented!(),
            }
        } else {
            self.weight().partial_cmp(&other.weight())
        }
    }
}

impl ClassicCardCombination {
    pub fn sanity_check(&self) -> bool {
        match self {
            Single(_) => true,

            Pair(x1, x2) => x1.get_value() == x2.get_value(),

            ThreeOfAKind(x1, x2, x3) => {
                x1.get_value() == x2.get_value() && x1.get_value() == x3.get_value()
            }

            ThreeWithOne(x1, x2, x3, _) => {
                x1.get_value() == x2.get_value() && x1.get_value() == x3.get_value()
            }

            ThreeWithPair(x1, x2, x3, y1, y2) => {
                let condition1 = ThreeOfAKind(*x1, *x2, *x3).sanity_check();
                let condition2 = Pair(*y1, *y2).sanity_check();

                condition1 && condition2
            }

            Straight(x) => {
                if x.len() < 5 {
                    return false;
                }
                let last_card = x.last().unwrap();
                if last_card.weight() > Value::Ace.weight() {
                    return false;
                }

                x.windows(2).all(|x| x[0].weight() == x[1].weight() + 1)
            }

            DoubleStraight(x) => {
                let condition1 = x.iter().all(|(t1, t2)| t1.get_value() == t2.get_value());

                let stright = x.iter().map(|x| x.0).collect::<Vec<_>>();
                if stright.len() < 3 {
                    return false;
                }
                let last_card = stright.last().unwrap();
                if last_card.weight() > Value::Ace.weight() {
                    return false;
                }

                let condition2 = stright
                    .windows(2)
                    .all(|y| y[0].weight() == y[1].weight() + 1);

                condition1 && condition2
            }

            TripleStraight(x) => {
                let condition1 = x.iter().all(|(t1, t2, t3)| {
                    t1.get_value() == t2.get_value() && t2.get_value() == t3.get_value()
                });

                let stright = x.iter().map(|x| x.0).collect::<Vec<_>>();
                if stright.len() < 2 {
                    return false;
                }
                let last_card = stright.last().unwrap();
                if last_card.weight() > Value::Ace.weight() {
                    return false;
                }

                let condition2 = stright
                    .windows(2)
                    .all(|y| y[0].weight() == y[1].weight() + 1);

                condition1 && condition2
            }

            TripleStraightWithOne(x) => {
                let triple_stright = x.iter().map(|x| (x.0, x.1, x.2)).collect::<Vec<_>>();
                TripleStraight(triple_stright).sanity_check()
            }

            TripleStraightWithPair(x) => {
                let triple_stright = x.iter().map(|x| (x.0, x.1, x.2)).collect::<Vec<_>>();
                let condition1 = TripleStraight(triple_stright).sanity_check();
                let condition2 = x.iter().all(|x| x.3.get_value() == x.4.get_value());

                condition1 && condition2
            }

            FourWithTwoSingle(x1, x2, x3, x4, _, _) => {
                x1.get_value() == x2.get_value()
                    && x2.get_value() == x3.get_value()
                    && x3.get_value() == x4.get_value()
            }

            FourWithTwoPairs(x1, x2, x3, x4, y1, y2, y3, y4) => {
                // todo Joker is a single or a pair
                let condition1 = x1.get_value() == x2.get_value()
                    && x2.get_value() == x3.get_value()
                    && x3.get_value() == x4.get_value();
                let condition2 = y1.get_value() == y2.get_value();
                let condition3 = y3.get_value() == y4.get_value();

                condition1 && condition2 && condition3
            }

            Bomb(x1, x2, x3, x4) => {
                x1.get_value() == x2.get_value()
                    && x2.get_value() == x3.get_value()
                    && x3.get_value() == x4.get_value()
            }

            Rocket(_, _) => todo!(),
        }
    }
}

pub type CryptoCardCombination = Combination<zshuffle::Card>;

impl CryptoCardCombination {
    pub fn flatten(&self) -> Vec<ark_bn254::Fr> {
        match self {
            Single(c) => {
                let (x, y) = c.into_affine().xy().unwrap();
                vec![x, y]
            }
            Pair(c1, c2) => {
                let (x1, y1) = c1.into_affine().xy().unwrap();
                let (x2, y2) = c2.into_affine().xy().unwrap();
                vec![x1, y1, x2, y2]
            }
            ThreeOfAKind(c1, c2, c3) => {
                let (x1, y1) = c1.into_affine().xy().unwrap();
                let (x2, y2) = c2.into_affine().xy().unwrap();
                let (x3, y3) = c3.into_affine().xy().unwrap();
                vec![x1, y1, x2, y2, x3, y3]
            }
            ThreeWithOne(c1, c2, c3, c4) => {
                let (x1, y1) = c1.into_affine().xy().unwrap();
                let (x2, y2) = c2.into_affine().xy().unwrap();
                let (x3, y3) = c3.into_affine().xy().unwrap();
                let (x4, y4) = c4.into_affine().xy().unwrap();
                vec![x1, y1, x2, y2, x3, y3, x4, y4]
            }
            ThreeWithPair(c1, c2, c3, c4, c5) => {
                let (x1, y1) = c1.into_affine().xy().unwrap();
                let (x2, y2) = c2.into_affine().xy().unwrap();
                let (x3, y3) = c3.into_affine().xy().unwrap();
                let (x4, y4) = c4.into_affine().xy().unwrap();
                let (x5, y5) = c5.into_affine().xy().unwrap();
                vec![x1, y1, x2, y2, x3, y3, x4, y4, x5, y5]
            }
            Straight(c) => {
                let mut res = vec![];
                for i in c.iter() {
                    let (x, y) = i.into_affine().xy().unwrap();
                    res.push(x);
                    res.push(y)
                }
                res
            }
            DoubleStraight(c) => {
                let mut res = vec![];
                for (i1, i2) in c.iter() {
                    let (x1, y1) = i1.into_affine().xy().unwrap();
                    let (x2, y2) = i2.into_affine().xy().unwrap();
                    res.push(x1);
                    res.push(y1);
                    res.push(x2);
                    res.push(y2);
                }
                res
            }
            TripleStraight(c) => {
                let mut res = vec![];
                for (i1, i2, i3) in c.iter() {
                    let (x1, y1) = i1.into_affine().xy().unwrap();
                    let (x2, y2) = i2.into_affine().xy().unwrap();
                    let (x3, y3) = i3.into_affine().xy().unwrap();
                    res.push(x1);
                    res.push(y1);
                    res.push(x2);
                    res.push(y2);
                    res.push(x3);
                    res.push(y3);
                }
                res
            }
            TripleStraightWithOne(c) => {
                let mut res = vec![];
                for (i1, i2, i3, i4) in c.iter() {
                    let (x1, y1) = i1.into_affine().xy().unwrap();
                    let (x2, y2) = i2.into_affine().xy().unwrap();
                    let (x3, y3) = i3.into_affine().xy().unwrap();
                    let (x4, y4) = i4.into_affine().xy().unwrap();
                    res.push(x1);
                    res.push(y1);
                    res.push(x2);
                    res.push(y2);
                    res.push(x3);
                    res.push(y3);
                    res.push(x4);
                    res.push(y4);
                }
                res
            }
            TripleStraightWithPair(c) => {
                let mut res = vec![];
                for (i1, i2, i3, i4, i5) in c.iter() {
                    let (x1, y1) = i1.into_affine().xy().unwrap();
                    let (x2, y2) = i2.into_affine().xy().unwrap();
                    let (x3, y3) = i3.into_affine().xy().unwrap();
                    let (x4, y4) = i4.into_affine().xy().unwrap();
                    let (x5, y5) = i5.into_affine().xy().unwrap();
                    res.push(x1);
                    res.push(y1);
                    res.push(x2);
                    res.push(y2);
                    res.push(x3);
                    res.push(y3);
                    res.push(x4);
                    res.push(y4);
                    res.push(x5);
                    res.push(y5);
                }

                res
            }
            FourWithTwoSingle(c1, c2, c3, c4, c5, c6) => {
                let (x1, y1) = c1.into_affine().xy().unwrap();
                let (x2, y2) = c2.into_affine().xy().unwrap();
                let (x3, y3) = c3.into_affine().xy().unwrap();
                let (x4, y4) = c4.into_affine().xy().unwrap();
                let (x5, y5) = c5.into_affine().xy().unwrap();
                let (x6, y6) = c6.into_affine().xy().unwrap();
                vec![x1, y1, x2, y2, x3, y3, x4, y4, x5, y5, x6, y6]
            }
            FourWithTwoPairs(c1, c2, c3, c4, c5, c6, c7, c8) => {
                let (x1, y1) = c1.into_affine().xy().unwrap();
                let (x2, y2) = c2.into_affine().xy().unwrap();
                let (x3, y3) = c3.into_affine().xy().unwrap();
                let (x4, y4) = c4.into_affine().xy().unwrap();
                let (x5, y5) = c5.into_affine().xy().unwrap();
                let (x6, y6) = c6.into_affine().xy().unwrap();
                let (x7, y7) = c7.into_affine().xy().unwrap();
                let (x8, y8) = c8.into_affine().xy().unwrap();
                vec![
                    x1, y1, x2, y2, x3, y3, x4, y4, x5, y5, x6, y6, x7, y7, x8, y8,
                ]
            }
            Bomb(c1, c2, c3, c4) => {
                let (x1, y1) = c1.into_affine().xy().unwrap();
                let (x2, y2) = c2.into_affine().xy().unwrap();
                let (x3, y3) = c3.into_affine().xy().unwrap();
                let (x4, y4) = c4.into_affine().xy().unwrap();
                vec![x1, y1, x2, y2, x3, y3, x4, y4]
            }
            Rocket(c1, c2) => {
                let (x1, y1) = c1.into_affine().xy().unwrap();
                let (x2, y2) = c2.into_affine().xy().unwrap();
                vec![x1, y1, x2, y2]
            }
        }
    }
    pub fn morph_to_classic(&self) -> Result<ClassicCardCombination> {
        match self {
            Single(x) => {
                let c = ENCODING_CARDS_MAPPING
                    .get(x)
                    .ok_or(PokerError::MorphError)?;

                Ok(Single(*c))
            }

            Pair(x1, x2) => {
                let c_1 = ENCODING_CARDS_MAPPING
                    .get(x1)
                    .ok_or(PokerError::MorphError)?;
                let c_2 = ENCODING_CARDS_MAPPING
                    .get(x2)
                    .ok_or(PokerError::MorphError)?;

                Ok(Pair(*c_1, *c_2))
            }

            ThreeOfAKind(x1, x2, x3) => {
                let c_1 = ENCODING_CARDS_MAPPING
                    .get(x1)
                    .ok_or(PokerError::MorphError)?;
                let c_2 = ENCODING_CARDS_MAPPING
                    .get(x2)
                    .ok_or(PokerError::MorphError)?;
                let c_3 = ENCODING_CARDS_MAPPING
                    .get(x3)
                    .ok_or(PokerError::MorphError)?;

                Ok(ThreeOfAKind(*c_1, *c_2, *c_3))
            }

            ThreeWithOne(x1, x2, x3, x4) => {
                let c_1 = ENCODING_CARDS_MAPPING
                    .get(x1)
                    .ok_or(PokerError::MorphError)?;
                let c_2 = ENCODING_CARDS_MAPPING
                    .get(x2)
                    .ok_or(PokerError::MorphError)?;
                let c_3 = ENCODING_CARDS_MAPPING
                    .get(x3)
                    .ok_or(PokerError::MorphError)?;
                let c_4 = ENCODING_CARDS_MAPPING
                    .get(x4)
                    .ok_or(PokerError::MorphError)?;

                Ok(ThreeWithOne(*c_1, *c_2, *c_3, *c_4))
            }

            ThreeWithPair(x1, x2, x3, x4, x5) => {
                let c_1 = ENCODING_CARDS_MAPPING
                    .get(x1)
                    .ok_or(PokerError::MorphError)?;
                let c_2 = ENCODING_CARDS_MAPPING
                    .get(x2)
                    .ok_or(PokerError::MorphError)?;
                let c_3 = ENCODING_CARDS_MAPPING
                    .get(x3)
                    .ok_or(PokerError::MorphError)?;
                let c_4 = ENCODING_CARDS_MAPPING
                    .get(x4)
                    .ok_or(PokerError::MorphError)?;
                let c_5 = ENCODING_CARDS_MAPPING
                    .get(x5)
                    .ok_or(PokerError::MorphError)?;

                Ok(ThreeWithPair(*c_1, *c_2, *c_3, *c_4, *c_5))
            }

            Straight(x) => {
                let mut classic_card = vec![];
                for y in x.iter() {
                    let c = ENCODING_CARDS_MAPPING
                        .get(y)
                        .ok_or(PokerError::MorphError)?;
                    classic_card.push(*c)
                }

                Ok(Straight(classic_card))
            }

            DoubleStraight(x) => {
                let mut classic_card = vec![];
                for (y1, y2) in x.iter() {
                    let c1 = ENCODING_CARDS_MAPPING
                        .get(y1)
                        .ok_or(PokerError::MorphError)?;
                    let c2 = ENCODING_CARDS_MAPPING
                        .get(y2)
                        .ok_or(PokerError::MorphError)?;
                    classic_card.push((*c1, *c2))
                }

                Ok(DoubleStraight(classic_card))
            }

            TripleStraight(x) => {
                let mut classic_card = vec![];
                for (y1, y2, y3) in x.iter() {
                    let c1 = ENCODING_CARDS_MAPPING
                        .get(y1)
                        .ok_or(PokerError::MorphError)?;
                    let c2 = ENCODING_CARDS_MAPPING
                        .get(y2)
                        .ok_or(PokerError::MorphError)?;
                    let c3 = ENCODING_CARDS_MAPPING
                        .get(y3)
                        .ok_or(PokerError::MorphError)?;
                    classic_card.push((*c1, *c2, *c3))
                }

                Ok(TripleStraight(classic_card))
            }

            TripleStraightWithOne(x) => {
                let mut classic_card = vec![];
                for (y1, y2, y3, y4) in x.iter() {
                    let c1 = ENCODING_CARDS_MAPPING
                        .get(y1)
                        .ok_or(PokerError::MorphError)?;
                    let c2 = ENCODING_CARDS_MAPPING
                        .get(y2)
                        .ok_or(PokerError::MorphError)?;
                    let c3 = ENCODING_CARDS_MAPPING
                        .get(y3)
                        .ok_or(PokerError::MorphError)?;
                    let c4 = ENCODING_CARDS_MAPPING
                        .get(y4)
                        .ok_or(PokerError::MorphError)?;
                    classic_card.push((*c1, *c2, *c3, *c4))
                }

                Ok(TripleStraightWithOne(classic_card))
            }

            TripleStraightWithPair(x) => {
                let mut classic_card = vec![];
                for (y1, y2, y3, y4, y5) in x.iter() {
                    let c1 = ENCODING_CARDS_MAPPING
                        .get(y1)
                        .ok_or(PokerError::MorphError)?;
                    let c2 = ENCODING_CARDS_MAPPING
                        .get(y2)
                        .ok_or(PokerError::MorphError)?;
                    let c3 = ENCODING_CARDS_MAPPING
                        .get(y3)
                        .ok_or(PokerError::MorphError)?;
                    let c4 = ENCODING_CARDS_MAPPING
                        .get(y4)
                        .ok_or(PokerError::MorphError)?;
                    let c5 = ENCODING_CARDS_MAPPING
                        .get(y5)
                        .ok_or(PokerError::MorphError)?;
                    classic_card.push((*c1, *c2, *c3, *c4, *c5))
                }

                Ok(TripleStraightWithPair(classic_card))
            }

            FourWithTwoSingle(x1, x2, x3, x4, x5, x6) => {
                let c_1 = ENCODING_CARDS_MAPPING
                    .get(x1)
                    .ok_or(PokerError::MorphError)?;
                let c_2 = ENCODING_CARDS_MAPPING
                    .get(x2)
                    .ok_or(PokerError::MorphError)?;
                let c_3 = ENCODING_CARDS_MAPPING
                    .get(x3)
                    .ok_or(PokerError::MorphError)?;
                let c_4 = ENCODING_CARDS_MAPPING
                    .get(x4)
                    .ok_or(PokerError::MorphError)?;
                let c_5 = ENCODING_CARDS_MAPPING
                    .get(x5)
                    .ok_or(PokerError::MorphError)?;
                let c_6 = ENCODING_CARDS_MAPPING
                    .get(x6)
                    .ok_or(PokerError::MorphError)?;

                Ok(FourWithTwoSingle(*c_1, *c_2, *c_3, *c_4, *c_5, *c_6))
            }

            FourWithTwoPairs(x1, x2, x3, x4, x5, x6, x7, x8) => {
                let c_1 = ENCODING_CARDS_MAPPING
                    .get(x1)
                    .ok_or(PokerError::MorphError)?;
                let c_2 = ENCODING_CARDS_MAPPING
                    .get(x2)
                    .ok_or(PokerError::MorphError)?;
                let c_3 = ENCODING_CARDS_MAPPING
                    .get(x3)
                    .ok_or(PokerError::MorphError)?;
                let c_4 = ENCODING_CARDS_MAPPING
                    .get(x4)
                    .ok_or(PokerError::MorphError)?;
                let c_5 = ENCODING_CARDS_MAPPING
                    .get(x5)
                    .ok_or(PokerError::MorphError)?;
                let c_6 = ENCODING_CARDS_MAPPING
                    .get(x6)
                    .ok_or(PokerError::MorphError)?;
                let c_7 = ENCODING_CARDS_MAPPING
                    .get(x7)
                    .ok_or(PokerError::MorphError)?;
                let c_8 = ENCODING_CARDS_MAPPING
                    .get(x8)
                    .ok_or(PokerError::MorphError)?;

                Ok(FourWithTwoPairs(
                    *c_1, *c_2, *c_3, *c_4, *c_5, *c_6, *c_7, *c_8,
                ))
            }

            Bomb(x1, x2, x3, x4) => {
                let c_1 = ENCODING_CARDS_MAPPING
                    .get(x1)
                    .ok_or(PokerError::MorphError)?;
                let c_2 = ENCODING_CARDS_MAPPING
                    .get(x2)
                    .ok_or(PokerError::MorphError)?;
                let c_3 = ENCODING_CARDS_MAPPING
                    .get(x3)
                    .ok_or(PokerError::MorphError)?;
                let c_4 = ENCODING_CARDS_MAPPING
                    .get(x4)
                    .ok_or(PokerError::MorphError)?;

                Ok(Bomb(*c_1, *c_2, *c_3, *c_4))
            }

            Rocket(x1, x2) => {
                let c_1 = ENCODING_CARDS_MAPPING
                    .get(x1)
                    .ok_or(PokerError::MorphError)?;
                let c_2 = ENCODING_CARDS_MAPPING
                    .get(x2)
                    .ok_or(PokerError::MorphError)?;

                Ok(Rocket(*c_1, *c_2))
            }
        }
    }
}
