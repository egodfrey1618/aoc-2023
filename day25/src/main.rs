use rand::prelude::*;
use std::collections::HashMap;
use std::collections::HashSet;

struct Graph {
    edges: HashMap<String, Vec<String>>,
}

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
struct Index(usize);

#[derive(Debug)]
struct KargerState {
    index_to_strings: HashMap<Index, HashSet<String>>,
    edges: HashMap<Index, HashMap<Index, usize>>,
}

fn karger(g: &Graph, target_min_cut: usize) -> (usize, usize) {
    // while True:
    // Create a new structure
    // Repeatedly contract an edge. Move weight onto other vertices.
    // Until your state has 2 vertices. Then check the weight of the edge between them.

    loop {
        // Set up Karger State.
        let string_to_index: HashMap<String, Index> = g
            .edges
            .keys()
            .enumerate()
            .map(|(i, s)| (s.clone(), Index(i)))
            .collect();

        let index_to_strings: HashMap<Index, HashSet<String>> = string_to_index
            .iter()
            .map(|(string, index)| (index.clone(), vec![string.clone()].into_iter().collect()))
            .collect();

        let edges: HashMap<Index, HashMap<Index, usize>> = g
            .edges
            .iter()
            .map(|(s, neighbours)| {
                let i = string_to_index.get(s).unwrap().clone();
                let neighbours = neighbours
                    .iter()
                    .map(|s| {
                        let i = string_to_index.get(s).unwrap().clone();
                        (i, 1)
                    })
                    .collect();
                (i, neighbours)
            })
            .collect();

        let mut karger_state = KargerState {
            index_to_strings,
            edges,
        };

        // While it has size >2: Repeatedly try and contract an edge.

        while karger_state.edges.len() > 2 {
            // Pick a random edge.
            let keys: Vec<&Index> = karger_state.edges.keys().collect();

            let v1: Index = keys[random::<usize>() % keys.len()].clone();
            let n1 = &karger_state.edges[&v1];

            let keys: Vec<&Index> = n1.keys().collect();
            let v2: Index = keys[random::<usize>() % keys.len()].clone();

            // Contract (v1, v2) by turning v2 into v1.
            let strings2 = karger_state.index_to_strings.remove(&v2).unwrap();
            karger_state
                .index_to_strings
                .get_mut(&v1)
                .unwrap()
                .extend(strings2);

            let old_connections_to_v2 = karger_state.edges.remove(&v2).unwrap();

            // Fix up v1.
            for (edge, size) in old_connections_to_v2 {
                if edge != v1 {
                    *karger_state
                        .edges
                        .get_mut(&v1)
                        .unwrap()
                        .entry(edge)
                        .or_insert(0) += size;
                }
            }

            for (index, edges) in &mut karger_state.edges {
                let old_edges_to_v2 = edges.remove(&v2);

                if old_edges_to_v2.is_some() && index != &v1 {
                    // Move the edges over to v1.
                    *edges.entry(v1.clone()).or_insert(0) += old_edges_to_v2.unwrap();
                }
            }
        }

        // Have we hit our target min_cut?
        let KargerState {
            index_to_strings,
            edges,
        } = karger_state;
        let keys: Vec<Index> = edges.keys().cloned().collect();
        assert!(keys.len() == 2);

        let score = edges[&keys[0]][&keys[1]];
        if score == target_min_cut {
            println!("Randomized algo found a min cut!");
            break (
                index_to_strings[&keys[0]].len(),
                index_to_strings[&keys[1]].len(),
            );
        }
    }
}

fn parse(s: &str) -> HashMap<String, Vec<String>> {
    let mut result = HashMap::new();

    for line in s.trim().lines() {
        let (from, to) = line.split_once(": ").unwrap();
        let from = from.to_string();
        let to: Vec<String> = to
            .split_whitespace()
            .map(|s| s.trim().to_string())
            .collect();

        for s in &to {
            result.entry(s.clone()).or_insert(vec![]).push(from.clone());
        }
        result.entry(from).or_insert(vec![]).extend(to);
    }
    result
}

fn main() {
    let s = include_str!("input");
    let edges = parse(s);
    let graph = Graph { edges };
    let result = karger(&graph, 3);
    println!("{:?}", result);
}
