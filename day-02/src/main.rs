use std::fs;

use regex::Regex;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
struct ColorRecord {
    red: Option<u32>,
    green: Option<u32>,
    blue: Option<u32>,
}

impl ColorRecord {
    fn new(red: Option<u32>, green: Option<u32>, blue: Option<u32>) -> Self {
        Self { red, green, blue }
    }

    fn parse(input: &str) -> ColorRecord {
        let red_pattern = Regex::new(r"(?<count>\d+) red").unwrap();
        let green_pattern = Regex::new(r"(?<count>\d+) green").unwrap();
        let blue_pattern = Regex::new(r"(?<count>\d+) blue").unwrap();
        let get_count = |c: regex::Captures| c["count"].parse().unwrap();
        let red_count = red_pattern.captures(input).map(get_count);
        let green_count = green_pattern.captures(input).map(get_count);
        let blue_count = blue_pattern.captures(input).map(get_count);
        ColorRecord::new(red_count, green_count, blue_count)
    }

    fn possible(&self, max_cubes: &ColorRecord) -> bool {
        (self.red <= max_cubes.red)
            && (self.green <= max_cubes.green)
            && (self.blue <= max_cubes.blue)
    }

    fn power(&self) -> u32 {
        self.red.unwrap_or_default()
            * self.green.unwrap_or_default()
            * self.blue.unwrap_or_default()
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
struct GameRecord {
    id: u32,
    records: Vec<ColorRecord>,
}

impl GameRecord {
    fn new(id: u32, records: Vec<ColorRecord>) -> Self {
        Self { id, records }
    }

    fn parse(input: &str) -> Option<GameRecord> {
        let row_pattern = Regex::new(r"^Game (?<ID>\d+): (?<records>.*)$").unwrap();
        let caps = row_pattern.captures(input)?;

        let record = GameRecord::new(
            caps["ID"].parse().unwrap(),
            caps["records"].split(';').map(ColorRecord::parse).collect(),
        );
        Some(record)
    }

    fn max(&self) -> ColorRecord {
        self.records
            .iter()
            .fold(ColorRecord::new(None, None, None), |acc, x| {
                ColorRecord::new(
                    acc.red.max(x.red),
                    acc.green.max(x.green),
                    acc.blue.max(x.blue),
                )
            })
    }
    fn max_power(&self) -> u32 {
        self.max().power()
    }
}

impl Iterator for GameRecord {
    type Item = ColorRecord;
    fn next(&mut self) -> Option<Self::Item> {
        self.records.pop()
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
struct GameRecords(Vec<GameRecord>);

impl GameRecords {
    fn new(records: Vec<GameRecord>) -> Self {
        Self(records)
    }

    fn iter(&self) -> impl Iterator<Item = &GameRecord> {
        self.0.iter()
    }
}

impl From<&str> for GameRecords {
    fn from(input: &str) -> Self {
        let mut records = Vec::new();
        for line in input.lines() {
            if let Some(record) = GameRecord::parse(line) {
                records.push(record);
            }
        }
        GameRecords::new(records)
    }
}

impl From<String> for GameRecords {
    fn from(input: String) -> Self {
        GameRecords::from(input.as_str())
    }
}

impl From<Vec<GameRecord>> for GameRecords {
    fn from(records: Vec<GameRecord>) -> Self {
        GameRecords::new(records)
    }
}

fn get_possible_games(game_records: &GameRecords, max_cubes: ColorRecord) -> Vec<u32> {
    let max_by_game: Vec<(u32, ColorRecord)> =
        game_records.iter().map(|x| (x.id, x.max())).collect();
    let possible_games: Vec<(u32, ColorRecord)> = max_by_game
        .into_iter()
        .filter(|x| x.1.possible(&max_cubes))
        .collect();
    possible_games.iter().map(|x| x.0).collect()
}

fn main() {
    let game_records: GameRecords = fs::read_to_string("./data/input.txt").unwrap().into();
    let max_cubes = ColorRecord::new(Some(12), Some(13), Some(14));
    let possible_ids = get_possible_games(&game_records, max_cubes);
    println!(
        "Sum of possible IDs: {:?}",
        possible_ids.iter().sum::<u32>()
    );

    let max_power = game_records.iter().map(|x| x.max_power()).sum::<u32>();
    println!("Max power: {:?}", max_power);
}

#[test]
fn test_parse_row() {
    let input = "Game 1: 3 blue, 4 red; 1 red, 2 green, 6 blue; 2 green";
    let expected = GameRecord::new(
        1,
        vec![
            ColorRecord::new(Some(4), None, Some(3)),
            ColorRecord::new(Some(1), Some(2), Some(6)),
            ColorRecord::new(None, Some(2), None),
        ],
    );
    assert_eq!(GameRecord::parse(input), Some(expected));
}

#[test]
fn test_fake_record() {
    let game_records: GameRecords = fs::read_to_string("./data/test_record.txt").unwrap().into();
    let max_cubes = ColorRecord::new(Some(12), Some(13), Some(14));
    let possible_ids = get_possible_games(&game_records, max_cubes);
    let total_possible = possible_ids.iter().sum::<u32>();
    assert_eq!(total_possible, 8);
}

#[test]
fn test_fake_record_power() {
    let game_records: GameRecords = fs::read_to_string("./data/test_record.txt").unwrap().into();
    let max_power = game_records.iter().map(|x| x.max_power()).sum::<u32>();
    assert_eq!(max_power, 2286);
}
