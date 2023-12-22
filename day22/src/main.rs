use std::collections::HashMap;
use std::collections::HashSet;

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
struct Position(usize, usize, usize);

impl Position {
    fn parse(s: &str) -> Self {
        let numbers: Vec<usize> = s.split(",").map(|s| s.parse::<usize>().unwrap()).collect();

        match &numbers[..] {
            [x, y, z] => Position(*x, *y, *z),
            _ => panic!("Didn't find one number"),
        }
    }
}

#[derive(Debug)]
enum Orientation {
    X,
    Y,
    Z,
}
use Orientation::*;

#[derive(Debug)]
struct Brick {
    start_point: Position,
    orientation: Orientation,
    size: usize,
    brick_id: usize,
}

impl Brick {
    fn points(&self) -> Vec<Position> {
        let Position(x, y, z) = self.start_point;

        let (x, y, z) = match &self.orientation {
            X => ((x..=x + self.size - 1), (y..=y), (z..=z)),
            Y => ((x..=x), (y..=y + self.size - 1), (z..=z)),
            Z => ((x..=x), (y..=y), (z..=z + self.size - 1)),
        };

        let mut result = vec![];
        for x in x.clone() {
            for y in y.clone() {
                for z in z.clone() {
                    result.push(Position(x, y, z))
                }
            }
        }

        result
    }

    fn of_positions(start_position: Position, end_position: Position, brick_id: usize) -> Self {
        // Assumes that the first point is lower, which looks like it's the case at a quick glance from the input.
        // If not, Rust will make sure this crashes.
        let x = end_position.0 - start_position.0;
        let y = end_position.1 - start_position.1;
        let z = end_position.2 - start_position.2;

        let (size, orientation) = {
            match (x, y, z) {
                (0, 0, 0) => (1, X),
                (x, 0, 0) => (x + 1, X),
                (0, y, 0) => (y + 1, Y),
                (0, 0, z) => (z + 1, Z),
                _ => panic!("More than one of the dimensions were greater than 1?"),
            }
        };

        Brick {
            start_point: start_position,
            orientation,
            size,
            brick_id,
        }
    }
}

#[derive(Debug)]
struct State {
    bricks: Vec<Brick>,
    position_to_brick_id: HashMap<Position, usize>,
}

impl State {
    fn parse(s: &str) -> Self {
        let lines = s.lines();
        let mut bricks: Vec<Brick> = vec![];

        for line in lines {
            let (pos1, pos2) = line.split_once("~").unwrap();
            let pos1 = Position::parse(pos1);
            let pos2 = Position::parse(pos2);
            let brick_id = bricks.len();
            let brick = Brick::of_positions(pos1, pos2, brick_id);
            bricks.push(brick)
        }

        let mut position_to_brick_id = HashMap::new();

        for brick in &bricks {
            for position in brick.points() {
                let old_brick_id = position_to_brick_id.insert(position, brick.brick_id);
                assert!(old_brick_id.is_none());
            }
        }
        State {
            bricks,
            position_to_brick_id,
        }
    }

    fn drop_bricks(&mut self) {
        // Drop all the bricks to the floor.

        // Step 1: Sort the bricks by their z co-ordinate.
        self.bricks
            .sort_by(|brick1, brick2| brick1.start_point.2.cmp(&brick2.start_point.2));

        // Step 2: Run through each brick one by one, and see if we can drop it.
        let State {
            bricks,
            position_to_brick_id,
        } = self;

        for brick in bricks.iter_mut() {
            // How far, if at all, can this brick drop?
            let mut amount_to_drop = 0;
            let brick_points = brick.points();

            for next_amount_to_drop in 1..brick.start_point.2 {
                let blocked = brick_points.iter().cloned().any(|p| {
                    let dropped_square = Position(p.0, p.1, p.2 - next_amount_to_drop);
                    match position_to_brick_id.get(&dropped_square) {
                        None => false,
                        Some(other_brick_id) => *other_brick_id != brick.brick_id,
                    }
                });

                if !blocked {
                    amount_to_drop = next_amount_to_drop;
                } else {
                    break;
                }
            }

            // Drop the brick!
            for old_position in &brick_points {
                let old_value = position_to_brick_id.remove(&old_position);
                assert!(old_value == Some(brick.brick_id));
            }
            brick.start_point = Position(
                brick.start_point.0,
                brick.start_point.1,
                brick.start_point.2 - amount_to_drop,
            );
            for new_position in brick.points() {
                let old_value = position_to_brick_id.insert(new_position, brick.brick_id);
                // We should have checked this doesn't contain anything.
                assert!(old_value.is_none());
            }
        }
    }

