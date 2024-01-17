use crate::{
    cards::{ClassicCard, Value},
    combination::Combination::*,
};

/// Different card play combinations
pub enum Combination {
    // Single card
    Single(ClassicCard),

    // Pair of cards
    Pair(ClassicCard, ClassicCard),

    // Three cards of the same rank
    ThreeOfAKind(ClassicCard, ClassicCard, ClassicCard),

    // Three cards of the same rank with one single card
    ThreeWithOne(ClassicCard, ClassicCard, ClassicCard, ClassicCard),

    // Three cards of the same rank with one pair
    ThreeWithPair(
        ClassicCard,
        ClassicCard,
        ClassicCard,
        ClassicCard,
        ClassicCard,
    ),

    // Five or more consecutive single cards
    Straight(Vec<ClassicCard>),

    // Three or more consecutive pairs
    DoubleStraight(Vec<(ClassicCard, ClassicCard)>),

    // Two or more consecutive three of a kind
    TripleStraight(Vec<(ClassicCard, ClassicCard, ClassicCard)>),

    // Triple straight with one single card
    TripleStraightWithOne(Vec<(ClassicCard, ClassicCard, ClassicCard, ClassicCard)>),

    // Triple straight with one pair
    TripleStraightWithPair(
        Vec<(
            ClassicCard,
            ClassicCard,
            ClassicCard,
            ClassicCard,
            ClassicCard,
        )>,
    ),

    // Four cards of the same rank with two single cards
    FourWithTwoSingle(
        ClassicCard,
        ClassicCard,
        ClassicCard,
        ClassicCard,
        ClassicCard,
        ClassicCard,
    ),

    // Four cards of the same rank with two pairs
    FourWithTwoPairs(
        ClassicCard,
        ClassicCard,
        ClassicCard,
        ClassicCard,
        ClassicCard,
        ClassicCard,
        ClassicCard,
        ClassicCard,
    ),

    // Four cards of the same rank
    Bomb(ClassicCard, ClassicCard, ClassicCard, ClassicCard),

    // Both Jokers in a standard deck
    Rocket(ClassicCard, ClassicCard),
    // TODO Add more combinations //
}

impl PartialEq for Combination {
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

impl PartialOrd for Combination {
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

impl Combination {
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

    pub fn check_rank(&self) -> bool {
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
                let condition1 = ThreeOfAKind(*x1, *x2, *x3).check_rank();
                let condition2 = Pair(*y1, *y2).check_rank();

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
                TripleStraight(triple_stright).check_rank()
            }

            TripleStraightWithPair(x) => {
                let triple_stright = x.iter().map(|x| (x.0, x.1, x.2)).collect::<Vec<_>>();
                let condition1 = TripleStraight(triple_stright).check_rank();
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
