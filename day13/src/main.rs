use std::collections::HashMap;

#[derive(Clone)]
struct Grid(Vec<Vec<char>>);

fn parse_grid(s: &str) -> Grid {
    Grid(s.lines().map(|line| line.chars().collect()).collect())
}

#[derive(Debug)]
struct GridLineIds {
    // A map from row or column numbers -> ID. Two values have the same key iff they're the same string.
    by_row: HashMap<usize, usize>,
    by_col: HashMap<usize, usize>,
}

fn to_ids(g: &Grid) -> GridLineIds {
    let mut string_to_row_id = HashMap::new();
    let mut string_to_col_id = HashMap::new();
    let mut by_row = HashMap::new();
    let mut by_col = HashMap::new();

    // Insert strings by row.
    for (row_id, row) in g.0.iter().enumerate() {
        let string: String = row.iter().collect();
        let size = string_to_row_id.len();
        let resolved_row_id = string_to_row_id.entry(string).or_insert(size);
        by_row.insert(row_id, *resolved_row_id);
    }

    // Insert strings by column
    for col_id in 0..g.0[0].len() {
        let string: String = (0..g.0.len()).map(|j| g.0[j][col_id]).collect();
        let size = string_to_col_id.len();
        let resolved_col_id = string_to_col_id.entry(string).or_insert(size);
        by_col.insert(col_id, *resolved_col_id);
    }

    GridLineIds { by_row, by_col }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
enum Dir {
    Horizontal,
    Vertical,
}

fn is_reflection_line(g: &GridLineIds, index: usize, dir: &Dir) -> bool {
    // index is the value just *after* the reflection line - equivalently, the number
    // of lines on one side of the reflection line.

    let ids = {
        match dir {
            Dir::Horizontal => &g.by_row,
            Dir::Vertical => &g.by_col,
        }
    };

    if index <= 0 || index >= ids.len() {
        // We have to reflect at least one line.
        panic!("Index out of bounds in [is_reflection_line]")
    }

    // All of the indices before index need to be reflected correctly.
    (0..index)
        .map(|x| {
            let reflected_x = 2 * index - 1 - x;

            if reflected_x >= ids.len() {
                // Nothing to check - this point would reflect off the graph.
                true
            } else {
                ids[&x] == ids[&reflected_x]
            }
        })
        .all(|x| x)
}

fn find_reflection_line(g: &Grid) -> Vec<(usize, Dir)> {
    let ids = to_ids(g);

    let mut result = vec![];

    let results_horizontal = (1..g.0.len()).filter_map(|i| {
        let dir = Dir::Horizontal;
        if is_reflection_line(&ids, i, &dir) {
            Some((i, dir))
        } else {
            None
        }
    });
    let results_vertical = (1..g.0[0].len()).filter_map(|i| {
        let dir = Dir::Vertical;
        if is_reflection_line(&ids, i, &dir) {
            Some((i, dir))
        } else {
            None
        }
    });
    result.extend(results_horizontal);
    result.extend(results_vertical);
    result
}

fn find_other_reflection_line(g: &Grid) -> Vec<(usize, Dir)> {
    // There is exactly one point in the grid that can be flipped to get a different reflection line. Find it.
    let original_line = find_reflection_line(g);

    // Not a smart way, just brute-force each other position, and find which one works. A better way would be
    // to identify which things are "almost" reflection lines, to filter down faster, but this is easily
    // fast enough in a compiled language.
    let mut grid = g.clone();
    let mut result = vec![];

    for row_id in 0..grid.0.len() {
        for col_id in 0..grid.0[0].len() {
            // Flip this point, and check it. I'm lazy - I try both possibilities, even though one of
            // them will have been what's already in the grid.
            grid.0[row_id][col_id] = '#';

            let lines: Vec<(usize, Dir)> = find_reflection_line(&grid)
                .iter()
                .filter(|x| !original_line.contains(x))
                .copied()
                .collect();

            grid.0[row_id][col_id] = '.';
            let lines2: Vec<(usize, Dir)> = find_reflection_line(&grid)
                .iter()
                .filter(|x| !original_line.contains(x))
                .copied()
                .collect();

            grid.0[row_id][col_id] = g.0[row_id][col_id];

            result.extend(lines);
            result.extend(lines2);
        }
    }

    result.dedup();
    result
}

fn line_to_score(x: usize, dir: &Dir) -> usize {
    let n = x * match dir {
        Dir::Horizontal => 100,
        Dir::Vertical => 1,
    };
    n
}

fn main() {
    let cases: Vec<Grid> = include_str!("input")
        .split("\n\n")
        .map(parse_grid)
        .collect();

    let mut total = 0;

    for case in &cases {
        let reflection_lines = find_reflection_line(case);

        match reflection_lines.len() {
            1 => {
                let (index, dir) = &reflection_lines[0];
                total += line_to_score(*index, dir);
            }
            _ => panic!("Didn't find a unique reflection line"),
        }
    }
    println!("Result for part 1: {}", total);

    let mut total = 0;
    for case in &cases {
        let reflection_lines = find_other_reflection_line(case);
        match reflection_lines.len() {
            1 => {
                let (index, dir) = &reflection_lines[0];
                total += line_to_score(*index, dir);
            }
            _ => panic!("Didn't find a unique reflection line"),
        }
    }
    println!("Result for part 2: {}", total);
}
