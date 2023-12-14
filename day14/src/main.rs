use std::collections::HashMap;

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
enum Square {
    Empty,
    MoveableBlock,
    StaticBlock,
}

#[derive(PartialEq, Eq, Hash)]
enum Dir {
    North,
    West,
    South,
    East,
}
use Dir::*;

use Square::*;

#[derive(Debug)]
struct Grid {
    grid: Vec<Vec<Square>>,
}

fn parse(s: &str) -> Grid {
    let grid: Vec<Vec<Square>> = s
        .trim()
        .lines()
        .map(|line| {
            line.chars()
                .map(|c| match c {
                    '.' => Empty,
                    'O' => MoveableBlock,
                    '#' => StaticBlock,
                    _ => panic!("Unknown char"),
                })
                .collect()
        })
        .collect();

    Grid { grid }
}

fn to_string(grid: &Grid) -> String {
    let mut result = vec![];

    for line in &grid.grid {
        let chars = line.iter().map(|c| match c {
            Empty => '.',
            MoveableBlock => 'O',
            StaticBlock => '#',
        });
        result.extend(chars);
        result.push('\n');
    }
    result.iter().collect()
}

fn move_block_if_moveable(grid: &mut Grid, position: (usize, usize), dir: &Dir) {
    let (row_id, col_id) = position;

    if grid.grid[row_id][col_id] != MoveableBlock {
        // Nothing to do, this block isn't moveable.
        return;
    }

    // Find the place it should fall to.
    let move_in_dir = |position: &(usize, usize)| -> Option<(usize, usize)> {
        let x = position.0 as i64;
        let y = position.1 as i64;

        let (x, y) = match dir {
            North => (x - 1, y),
            South => (x + 1, y),
            West => (x, y - 1),
            East => (x, y + 1),
        };

        // Bounds check
        if x < 0 || y < 0 || (x as usize) >= grid.grid.len() || (y as usize) >= grid.grid[0].len() {
            return None;
        }

        // Is empty?
        let x = x as usize;
        let y = y as usize;
        if grid.grid[x][y] != Empty {
            return None;
        }

        return Some((x, y));
    };

    let mut target_position = position.clone();
    let target_position = loop {
        let next_ = move_in_dir(&target_position);
        if next_.is_some() {
            target_position = next_.unwrap();
        } else {
            break target_position;
        }
    };

    grid.grid[row_id][col_id] = Empty;
    grid.grid[target_position.0][target_position.1] = MoveableBlock;
}

fn move_all_blocks(grid: &mut Grid, dir: &Dir) {
    // Move all blocks in this dir.

    // Iterate through the blocks in the right order! So if we're moving North, we need to go top-to-bottom, etc.
    // This is a bit silly - I should precache this so I don't recompute it each time - but whatever.
    let mut positions: Vec<(usize, usize)> = (0..grid.grid.len())
        .flat_map(|x| (0..grid.grid[0].len()).map(move |y| (x, y)))
        .collect();

    let sort_key = |&p: &(usize, usize)| -> i64 {
        let (x, y) = p;
        match dir {
            North => x as i64,
            South => (x as i64) * -1,
            West => y as i64,
            East => (y as i64) * -1,
        }
    };

    positions.sort_by_key(sort_key);

    for p in positions {
        move_block_if_moveable(grid, p, dir);
    }
}

fn load(grid: &Grid) -> usize {
    let mut total = 0;
    for (i, line) in grid.grid.iter().enumerate() {
        let height = grid.grid.len() - i;
        let number_of_blocks = line.iter().filter(|c| **c == MoveableBlock).count();
        total += height * number_of_blocks;
    }
    total
}

fn solve_part1(grid: &mut Grid) -> usize {
    move_all_blocks(grid, &North);
    load(grid)
}

fn solve_part2(grid: &mut Grid) -> usize {
    let mut state_to_index: HashMap<String, usize> = HashMap::new();

    let mut number_of_cycles = 0usize;

    // Loop until we find a cycle, and return the cycle length from here.
    let cycle_length = loop {
        move_all_blocks(grid, &North);
        move_all_blocks(grid, &West);
        move_all_blocks(grid, &South);
        move_all_blocks(grid, &East);

        number_of_cycles += 1;

        let s = to_string(grid);
        if state_to_index.contains_key(&s) {
            println!("OK, found a cycle. So I can extend us to there.");
            let old_index = state_to_index.get(&s).unwrap();
            let cycle_length = number_of_cycles - old_index;
            break cycle_length;
        }

        state_to_index.insert(s, number_of_cycles);
    };

    println!("Found a cycle of length {}", cycle_length);
    let target = 1_000_000_000usize;

    number_of_cycles += ((target - number_of_cycles) / cycle_length) * cycle_length;
    assert!(number_of_cycles <= target);

    while number_of_cycles < target {
        move_all_blocks(grid, &North);
        move_all_blocks(grid, &West);
        move_all_blocks(grid, &South);
        move_all_blocks(grid, &East);

        number_of_cycles += 1;
    }

    load(grid)
}

fn main() {
    let mut grid = parse(include_str!("input"));

    println!("{:?}", solve_part1(&mut grid));
    println!("{}", to_string(&grid));

    let mut grid2 = parse(include_str!("input"));
    println!("{:?}", solve_part2(&mut grid2));
}
