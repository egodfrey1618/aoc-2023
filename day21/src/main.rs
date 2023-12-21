use std::cmp::Ordering;
use std::collections::BinaryHeap;
use std::collections::HashMap;
use std::collections::HashSet;

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
struct Position(usize, usize);

#[derive(Debug)]
struct Grid {
    adjacency_list: HashMap<Position, Vec<Position>>,
    elf_position: Position,
    height: usize,
    width: usize,
}

fn parse(s: &str) -> Grid {
    let chars: Vec<Vec<char>> = s
        .trim()
        .lines()
        .map(|line| line.chars().collect())
        .collect();

    let mut elf_position = None;

    let height = chars.len();
    let width = chars[0].len();

    let mut empty_positions = HashSet::new();
    let mut add_empty_position = |p: Position| empty_positions.insert(p);

    for i in 0..height {
        for j in 0..width {
            let char = chars[i][j];

            match char {
                '.' => {
                    add_empty_position(Position(i, j));
                }
                '#' => (),
                'S' => {
                    elf_position = Some(Position(i, j));
                    add_empty_position(Position(i, j));
                }
                _ => panic!("Unexpected char in grid"),
            }
        }
    }

    let elf_position = elf_position.unwrap();

    let mut adjacency_list = HashMap::new();

    for position in &empty_positions {
        let mut neighbours = vec![];
        let Position(x, y) = position;

        let mut add_if_empty = |x: usize, y: usize| {
            let p = Position(x, y);
            if empty_positions.contains(&p) {
                neighbours.push(p)
            }
        };

        if *x > 0 {
            add_if_empty(*x - 1, *y);
        }
        if *x < height - 1 {
            add_if_empty(*x + 1, *y);
        }
        if *y > 0 {
            add_if_empty(*x, *y - 1);
        }
        if *y < width - 1 {
            add_if_empty(*x, *y + 1);
        }

        adjacency_list.insert(position.clone(), neighbours);
    }

    Grid {
        adjacency_list,
        elf_position,
        height,
        width,
    }
}

fn dijkstra(grid: &Grid, point: &Position) -> HashMap<Position, usize> {
    // Do Dijkstra from a starting point to every other point.
    #[derive(PartialEq, Eq)]
    struct Key {
        cell: Position,
        distance: usize,
    }

    impl PartialOrd for Key {
        fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
            // Reverse sorting, because [BinaryHeap] is a max-heap.
            other.distance.partial_cmp(&self.distance)
        }
    }

    impl Ord for Key {
        fn cmp(&self, other: &Self) -> Ordering {
            other.distance.cmp(&self.distance)
        }
    }

    let mut best_distances = HashMap::new();
    let mut process_queue: BinaryHeap<Key> = BinaryHeap::new();

    process_queue.push(Key {
        cell: point.clone(),
        distance: 0,
    });

    while !process_queue.is_empty() {
        let Key { cell, distance } = process_queue.pop().unwrap();

        if !best_distances.contains_key(&cell) {
            for neighbour in &grid.adjacency_list[&cell] {
                if !best_distances.contains_key(neighbour) {
                    process_queue.push(Key {
                        cell: neighbour.clone(),
                        distance: distance + 1,
                    });
                }
            }
            best_distances.insert(cell, distance);
        }
    }
    best_distances
}

fn solve_part1(grid: &Grid, target_distance: usize) -> usize {
    // Count how many squares are exactly 64 steps away.
    // We can do this with Dijkstra - find the distance from the elf to any other cell.
    // Then exactly 64 steps away <--> exactly N steps away for some even N, N <= 64.
    //
    // (Why? Any cell is either an even or odd number of steps away - think of a chessboard colouring. And
    // if we're N steps away for N < 64, we're exactly 64 steps away by doing some redundant moves at the
    // start.)

    // Count how many spaces are an even distance away, and within 64 steps.
    dijkstra(grid, &grid.elf_position)
        .into_iter()
        .filter_map(|(space, distance)| {
            if distance % 2 == target_distance % 2 && distance <= target_distance {
                Some(space)
            } else {
                None
            }
        })
        .count()
}

