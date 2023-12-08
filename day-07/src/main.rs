#![allow(dead_code)]

use itertools::Itertools;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum Card {
    Joker,
    Two,
    Three,
    Four,
    Five,
    Six,
    Seven,
    Eight,
    Nine,
    Ten,
    Queen,
    King,
    Ace,
}

impl From<char> for Card {
    fn from(c: char) -> Self {
        match c {
            '2' => Card::Two,
            '3' => Card::Three,
            '4' => Card::Four,
            '5' => Card::Five,
            '6' => Card::Six,
            '7' => Card::Seven,
            '8' => Card::Eight,
            '9' => Card::Nine,
            'T' => Card::Ten,
            'J' => Card::Joker,
            'Q' => Card::Queen,
            'K' => Card::King,
            'A' => Card::Ace,
            _ => unreachable!(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum HandType {
    HighCard,
    Pair,
    TwoPair,
    ThreeKind,
    FullHouse,
    FourKind,
    FiveKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct Cards([Card; 5]);

impl From<&Cards> for HandType {
    fn from(cards: &Cards) -> Self {
        let mut counter = cards.0.iter().counts();
        let n_joker = counter.remove(&Card::Joker).unwrap_or(0);
        if n_joker == 5 {
            return HandType::FiveKind;
        }

        let mut max_nvals = counter.values().sorted().rev();
        let best_nval = *max_nvals.next().unwrap() + n_joker;
        if best_nval == 5 {
            return HandType::FiveKind;
        } else if best_nval == 4 {
            return HandType::FourKind;
        }
        let second_nval = *max_nvals.next().unwrap();
        if best_nval == 3 && second_nval == 2 {
            return HandType::FullHouse;
        }

        if best_nval == 3 {
            return HandType::ThreeKind;
        } else if best_nval == 2 && second_nval == 2 {
            return HandType::TwoPair;
        } else if best_nval == 2 {
            return HandType::Pair;
        }

        HandType::HighCard
    }
}

impl FromIterator<char> for Cards {
    fn from_iter<T: IntoIterator<Item = char>>(iter: T) -> Self {
        let mut cards: [Card; 5] = [Card::Two; 5];
        for (i, char) in iter.into_iter().enumerate() {
            cards[i] = Card::from(char);
        }
        Self(cards)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct Hand {
    hand_type: HandType,
    cards: Cards,
    bid: u32,
}

impl Hand {
    fn new(cards: Cards, bid: Option<u32>) -> Self {
        Self {
            hand_type: (&cards).into(),
            cards,
            bid: bid.unwrap_or(0),
        }
    }
    fn parse(row: &str) -> Self {
        let mut col_iter = row.split_whitespace();
        let cards = col_iter.next().unwrap();
        let bid = col_iter.next().map(|bid| bid.parse::<u32>().unwrap());
        return Self::new(cards.chars().collect(), bid);
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct Hands(Vec<Hand>);

impl Hands {
    fn new(hands: Vec<Hand>) -> Self {
        Self(hands)
    }

    fn parse(input: &str) -> Self {
        Hands::new(input.lines().map(Hand::parse).sorted().collect())
    }

    fn from_file(path: &str) -> Self {
        let contents = std::fs::read_to_string(path).unwrap();
        Self::parse(&contents)
    }

    fn score(&self) -> u32 {
        self.0
            .iter()
            .sorted()
            .enumerate()
            .map(|(i, hand)| (i + 1) as u32 * hand.bid)
            .sum()
    }
}

fn main() {
    let hand = Hands::from_file("data/input.txt");
    println!("{}", hand.score());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hand_type() {
        let hand = Hand::parse("32T3K");
        assert_eq!(hand.hand_type, HandType::Pair);

        let hand = Hand::parse("QQQJA");
        assert_eq!(hand.hand_type, HandType::ThreeKind);
    }

    #[test]
    fn test_part1() {
        let hands = Hands::from_file("./data/test_input.txt");
        println!("{:?}", hands);
        println!("{}", hands.score());
    }
}
