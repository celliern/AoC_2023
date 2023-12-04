#![allow(dead_code)]
use regex::Regex;
use std::{cmp::Reverse, collections::HashSet, fs};

fn score(n_win: u32) -> u32 {
    if n_win < 1 {
        return 0;
    }
    2_u32.pow(n_win - 1)
}

#[derive(Debug, Clone)]
struct Scratchcard {
    id: Reverse<u32>,
    win_num: HashSet<u32>,
    my_num: Vec<u32>,
}

impl Scratchcard {
    fn new(id: u32, win_num: Vec<u32>, my_num: Vec<u32>) -> Scratchcard {
        Scratchcard {
            id: Reverse(id),
            win_num: win_num.into_iter().collect(),
            my_num,
        }
    }

    fn id(&self) -> u32 {
        self.id.0
    }

    fn nums_winning(&self) -> Vec<u32> {
        self.my_num
            .iter()
            .filter(|n| self.win_num.contains(n))
            .copied()
            .collect()
    }

    fn n_winning(&self) -> u32 {
        self.nums_winning().len() as u32
    }

    fn score(&self) -> u32 {
        score(self.n_winning())
    }

    fn parse(s: &str) -> Option<Scratchcard> {
        let pattern = Regex::new(r"^Card +(\d+): (.*) \| (.*)$").expect("Regex invalid");
        let cap = pattern.captures(s)?;
        let id: u32 = cap[1].parse().expect("unable to parse id");
        let win_num: Vec<u32> = cap[2]
            .split_whitespace()
            .map(|s| s.parse().expect("unable to parse win_num"))
            .collect();
        let my_num: Vec<u32> = cap[3]
            .split_whitespace()
            .map(|s| s.parse().expect("unable to parse my_num"))
            .collect();
        Some(Scratchcard::new(id, win_num, my_num))
    }
}

#[derive(Debug, Clone)]
struct Pile {
    cards: Vec<Scratchcard>,
    duplicates: Vec<u32>,
}

impl Default for Pile {
    fn default() -> Self {
        Pile::new(vec![])
    }
}

impl Pile {
    fn new(cards: Vec<Scratchcard>) -> Pile {
        let n_cards = cards.len();
        Pile {
            cards,
            duplicates: vec![1; n_cards],
        }
    }

    fn parse(s: &str) -> Pile {
        let cards: Vec<Scratchcard> = s.lines().filter_map(Scratchcard::parse).collect();
        Pile::new(cards)
    }

    fn scratchcards(&self) -> &[Scratchcard] {
        &self.cards
    }

    fn scores(&self) -> Vec<u32> {
        self.scratchcards().iter().map(|c| c.score()).collect()
    }

    fn duplicate_card(&mut self, card: &Scratchcard) {
        let id = card.id();
        let n_winning = card.n_winning();
        let curr_duplicate = self.duplicates[id as usize - 1];
        for incr in 1..=n_winning {
            let i = (id + incr) as usize - 1;
            if i > self.duplicates.len() {
                break;
            }
            self.duplicates[(id + incr) as usize - 1] += curr_duplicate;
        }
    }
}

fn part01(pile: &Pile) -> u32 {
    pile.scores().iter().sum::<u32>()
}

fn part02(pile: &mut Pile) -> u32 {
    let cards = pile.cards.clone();
    cards.iter().for_each(|c| pile.duplicate_card(c));
    pile.duplicates.iter().sum()
}

fn main() {
    let content = fs::read_to_string("data/input.txt").unwrap();
    let pile = Pile::parse(&content);
    println!("Part 1: {}", part01(&pile));
    println!("Part 2: {}", part02(&mut pile.clone()));
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::*;

    #[test]
    fn test_p1() {
        let content = fs::read_to_string("data/test_input.txt").unwrap();
        let pile = Pile::parse(&content);
        assert_eq!(13, part01(&pile));
    }

    #[test]
    fn test_p2() {
        let content = fs::read_to_string("data/test_input.txt").unwrap();
        let mut pile = Pile::parse(&content);
        assert_eq!(30, part02(&mut pile));
    }
}
