use std::collections::HashMap;
use std::collections::HashSet;

#[derive(Debug, PartialEq, Eq, Hash, Copy, Clone)]
struct Position(i64, i64);

#[derive(Debug)]
enum Dir {
    Up,
    Down,
    Left,
    Right,
}
use Dir::*;

impl Dir {
    fn parse(s: &str) -> Self {
        match s {
            "U" => Up,
            "D" => Down,
            "L" => Left,
            "R" => Right,
            _ => panic!("Could not parse as Dir"),
        }
    }

    fn move_in_dir(&self, position: &Position, length: &usize) -> Position {
        let length = *length as i64;
        match self {
            Up => Position(position.0 - length, position.1),
            Down => Position(position.0 + length, position.1),
            Left => Position(position.0, position.1 - length),
            Right => Position(position.0, position.1 + length),
        }
    }
}

#[derive(Debug)]
struct Step {
    dir: Dir,
    length: usize,
}

impl Step {
    fn parse1(s: &str) -> Self {
        let [dir, length, _colour]: [&str; 3] = s
            .split_whitespace()
            .collect::<Vec<&str>>()
            .try_into()
            .expect("Expected 3 tokens");

        let dir = Dir::parse(dir);
        let length: usize = length.parse().expect("Couldn't parse length as a usize");

        Step { dir, length }
    }

    fn parse2(s: &str) -> Self {
        let [_dir, _length, colour]: [&str; 3] = s
            .split_whitespace()
            .collect::<Vec<&str>>()
            .try_into()
            .expect("Expected 3 tokens");

        let colour = colour
            .strip_prefix("(#")
            .unwrap()
            .strip_suffix(")")
            .unwrap();
        let length_and_dir = usize::from_str_radix(colour, 16).unwrap();
        // The length is the first 5 hex digits.
        let length = length_and_dir / 16;
        let dir = match length_and_dir % 16 {
            0 => Right,
            1 => Down,
            2 => Left,
            3 => Up,
            _ => panic!(
                "Parse error - expected last digit of colour to be 0-3. {}",
                colour
            ),
        };
        Step { dir, length }
    }
}

