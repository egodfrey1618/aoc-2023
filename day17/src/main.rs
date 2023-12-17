use std::cmp::Ordering;
use std::collections::BinaryHeap;
use std::collections::HashMap;

#[derive(Hash, PartialEq, Eq, Debug)]
struct Position(usize, usize);

#[derive(Hash, PartialEq, Eq, Debug)]
enum Dir {
    Up,
    Left,
    Down,
    Right,
}
use Dir::*;

impl Dir {
    fn flip(&self) -> Self {
        match self {
            Up => Down,
            Down => Up,
            Left => Right,
            Right => Left,
        }
    }
}

#[derive(Hash, PartialEq, Eq, Debug)]
struct State {
    position: Position,
    dir: Dir,
    moves_so_far: usize,
}

struct Grid(Vec<Vec<usize>>);

fn neighbours(
    grid: &Grid,
    state: &State,
    min_moves_before_turn: usize,
    max_moves_before_turn: usize,
) -> Vec<State> {
    let mut result = vec![];

    let Position(x, y) = state.position;
    let height = grid.0.len();
    let width = grid.0[0].len();

    for dir in vec![Up, Left, Down, Right] {
        // We aren't allowed to flip direction.
        if dir == state.dir.flip() {
            continue;
        }

        // Get the new position, bounds checking.
        let new_position: Option<Position> = {
            match dir {
                Up => {
                    if x == 0 {
                        None
                    } else {
                        Some(Position(x - 1, y))
                    }
                }
                Down => {
                    if x == height - 1 {
                        None
                    } else {
                        Some(Position(x + 1, y))
                    }
                }
                Left => {
                    if y == 0 {
                        None
                    } else {
                        Some(Position(x, y - 1))
                    }
                }
                Right => {
                    if y == width - 1 {
                        None
                    } else {
                        Some(Position(x, y + 1))
                    }
                }
            }
        };

        let moves_so_far: Option<usize> = {
            if dir == state.dir {
                if state.moves_so_far == max_moves_before_turn {
                    // We must turn, so this isn't allowed.
                    None
                } else {
                    Some(state.moves_so_far + 1)
                }
            } else {
                if state.moves_so_far < min_moves_before_turn {
                    // We haven't moved far enough in this direction.
                    None
                } else {
                    Some(1)
                }
            }
        };

        match (new_position, moves_so_far) {
            (Some(new_position), Some(moves_so_far)) => {
                let state = State {
                    position: new_position,
                    moves_so_far,
                    dir,
                };
                result.push(state)
            }
            _ => (),
        }
    }
    result
}

fn dijkstra_to_bottom_right_corner(
    grid: &Grid,
    min_moves_before_turn: usize,
    max_moves_before_turn: usize,
) -> usize {
    // Do Dijkstra to the bottom-right corner of the grid, and return the shortest route there.

    #[derive(PartialEq, Eq)]
    struct BoundaryPoint {
        state: State,
        cost: usize,
    }

    impl PartialOrd for BoundaryPoint {
        fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
            // Flipping the order, because BinaryHeap is a max-heap, but we want to pick the smaller element.
            Some(other.cost.cmp(&self.cost))
        }
    }

    impl Ord for BoundaryPoint {
        fn cmp(&self, other: &Self) -> Ordering {
            // Flipping the order, because BinaryHeap is a max-heap, but we want to pick the smaller element.
            other.cost.cmp(&self.cost)
        }
    }

    let mut queue: BinaryHeap<BoundaryPoint> = BinaryHeap::new();
    let mut best_cost: HashMap<State, usize> = HashMap::new();

    // Initialise the priority queue with starting states.
    for dir in vec![Up, Down, Left, Right] {
        let state = State {
            position: Position(0, 0),
            dir,
            moves_so_far: 0,
        };

        queue.push(BoundaryPoint { state, cost: 0 })
    }

    while !queue.is_empty() {
        // Find the boundary spot with the smallest distance.
        let BoundaryPoint { state, cost } = queue.pop().unwrap();
        if best_cost.contains_key(&state) {
            // We already found this state with a smaller cost, so can continue.
            continue;
        }

        let neighbours = neighbours(&grid, &state, min_moves_before_turn, max_moves_before_turn);

        // Add this state to the best distance.
        best_cost.insert(state, cost);

        // Add all of its neighbours to the queue.
        for neighbour in neighbours {
            if !best_cost.contains_key(&neighbour) {
                let cost_to_neighbour_state =
                    cost + grid.0[neighbour.position.0][neighbour.position.1];
                queue.push(BoundaryPoint {
                    state: neighbour,
                    cost: cost_to_neighbour_state,
                });
            }
        }
    }

    // And then we want to find the lowest cost to get to any state in the bottom-right corner.
    // We also want to make sure it's taken at least [min_moves_before_turn] to get there.
    //
    // I could make this faster by bailing out of the search as soon as I hit the bottom-right
    // corner, rather than exploring the whole state space, but whatever. This works.
    let height = grid.0.len();
    let width = grid.0[0].len();

    let best_score = best_cost
        .iter()
        .filter(|(state, _cost)| {
            state.position == Position(height - 1, width - 1)
                && state.moves_so_far >= min_moves_before_turn
        })
        .map(|(_state, cost)| cost)
        .min()
        .unwrap();

    *best_score
}

fn main() {
    let grid: Vec<Vec<usize>> = include_str!("input")
        .trim()
        .lines()
        .map(|line| {
            line.chars()
                .map(|c| c.to_digit(10).unwrap() as usize)
                .collect()
        })
        .collect();
    let grid = Grid(grid);

    let part1 = dijkstra_to_bottom_right_corner(&grid, 0, 3);
    println!("Solution to part 1: {}", part1);
    let part2 = dijkstra_to_bottom_right_corner(&grid, 4, 10);
    println!("Solution to part 2: {}", part2);
}
