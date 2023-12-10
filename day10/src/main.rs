use std::collections::HashSet;

#[derive(PartialEq, Eq, Copy, Clone, Debug)]
enum Dir {
    North,
    South,
    East,
    West,
}

use Dir::*;

impl Dir {
    fn flip(&self) -> Self {
        match self {
            North => South,
            South => North,
            East => West,
            West => East,
        }
    }
}

#[derive(Debug)]
enum Square {
    Pipe(Dir, Dir),
    Ground,
    Unknown,
}
use Square::*;

#[derive(Debug)]
struct Grid {
    rabbit_pos: (usize, usize),
    grid: Vec<Vec<Square>>,
}

enum MoveResult {
    FailedToMove,
    MovedOntoStartSpace,
    Moved {
        new_pos: (usize, usize),
        new_dir: Dir,
    },
}
use MoveResult::*;

impl Grid {
    fn move_(&self, starting_pos: (usize, usize), dir: Dir) -> MoveResult {
        // From a starting point in the grid, if you move in some direction, does that work?
        // If None, that means you can't move this way - either you've hit a wall, or a pipe that isn't connected.
        // If Some(pos, dir), then you've moved this way and maybe have changed direction.
        let (x, y) = starting_pos;

        let new_pos = match dir {
            North => (x - 1, y),
            South => (x + 1, y),
            East => (x, y + 1),
            West => (x, y - 1),
        };

        let validate = |x, lower, upper| x >= lower && x < upper;

        // Bounds check - are we inside?
        if !validate(new_pos.0, 0, self.grid.len()) {
            return FailedToMove;
        }
        if !validate(new_pos.1, 0, self.grid[0].len()) {
            return FailedToMove;
        }

        // Now check what square we're moving onto.
        match &self.grid[new_pos.0][new_pos.1] {
            Ground => FailedToMove,
            Unknown => MovedOntoStartSpace,
            Pipe(dir1, dir2) => {
                // OK, so we've hit a pipe. One of these ends needs to connect to us - the other one tells us
                // where we'll point next.
                //
                // The convention I've gone for is that the pipe ends point outwards, so I need to flip my direction.
                if *dir1 == dir.flip() {
                    Moved {
                        new_pos,
                        new_dir: *dir2,
                    }
                } else if *dir2 == dir.flip() {
                    Moved {
                        new_pos,
                        new_dir: *dir1,
                    }
                } else {
                    FailedToMove
                }
            }
        }
    }
}

#[derive(Debug)]
struct FindLoopResult {
    // The loop, starting and ending where the rabbit is
    loop_: Vec<(usize, usize)>,
}

fn find_loop(g: &Grid) -> FindLoopResult {
    let dirs = vec![North, East, South, West];

    // Plan: For each direction, start the rabbit off there. See if we ever get back to the starting point.
    for start_dir in dirs {
        let mut loop_ = vec![g.rabbit_pos];
        let mut dir = start_dir;

        // Everything has degree 2 here so I don't need to worry about getting stuck inside
        // a smaller loop - only check if I get back to the start.
        let success = loop {
            match g.move_(loop_[loop_.len() - 1], dir) {
                FailedToMove => break false,
                MovedOntoStartSpace => {
                    loop_.push(g.rabbit_pos);
                    break true;
                }
                Moved { new_pos, new_dir } => {
                    loop_.push(new_pos);
                    dir = new_dir;
                }
            }
        };

        if success {
            return FindLoopResult { loop_ };
        }
    }

    panic!("Failed to find any direction which made the rabbit run in a loop.")
}

fn parse(s: &str) -> Grid {
    let parse_char = |c| match c {
        '.' => Ground,
        '|' => Pipe(North, South),
        '-' => Pipe(West, East),
        'L' => Pipe(North, East),
        'J' => Pipe(North, West),
        '7' => Pipe(West, South),
        'F' => Pipe(South, East),
        'S' => Unknown,
        _ => panic!("Unrecognised char"),
    };

    let grid: Vec<Vec<Square>> = s
        .trim()
        .lines()
        .map(|line| line.chars().map(parse_char).collect::<Vec<Square>>())
        .collect();

    let find_rabbit = || -> (usize, usize) {
        for i in 0..grid.len() {
            for j in 0..grid[0].len() {
                match grid[i][j] {
                    Unknown => return (i, j),
                    _ => (),
                }
            }
        }
        panic!("Couldn't find unknown square in grid")
    };
    let rabbit_pos = find_rabbit();
    Grid { grid, rabbit_pos }
}

