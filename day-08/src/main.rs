#![allow(dead_code)]

use num::Integer;
use std::{collections::HashMap, fs};

use itertools::Itertools;
use regex::Regex;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct Node {
    name: String,
    left: String,
    right: String,
}

impl Node {
    fn new(name: String, left: String, right: String) -> Self {
        Self { name, left, right }
    }

    fn parse(input: &str) -> Self {
        let re = Regex::new(r"(\w+) = \((\w+), (\w+)\)").unwrap();
        let cap = re.captures(input).expect("Invalid input");
        let (name, left, right) = cap
            .iter()
            .skip(1)
            .map(|x| x.unwrap().as_str().to_string())
            .next_tuple()
            .unwrap();
        Self::new(name, left, right)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum Direction {
    Left,
    Right,
}

impl From<char> for Direction {
    fn from(input: char) -> Self {
        match input {
            'L' => Self::Left,
            'R' => Self::Right,
            _ => unreachable!(),
        }
    }
}

#[derive(Debug, Clone, Default)]
struct Network {
    path: Vec<Direction>,
    nodes: HashMap<String, Node>,
}

impl Network {
    fn new(path: Vec<Direction>, nodes: Vec<Node>) -> Self {
        Self {
            path,
            nodes: HashMap::from_iter(
                nodes
                    .iter()
                    .map(|node| (node.name.to_string(), node.clone())),
            ),
        }
    }

    fn parse(input: &str) -> Self {
        let mut lines = input.lines();
        let directions: Vec<Direction> = lines
            .next()
            .expect("Invalid input")
            .chars()
            .map_into()
            .collect_vec();
        let nodes = lines.skip(1).map(Node::parse).collect();
        Self::new(directions, nodes)
    }

    fn from_file(path: &str) -> Self {
        Network::parse(fs::read_to_string(path).unwrap().as_str())
    }

    fn get_node(&self, name: &str) -> &Node {
        self.nodes.get(name).unwrap()
    }

    fn find_next(&self, name: &str, direction: &Direction) -> String {
        let node = self.get_node(name);
        match direction {
            Direction::Left => node.left.clone(),
            Direction::Right => node.right.clone(),
        }
    }

    fn walk_from(&self, start: &str, target: &str) -> usize {
        let mut current_node = start.to_string();
        for (step, direction) in self.path.iter().cycle().enumerate() {
            println!("step {}: {:?} ({:?})", step, current_node, direction);
            current_node = self.find_next(&current_node, direction);
            println!("step {}: {}", step + 1, current_node);
            if current_node == target {
                return step + 1;
            }
        }
        unreachable!();
    }

    fn walk_ghosts(&self, start: &str) -> usize {
        let mut current_node = start.to_string();
        for (step, direction) in self.path.iter().cycle().enumerate() {
            println!("step {}: {:?} ({:?})", step, current_node, direction);
            current_node = self.find_next(&current_node, direction);
            println!("step {}: {}", step + 1, current_node);
            if current_node.ends_with('Z') {
                return step + 1;
            }
        }
        unreachable!();
    }

    fn find_steps_ghosts(&self) -> usize {
        self.nodes
            .keys()
            .filter(|x| x.ends_with('A'))
            .map(|x| x.to_string())
            .map(|x| self.walk_ghosts(&x))
            .fold(1, |a, b| a.lcm(&b))
    }
}

fn main() {
    let network = Network::from_file("data/input.txt");
    println!("Part 1: {}", network.walk_from("AAA", "ZZZ"));

    println!("\n");

    println!("Part 2: {}", network.find_steps_ghosts());
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_part1() {
        let network = Network::from_file("data/test_input_1.txt");
        assert_eq!(network.walk_from("AAA", "ZZZ"), 2);

        let network = Network::from_file("data/test_input_2.txt");
        assert_eq!(network.walk_from("AAA", "ZZZ"), 6);
    }

    #[test]
    fn test_part2() {
        let network = Network::from_file("data/test_input_3.txt");
        assert_eq!(6, network.find_steps_ghosts());
    }
}