fn solve(steps: &[Step]) -> usize {
    /*
    This was fun!

    High-level guide: I'm going to translate the shape I get into "mini-space".
    Suppose the x co-ordinates of the knots I get are x_0, ..., x_{n-1} (in order, deduplicated).

    Then in mini-space, I'll represent these as 0, 2, ..., 2*(n-1).

    So:
    0 --> represents a box corresponding to interval [x_0, x_0] in "real space"
    1 --> represents a box corresponding to interval (x_0, x_1) in "real space"
    2 --> represents a box corresponding to interval [x_1, x_1] in "real space"
    etc. So even and odd co-ordinates are a little different.

    This gives me a grid in mini-space where mini-squares correspond to disjoint regions in real space.
    It's pretty simple to construct a formula for the real-area of a minisquare.

    Then in mini-space, we look at where the boundary is, flood fill to find what's inside/outside, and
    then convert back to find out how much real-area is contained.
    */

    // Find the boundary knots.
    let mut knots = vec![Position(0, 0)];
    let mut position = Position(0, 0);

    for Step { dir, length } in steps {
        position = dir.move_in_dir(&position, length);
        knots.push(position);
    }

    // Sanity-check - we should have ended up at the start.
    assert!(position == Position(0, 0));

    // Figure out a map converting real co-ordinates to mini-coordinates.
    // Suppose we sort the x co-ordinates of the knots in order. Then the smallest will be 0, the next smallest will be 2, etc.
    // The reason for the jumps of 2 is it makes summing up the area at the end a bit easier, because all of the resulting
    // squares I get correspond to disjoint ranges.
    let all_xs: Vec<i64> = knots.iter().map(|p| p.0).collect();
    let all_ys: Vec<i64> = knots.iter().map(|p| p.1).collect();

    let make_real_to_mini = |mut v: Vec<i64>| -> HashMap<i64, i64> {
        v.sort();
        v.dedup();

        v.into_iter()
            .enumerate()
            .map(|(i, x)| (x, (2 * i) as i64))
            .collect()
    };

    let real_to_mini_x: HashMap<i64, i64> = make_real_to_mini(all_xs);
    let real_to_mini_y: HashMap<i64, i64> = make_real_to_mini(all_ys);

    let real_to_mini =
        |p: &Position| -> Position { Position(real_to_mini_x[&p.0], real_to_mini_y[&p.1]) };

    // Only contains keys for the even co-ordinates.
    let mini_to_real_x: HashMap<i64, i64> = real_to_mini_x.iter().map(|(a, b)| (*b, *a)).collect();
    let mini_to_real_y: HashMap<i64, i64> = real_to_mini_y.iter().map(|(a, b)| (*b, *a)).collect();

    // Create a grid in mini co-ordinate space, and fill in the steps between the knots.
    let mut boundary_cells_in_mini_space: HashSet<Position> = HashSet::new();

    for (i, knot) in knots.iter().enumerate() {
        let next_knot = {
            if i == knots.len() - 1 {
                &knots[0]
            } else {
                &knots[i + 1]
            }
        };

        let Position(x0, y0) = real_to_mini(knot);
        let Position(x1, y1) = real_to_mini(next_knot);

        // Find the route between them.
        let path: Vec<Position> = if x0 == x1 {
            let min_y = y0.min(y1);
            let max_y = y0.max(y1);
            (min_y..=max_y).map(|y| Position(x0, y)).collect()
        } else if y0 == y1 {
            let min_x = x0.min(x1);
            let max_x = x0.max(x1);
            (min_x..=max_x).map(|x| Position(x, y0)).collect()
        } else {
            panic!("BUG. My knots shouldn't be not in a straight line.")
        };

        for position in path {
            boundary_cells_in_mini_space.insert(position);
        }
    }

    // Do a flood fill in mini-space to figure out which squares are full/empty.
    let mut outside_cells_in_mini_space: HashSet<Position> = HashSet::new();
    let mut explore_queue: Vec<Position> = vec![];

    let min_x = *mini_to_real_x.keys().min().unwrap();
    let max_x = *mini_to_real_x.keys().max().unwrap();
    let min_y = *mini_to_real_y.keys().min().unwrap();
    let max_y = *mini_to_real_y.keys().max().unwrap();

    let mut insert_outside_cell = |p: Position| {
        if !boundary_cells_in_mini_space.contains(&p) {
            explore_queue.push(p.clone());
            outside_cells_in_mini_space.insert(p);
        }
    };

    for x in min_x..=max_x {
        for y in vec![min_y, max_y] {
            let p = Position(x, y);
            insert_outside_cell(p);
        }
    }

    for x in vec![min_x, max_x] {
        for y in min_y..=max_y {
            let p = Position(x, y);
            insert_outside_cell(p);
        }
    }

    let neighbours = |p: &Position| -> Vec<Position> {
        let mut result = vec![];

        let mut push = |p: Position| {
            if p.0 >= min_x
                && p.0 <= max_x
                && p.1 >= min_y
                && p.1 <= max_y
                && !boundary_cells_in_mini_space.contains(&p)
            {
                result.push(p);
            }
        };

        push(Position(p.0 + 1, p.1));
        push(Position(p.0 - 1, p.1));
        push(Position(p.0, p.1 + 1));
        push(Position(p.0, p.1 - 1));

        result
    };

    while !explore_queue.is_empty() {
        let p = explore_queue.pop().unwrap();
        let neighbours = neighbours(&p);
        for n in neighbours {
            if !outside_cells_in_mini_space.contains(&n) {
                explore_queue.push(n);
                outside_cells_in_mini_space.insert(n);
            }
        }
    }

    // Now we've got all the cells in the boundary and outside, we want to sum up their area!
    // We can find the area by mapping back to "real" co-ordinates.
    let get_area_of_box = |p: &Position| -> usize {
        let Position(x, y) = p;

        let height = {
            if (x % 2) == 0 {
                1
            } else {
                mini_to_real_x[&(x + 1)] - mini_to_real_x[&(x - 1)] - 1
            }
        };
        let width = {
            if (y % 2) == 0 {
                1
            } else {
                mini_to_real_y[&(y + 1)] - mini_to_real_y[&(y - 1)] - 1
            }
        };
        assert!(height >= 0);
        assert!(width >= 0);
        (height * width) as usize
    };

    let mut total = 0;
    for x in min_x..=max_x {
        for y in min_y..=max_y {
            let position = Position(x, y);
            let inside_lake = match (
                boundary_cells_in_mini_space.contains(&position),
                outside_cells_in_mini_space.contains(&position),
            ) {
                (false, true) => {
                    // This cell is outside of the shape - found in our flood fill - so we don't have to include.
                    false
                }
                (true, false) => {
                    // On the boundary, so we should include
                    true
                }
                (false, false) => {
                    // On the inside, so should include
                    true
                }
                (true, true) => {
                    // This shouldn't happen, so would be a bug in the flood fill.
                    panic!("Shouldn't happen! We shouldn't be able to be both outside and on the boundary")
                }
            };
            if inside_lake {
                total += get_area_of_box(&position);
            }
        }
    }
    total
}

fn main() {
    let s = include_str!("input").trim();
    let steps1: Vec<Step> = s.lines().map(Step::parse1).collect();

    let result1 = solve(&steps1);
    println!("Result for part 1: {}", result1);

    let steps2: Vec<Step> = s.lines().map(Step::parse2).collect();
    let result2 = solve(&steps2);
    println!("Result for part 2: {}", result2);
}