fn find_points_inside_loop(grid: &Grid, loop_: &Vec<(usize, usize)>) -> Vec<(usize, usize)> {
    // Find the points that are inside [loop_].
    // This function isn't quite right. We do a flood fill from the outside, which might mean we
    // end up including points that are inside a different loop. Hopefully the input won't have those.
    //
    // To deal with the fact that we can squeeze between pipes, we do a flood fill on a grid that's
    // twice the size. So effectively each square on the grid gets split into quarters.
    let mut big_grid = vec![];
    for line in &grid.grid {
        big_grid.push(vec![false; 2 * line.len()]);
        big_grid.push(vec![false; 2 * line.len()]);
    }

    // This is a huge block of code to identify which cells are neighbours in the big grid.
    // To do this, we need to look at where the pipes are, which might block off a section of that cell.
    let neighbours = |big_grid: &Vec<Vec<bool>>, (x, y)| -> Vec<(usize, usize)> {
        let grid_contains_pipe_with_this_dir = |x: usize, y: usize, dir| -> bool {
            match grid.grid[x][y] {
                Ground => false,
                Unknown => panic!("Should not have any unknown cells at this stage"),
                Pipe(dir1, dir2) => dir1 == dir || dir2 == dir,
            }
        };

        // Return the set of neighbours of a node in the big grid. This is quite gnarly, unfortunately.
        let even_x = x % 2 == 0;
        let even_y = y % 2 == 0;

        let mut result = vec![];

        // This bounds checks, and deals with checking the pipe.
        let push_neighbour_in_dir_if_no_pipe =
            |result: &mut Vec<(usize, usize)>, move_dir: Dir, if_no_pipe_in_dir: Option<Dir>| {
                let new_position = match move_dir {
                    North => {
                        if x != 0 {
                            Some((x - 1, y))
                        } else {
                            None
                        }
                    }
                    South => {
                        if x != big_grid.len() - 1 {
                            Some((x + 1, y))
                        } else {
                            None
                        }
                    }
                    West => {
                        if y != 0 {
                            Some((x, y - 1))
                        } else {
                            None
                        }
                    }
                    East => {
                        if y != big_grid[0].len() - 1 {
                            Some((x, y + 1))
                        } else {
                            None
                        }
                    }
                };

                match new_position {
                    None => {
                        // We'd be moving out of the grid. Don't need to do anything.
                        ()
                    }
                    Some(new_position) => {
                        let should_add = match if_no_pipe_in_dir {
                            None => {
                                // We've been told to unconditionally add this neighbour.
                                true
                            }
                            Some(dir) => !grid_contains_pipe_with_this_dir(x / 2, y / 2, dir),
                        };
                        if should_add {
                            result.push(new_position);
                        }
                    }
                }
            };

        if even_x && even_y {
            // We're in the top-left of one of the little boxes, so:
            // - We always have the top and left neighbours (assuming they're in bounds)
            // - We have the right neighbour if this cell doesn't contain a North pipe
            // - We have the down neighbour if this cell doesn't contain a West pipe
            push_neighbour_in_dir_if_no_pipe(&mut result, North, None);
            push_neighbour_in_dir_if_no_pipe(&mut result, West, None);
            push_neighbour_in_dir_if_no_pipe(&mut result, East, Some(North));
            push_neighbour_in_dir_if_no_pipe(&mut result, South, Some(West));
        } else if !even_x && even_y {
            // We're in the bottom-left of one of the cells, so:
            // - We always have the bottom and left neighbours (assuming they're in bounds)
            // - We have the right neighbour if this cell doesn't contain a South pipe
            // - We have the top neighbout if this cell doesn't contain a West pipe.
            push_neighbour_in_dir_if_no_pipe(&mut result, North, Some(West));
            push_neighbour_in_dir_if_no_pipe(&mut result, West, None);
            push_neighbour_in_dir_if_no_pipe(&mut result, East, Some(South));
            push_neighbour_in_dir_if_no_pipe(&mut result, South, None);
        } else if even_x && !even_y {
            // We're in the top-right of one of the cells, so:
            // - We always have the top and right neighbours (assuming they're in bounds)
            // - We have the left neighbour if this cell doesn't contain a North pipe.
            // - We have the down neighbour if this cell doesn't contain an East pipe.
            push_neighbour_in_dir_if_no_pipe(&mut result, North, None);
            push_neighbour_in_dir_if_no_pipe(&mut result, West, Some(North));
            push_neighbour_in_dir_if_no_pipe(&mut result, East, None);
            push_neighbour_in_dir_if_no_pipe(&mut result, South, Some(East));
        } else if !even_x && !even_y {
            // We're in the bottom-right of one of the cells, so:
            // - We always have the bottom and right neighbours (assuming they're in bounds)
            // - We have the left neighbour if this cell doesn't contain a South pipe.
            // - We have the up neighbour if this cell doesn't contain an East pipe.
            push_neighbour_in_dir_if_no_pipe(&mut result, North, Some(East));
            push_neighbour_in_dir_if_no_pipe(&mut result, West, Some(South));
            push_neighbour_in_dir_if_no_pipe(&mut result, East, None);
            push_neighbour_in_dir_if_no_pipe(&mut result, South, None);
        };
        result
    };

    // Now do a flood fill from the outside of the big grid.
    let mut in_queue: Vec<(usize, usize)> = vec![];
    let mut fully_processed: HashSet<(usize, usize)> = HashSet::new();

    // Add the border cells
    for i in 0..big_grid.len() {
        in_queue.push((i, 0));
        in_queue.push((i, big_grid[0].len() - 1));
    }
    for j in 0..big_grid[0].len() {
        in_queue.push((0, j));
        in_queue.push((big_grid.len() - 1, j));
    }

    // Do the flood fill.
    while !in_queue.is_empty() {
        let a = in_queue.pop().unwrap();
        if !fully_processed.contains(&a) {
            big_grid[a.0][a.1] = true;
            fully_processed.insert(a);
            for neighbour in neighbours(&big_grid, a) {
                in_queue.push(neighbour);
            }
        }
    }

    // And then count up which points are inside the loop! We'll assume a point is inside the loop
    // if all 4 of its quarters weren't hit by the flood fill - this should (hopefully?) exclude points
    // that are on the loop itself.
    let mut points_inside_loop = vec![];
    for x in 0..grid.grid.len() {
        for y in 0..grid.grid[0].len() {
            // Did we find any sections of this point in the flood fill?
            let flood_fill_points: usize = (vec![
                big_grid[2 * x][2 * y],
                big_grid[2 * x + 1][2 * y],
                big_grid[2 * x][2 * y + 1],
                big_grid[2 * x + 1][2 * y + 1],
            ])
            .iter()
            .map(|x| if *x { 1 } else { 0 })
            .sum();

            if flood_fill_points == 0 {
                points_inside_loop.push((x, y))
            }
        }
    }

    println!("{:?}", points_inside_loop);
    println!("{:?}", loop_);
    println!("{}", points_inside_loop.len());
    points_inside_loop
}

