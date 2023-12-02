#![allow(unused, dead_code)]
use regex::{Regex, RegexSet};
use std::{collections::HashMap, fs};

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
struct CalibrationDigits {
    position: usize,
    digit: u32,
}

impl CalibrationDigits {
    fn new(position: usize, digit: u32) -> Self {
        Self { position, digit }
    }
}

impl PartialOrd for CalibrationDigits {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.position.cmp(&other.position))
    }
}

impl Ord for CalibrationDigits {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.position.cmp(&other.position)
    }
}

const WORD_DIGIT: [&str; 9] = [
    "one", "two", "three", "four", "five", "six", "seven", "eight", "nine",
];

fn detect_word_digits(row: &str) -> Vec<CalibrationDigits> {
    let mut result: Vec<CalibrationDigits> = Vec::new();
    for i in 0..row.len() {
        let substr = &row[i..];
        for (digit, word) in (1..=9).zip(WORD_DIGIT) {
            if substr.starts_with(word) {
                result.push(CalibrationDigits::new(i, digit));
            }
        }
    }
    result
}

fn detect_digits(row: &str) -> Vec<CalibrationDigits> {
    let pattern = Regex::new(r"\d").unwrap();
    pattern
        .find_iter(row)
        .map(|m| CalibrationDigits::new(m.start(), m.as_str().parse().unwrap()))
        .collect()
}

fn parse_row(row: &str) -> Option<u32> {
    let mut calibration_results: Vec<CalibrationDigits> = Vec::new();
    calibration_results.extend(detect_digits(row));
    calibration_results.extend(detect_word_digits(row));
    calibration_results.sort();
    let first_digit = calibration_results.first()?.digit;
    let last_digit = calibration_results.last()?.digit;

    let mut str_result = String::new();
    str_result.push_str(first_digit.to_string().as_str());
    str_result.push_str(last_digit.to_string().as_str());

    str_result.parse().ok()
}

fn main() {
    let contents =
        fs::read_to_string("./data/input.txt").expect("Should have been able to read the file");

    let total_cal: u32 = contents
        .split('\n')
        .map(parse_row)
        .sum::<Option<u32>>()
        .unwrap();
    println!("{}", total_cal);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_row() {
        let results = [
            ("1abc2", Some(12)),
            ("pqr3stu8vwx", Some(38)),
            ("a1b2c3d4e5f", Some(15)),
            ("treb7uchet", Some(77)),
            ("1oneninegspfm3four43", Some(13)),
        ];
        for (row, expected) in results {
            assert_eq!(parse_row(row), expected);
        }
    }

    #[test]
    fn test_parse_row_word_digits() {
        let results = [
            ("two1nine", Some(29)),
            ("eightwothree", Some(83)),
            ("abcone2threexyz", Some(13)),
            ("xtwone3four", Some(24)),
            ("4nineeightseven2", Some(42)),
            ("zoneight234", Some(14)),
            ("7pqrstsixteen", Some(76)),
            ("1oneninegspfm3four43one", Some(11)),
            ("1oneninegspfm3four43eightwo", Some(12)),
        ];
        for (row, expected) in results {
            assert_eq!(parse_row(row), expected);
        }
    }
}
