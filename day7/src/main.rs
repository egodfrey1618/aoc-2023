use std::cmp::Ordering;
use std::collections::HashMap;

#[derive(Debug, PartialEq, Eq)]
enum Category {
    FiveOfAKind,
    FourOfAKind,
    FullHouse,
    ThreeOfAKind,
    TwoPair,
    OnePair,
    HighCard,
}

impl Category {
    fn to_usize(&self) -> usize {
        // Higher means better.
        match self {
            Category::FiveOfAKind => 6,
            Category::FourOfAKind => 5,
            Category::FullHouse => 4,
            Category::ThreeOfAKind => 3,
            Category::TwoPair => 2,
            Category::OnePair => 1,
            Category::HighCard => 0,
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
struct Hand {
    category: Category,
    raw_hand: [usize; 5],
    bid: usize,
}

impl Hand {
    fn cmp(&self, other: &Self) -> Ordering {
        // First of all, compare the category.
        match self.category.to_usize().cmp(&other.category.to_usize()) {
            Ordering::Less => Ordering::Less,
            Ordering::Greater => Ordering::Greater,
            Ordering::Equal => {
                // Of the same category - fall back to comparing the raw hands.
                self.raw_hand.cmp(&other.raw_hand)
            }
        }
    }

    fn parse(line: &str, apply_joker_rule: bool) -> Self {
        // This supports both with and without jokers. The joker only comes in in one place - whether or not
        // we interpret jokers as "1" or "11". The rest of the code then works in the same way.
        let words: [&str; 2] = line
            .split_whitespace()
            .collect::<Vec<&str>>()
            .try_into()
            .expect("Expected 2 words.");

        let bid = words[1]
            .parse::<usize>()
            .expect("Failed to read bid as string");

        let raw_hand: [usize; 5] = words[0]
            .chars()
            .map(|x| match x.to_digit(10) {
                Some(x) => x as usize,
                None => match x {
                    'T' => 10,
                    'J' => {
                        if apply_joker_rule {
                            1
                        } else {
                            11
                        }
                    }
                    'Q' => 12,
                    'K' => 13,
                    'A' => 14,
                    _ => panic!("unrecognised letter: {}", x),
                },
            })
            .collect::<Vec<usize>>()
            .try_into()
            .expect("Expected 5 characters");

        let mut numbers_to_count = HashMap::new();
        for x in raw_hand {
            let y = numbers_to_count.entry(x).or_insert(0usize);
            *y += 1
        }

        // Return the number of "normal" numbers - i.e. not including jokers - that appear n times.
        let get_numbers_appearing_n_times = |n: usize| -> Vec<usize> {
            numbers_to_count
                .iter()
                .filter(|(x, y)| **x != 1 && **y == n)
                .map(|(x, _y)| x)
                .copied()
                .collect()
        };

        let five = get_numbers_appearing_n_times(5);
        let four = get_numbers_appearing_n_times(4);
        let three = get_numbers_appearing_n_times(3);
        let two = get_numbers_appearing_n_times(2);
        let one = get_numbers_appearing_n_times(1);
        let number_of_jokers = *numbers_to_count.get(&1).unwrap_or(&0);

        let category = {
            match (
                five.len(),
                four.len(),
                three.len(),
                two.len(),
                one.len(),
                number_of_jokers,
            ) {
                // Possibilities with 5 jokers.
                (0, 0, 0, 0, 0, 5) => Category::FiveOfAKind,
                // Possibilities with 4 jokers.
                (0, 0, 0, 0, 1, 4) => Category::FiveOfAKind,
                // Possibilities with 3 jokers.
                (0, 0, 0, 0, 2, 3) => Category::FourOfAKind,
                (0, 0, 0, 1, 0, 3) => Category::FiveOfAKind,
                // Possibilities with 2 jokers.
                (0, 0, 0, 0, 3, 2) => Category::ThreeOfAKind,
                (0, 0, 0, 1, 1, 2) => Category::FourOfAKind,
                (0, 0, 1, 0, 0, 2) => Category::FiveOfAKind,
                // Possibilities with 1 joker.
                (0, 0, 0, 0, 4, 1) => Category::OnePair,
                (0, 0, 0, 1, 2, 1) => Category::ThreeOfAKind,
                (0, 0, 0, 2, 0, 1) => Category::FullHouse,
                (0, 0, 1, 0, 1, 1) => Category::FourOfAKind,
                (0, 1, 0, 0, 0, 1) => Category::FiveOfAKind,
                // Possibilities with 0 jokers.
                (1, 0, 0, 0, 0, 0) => Category::FiveOfAKind,
                (0, 1, 0, 0, 1, 0) => Category::FourOfAKind,
                (0, 0, 1, 1, 0, 0) => Category::FullHouse,
                (0, 0, 1, 0, 2, 0) => Category::ThreeOfAKind,
                (0, 0, 0, 2, 1, 0) => Category::TwoPair,
                (0, 0, 0, 1, 3, 0) => Category::OnePair,
                (0, 0, 0, 0, 5, 0) => Category::HighCard,
                _ => panic!("Shouldn't be able to get any other ways of making 5."),
            }
        };

        Hand {
            raw_hand,
            category,
            bid,
        }
    }
}

fn main() {
    let mut hands1: Vec<Hand> = include_str!("input2")
        .trim()
        .lines()
        .map(|s| Hand::parse(s, false))
        .collect();

    let mut hands2: Vec<Hand> = include_str!("input2")
        .trim()
        .lines()
        .map(|s| Hand::parse(s, true))
        .collect();

    hands1.sort_by(Hand::cmp);
    hands2.sort_by(Hand::cmp);

    let result: usize = hands1
        .iter()
        .enumerate()
        .map(|(i, h)| (i + 1) * h.bid)
        .sum();
    println!("Solution for part 1: {}", result);

    let result: usize = hands2
        .iter()
        .enumerate()
        .map(|(i, h)| (i + 1) * h.bid)
        .sum();
    println!("Solution for part 2: {}", result);
}
