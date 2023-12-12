use std::collections::HashMap;
use std::ops::Range;

/*
OK, so how do we count the possibilities? The obvious answer is some sort of dynamic programming thing.

Let f(S, M) be the number of ways of filling in unknowns in S so it matches pattern M.
Let g(S, M) be the number of ways of filling in unknowns in S so it matches pattern M, with no spaces at the end.

If len(M) == 1, then it's not too bad to do this directly.

If len(M) > 1, then we can partition these based on "when does the first one finish", add a space after it.
*/

#[derive(Copy, Clone, PartialEq, Eq)]
enum Status {
    Empty,
    Full,
    Unknown,
}
use Status::*;

impl Status {
    fn is_fillable(&self) -> bool {
        match self {
            Unknown | Full => true,
            Empty => false,
        }
    }

    fn is_blankable(&self) -> bool {
        match self {
            Unknown | Empty => true,
            Full => false,
        }
    }
}

fn number_of_ways(s: &[Status], pattern: &[usize]) -> usize {
    #[derive(PartialEq, Eq, Hash, Clone, Debug)]
    struct Key {
        status_index: Range<usize>,
        pattern_index: Range<usize>,
    }

    let mut cache: HashMap<Key, usize> = HashMap::new();

    // This needs to be a [fn], not a closure, because it's recursive.
    // All arguments other than key here are just meant to capture from the environment.
    fn solve(
        full_status: &[Status],
        full_pattern: &[usize],
        key: Key,
        cache: &mut HashMap<Key, usize>,
    ) -> usize {
        // Check cache
        if cache.contains_key(&key) {
            return *cache.get(&key).unwrap();
        }

        let s = &full_status[key.status_index.clone()];
        let pattern = &full_pattern[key.pattern_index.clone()];

        // Solve, assuming that we've included everything we'll need in the cache.
        let result = match (s.len(), pattern.len()) {
            (_, 0) => {
                // Base case. 0 is true if we can fill in all squares with Empty.
                // (This covers the case when s is empty as well.)
                if s.iter().all(|t| t.is_blankable()) {
                    1
                } else {
                    0
                }
            }
            (0, _) => {
                // Base case - we have a non-empty pattern but an empty string, so nothing we can do.
                0
            }
            _ => {
                // Otherwise, both s and pattern are non-empty.
                let prefix_length = {
                    if pattern.len() == 1 {
                        pattern[0]
                    } else {
                        pattern[0] + 1
                    }
                };

                if s.len() < prefix_length {
                    // Bail out early - the length of the string isn't going to be long enough.
                    0
                } else {
                    let mut total = 0;

                    // Case 1 (only possible if first cell is blankable) - the pattern starts at position >= 1.
                    if s[0].is_blankable() {
                        let status_index = key.status_index.start + 1..key.status_index.end;
                        let key = Key {
                            status_index,
                            pattern_index: key.pattern_index.clone(),
                        };
                        total += solve(full_status, full_pattern, key, cache);
                    }

                    // Case 2 (only possible if first pattern[0] cells are fillable, and pattern[0] cell is blankable).
                    // The pattern starts at position 0.
                    let pattern_starting_here_is_possible = {
                        let first_cells_are_fillable =
                            s[0..pattern[0]].iter().all(|s| s.is_fillable());
                        let next_cell_is_blankable = {
                            // We only need to check this if pattern.len() > 1
                            (pattern.len() == 1)
                                || (pattern.len() > 1 && s[pattern[0]].is_blankable())
                        };
                        first_cells_are_fillable && (next_cell_is_blankable || pattern.len() == 1)
                    };
                    if pattern_starting_here_is_possible {
                        let status_index =
                            key.status_index.start + prefix_length..key.status_index.end;
                        let pattern_index = key.pattern_index.start + 1..key.pattern_index.end;

                        let key = Key {
                            status_index,
                            pattern_index,
                        };

                        total += solve(full_status, full_pattern, key, cache);
                    }
                    total
                }
            }
        };
        cache.insert(key, result);
        result
    }

    let full_key = Key {
        status_index: 0..s.len(),
        pattern_index: 0..pattern.len(),
    };

    let result = solve(s, pattern, full_key, &mut cache);
    result
}

fn main() {
    let s = include_str!("input2").trim();

    let cases: Vec<(Vec<Status>, Vec<usize>)> = s
        .lines()
        .map(|line| {
            let [s, pattern]: [&str; 2] = line
                .split_whitespace()
                .collect::<Vec<&str>>()
                .try_into()
                .unwrap();

            let s: Vec<Status> = s
                .chars()
                .map(|c| match c {
                    '?' => Unknown,
                    '.' => Empty,
                    '#' => Full,
                    _ => panic!("Unrecognised char"),
                })
                .collect();

            let pattern: Vec<usize> = pattern
                .split(",")
                .map(|c| c.parse().expect("Couldn't parse as usize"))
                .collect();
            (s, pattern)
        })
        .collect();

    let mut total = 0;
    for case in &cases {
        let result = number_of_ways(&case.0, &case.1);
        total += result;
    }
    println!("Solution for part 1: {}", total);

    // Part 2 - copy each of the cases by 5 times!
    let mut total = 0;
    for (status, pattern) in cases {
        // Expand each of them by 5. I did this in a bit of a lazy way.
        let mut new_status = status.clone();
        for _ in 0..4 {
            new_status.push(Unknown);
            new_status.extend(status.iter());
        }
        let mut new_pattern = pattern.clone();
        for _ in 0..4 {
            new_pattern.extend(pattern.iter());
        }

        let result = number_of_ways(&new_status, &new_pattern);
        total += result;
    }
    println!("Solution for part 2: {}", total);
}
