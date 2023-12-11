use std::collections::HashMap;
use std::collections::HashSet;

fn expand_grid_and_get_galaxies(
    grid: &Vec<Vec<bool>>,
    expansion_amount: usize,
) -> HashSet<(usize, usize)> {
    // Expand grid. Because of part 2, representing this as a grid no longer works!
    let height = grid.len();
    let width = grid[0].len();

    // Find empty rows and empty columns
    let empty_rows: Vec<usize> = (0..height)
        .filter(|i| {
            let any_nonempty = (0..width).any(|j| grid[*i][j]);
            !any_nonempty
        })
        .collect();
    let empty_cols: Vec<usize> = (0..width)
        .filter(|j| {
            let any_nonempty = (0..height).any(|i| grid[i][*j]);
            !any_nonempty
        })
        .collect();

    // Create a mapping from old row/column to new row/column.
    let mut old_to_new_row: HashMap<usize, usize> = HashMap::new();
    let mut old_to_new_col: HashMap<usize, usize> = HashMap::new();

    for i in 0..height {
        // The location this maps to in the new grid is the number of empty rows before this one.
        // (This is quadratic, I should keep a running total, but whatever.)
        let real_i = i + (empty_rows.iter().filter(|j| **j < i).count() * expansion_amount);
        old_to_new_row.insert(i, real_i);
    }
    for i in 0..width {
        let real_i = i + (empty_cols.iter().filter(|j| **j < i).count() * expansion_amount);
        old_to_new_col.insert(i, real_i);
    }

    let mut result = HashSet::new();

    for i in 0..height {
        for j in 0..width {
            if grid[i][j] {
                let new_i = old_to_new_row
                    .get(&i)
                    .expect("BUG: Couldn't find mapping for row");
                let new_j = old_to_new_col
                    .get(&j)
                    .expect("BUG: Couldn't find mapping for col");

                result.insert((*new_i, *new_j));
            }
        }
    }

    result
}

fn solve(s: &str, expansion_amount: usize) -> usize {
    // Parse the grid.
    let mut grid = vec![];
    for line in s.lines() {
        let line = line
            .chars()
            .map(|c| match c {
                '#' => true,
                '.' => false,
                _ => panic!("Couldn't recognise character"),
            })
            .collect();
        grid.push(line)
    }

    // Expand the grid and get galaxies
    let galaxies = expand_grid_and_get_galaxies(&grid, expansion_amount);

    // Find distances between each galaxy, sum them up, return!
    let mut result = 0;
    for (x1, x2) in &galaxies {
        for (y1, y2) in &galaxies {
            result += x1.abs_diff(*y1) + x2.abs_diff(*y2);
        }
    }

    // And divide by 2, because we counted each pair twice above.
    result / 2
}

fn main() {
    let s = include_str!("input").trim();
    let result1 = solve(s, 1);
    println!("{}", result1);

    let result1 = solve(s, 1_000_000 - 1);
    println!("{}", result1);
}
