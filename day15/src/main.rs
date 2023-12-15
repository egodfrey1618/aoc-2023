fn string_hash(s: &str) -> usize {
    s.chars().map(|c| c as u8).fold(0, |acc, x| {
        let acc = acc as i64;
        let x = x as i64;

        let y = ((acc + x) * 17) % 256;
        y as u8
    }) as usize
}

enum Move {
    Add {
        r#box: usize,
        lens_number: usize,
        label: String,
    },
    Remove {
        r#box: usize,
        label: String,
    },
}

#[derive(Debug, Clone)]
struct Lens {
    label: String,
    lens_number: usize,
}

#[derive(Debug)]
struct Boxes(Vec<Vec<Lens>>);

fn apply_move(boxes: &mut Boxes, r#move: Move) {
    // None of this is very efficient.

    match r#move {
        Move::Remove { r#box, label } => {
            let this_box = &mut boxes.0[r#box];

            // This is linear time - could do O(log n) with some sort of tree structure - but whatever.
            // Also unnecessarily clones the strings, but I'm not sure how to fix that.
            boxes.0[r#box] = this_box
                .iter()
                .filter(|lens| {
                    let Lens {
                        label: this_label,
                        lens_number: _,
                    } = lens;
                    *this_label != label
                })
                .cloned()
                .collect();
        }
        Move::Add {
            r#box,
            lens_number,
            label,
        } => {
            let this_box = &mut boxes.0[r#box];

            // Does the box already contain something of this label? If so, replace the lense number.
            let mut found_right_lens = false;

            for lens in this_box.iter_mut() {
                if lens.label == label {
                    lens.lens_number = lens_number;
                    found_right_lens = true;
                }
            }

            // And if not, push it onto the end.
            if !found_right_lens {
                this_box.push(Lens { label, lens_number })
            }
        }
    }
}

fn parse_move(s: &str) -> Move {
    // Moves are either:
    // STRING-
    // STRING=NUMBER

    if s.ends_with("-") {
        // We're a remove move.
        let label = s.strip_suffix("-").unwrap().to_string();
        let r#box = string_hash(&label);
        Move::Remove { r#box, label }
    } else {
        // We're an add move.
        let [label, lens_number]: [&str; 2] =
            s.split("=").collect::<Vec<&str>>().try_into().unwrap();
        let label = label.to_string();
        let r#box = string_hash(&label);
        let lens_number = lens_number.parse().unwrap();
        Move::Add {
            r#box,
            lens_number,
            label,
        }
    }
}

fn solve_part2(cases: Vec<&str>) -> usize {
    let mut boxes = Boxes(vec![]);
    for _ in 0..256 {
        boxes.0.push(vec![]);
    }

    let moves: Vec<Move> = cases.into_iter().map(parse_move).collect();

    for r#move in moves {
        apply_move(&mut boxes, r#move)
    }

    // Now compute the focusing power.
    let mut total = 0;

    for (box_index, r#box) in boxes.0.iter().enumerate() {
        let box_index = box_index + 1;

        for (lens_index, lens) in r#box.iter().enumerate() {
            let lens_index = lens_index + 1;
            total += box_index * lens_index * lens.lens_number
        }
    }

    println!("{:?}", boxes);
    total
}

fn main() {
    let cases: Vec<&str> = include_str!("input").trim().split(",").collect();

    let total1: usize = cases.iter().map(|x| string_hash(x)).sum();
    println!("{}", total1);

    println!("{}", solve_part2(cases));
}
