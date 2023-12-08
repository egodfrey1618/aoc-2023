use std::collections::HashMap;

#[derive(Debug)]
enum Dir {
    Left,
    Right,
}
#[derive(Debug)]
struct Node {
    left: String,
    right: String,
}

#[derive(Debug)]
struct Game {
    nodes: HashMap<String, Node>,
    dirs: Vec<Dir>,
}

fn read_lines(s: &str) -> Game {
    let (dirs, s) = s.split_once("\n").expect("Couldn't find newline.");

    let dirs: Vec<Dir> = dirs
        .trim()
        .chars()
        .map(|c| match c {
            'L' => Dir::Left,
            'R' => Dir::Right,
            _ => panic!("Unrecognised dir"),
        })
        .collect();

    let mut nodes = HashMap::new();

    for line in s.trim().lines() {
        let (name, outputs) = line.split_once(" = ").expect("Expected equals in string");
        let (left, right) = outputs
            .strip_prefix("(")
            .unwrap()
            .strip_suffix(")")
            .unwrap()
            .split_once(", ")
            .unwrap();
        let node = Node {
            left: left.to_string(),
            right: right.to_string(),
        };
        let name = name.to_string();
        nodes.insert(name, node);
    }

    Game { nodes, dirs }
}

fn part1(game: &Game) -> usize {
    // Part 1 - how many times does it take us to reach ZZZ from AAA?
    let mut name = "AAA";
    let mut node = game.nodes.get("AAA").expect("No AAA node in input?");
    let mut count = 0;

    let finish_name = "ZZZ";

    while name.ne(finish_name) {
        let dir = &game.dirs[count % game.dirs.len()];
        name = {
            match dir {
                Dir::Left => &node.left,
                Dir::Right => &node.right,
            }
        };
        node = game.nodes.get(name).unwrap();

        count += 1;
    }
    count
}

fn part2(game: &Game) -> usize {
    // Part 2 is weird!
    // We want to start on ALL of the nodes that start with 'A', and move simultaeneously until
    // we're all on nodes that end with 'Z'.
    //
    // The question hints this is going to be significantly more steps. So I might need to do some sort
    // of periodic thing.

    let starting_nodes: Vec<&String> = game.nodes.keys().filter(|s| s.ends_with("A")).collect();

    // Given any node, it eventually reaches some form of cycle - the path is fully determined by the [Key] type
    // below. So we must hit some point where the thing starts cycling, as there's only (dirs * nodes) number of
    // possible keys. In our real input, that's about 200_000.
    //
    // Technically I should probably also store which spots are good before we hit the cycle, but in practice
    // the input's going to take longer than that.

    #[derive(Debug)]
    struct CycleStructure {
        moves_before_cycle: usize,
        cycle_length: usize,
        good_nodes_on_path: Vec<usize>,
    }

    #[derive(Eq, PartialEq, Hash)]
    struct Key {
        count_modulo_dir_length: usize,
        node: String,
    }

    fn determine_cycle_structure(game: &Game, name: &str) -> CycleStructure {
        let mut key_to_first_visit: HashMap<Key, usize> = HashMap::new();
        let mut count = 0;
        let mut name = name;
        let mut good_nodes_on_path = vec![];

        loop {
            let count_modulo_dir_length = count % game.dirs.len();
            // Check if we're at a loop.
            let key = Key {
                count_modulo_dir_length,
                node: name.to_string(),
            };

            match key_to_first_visit.get(&key) {
                Some(spot) => {
                    // We've previously visited this spot, so we have our cycle!
                    let moves_before_cycle = *spot;
                    let cycle_length = count - moves_before_cycle;
                    return CycleStructure {
                        moves_before_cycle,
                        cycle_length,
                        good_nodes_on_path,
                    };
                }
                None => {
                    // First time we're visiting this key.
                    key_to_first_visit.insert(key, count);
                    if name.ends_with("Z") {
                        good_nodes_on_path.push(count);
                    }
                }
            }

            let dir = &game.dirs[count % game.dirs.len()];
            let node = game.nodes.get(name).unwrap();

            name = {
                match dir {
                    Dir::Left => &node.left,
                    Dir::Right => &node.right,
                }
            };
            count += 1;
        }
    }

    for node in starting_nodes {
        let structure = determine_cycle_structure(&game, node);
        println!("{}: {:?}", node, structure);
    }

    // I didn't finish the code to solve this, I just plugged the cycle structures into an online LCM calculator.
    // It turns out the input is a very special case. E.g. one of my cycles:
    // CVA: CycleStructure { moves_before_cycle: 2, cycle_length: 22357, good_nodes_on_path: [22357] }
    //
    // They're all of this form, where the answer is "it hits a Z node every N nodes from the start". This is
    // a special case for 2 reasons:
    // - Only having one node on the path which is a Z node.
    // - There's no shifting - it's just a multiple.
    //
    // So we don't need to Chinese Remainder Theorem this, or do any sort of keeping track of multiple places
    // which intersect a Z-node. We can just take LCM of the cycle lengths.
    0
}

fn main() {
    let game = read_lines(include_str!("input"));

    println!("Solution for part 1: {:?}", part1(&game));
    println!("Solution for part 2: {:?}", part2(&game));
}