fn main() {
    let mut grid = parse(include_str!("input2"));

    // Part 1: Find the loop the rabbit is in, which requires figuring out the pipe
    // it's on.
    let FindLoopResult { loop_ } = find_loop(&grid);

    let max_distance = loop_
        .iter()
        .enumerate()
        .map(|(i, _point)| {
            let distance = i.min(loop_.len() - 1 - i);
            distance
        })
        .max()
        .expect("Must be at least one point in the loop.");

    println!("Max distance in loop: {}", max_distance);

    // Modify the grid so we don't have any unknown squares. Use the loop we've found to fill it in.
    fn get_direction(start_point: (usize, usize), end_point: (usize, usize)) -> Dir {
        if end_point.0 == start_point.0 + 1 {
            return South;
        }
        if end_point.0 + 1 == start_point.0 {
            return North;
        }
        if end_point.1 + 1 == start_point.1 {
            return West;
        }
        if end_point.1 == start_point.1 + 1 {
            return East;
        }
        panic!("Doesn't look like the end_point is 1 away from start_point");
    }
    // Get the outward facing directions from the starting point, and set the pipe.
    let dir1 = get_direction(grid.rabbit_pos, loop_[1]);
    let dir2 = get_direction(grid.rabbit_pos, loop_[loop_.len() - 2]);
    grid.grid[grid.rabbit_pos.0][grid.rabbit_pos.1] = Pipe(dir1, dir2);

    // Part 2: We want to do a flood fill from the outside to try and identify which points are inside
    // the big loop. Technically this will identify points that are inside any loop - hopefully the question
    // doesn't have other loops in!
    let number_of_points_inside_loop = find_points_inside_loop(&grid, &loop_).len();
    println!("Solution for part 2: {}", number_of_points_inside_loop);
}
