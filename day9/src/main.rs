fn lagrange_interpolate(v: &Vec<f64>, target: f64) -> f64 {
    // Given a vector of f64's, come up with the polynomial that interpolates through (i, x_i), and evaluate
    // it at target.
    // This isn't going to be exact - will it matter? I don't know.

    let mut total = 0f64;

    for (i, x_i) in v.iter().enumerate() {
        // Compute and add on the term for the multiple of the ith Lagrange polynomial.
        // Define l_i(X) = \prod_{j \neq i} (X - j) / (i - j).
        // It's the polynomial that's 1 at i, and 0 at all of the other j
        //
        // Then the term of l_i(X) in the Lagrange interpolation is x_i.

        // This computes l_i(target)
        let l_i = (0..v.len())
            .filter(|j| *j != i)
            .map(|j| {
                let j = j as f64;
                let i = i as f64;
                (target - j) / (i - j)
            })
            .reduce(|x, y| x * y)
            .expect("");

        total += l_i * x_i;
    }
    total
}

fn main() {
    let lines: Vec<Vec<f64>> = include_str!("input")
        .lines()
        .map(|s| {
            let tokens = s
                .split_whitespace()
                .map(|s| s.parse::<f64>().expect("Failed to parse as float"))
                .collect();
            tokens
        })
        .collect();

    let result_part1: f64 = lines
        .iter()
        .map(|line| lagrange_interpolate(&line, line.len() as f64).round())
        .sum();
    println!("Result for part 1: {}", result_part1);

    let result_part2: f64 = lines
        .iter()
        .map(|line| lagrange_interpolate(&line, -1.0))
        .sum();
    println!("Result for part 2: {}", result_part2);
}
