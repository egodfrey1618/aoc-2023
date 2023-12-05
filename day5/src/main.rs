use std::ops::Range;

#[derive(Debug)]
struct SoilMapOne {
    input_start: usize,
    output_start: usize,
    length: usize,
}

#[derive(Debug)]
struct SoilMap(Vec<SoilMapOne>);

#[derive(Debug)]
struct Problem {
    input_seeds: Vec<usize>,
    soil_maps: Vec<SoilMap>,
}

struct ApplyRangeResult {
    mapped: Option<Range<usize>>,
    // The vector might be length 0, 1 or 2, depending on how much of the range is mapped.
    unmapped: Vec<Range<usize>>,
}

impl SoilMapOne {
    fn apply(&self, v: usize) -> Option<usize> {
        let SoilMapOne {
            input_start,
            output_start,
            length,
        } = self;
        if v >= *input_start && v < *input_start + *length {
            return Some(output_start + v - *input_start);
        } else {
            return None;
        }
    }

    fn apply_range(&self, input_range: Range<usize>) -> ApplyRangeResult {
        // 5 cases to deal with:
        // (1) The range is fully outside of the map - if so, just return it.
        // (2) The range is fully inside of the map.
        // (3) The range is outside on the left, inside on the right.
        // (4) The range is inside on the left, outside on the right.
        // (5) The range spans the whole of the map, so we break it into 3 parts.
        //
        // I can probably combine these cases in some smarter way, but whatever.

        let map_range = self.input_start..self.input_start + self.length;

        fn shift(x: &SoilMapOne, range: Range<usize>) -> Range<usize> {
            (range.start + x.output_start - x.input_start)
                ..(range.end + x.output_start - x.input_start)
        }

        // Case 1 - fully outside.
        if input_range.end <= map_range.start || map_range.end <= input_range.start {
            return ApplyRangeResult {
                mapped: None,
                unmapped: vec![input_range],
            };
        }

        // Case 2 - fully inside of the map.
        if input_range.end <= map_range.end && input_range.start >= map_range.start {
            return ApplyRangeResult {
                mapped: Some(shift(&self, input_range)),
                unmapped: vec![],
            };
        }

        // Case 3 - outside on the left, inside on the right.
        if input_range.end <= map_range.end && input_range.start < map_range.start {
            return ApplyRangeResult {
                mapped: Some(shift(&self, map_range.start..input_range.end)),
                unmapped: vec![input_range.start..map_range.start],
            };
        }

        // Case 4 - inside on the left, outside on the right.
        if input_range.start >= map_range.start && input_range.end > map_range.end {
            return ApplyRangeResult {
                mapped: Some(shift(&self, input_range.start..map_range.end)),
                unmapped: vec![map_range.end..input_range.end],
            };
        }

        // Case 5 - fully overlaps the map.
        if input_range.start < map_range.start && input_range.end > map_range.end {
            return ApplyRangeResult {
                mapped: Some(shift(&self, map_range.start..map_range.end)),
                unmapped: vec![
                    input_range.start..map_range.start,
                    map_range.end..input_range.end,
                ],
            };
        }

        unreachable!("Should have covered every case above");
    }
}

impl SoilMap {
    fn apply(&self, v: usize) -> usize {
        // If any of the inner maps match it, return, otherwise default to the current value.
        self.0.iter().find_map(|s| s.apply(v)).unwrap_or(v)
    }

    fn apply_range(&self, range: Range<usize>) -> Vec<Range<usize>> {
        // Fun! Apply the map to a range. This might split a contiguous range up into multiple pieces.

        struct State {
            mapped: Vec<Range<usize>>,
            unmapped: Vec<Range<usize>>,
        }

        let state = State {
            mapped: vec![],
            unmapped: vec![range],
        };

        let state = self.0.iter().fold(state, |state, s| {
            // For each of our unmapped states, see if we can pass it through the map.
            let mut new_mapped = vec![];
            let mut new_unmapped = vec![];
            let old_mapped = state.mapped;
            let old_unmapped = state.unmapped;

            for range in old_unmapped.into_iter() {
                let ApplyRangeResult { mapped, unmapped } = s.apply_range(range);
                new_mapped.extend(mapped);
                new_unmapped.extend(unmapped);
            }

            let mut mapped = old_mapped;
            mapped.extend(new_mapped);

            State {
                mapped: mapped,
                unmapped: new_unmapped,
            }
        });

        let mut result = vec![];
        result.extend(state.mapped);
        // Anything unmapped after the loop just gets mapped to itself.
        result.extend(state.unmapped);
        result
    }
}

fn parse(s: &str) -> Problem {
    // I'm going to be lazy and assume the maps are in the right order in the file, so not read the X-to-Y map lines.
    // Maybe I'll need those for Part 2 and it'll come back to bite me!

    fn find_all_numbers(line: &str) -> Vec<usize> {
        // Returns all numbers in a line.
        line.split_whitespace()
            .filter_map(|s| s.parse::<usize>().ok())
            .collect()
    }

    let (first_line, rest) = s.split_once("\n").expect("Expected at least one new-line.");
    let input_seeds = find_all_numbers(first_line);
    assert!(input_seeds.len() != 0);

    let chunks: Vec<&str> = rest.split("\n\n").collect();
    let mut soil_maps = vec![];

    for chunk in chunks {
        let soil_maps_from_chunk = chunk
            .lines()
            .map(find_all_numbers)
            .filter(|v| !v.is_empty())
            .map(|v| {
                if v.len() != 3 {
                    panic!("Found a line with not 3 numbers: {:?}", v);
                }

                SoilMapOne {
                    input_start: v[1],
                    output_start: v[0],
                    length: v[2],
                }
            })
            .collect();

        soil_maps.push(SoilMap(soil_maps_from_chunk));
    }

    Problem {
        input_seeds,
        soil_maps,
    }
}

fn solve1(p: &Problem) -> usize {
    fn location_for_seed(p: &Problem, s: &usize) -> usize {
        let mut x = *s;
        for m in &p.soil_maps {
            x = m.apply(x);
        }
        x
    }

    p.input_seeds
        .iter()
        .map(|s| location_for_seed(p, s))
        .min()
        .expect("Validated there was at least one seed")
}

fn solve2(p: &Problem) -> usize {
    let mut ranges = vec![];
    for i in 0..p.input_seeds.len() {
        if i % 2 == 0 {
            ranges.push(p.input_seeds[i]..p.input_seeds[i] + p.input_seeds[i + 1])
        }
    }

    let x: Vec<Range<usize>> = p.soil_maps.iter().fold(ranges, |ranges, m| {
        let ranges = ranges
            .into_iter()
            .map(|r| m.apply_range(r).into_iter())
            .flatten()
            .collect();
        println!("Ranges after next stage: {:?}", ranges);
        ranges
    });

    x.iter()
        .map(|x| x.start)
        .min()
        .expect("Expected at least one input range")
}

fn main() {
    let s = include_str!("out2");
    let p = parse(s);
    println!("{:?}", solve1(&p));
    println!("{:?}", solve2(&p));
}
