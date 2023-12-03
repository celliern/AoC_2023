#![allow(dead_code)]

use regex::Regex;
use std::{collections::HashSet, fs};
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct PartNumber {
    number: u32,
    col_start: u32,
    col_end: u32,
    row: u32,
}

impl PartNumber {
    fn new(number: u32, col: u32, row: u32) -> Self {
        let ndigits = number.to_string().len();
        Self {
            number,
            col_start: col,
            col_end: col + ndigits as u32 - 1,
            row,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct Symbol {
    symb: char,
    row: u32,
    col: u32,
}

impl Symbol {
    fn new(symb: char, row: u32, col: u32) -> Self {
        Self { symb, row, col }
    }

    fn find_adjacent(self, parts: &[PartNumber]) -> Vec<PartNumber> {
        parts
            .iter()
            .filter(|p| p.row >= self.row - 1 && p.row <= self.row + 1)
            .filter(|p| p.col_start <= self.col + 1 && p.col_end >= self.col - 1)
            .copied()
            .collect()
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct Schematic {
    parts: Vec<PartNumber>,
    symbols: Vec<Symbol>,
}

impl Schematic {
    fn new(parts: Vec<PartNumber>, symbols: Vec<Symbol>) -> Self {
        Self { parts, symbols }
    }

    fn add_part(&mut self, part: PartNumber) {
        self.parts.push(part);
    }

    fn add_symbol(&mut self, symbol: Symbol) {
        self.symbols.push(symbol);
    }

    fn parse(input: &str) -> Self {
        let number_re = Regex::new(r"\d+").unwrap();
        let mut schematic = Self::default();

        for (row, line) in input.lines().enumerate() {
            number_re.find_iter(line).for_each(|m| {
                schematic.add_part(PartNumber::new(
                    m.as_str().parse().unwrap(),
                    m.start() as u32,
                    row as u32,
                ))
            });

            for (col, symbol) in line.chars().enumerate() {
                if !symbol.is_ascii_digit() && symbol != '.' {
                    schematic.add_symbol(Symbol::new(symbol, row as u32, col as u32));
                }
            }
        }

        schematic
    }

    fn get_valid_parts(&self) -> Vec<PartNumber> {
        self.symbols
            .iter()
            .flat_map(|s| s.find_adjacent(&self.parts))
            .collect::<HashSet<_>>()
            .into_iter()
            .collect()
    }

    fn get_gears(&self) -> Vec<u32> {
        self.symbols
            .iter()
            .filter(|x| x.symb == '*')
            .map(|x| x.find_adjacent(&self.parts))
            .filter(|x| x.len() == 2)
            .map(|x| x[0].number * x[1].number)
            .collect()
    }
}

fn main() {
    let content = fs::read_to_string("data/input.txt").unwrap();
    let schematic = Schematic::parse(&content);
    let total = schematic
        .get_valid_parts()
        .iter()
        .map(|x| x.number)
        .sum::<u32>();
    println!("{}", total);

    let gears = schematic.get_gears();
    println!("{}", gears.iter().sum::<u32>());
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::*;

    #[test]
    fn test_parse() {
        let content = fs::read_to_string("data/test_input.txt").unwrap();
        let schematic = Schematic::parse(&content);
        println!("{:#?}", schematic);
        assert_eq!(schematic.parts.len(), 10);
        assert_eq!(schematic.symbols.len(), 6);
    }

    #[test]
    fn test_part01() {
        let content = fs::read_to_string("data/test_input.txt").unwrap();
        let schematic = Schematic::parse(&content);
        let total = schematic
            .get_valid_parts()
            .iter()
            .map(|x| x.number)
            .sum::<u32>();
        assert_eq!(total, 4361);
    }
}
