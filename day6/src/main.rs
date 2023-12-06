fn number_of_ways_beating_bound(n: usize, bound: usize) -> usize {
    // Naive way - just try all the possibilities.

    (0..=n)
        .filter(|x| {
            let m = *x;
            m * (n - m) > bound
        })
        .map(|_x| 1)
        .sum()
}

fn parse_input(s: &str) -> Vec<(usize, usize)> {
    let rows: Vec<Vec<usize>> = s
        .trim()
        .lines()
        .map(|s| s.split_whitespace().filter_map(|x| x.parse::<usize>().ok()))
        .map(|x| x.collect())
        .collect();

    assert!(rows.len() == 2, "Only expected 2 rows in input");

    (0..rows[0].len())
        .map(|i| (rows[0][i], rows[1][i]))
        .collect()
}

fn run_for_input(s: &str) -> usize {
    let rows = parse_input(s);

    let total: usize = rows
        .iter()
        .map(|(x, y)| number_of_ways_beating_bound(*x, *y))
        .reduce(|x, y| x * y)
        .expect("There should be at least one number in the input");

    total
}

fn main() {
    println!(
        "Solution for part 1: {}",
        run_for_input(include_str!("input"))
    );

    // I just manually edited the file for part 2. I feel like I could have done something
    // cleverer for this (with roots of a quadratic formula), but doing the naive thing with
    // --release still runs instantly.
    println!(
        "Solution for part 2: {}",
        run_for_input(include_str!("input2"))
    );
}
