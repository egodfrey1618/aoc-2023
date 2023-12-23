use std::collections::HashMap;
use std::collections::HashSet;

#[derive(Debug, PartialEq, Eq)]
enum Space {
    Empty,
    Full,
    SlopeUp,
    SlopeDown,
    SlopeRight,
    SlopeLeft,
}
use Space::*;

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
struct Position(usize, usize);

struct Grid {
    grid: HashMap<Position, Space>,
    start: Position,
    end: Position,
}

struct Graph {
    forward_edges_with_edge_weights: HashMap<Position, HashMap<Position, usize>>,
    backwards_edges_with_edge_weights: HashMap<Position, HashMap<Position, usize>>,
    start: Position,
    end: Position,
}

impl Graph {
    fn add_edge(&mut self, v1: &Position, v2: &Position, weight: usize) {
        self.forward_edges_with_edge_weights
            .entry(v1.clone())
            .or_insert_with(|| HashMap::new())
            .insert(v2.clone(), weight);
        self.backwards_edges_with_edge_weights
            .entry(v2.clone())
            .or_insert_with(|| HashMap::new())
            .insert(v1.clone(), weight);
    }

    fn remove_edge(&mut self, v1: &Position, v2: &Position) {
        self.forward_edges_with_edge_weights
            .get_mut(v1)
            .unwrap()
            .remove(v2);
        self.backwards_edges_with_edge_weights
            .get_mut(v2)
            .unwrap()
            .remove(v1);
    }
}

fn parse(s: &str) -> Grid {
    let lines: Vec<&str> = s.trim().lines().collect();
    let mut grid = HashMap::new();

    for (i, line) in lines.iter().enumerate() {
        for (j, c) in line.chars().enumerate() {
            let position = Position(i, j);
            let state = match c {
                '.' => Empty,
                '#' => Full,
                '^' => SlopeUp,
                '>' => SlopeRight,
                '<' => SlopeLeft,
                'v' => SlopeDown,
                _ => panic!("Unknown char"),
            };
            grid.insert(position, state);
        }
    }

    let find_point_with_row = |row: usize| {
        grid.iter()
            .filter(|(p, s)| p.0 == row && s == &&Empty)
            .map(|(p, _s)| p)
            .next()
            .unwrap()
            .clone()
    };
    let start = find_point_with_row(0);
    let end = find_point_with_row(lines.len() - 1);

    Grid { grid, start, end }
}

fn neighbours(grid: &Grid, position: &Position, ignore_ice: bool) -> Vec<Position> {
    match (grid.grid.get(position).unwrap(), ignore_ice) {
        (SlopeUp, false) => vec![Position(position.0 - 1, position.1)],
        (SlopeDown, false) => vec![Position(position.0 + 1, position.1)],
        (SlopeLeft, false) => vec![Position(position.0, position.1 - 1)],
        (SlopeRight, false) => vec![Position(position.0, position.1 + 1)],
        (Full, _) => panic!("Can't find neighbours of full cell"),
        (Empty, _)
        | (SlopeUp, true)
        | (SlopeDown, true)
        | (SlopeLeft, true)
        | (SlopeRight, true) => {
            let mut result: Vec<Position> = vec![];

            let mut push_if_not_full = |position: Position| match grid.grid.get(&position) {
                Some(Full) => (),
                None => (),
                Some(_) => result.push(position),
            };

            if position.0 > 0 {
                push_if_not_full(Position(position.0 - 1, position.1));
            }
            if position.1 > 0 {
                push_if_not_full(Position(position.0, position.1 - 1));
            }
            push_if_not_full(Position(position.0 + 1, position.1));
            push_if_not_full(Position(position.0, position.1 + 1));

            result
        }
    }
}

fn to_graph(grid: &Grid, ignore_ice: bool) -> Graph {
    let mut forward_edges_with_edge_weights = HashMap::new();
    let mut backwards_edges_with_edge_weights = HashMap::new();

    for (point, space) in grid.grid.iter() {
        match space {
            Full => (),
            _ => {
                let neighbours = neighbours(&grid, &point, ignore_ice);
                let neighbours_with_weight = neighbours.iter().map(|s| (s.clone(), 1)).collect();
                forward_edges_with_edge_weights.insert(point.clone(), neighbours_with_weight);

                for neighbour in &neighbours {
                    backwards_edges_with_edge_weights
                        .entry(neighbour.clone())
                        .or_insert_with(|| HashMap::new())
                        .insert(point.clone(), 1);
                }
            }
        }
    }

    Graph {
        forward_edges_with_edge_weights,
        backwards_edges_with_edge_weights,
        start: grid.start.clone(),
        end: grid.end.clone(),
    }
}