    fn brick_supporting_map(&self) -> HashMap<usize, HashSet<usize>> {
        // Returns a map from brick -> bricks, giving a map for which bricks are supporting others.

        let mut result = HashMap::new();

        for brick in &self.bricks {
            let supporting_bricks: HashSet<usize> = brick
                .points()
                .into_iter()
                .filter_map(|p| {
                    let Position(x, y, z) = p;
                    let below_point = Position(x, y, z - 1);
                    self.position_to_brick_id.get(&below_point)
                })
                .filter(|x| **x != brick.brick_id)
                .copied()
                .collect();
            result.insert(brick.brick_id, supporting_bricks);
        }
        result
    }
}

fn solve_part2(state: &mut State) -> usize {
    // Assuming that the state's already had all the blocks dropped to the floor.

    // Step 1: Sort the bricks by their z co-ordinate.
    state
        .bricks
        .sort_by(|brick1, brick2| brick1.start_point.2.cmp(&brick2.start_point.2));

    let brick_supporting_map = state.brick_supporting_map();

    // Step 2: Now work top-to-bottom, constructing the transitive closure of all the blocks that would fall.
    let mut bricks_to_transitive_closure_bricks: HashMap<usize, HashSet<usize>> = HashMap::new();

    for i in 0..state.bricks.len() {
        let brick = &state.bricks[i];

        let mut result = HashSet::new();

        // We're going bottom-to-top, so we're checking things in the right order.
        for j in (i + 1)..state.bricks.len() {
            let other_brick = &state.bricks[j];

            // Is everything supporting this brick in the set that's going to fall?

            let everything_supporting_this_brick_will_fall = {
                if other_brick.start_point.2 == 1 {
                    // It's already in the floor, so not going to fall.
                    false
                } else {
                    // Is everything supporting it on the floor?
                    brick_supporting_map[&other_brick.brick_id]
                        .iter()
                        .copied()
                        .map(|x| (result.contains(&x) || x == brick.brick_id))
                        .all(|x| x)
                }
            };

            if everything_supporting_this_brick_will_fall {
                result.insert(other_brick.brick_id);
            }
        }

        bricks_to_transitive_closure_bricks.insert(brick.brick_id, result);
    }

    let result: usize = bricks_to_transitive_closure_bricks
        .values()
        .map(|x| x.len())
        .sum();
    result
}

fn main() {
    let s = include_str!("input");
    let mut state = State::parse(s);

    state.drop_bricks();

    let brick_to_supporting_bricks = state.brick_supporting_map();
    let bricks_which_are_a_unique_support: HashSet<usize> = brick_to_supporting_bricks
        .values()
        .filter_map(|supporting_bricks| match supporting_bricks.len() {
            1 => Some(supporting_bricks.iter().copied().collect::<Vec<usize>>()[0]),
            _ => None,
        })
        .collect();

    println!(
        "Result for part 1 (number of bricks that AREN'T a unique support): {}",
        brick_to_supporting_bricks.len() - bricks_which_are_a_unique_support.len()
    );

    // Part 2. This is fun!
    println!("Solution for part 2: {}", solve_part2(&mut state));
}
