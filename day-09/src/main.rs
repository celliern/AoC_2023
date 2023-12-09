use std::fs;

fn find_diffs(history: Vec<i64>) -> Vec<Vec<i64>> {
    let mut diffs: Vec<Vec<i64>> = Vec::new();
    diffs.push(history);
    while !diffs.last().unwrap().iter().all(|x| *x == 0) {
        let last_diff = diffs.last().unwrap();
        diffs.push(
            last_diff
                .iter()
                .zip(last_diff.iter().skip(1))
                .map(|(a, b)| b - a)
                .collect::<Vec<i64>>(),
        );
    }
    diffs
}

fn extrapolate_pred(history: Vec<i64>) -> i64 {
    let diffs = find_diffs(history);
    diffs.iter().map(|x| *x.last().unwrap()).rev().sum()
}

fn extrapolate_pred_backward(history: Vec<i64>) -> i64 {
    let diffs = find_diffs(history);
    diffs
        .iter()
        .map(|x| *x.first().unwrap())
        .rev()
        .fold(0, |a, b| b - a)
}

fn main() {
    let history: Vec<Vec<i64>> = fs::read_to_string("data/input.txt")
        .unwrap()
        .lines()
        .map(|x| {
            x.split_whitespace()
                .map(|x| x.parse::<i64>().unwrap())
                .collect::<Vec<i64>>()
        })
        .collect();

    let extrapolated_data = history
        .iter()
        .map(|x| extrapolate_pred(x.to_vec()))
        .collect::<Vec<i64>>();

    let part_1: i64 = extrapolated_data.iter().sum();
    println!("Part 1: {}", part_1);

    let extrapolated_data_back = history
        .iter()
        .map(|x| extrapolate_pred_backward(x.to_vec()))
        .collect::<Vec<i64>>();

    let part_2: i64 = extrapolated_data_back.iter().sum();
    println!("Part 2: {}", part_2);
}
