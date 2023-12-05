#![allow(dead_code)]
use itertools::Itertools;
use kdam::tqdm;
use rayon::prelude::*;
use std::collections::HashSet;
use std::fs;
use std::ops::Range;
use std::time::Instant;

use regex::Regex;

const CHUNK_SIZE: usize = 1_000_000;

fn build_location_regex(from: &str, to: &str) -> Regex {
    Regex::new(format!(r"(?m){}-to-{} map:\n(?<map>(\d+\s?)+)", from, to).as_str()).unwrap()
}

fn parse_seeds(input: String) -> Vec<usize> {
    let seed_regex = Regex::new(r"seeds: (?<seeds>(\d+\s)+)").unwrap();
    let c = seed_regex.captures(&input).expect("failed to parse seeds");
    c["seeds"]
        .to_string()
        .split_whitespace()
        .map(|s| s.parse::<usize>().unwrap())
        .collect()
}

fn process_seeds_raw(seeds: Vec<usize>) -> (usize, impl Iterator<Item = usize>) {
    (seeds.len(), seeds.into_iter())
}

fn process_seeds_range(seeds: Vec<usize>) -> (usize, impl Iterator<Item = usize>) {
    let start_len: Vec<(usize, usize)> = seeds.into_iter().tuples::<(_, _)>().collect();
    let total = start_len.iter().map(|(_, len)| len).sum::<usize>();
    (total, start_len.into_iter().flat_map(|(a, b)| (a..a + b)))
}

const SEED_STEPS: [&str; 8] = [
    "seed",
    "soil",
    "fertilizer",
    "water",
    "light",
    "temperature",
    "humidity",
    "location",
];

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct LocationRange {
    dst_start: usize,
    src_start: usize,
    length: usize,
}

impl LocationRange {
    fn new(dst_start: usize, src_start: usize, length: usize) -> LocationRange {
        LocationRange {
            dst_start,
            src_start,
            length,
        }
    }

    fn parse(row: &str) -> LocationRange {
        let mut parts = row.split_whitespace();
        let dst_start = parts.next().unwrap().parse::<usize>().unwrap();
        let src_start = parts.next().unwrap().parse::<usize>().unwrap();
        let length = parts.next().unwrap().parse::<usize>().unwrap();
        LocationRange::new(dst_start, src_start, length)
    }

    fn src_range(&self) -> Range<usize> {
        self.src_start..(self.src_start + self.length)
    }

    fn dst_range(&self) -> Range<usize> {
        self.dst_start..(self.dst_start + self.length)
    }

    fn map(&self, src: usize) -> Option<usize> {
        if !self.src_range().contains(&src) {
            return None;
        }
        let offset = src - self.src_start;
        Some(self.dst_range().nth(offset).unwrap())
    }

    fn reverse_map(&self, dst: usize) -> Option<usize> {
        if !self.dst_range().contains(&dst) {
            return None;
        }
        let offset = dst - self.dst_start;
        Some(self.src_range().nth(offset).unwrap())
    }
}

struct Location {
    from: String,
    to: String,
    ranges: HashSet<LocationRange>,
}

impl Location {
    fn new(from: &str, to: &str) -> Location {
        Location {
            from: from.to_string(),
            to: to.to_string(),
            ranges: HashSet::new(),
        }
    }

    fn map(&self, input: usize) -> usize {
        for range in &self.ranges {
            if let Some(mapped) = range.map(input) {
                return mapped;
            }
        }
        input
    }

    fn reverse_map(&self, input: usize) -> usize {
        for range in &self.ranges {
            if let Some(mapped) = range.reverse_map(input) {
                return mapped;
            }
        }
        input
    }

    fn parse_ranges(&mut self, input: &str) {
        let location_regex = build_location_regex(&self.from, &self.to);
        let c = location_regex
            .captures(input)
            .unwrap_or_else(|| panic!("failed to parse ranges ({}-to-{})", self.from, self.to));
        let map = &c["map"];
        let ranges: HashSet<LocationRange> = map.lines().map(LocationRange::parse).collect();
        self.ranges.extend(ranges);
    }
}