fn solve_part2(grid: &Grid, target_distance: usize) -> usize {
    // We're going to use a bunch of special properties of the grid:
    // - our elf starts in the middle of the square grid.
    // - there's a line of empty squares from the middle to the border (both horizontally and vertically).
    // - There's also a line of empty squares around the border.
    // (This is *not* true for the example though, which is kind of infuriating for checking!)
    //
    // So, let's take a square (i, j) in the grid. How many points in the big-grid have this reachable?
    // Well, if it's reachable in the big-grid in exactly [target_distance] steps, both of these need
    // to hold:
    //
    // (1) It's reachable in at most [target_distance] steps.
    // (2) We're of the right parity of steps.
    //
    // For (1): Suppose we're trying to get to a big-square that's to the North-East of us - concretely,
    // let's say it's (A, B) in the big grid, where we start at (0, 0), and A, B > 0. So we're
    // going right and up. The best thing for us to do is to go along the middle route of empty squares
    // until we get to the bottom or left boundary of that big-square, and then use whatever the
    // shortest path is to get to that square.
    //
    // To get there, we must go (A-1) * width + (B-1) * height + (1/2 * width) + (1/2 * height) to get to the corner
    // of that square, and then use shortest path to find if it's reachable. So we can use this to bound the
    // sum of A and B, so find how many spots can be reached.
    // The cases where we go to a square that's exactly to the North/East/West/South of us are similar. There
    // are 8 cases below.

    // First of all, let's assert at least one of those assumptions above - that the grid is square, and the elf
    // starts in the middle.
    assert!(grid.height == grid.width);
    assert!(grid.height % 2 == 1);
    assert!(grid.elf_position == Position(grid.height / 2, grid.height / 2));

    // Do Dijkstra from each of the border squares, and also from the elf square.
    let left_middle_square = Position(grid.height / 2, 0);
    let right_middle_square = Position(grid.height / 2, grid.width - 1);
    let top_middle_square = Position(0, grid.width / 2);
    let bottom_middle_square = Position(grid.height - 1, grid.width / 2);
    let top_left_corner = Position(0, 0);
    let top_right_corner = Position(0, grid.width - 1);
    let bottom_left_corner = Position(grid.height - 1, 0);
    let bottom_right_corner = Position(grid.height - 1, grid.width - 1);

    let distances_from_left_middle = dijkstra(&grid, &left_middle_square);
    let distances_from_right_middle = dijkstra(&grid, &right_middle_square);
    let distances_from_top_middle = dijkstra(&grid, &top_middle_square);
    let distances_from_bottom_middle = dijkstra(&grid, &bottom_middle_square);
    let distances_from_top_left = dijkstra(&grid, &top_left_corner);
    let distances_from_top_right = dijkstra(&grid, &top_right_corner);
    let distances_from_bottom_left = dijkstra(&grid, &bottom_left_corner);
    let distances_from_bottom_right = dijkstra(&grid, &bottom_right_corner);
    let distances_from_start = dijkstra(&grid, &grid.elf_position);

    let mut best_distances = HashMap::new();

    for i in 0..grid.height {
        for j in 0..grid.width {
            let position = Position(i, j);

            if !grid.adjacency_list.contains_key(&position) {
                // This case isn't empty, so doesn't contribute anything.
                continue;
            }
            if !distances_from_left_middle.contains_key(&position) {
                // This is not connected to the boundary, so we're definitely not reaching it.
                continue;
            }

            let mut total = 0;

            // In how many big squares is this point reachable? We have 8 cases to consider:
            // (A) Going in a straight line out from the origin - i.e. a big square to the left/top/right/bottom.
            // (B) Not in a straight line.
            // (C) Within this square.
            //
            // For (A), if we're reaching this square, we need to go:
            // - To the border (costs grid.height / 2 + 1 squares)
            // - And then from there to the square
            //
            // For (B) - for example if we're going to a square North-East of us - we need to go:
            // (1) To the bottom-right corner of the square at (1, 1) (costs 2 * grid.height / 2 + 1)
            // (2) Then A squares to the right/up (costs A * grid.height)
            // (3) Then from the bottom-right corner to the target square.

            let number_of_squares_case_a = |border_map: &HashMap<Position, usize>| {
                let to_first_border = grid.height / 2 + 1;
                let from_last_border = border_map.get(&position).unwrap();
                let min_distance = from_last_border + to_first_border;

                let number_of_full_squares_we_can_travel = {
                    if target_distance < min_distance {
                        0
                    } else {
                        // The [min_distance] already accounts for moving 1 square over.
                        1 + (target_distance - min_distance) / grid.height
                    }
                };

                // OK, so the number of possible squares is in [1..=number_of_full_squares_we_can_travel].
                // But! We also need to check the parity. Going a square flips the parity of the square that
                // we're on.
                let parity_of_target_square_from_elf =
                    distances_from_start.get(&position).unwrap() % 2;
                let parity_of_target_distance = target_distance % 2;

                if parity_of_target_square_from_elf == parity_of_target_distance {
                    // Can only visit the even squares
                    number_of_full_squares_we_can_travel / 2
                } else {
                    // Can only visit the odd squares
                    (number_of_full_squares_we_can_travel + 1) / 2
                }
            };

            let number_of_squares_case_b = |corner_map: &HashMap<Position, usize>| {
                // This needs 2* the above, because we need to go both one direction and the other.
                let to_first_corner = 2 * (grid.height / 2 + 1);
                let from_last_corner = corner_map.get(&position).unwrap();
                let min_distance = from_last_corner + to_first_corner;

                let number_of_full_squares_we_can_travel = {
                    if target_distance < min_distance {
                        0
                    } else {
                        // The [min_distance] already accounts for moving 2 square over.
                        // (This is relying on the grid being square.)
                        2 + (target_distance - min_distance) / grid.height
                    }
                };

                // The number of squares we can get is the number of possibilities of (A, B) with:
                // A+B matching the parity that we need
                // A+B <= number_of_full_squares_we_can_travel
                let parity_of_target_square_from_elf =
                    distances_from_start.get(&position).unwrap() % 2;
                let parity_of_target_distance = target_distance % 2;
                let desired_big_square_parity = {
                    if parity_of_target_distance == parity_of_target_square_from_elf {
                        0
                    } else {
                        1
                    }
                };

                // This is lazy of me - I should be able to find a closed form for this.
                // But I'm tired, and this works well enough (my whole code runs in under 2s).
                let mut total = 0;
                for a in 1..number_of_full_squares_we_can_travel {
                    let b = number_of_full_squares_we_can_travel - a;

                    let desired_b_parity = {
                        if a % 2 == 0 {
                            desired_big_square_parity
                        } else {
                            1 - desired_big_square_parity
                        }
                    };

                    if desired_b_parity == 0 {
                        total += b / 2
                    } else {
                        total += (b + 1) / 2
                    }
                }
                total
            };

            total += number_of_squares_case_a(&distances_from_left_middle);
            total += number_of_squares_case_a(&distances_from_top_middle);
            total += number_of_squares_case_a(&distances_from_right_middle);
            total += number_of_squares_case_a(&distances_from_bottom_middle);
            total += number_of_squares_case_b(&distances_from_top_left);
            total += number_of_squares_case_b(&distances_from_top_right);
            total += number_of_squares_case_b(&distances_from_bottom_left);
            total += number_of_squares_case_b(&distances_from_bottom_right);

            // Don't forget the ones inside this square too!
            total += {
                let x = *distances_from_start.get(&position).unwrap();
                if x <= target_distance && (x % 2) == (target_distance % 2) {
                    1
                } else {
                    0
                }
            };
            // println!("{:?}, {}", position, total);
            best_distances.insert(position, total);
        }
    }

    best_distances.values().cloned().sum::<usize>()
}

fn main() {
    let grid = include_str!("input");
    let grid = parse(grid);
    println!("Solution for part 1: {}", solve_part1(&grid, 64));

    // Part 2.

    // I used this for testing my solution to Part 2. [input_exploded] was a version of my input
    // expanded by some amount.

    /*
    let grid_exploded = include_str!("input_exploded");
    let grid_exploded = parse(grid_exploded);
    let best_distances = dijkstra(&grid_exploded, &grid_exploded.elf_position);

    for target_distance in 120..200 {
        let using_this_method = solve_part2(&grid, target_distance);

        let using_dijkstra_on_big_grid = best_distances
            .iter()
            .filter(|(position, distance)| {
                **distance <= target_distance && (**distance % 2) == (target_distance % 2)
            })
            .count();

        println!(
            "{}, {}, {}",
            target_distance, using_this_method, using_dijkstra_on_big_grid
        );
    }
    */

    println!("Solution for part 2: {}", solve_part2(&grid, 26501365));
}
