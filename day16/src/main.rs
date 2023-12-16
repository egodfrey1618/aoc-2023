use std::collections::HashSet;

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
enum Dir {
    Left,
    Right,
    Up,
    Down,
}
use Dir::*;

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
struct Position(usize, usize);

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
struct BeamState {
    position: Position,
    dir: Dir,
}

#[derive(Debug, Copy, Clone)]
enum GridSpace {
    Empty,
    MirrorTopLeft,
    MirrorTopRight,
    SplitterHorizontal,
    SplitterVertical,
}
use GridSpace::*;

struct Grid(Vec<Vec<GridSpace>>);

fn adjust_dirs(beam: &BeamState, grid: &Grid) -> Vec<BeamState> {
    // When we move a beam, we need to:
    // (1) Move its position
    // (2) Maybe adjust its direction, to account for the fact that we're on a mirror.
    //
    // This function just does (2). Useful because we also use it when initialising the beam.
    let BeamState { position, dir } = *beam;
    let grid_space = grid.0[position.0][position.1];

    let new_dirs = match (dir, grid_space) {
        // If the spot is empty, we don't change direction
        (dir, GridSpace::Empty) => vec![dir],
        // The cases where the mirror starts at the top left, i.e. \
        (Up, MirrorTopLeft) => vec![Left],
        (Left, MirrorTopLeft) => vec![Up],
        (Right, MirrorTopLeft) => vec![Down],
        (Down, MirrorTopLeft) => vec![Right],
        // The cases where the mirror starts at the top right, i.e. /
        (Up, MirrorTopRight) => vec![Right],
        (Right, MirrorTopRight) => vec![Up],
        (Left, MirrorTopRight) => vec![Down],
        (Down, MirrorTopRight) => vec![Left],
        // The horizontal splitter.
        (Left, SplitterHorizontal) => vec![dir],
        (Right, SplitterHorizontal) => vec![dir],
        (Up, SplitterHorizontal) => vec![Left, Dir::Right],
        (Down, SplitterHorizontal) => vec![Left, Dir::Right],
        // The vertical splitter
        (Up, SplitterVertical) => vec![dir],
        (Down, SplitterVertical) => vec![dir],
        (Left, SplitterVertical) => vec![Up, Dir::Down],
        (Right, SplitterVertical) => vec![Up, Dir::Down],
    };

    new_dirs
        .into_iter()
        .map(|new_dir| BeamState {
            position: position,
            dir: new_dir,
        })
        .collect()
}

fn move_beam(beam: &BeamState, grid: &Grid) -> Vec<BeamState> {
    let BeamState { position, dir } = beam;
    let Position(x, y) = *position;

    // I don't like how clunky rustfmt is here, this looked clearer when there were fewer line breaks in the if/else clauses...
    let new_position = match dir {
        Up => {
            if x == 0 {
                None
            } else {
                Some(Position(x - 1, y))
            }
        }
        Left => {
            if y == 0 {
                None
            } else {
                Some(Position(x, y - 1))
            }
        }
        Down => {
            if x + 1 == grid.0.len() {
                None
            } else {
                Some(Position(x + 1, y))
            }
        }
        Right => {
            if y + 1 == grid.0[0].len() {
                None
            } else {
                Some(Position(x, y + 1))
            }
        }
    };

    match new_position {
        None => vec![],
        Some(new_position) => {
            let beam_state_after_moving = BeamState {
                position: new_position,
                dir: *dir,
            };
            adjust_dirs(&beam_state_after_moving, &grid)
        }
    }
}

fn parse_grid(s: &str) -> Grid {
    let grid = s
        .trim()
        .lines()
        .map(|line| {
            line.chars()
                .map(|c| match c {
                    '.' => Empty,
                    '/' => MirrorTopRight,
                    '\\' => MirrorTopLeft,
                    '|' => SplitterVertical,
                    '-' => SplitterHorizontal,
                    _ => panic!("Unrecognised character"),
                })
                .collect()
        })
        .collect();
    Grid(grid)
}

fn energised_boxes_starting_from(beam: &BeamState, grid: &Grid) -> usize {
    let initial_states = {
        // I need to adjust the directions as the beam comes into the grid (e.g. if it immediately hits a mirror).
        adjust_dirs(&beam, &grid)
    };

    // Flood fill. (Our state space is BeamState, so position and a direction you're moving.)
    let mut processed: HashSet<BeamState> = HashSet::new();

    let mut queue = initial_states;

    while !queue.is_empty() {
        let next_ = queue.pop().unwrap();
        processed.insert(next_);
        let neighbours = move_beam(&next_, &grid);

        for neighbour in neighbours {
            if !processed.contains(&neighbour) {
                queue.push(neighbour);
            }
        }
    }

    // Then count just the positions we've reached.
    let positions: HashSet<Position> = processed
        .iter()
        .map(|beam_state| beam_state.position)
        .collect();
    positions.len()
}

fn main() {
    let grid = parse_grid(include_str!("input"));

    // Part 1
    let x = energised_boxes_starting_from(
        &BeamState {
            position: Position(0, 0),
            dir: Right,
        },
        &grid,
    );
    println!("Result for part 1: {}", x);

    // Part 2. Nothing smart here, just try each possible entry point. There's probably something better I can do here.
    // This takes about 0.5s in Rust's debug mode, so I'm pretty happy with this.
    //
    // I think there's a smarter thing where I could upper bound how well a fill is going to go, and then cut off some
    // parts of the search early, but this works fine.
    let mut edge_points: Vec<BeamState> = vec![];

    let height = grid.0.len();
    let width = grid.0[0].len();

    for x in 0..height {
        edge_points.push(BeamState {
            position: Position(x, 0),
            dir: Right,
        });
        edge_points.push(BeamState {
            position: Position(x, width - 1),
            dir: Left,
        });
    }
    for y in 0..width {
        edge_points.push(BeamState {
            position: Position(0, y),
            dir: Down,
        });
        edge_points.push(BeamState {
            position: Position(height - 1, y),
            dir: Left,
        });
    }

    let result = edge_points
        .iter()
        .map(|beam| energised_boxes_starting_from(beam, &grid))
        .max()
        .unwrap();
    println!("Result for part 2: {:?}", result);
}
