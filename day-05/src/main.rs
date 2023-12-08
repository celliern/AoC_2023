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

fn parse_seeds(input: String) -> Vec<i32> {
    let seed_regex = Regex::new(r"seeds: (?<seeds>(\d+\s)+)").unwrap();
    let c = seed_regex.captures(&input).expect("failed to parse seeds");
    c["seeds"]
        .to_string()
        .split_whitespace()
        .map(|s| s.parse::<i32>().unwrap())
        .collect()
}

fn process_seeds_raw(seeds: Vec<i32>) -> (i32, impl Iterator<Item = i32>) {
    (seeds.len() as i32, seeds.into_iter())
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
    dst_start: i32,
    src_start: i32,
    length: i32,
}

impl LocationRange {
    fn new(dst_start: i32, src_start: i32, length: i32) -> LocationRange {
        LocationRange {
            dst_start,
            src_start,
            length,
        }
    }

    fn parse(row: &str) -> LocationRange {
        let mut parts = row.split_whitespace();
        let dst_start = parts.next().unwrap().parse::<i32>().unwrap();
        let src_start = parts.next().unwrap().parse::<i32>().unwrap();
        let length = parts.next().unwrap().parse::<i32>().unwrap();
        LocationRange::new(dst_start, src_start, length)
    }

    fn src_range(&self) -> Range<i32> {
        self.src_start..(self.src_start + self.length)
    }

    fn dst_range(&self) -> Range<i32> {
        self.dst_start..(self.dst_start + self.length)
    }

    fn map(&self, src: i32) -> Option<i32> {
        if !self.src_range().contains(&src) {
            return None;
        }
        let offset = src - self.src_start;
        Some(self.dst_range().nth(offset as usize).unwrap())
    }

    fn reverse_map(&self, dst: i32) -> Option<i32> {
        if !self.dst_range().contains(&dst) {
            return None;
        }
        let offset = dst - self.dst_start;
        Some(self.src_range().nth(offset as usize).unwrap())
    }
    #[allow(clippy::single_range_in_vec_init)]
    fn split_range(&self, src_range: Range<i32>) -> (Vec<Range<i32>>, Option<Range<i32>>) {
        let loc_src_range = self.src_range();
        let left = src_range.start.max(loc_src_range.start);
        let right = src_range.end.min(loc_src_range.end);
        let in_loc_range = left..right;
        let (src_ranges, in_loc_range) = if in_loc_range.start < in_loc_range.end {
            (vec![src_range], None)
        } else if in_loc_range == src_range {
            (vec![], Some(src_range))
        } else if right == loc_src_range.end {
            (
                vec![src_range.start..in_loc_range.start],
                Some(in_loc_range),
            )
        } else if left == loc_src_range.start {
            (vec![in_loc_range.end..src_range.end], Some(in_loc_range))
        } else {
            (
                vec![
                    src_range.start..in_loc_range.start,
                    in_loc_range.end..src_range.end,
                ],
                Some(in_loc_range),
            )
        };
        let dst_range = in_loc_range.map(|in_loc_range| {
            let offset = self.dst_start - self.src_start;
            (in_loc_range.start + offset)..(in_loc_range.end + offset)
        });
        (src_ranges, dst_range)
    }
}

#[derive(Debug, Clone)]
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

    fn map(&self, input: i32) -> i32 {
        for range in &self.ranges {
            if let Some(mapped) = range.map(input) {
                return mapped;
            }
        }
        input
    }

    fn reverse_map(&self, input: i32) -> i32 {
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

    fn map_ranges(&self, ranges: Vec<Range<i32>>) -> Vec<Range<i32>> {
        let mut dst_ranges = Vec::new();
        let mut unseen = ranges.clone();

        for loc_range in &self.ranges {
            println!("{:?}", loc_range);
            let mut still_here = Vec::new();
            while let Some(range) = unseen.pop() {
                let (src_ranges, dst_range) = loc_range.split_range(range);

                if let Some(dst_range) = dst_range {
                    dst_ranges.push(dst_range);
                }
                still_here.extend(src_ranges)
            }

            unseen.extend(still_here);
        }

        dst_ranges.into_iter().chain(unseen).collect()
    }
}

struct Almanach {
    seeds: Vec<i32>,
    locations: Vec<Location>,
}

impl Almanach {
    fn new(seeds: Vec<i32>, locations: Vec<Location>) -> Almanach {
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

    fn get_dst(&self, src: &i32) -> i32 {
        self.locations
            .iter()
            .fold(*src, |src, location| location.map(src))
    }

    fn get_src(&self, dst: &i32) -> i32 {
        self.locations
            .iter()
            .rev()
            .fold(*dst, |dst, location| location.reverse_map(dst))
    }

    fn process_raw(self) -> i32 {
        let (total, seeds) = process_seeds_raw(self.seeds);
        tqdm!(seeds, total = total as usize)
            .map(move |seed| {
                self.locations
                    .iter()
                    .fold(seed, |seed, location| location.map(seed))
            })
            .min()
            .unwrap()
    }

    fn process_range(&self) -> i32 {
        let intervals: Vec<Range<i32>> = self
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
    println!("Part 02: {}", almanach.process_range());
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
        let almanach = Almanach::parse(input);
        assert_eq!(almanach.process_raw(), 35);
    }

    #[test]
    fn test_p2() {
        let input = fs::read_to_string("./data/test_input.txt").expect("failed to read input");
        let almanach = Almanach::parse(input);
        assert_eq!(almanach.process_range(), 46);
    }

    #[test]
    fn test_map_range() {
        let input = fs::read_to_string("./data/test_input.txt").expect("failed to read input");
        let almanach = Almanach::parse(input);
        let intervals: Vec<Range<i32>> = almanach
            .seeds
            .clone()
            .into_iter()
            .tuples()
            .map(|(a, b)| a..a + b)
            .collect();
        let results = almanach
            .locations
            .iter()
            .fold(intervals, |intervals, location| {
                location.map_ranges(intervals)
            });
        println!("{:?}", results)
    }
}