struct Almanach {
    seeds: Vec<usize>,
    locations: Vec<Location>,
}

impl Almanach {
    fn new(seeds: Vec<usize>, locations: Vec<Location>) -> Almanach {
        Almanach { seeds, locations }
    }

    fn parse(input: String) -> Almanach {
        let steps = SEED_STEPS
            .into_iter()
            .zip(SEED_STEPS.into_iter().skip(1))
            .collect::<Vec<_>>();
        let locations: Vec<Location> = steps
            .into_iter()
            .map(|(from, to)| {
                let mut location = Location::new(from, to);
                location.parse_ranges(&input);
                location
            })
            .collect();
        let seeds = parse_seeds(input);
        Almanach::new(seeds, locations)
    }

    fn get_dst(&self, src: &usize) -> usize {
        self.locations
            .iter()
            .fold(*src, |src, location| location.map(src))
    }

    fn get_src(&self, dst: &usize) -> usize {
        self.locations
            .iter()
            .rev()
            .fold(*dst, |dst, location| location.reverse_map(dst))
    }

    fn process_raw(self) -> usize {
        let (total, seeds) = process_seeds_raw(self.seeds);
        tqdm!(seeds, total = total)
            .map(move |seed| {
                self.locations
                    .iter()
                    .fold(seed, |seed, location| location.map(seed))
            })
            .min()
            .unwrap()
    }
    fn process_range(&self) -> usize {
        let (total, seeds) = process_seeds_range(self.seeds.clone());
        tqdm!(seeds, total = total)
            .par_bridge()
            .map(|src| self.get_dst(&src))
            .min()
            .unwrap()
    }

    fn reverse_process_raw(self) -> usize {
        let src = tqdm!((1..).map(|dst| self.get_src(&dst)))
            .find(|src| self.seeds.contains(src))
            .unwrap();
        self.get_dst(&src)
    }

    fn reverse_process_range(&self) -> usize {
        let intervals: Vec<Range<usize>> = self
            .seeds
            .clone()
            .into_iter()
            .tuples()
            .map(|(a, b)| a..a + b)
            .collect();

        let src = tqdm!((1..).chunks(CHUNK_SIZE).into_iter().flat_map(|chunk| {
            chunk
                .collect::<Vec<_>>()
                .into_par_iter()
                .map(|dst| self.get_src(&dst))
                .find_first(|src| intervals.iter().any(|range| range.contains(src)))
        }))
        .next()
        .unwrap();
        self.get_dst(&src)
    }
}

fn main() {
    let input = fs::read_to_string("./data/input.txt").expect("failed to read input");
    let almanach = Almanach::parse(input.clone());
    let now = Instant::now();
    println!("Part 01: {}", almanach.process_raw());
    println!("Time: {:?}", now.elapsed());

    let almanach = Almanach::parse(input.clone());
    let now = Instant::now();
    println!("Part 02: {}", almanach.reverse_process_range());
    println!("Time: {:?}", now.elapsed());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse() {
        let input = fs::read_to_string("./data/test_input.txt").expect("failed to read input");
        let almanach = Almanach::parse(input.clone());
        assert_eq!(almanach.seeds, vec![79, 14, 55, 13]);
    }

    #[test]
    fn test_p1() {
        let input = fs::read_to_string("./data/test_input.txt").expect("failed to read input");
        let almanach = Almanach::parse(input.clone());
        assert_eq!(almanach.process_raw(), 35);
        let almanach = Almanach::parse(input.clone());
        assert_eq!(almanach.reverse_process_raw(), 35);
    }

    #[test]
    fn test_p2() {
        let input = fs::read_to_string("./data/test_input.txt").expect("failed to read input");
        let almanach = Almanach::parse(input.clone());
        assert_eq!(almanach.process_range(), 46);
        let almanach = Almanach::parse(input.clone());
        assert_eq!(almanach.reverse_process_range(), 46);
    }
}