fn optimise(graph: &mut Graph) {
    // Some optimisations to the graph to make the brute-force solution run faster.

    fn maybe_squash_vertex(graph: &mut Graph, point: &Position) {
        // For simplicity, we'll never try to squash the start or end nodes, so I don't have to think about updating them.
        if point == &graph.start || point == &graph.end {
            return;
        }

        let neighbours_out = graph.forward_edges_with_edge_weights.get(point).unwrap();
        let neighbours_in = graph.backwards_edges_with_edge_weights.get(point).unwrap();

        let mut neighbours: HashSet<Position> = neighbours_out.keys().cloned().collect();
        neighbours.extend(neighbours_in.keys().cloned());

        if neighbours.len() == 2 {
            // This only has two neighbours. We can remove this vertex, and repath any edges that went through
            // it to go direct.
            let mut i = neighbours.into_iter();
            let n1 = i.next().unwrap();
            let n2 = i.next().unwrap();

            assert!(n1 != n2);
            assert!(&n1 != point);
            assert!(&n2 != point);

            fn get_edge(graph: &Graph, p1: &Position, p2: &Position) -> Option<usize> {
                graph
                    .forward_edges_with_edge_weights
                    .get(&p1)
                    .unwrap()
                    .get(&p2)
                    .copied()
            }

            let old_direct_edge_weight_n1_to_n2 = get_edge(&graph, &n1, &n2);
            let old_direct_edge_weight_n2_to_n1 = get_edge(&graph, &n2, &n1);

            let indirect_edge_weight_n1_to_n2 = {
                match (get_edge(&graph, &n1, point), get_edge(&graph, &point, &n2)) {
                    (Some(v1), Some(v2)) => Some(v1 + v2),
                    _ => None,
                }
            };
            let indirect_edge_weight_n2_to_n1 = {
                match (get_edge(&graph, &n2, point), get_edge(&graph, &point, &n1)) {
                    (Some(v1), Some(v2)) => Some(v1 + v2),
                    _ => None,
                }
            };

            let new_weight_n1_to_n2 =
                old_direct_edge_weight_n1_to_n2.max(indirect_edge_weight_n1_to_n2);
            let new_weight_n2_to_n1 =
                old_direct_edge_weight_n2_to_n1.max(indirect_edge_weight_n2_to_n1);

            match new_weight_n1_to_n2 {
                Some(v) => graph.add_edge(&n1, &n2, v),
                _ => (),
            }
            match new_weight_n2_to_n1 {
                Some(v) => graph.add_edge(&n2, &n1, v),
                _ => (),
            }

            graph.remove_edge(&n1, point);
            graph.remove_edge(&n2, point);
            graph.remove_edge(point, &n1);
            graph.remove_edge(point, &n2);
            graph.forward_edges_with_edge_weights.remove(&point);
            graph.backwards_edges_with_edge_weights.remove(&point);
        }
    }

    let positions: Vec<Position> = graph
        .forward_edges_with_edge_weights
        .keys()
        .cloned()
        .collect();

    for position in positions {
        maybe_squash_vertex(graph, &position);
    }
}

fn solve_naive(graph: &Graph) {
    // Solve naively - just try brute-forcing every path.

    struct PartialPath {
        visited: HashSet<Position>,
        path: Vec<(Position, usize)>,
    }

    let mut finished_paths = vec![];

    let mut partial_paths = vec![PartialPath {
        visited: vec![graph.start.clone()].into_iter().collect(),
        path: vec![(graph.start.clone(), 0)],
    }];

    while !partial_paths.is_empty() {
        let PartialPath { visited, path } = partial_paths.pop().unwrap();

        let next_neighbours = graph
            .forward_edges_with_edge_weights
            .get(&path[path.len() - 1].0)
            .unwrap();

        let next_neighbours: Vec<(Position, usize)> = next_neighbours
            .into_iter()
            .filter(|(s, _weight)| !visited.contains(s))
            .map(|(s, t)| (s.clone(), *t))
            .collect();

        // Optimisation: If the end neighbour is in here, you have to go there.
        // (I don't know that this really helps that much - the code still runs fast enough without it.)
        let contains_end = next_neighbours
            .iter()
            .find(|(p, _s)| p == &graph.end)
            .is_some();

        if next_neighbours.is_empty() {
            finished_paths.push(PartialPath { visited, path });
            continue;
        }

        for (neighbour, weight) in next_neighbours {
            if contains_end && neighbour != graph.end {
                continue;
            }

            let mut next_visited = visited.clone();
            next_visited.insert(neighbour.clone());
            let mut next_path = path.clone();
            next_path.push((neighbour, weight));
            let partial_path = PartialPath {
                visited: next_visited,
                path: next_path,
            };
            partial_paths.push(partial_path);
        }
    }

    let finished_paths: Vec<PartialPath> = finished_paths
        .into_iter()
        .filter(|p| p.path[p.path.len() - 1].0 == graph.end)
        .collect();

    let longest_path: usize = finished_paths
        .iter()
        .map(|p| p.path.iter().map(|(_position, weight)| weight).sum())
        .max()
        .unwrap();

    println!("Longest path from start to end is: {}", longest_path);
}

fn main() {
    let s = include_str!("input");
    let grid = parse(s);
    let mut graph = to_graph(&grid, false);
    optimise(&mut graph);
    println!(
        "Size after optimising: {}",
        graph.forward_edges_with_edge_weights.len()
    );
    solve_naive(&graph);

    let mut graph = to_graph(&grid, true);
    println!("{}", graph.forward_edges_with_edge_weights.len());
    optimise(&mut graph);
    println!(
        "Size after optimising: {}",
        graph.forward_edges_with_edge_weights.len()
    );
    solve_naive(&graph);
}
