#![allow(dead_code)]
#![feature(tuple_trait)]

use std::fs;

use itertools::Itertools;

#[derive(Debug, PartialEq)]
struct RaceRecord {
    time: u64,
    rec_distance: u64,
}

#[derive(Debug)]
enum Errors {
    InvalidInput,
    InvalidTime,
}

impl RaceRecord {
    fn new(time: u64, distance: u64) -> RaceRecord {
        RaceRecord {
            time,
            rec_distance: distance,
        }
    }

    fn perform(&self, hold_time: u64) -> Result<u64, Errors> {
        if self.time < hold_time {
            return Err(Errors::InvalidTime);
        }
        let speed = hold_time;
        let remain_time = self.time - hold_time;
        Ok(speed * remain_time)
    }

    fn best_record_possible(&self) -> u64 {
        (self.time / 2) * (self.time - self.time / 2)
    }

    fn nth_break_dist_rec(&self) -> u64 {
        let all_hold_time: Vec<u64> = (0..=self.time).collect();
        let half_best = all_hold_time[all_hold_time.len() / 2..]
            .iter()
            .map(|hold_time| self.perform(*hold_time).unwrap())
            .take_while(|&r| r > self.rec_distance)
            .count() as u64;
        half_best * 2 - (self.time + 1) % 2
    }
}

#[derive(Debug)]
struct Races {
    records: Vec<RaceRecord>,
}

impl From<&str> for Races {
    fn from(input: &str) -> Self {
        Self::parse(input)
    }
}

impl From<String> for Races {
    fn from(input: String) -> Self {
        Self::parse(&input)
    }
}

impl From<Vec<RaceRecord>> for Races {
    fn from(input: Vec<RaceRecord>) -> Self {
        Self::new(input)
    }
}

impl Races {
    fn new(records: Vec<RaceRecord>) -> Races {
        Races { records }
    }

    fn times(&self) -> Vec<u64> {
        self.records.iter().map(|r| r.time).collect()
    }

    fn distances(&self) -> Vec<u64> {
        self.records.iter().map(|r| r.rec_distance).collect()
    }

    fn nth_break_prod(&self) -> u64 {
        self.records
            .iter()
            .map(|r| r.nth_break_dist_rec())
            .product::<u64>()
    }

    fn parse(input: &str) -> Races {
        let (times_line, records_line) = input.lines().next_tuple().unwrap();

        let times = times_line
            .strip_prefix("Time:")
            .unwrap()
            .split_whitespace()
            .map(|t| t.parse().unwrap());

        let distances = records_line
            .strip_prefix("Distance:")
            .unwrap()
            .split_whitespace()
            .map(|d| d.parse().unwrap());

        times
            .into_iter()
            .zip(distances)
            .map(|(time, distance)| RaceRecord::new(time, distance))
            .collect::<Vec<_>>()
            .into()
    }
}

fn main() {
    let races: Races = fs::read_to_string("./data/input.txt").unwrap().into();
    println!("Part 1: {}", races.nth_break_prod());

    let races: Races = fs::read_to_string("./data/input_unkerned.txt")
        .unwrap()
        .into();
    println!("Part 2: {}", races.nth_break_prod());
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::*;

    #[test]
    fn test_parse() {
        let races: Races = fs::read_to_string("./data/test_input.txt").unwrap().into();
        assert_eq!(races.times(), vec![7, 15, 30]);
        assert_eq!(races.distances(), vec![9, 40, 200]);
    }

    #[test]
    fn test_p1() {
        let races: Races = fs::read_to_string("./data/test_input.txt").unwrap().into();
        println!("{:?}", races.nth_break_prod());
    }
}
