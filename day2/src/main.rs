#[derive(Debug)]
struct Draw {
    red: usize,
    blue: usize,
    green: usize,
}

#[derive(Debug)]
struct Game {
    id: usize,
    draws: Vec<Draw>,
}

fn parse_draw(s: &str) -> Draw {
    // Format of a draw is
    // "N red, M green, K blue"
    // in some order

    let mut draw = Draw {
        red: 0,
        blue: 0,
        green: 0,
    };

    let parts: Vec<&str> = s.split(",").map(|s| s.trim()).collect();

    for part in parts {
        let words: Vec<&str> = part.split(" ").collect();
        let n: usize = words[0].parse().expect("Couldn't parse as a number");
        match words[1] {
            "red" => draw.red = n,
            "green" => draw.green = n,
            "blue" => draw.blue = n,
            _ => {
                panic!("Didn't recognise colour {}", words[1])
            }
        }

        if words.len() != 2 {
            panic!(
                "Expected exactly two words in each part of a draw, got {:?}",
                words
            )
        }
    }

    draw
}

fn parse_game(s: &str) -> Game {
    // Format of the game is:
    // "Game N: <semicolon separated games>"
    // where a game is
    // "N red, M green, K blue"
    // in some order

    println!("{}", s);
    let s = s
        .strip_prefix("Game ")
        .expect("Expected string to start with 'Game '");
    let parts: Vec<&str> = s.split(":").collect();
    let id: usize = parts[0].parse().expect("Expected game ID to be a number");

    if parts.len() != 2 {
        panic!("Expected exactly one semicolon in string");
    }

    let draws = parts[1];
    let draws: Vec<Draw> = draws.split(";").map(|s| s.trim()).map(parse_draw).collect();

    Game { id, draws }
}

fn game_valid1(game: &Game) -> bool {
    fn draw_valid(draw: &Draw) -> bool {
        draw.red <= 12 && draw.green <= 13 && draw.blue <= 14
    }

    game.draws.iter().all(draw_valid)
}

fn game_power(game: &Game) -> usize {
    // The product of the minimum number of cubes in this game.

    fn max_draw(d1: Draw, d2: &Draw) -> Draw {
        let red = d1.red.max(d2.red);
        let green = d1.green.max(d2.green);
        let blue = d1.blue.max(d2.blue);
        Draw { red, green, blue }
    }

    // urgh, wanted to use reduce here but I couldn't figure out how to get the borrow checker to like it.
    let mut draw = Draw {
        red: 0,
        green: 0,
        blue: 0,
    };
    for other_draw in &game.draws {
        draw = max_draw(draw, other_draw);
    }

    draw.red * draw.green * draw.blue
}

fn main() {
    let s = include_str!("out");
    let games: Vec<Game> = s.trim().split("\n").map(parse_game).collect();

    let solution1: usize = games.iter().filter(|g| game_valid1(*g)).map(|g| g.id).sum();
    println!("Solution for part 1 is {}", solution1);

    let solution2: usize = games.iter().map(game_power).sum();
    println!("Solution for part 2 is {}", solution2);
}
