#![allow(dead_code)]

use std::fs;

use anyhow::Result;
use geo::Contains;
use geo_types::{LineString, Point, Polygon};
use itertools::Itertools;
use ndarray::Array2;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum Direction {
    North,
    South,
    East,
    West,
}

impl Direction {
    fn opposite(self) -> Direction {
        match self {
            Direction::North => Direction::South,
            Direction::South => Direction::North,
            Direction::East => Direction::West,
            Direction::West => Direction::East,
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum PipeKind {
    P([Direction; 2]),
    G,
    S,
}

impl From<char> for PipeKind {
    fn from(c: char) -> Self {
        match c {
            '|' => PipeKind::P([Direction::North, Direction::South]),
            '-' => PipeKind::P([Direction::West, Direction::East]),
            'L' => PipeKind::P([Direction::North, Direction::East]),
            'J' => PipeKind::P([Direction::North, Direction::West]),
            '7' => PipeKind::P([Direction::South, Direction::West]),
            'F' => PipeKind::P([Direction::South, Direction::East]),
            'S' => PipeKind::S,
            '.' => PipeKind::G,
            _ => panic!("Invalid direction"),
        }
    }
}

impl PipeKind {
    fn traverse(&self, from: Direction) -> Option<Direction> {
        match self {
            PipeKind::P([d1, d2]) => {
                if from != *d1 && from != *d2 {
                    None
                } else {
                    Some(if from == *d1 { *d2 } else { *d1 })
                }
            }
            _ => None,
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
struct Pipe {
    kind: PipeKind,
    pos: (usize, usize),
}

impl Pipe {
    fn new(kind: PipeKind, pos: (usize, usize)) -> Self {
        Pipe { kind, pos }
    }
}

#[derive(Debug)]
struct Field {
    pipes: Array2<Pipe>,
}

impl Field {
    fn new(pipes: Array2<Pipe>) -> Self {
        Field { pipes }
    }

    fn parse(input: &str) -> Result<Self> {
        let nrow = input.lines().count();
        let flat_pipes: Vec<PipeKind> = input
            .lines()
            .flat_map(|s| s.chars())
            .map(PipeKind::from)
            .collect();
        let ncol = flat_pipes.len() / nrow;
        let pipes_kind = Array2::from_shape_vec((nrow, ncol), flat_pipes)?;
        let pipes = Array2::from_shape_vec(
            (nrow, ncol),
            pipes_kind
                .indexed_iter()
                .map(|((i, j), p)| Pipe::new(*p, (i, j)))
                .collect(),
        )?;

        Ok(Field::new(pipes))
    }

    fn find_start(&self) -> Option<&Pipe> {
        self.pipes.iter().find(|p| matches!(p.kind, PipeKind::S))
    }

    fn get_neighbor(&self, p: &Pipe, dir: Direction) -> Option<&Pipe> {
        let (i, j) = p.pos;
        let (ni, nj) = match dir {
            Direction::North => (i.checked_sub(1)?, j),
            Direction::South => (i + 1, j),
            Direction::East => (i, j + 1),
            Direction::West => (i, j.checked_sub(1)?),
        };
        self.pipes.get((ni, nj))
    }

    fn get_next_direction(&self, neighbor: &Pipe, dir: Direction) -> Option<Direction> {
        match neighbor.kind {
            PipeKind::G => None,
            PipeKind::S => None,
            PipeKind::P([_, _]) => neighbor.kind.traverse(dir.opposite()),
        }
    }

    fn get_next_step(&self, p: &Pipe, dir: Direction) -> Option<(&Pipe, Direction)> {
        let neighbor = self.get_neighbor(p, dir)?;
        let next_dir = self.get_next_direction(neighbor, dir)?;
        Some((neighbor, next_dir))
    }

    fn startpos2startdir(&self, start: &Pipe) -> Vec<(&Pipe, Direction)> {
        let directions = [
            Direction::North,
            Direction::South,
            Direction::East,
            Direction::West,
        ];
        let directions: Vec<(&Pipe, Direction)> = directions
            .into_iter()
            .filter_map(|dir| self.get_next_step(start, dir))
            .collect();

        directions
    }

    fn follow_paths(&self, p: &Pipe, dir: Direction) -> Vec<Pipe> {
        let mut p = p;
        let mut nodes = vec![*p];
        let mut dir = dir;
        while let Some((next_p, next_dir)) = self.get_next_step(p, dir) {
            p = next_p;
            dir = next_dir;
            nodes.push(*p);
        }
        nodes
    }

    fn get_poly_path(&self, path: &[Pipe]) -> Polygon<f64> {
        let coords: Vec<(f64, f64)> = path
            .iter()
            .map(|p| (p.pos.0 as f64, p.pos.1 as f64))
            .collect();
        Polygon::new(LineString::from(coords), vec![])
    }

    fn n_pipes_in_path(&self, path: &[Pipe]) -> usize {
        let poly = self.get_poly_path(path);
        self.pipes
            .iter()
            .filter(|p| !path.contains(p))
            .map(|p| (p.pos.0 as f64, p.pos.1 as f64))
            .filter(|p| poly.contains(&Point::new(p.0, p.1)))
            .collect_vec()
            .len()
    }
}

fn part_1(field: &Field) -> usize {
    let start_pos = field.find_start().unwrap();
    let dirs = field.startpos2startdir(start_pos);
    let (p, dir) = dirs[0];
    field.follow_paths(p, dir).len() / 2 + 1
}

fn part_2(field: &Field) -> usize {
    let start_pos = field.find_start().unwrap();
    let dirs = field.startpos2startdir(start_pos);
    let (p, dir) = dirs[0];
    let path = field.follow_paths(p, dir);
    field.n_pipes_in_path(&path)
}

fn main() {
    let input = fs::read_to_string("data/input.txt").unwrap();
    let field = Field::parse(&input).unwrap();
    println!("Part 1: {}", part_1(&field));
    println!("Part 2: {}", part_2(&field));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_p1_1() {
        let input = ".....
.S-7.
.|.|.
.L-J.
.....";
        let field = Field::parse(input).unwrap();
        assert_eq!(4, part_1(&field));
    }

    #[test]
    fn test_p1_2() {
        let input = "..F7.
.FJ|.
SJ.L7
|F--J
LJ...";
        let field = Field::parse(input).unwrap();
        assert_eq!(8, part_1(&field));
    }

    #[test]
    fn test_p2_1() {
        let input = "...........
.S-------7.
.|F-----7|.
.||.....||.
.||.....||.
.|L-7.F-J|.
.|..|.|..|.
.L--J.L--J.
...........";
        let field = Field::parse(input).unwrap();
        assert_eq!(4, part_2(&field));
    }

    #[test]
    fn test_p2_2() {
        let input = "..........
.S------7.
.|F----7|.
.||....||.
.||....||.
.|L-7F-J|.
.|..||..|.
.L--JL--J.
..........";
        let field = Field::parse(input).unwrap();
        assert_eq!(4, part_2(&field));
    }

    #[test]
    fn test_p2_3() {
        let input = "FF7FSF7F7F7F7F7F---7
L|LJ||||||||||||F--J
FL-7LJLJ||||||LJL-77
F--JF--7||LJLJ7F7FJ-
L---JF-JLJ.||-FJLJJ7
|F|F-JF---7F7-L7L|7|
|FFJF7L7F-JF7|JL---7
7-L-JL7||F7|L7F-7F7|
L.L7LFJ|||||FJL7||LJ
L7JLJL-JLJLJL--JLJ.L";
        let field = Field::parse(input).unwrap();
        assert_eq!(10, part_2(&field));
    }
}
