#[derive(Debug)]
struct Hailstone {
    position: (f64, f64, f64),
    velocity: (f64, f64, f64),
}

#[derive(Debug)]
enum IntersectionResult {
    Parallel,
    IntersectedInPast,
    IntersectsAt((f64, f64)),
}
use IntersectionResult::*;

fn get_ray_intersection_ignoring_z_axis(h1: &Hailstone, h2: &Hailstone) -> IntersectionResult {
    // We want to solve p1 + t1*v1 == p2 + t2*v2.
    // (Or, to be more precise, know if there is a solution with t1, t2 > 0.)
    // Equivalently, t2*v2 - t1*v1 == p1 - p2. Which is just Gaussian elimination.
    // ... Which would be easy with Numpy! Blah.
    let (x1, y1, _z1) = h1.position;
    let (x2, y2, _z2) = h2.position;
    let (a1, b1, _c1) = h1.velocity;
    let (a2, b2, _c2) = h2.velocity;

    let p1 = (x1, y1);
    let p2 = (x2, y2);
    let v1 = (a1, b1);
    let v2 = (a2, b2);

    // Let's write both lines in the form Y = MX + B
    #[derive(Debug)]
    enum Line {
        YEqualsMXPlusB { slope: f64, intercept: f64 },
        XEquals { value: f64 },
    }
    use Line::*;

    let convert = |p: (f64, f64), v: (f64, f64)| -> Line {
        if v == (0.0, 0.0) {
            panic!("Velocity zero");
        }

        if v.0 == 0.0 {
            return Line::XEquals { value: p.0 };
        }

        let slope = v.1 / v.0;
        let intercept = p.1 - (slope * p.0);
        Line::YEqualsMXPlusB { slope, intercept }
    };
    let line1 = convert(p1, v1);
    let line2 = convert(p2, v2);

    let intersection = match (line1, line2) {
        (XEquals { value: value1 }, XEquals { value: value2 }) => {
            if value1 != value2 {
                None
            } else {
                panic!("lines are the same, don't know what to do here")
            }
        }
        (XEquals { value }, YEqualsMXPlusB { slope, intercept })
        | (YEqualsMXPlusB { slope, intercept }, XEquals { value }) => {
            Some((value, value * slope + intercept))
        }
        (
            YEqualsMXPlusB {
                slope: slope1,
                intercept: intercept1,
            },
            YEqualsMXPlusB {
                slope: slope2,
                intercept: intercept2,
            },
        ) => {
            if slope1 == slope2 {
                if intercept1 == intercept2 {
                    panic!("lines are the same, don't know what to do here");
                } else {
                    None
                }
            } else {
                let x = (intercept1 - intercept2) / (slope2 - slope1);
                let y = slope1 * x + intercept1;
                Some((x, y))
            }
        }
    };

    if intersection.is_none() {
        return Parallel;
    }
    let intersection = intersection.unwrap();

    let get_time_to_intersection = |start: (f64, f64), velocity: (f64, f64), point: (f64, f64)| {
        if velocity.0 == 0.0 {
            (point.1 - start.1) / velocity.1
        } else {
            (point.0 - start.0) / velocity.0
        }
    };

    let t1 = get_time_to_intersection(p1, v1, intersection);
    let t2 = get_time_to_intersection(p2, v2, intersection);

    if t1 < 0.0 || t2 < 0.0 {
        return IntersectedInPast;
    }
    return IntersectsAt(intersection);
}

fn parse(s: &str) -> Vec<Hailstone> {
    let mut hailstones = vec![];
    let parse_tuple = |s: &str| {
        let [x, y, z]: [f64; 3] = s
            .split(", ")
            .map(|s| s.trim().parse::<f64>().unwrap())
            .collect::<Vec<f64>>()
            .try_into()
            .unwrap();
        (x, y, z)
    };

    for line in s.trim().lines() {
        let parts: (&str, &str) = line.split_once(" @ ").unwrap();

        let position = parse_tuple(parts.0);
        let velocity = parse_tuple(parts.1);
        let hailstone = Hailstone { position, velocity };
        hailstones.push(hailstone);
    }
    hailstones
}

fn main() {
    let hailstones = parse(include_str!("input"));

    // Part 1
    let mut result1 = 0;

    let min = 200_000_000_000_000f64;
    let max = 400_000_000_000_000f64;

    for i in 0..hailstones.len() {
        for j in i + 1..hailstones.len() {
            let h1 = &hailstones[i];
            let h2 = &hailstones[j];
            let intersection = get_ray_intersection_ignoring_z_axis(&h1, &h2);

            // remove
            match intersection {
                Parallel => {
                    println!("{:?}, {:?}", h1.velocity, h2.velocity);
                }
                _ => (),
            }

            match intersection {
                Parallel => (),
                IntersectedInPast => (),
                IntersectsAt((x, y)) => {
                    if x >= min && x <= max && y >= min && y <= max {
                        result1 += 1;
                    }
                }
            }
        }
    }
    println!("Result for part 1: {}", result1);
}
