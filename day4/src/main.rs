use std::collections::HashSet;

struct Card {
    winning_numbers: HashSet<usize>,
    our_numbers: Vec<usize>,
}

fn parse_line(s: &str) -> Card {
    println!("{}", s);
    let parts: Vec<&str> = s.split(":").collect();
    if parts.len() != 2 {
        panic!("Expected exactly one colon in string.")
    }

    let parts: Vec<&str> = parts[1].split(" | ").collect();
    if parts.len() != 2 {
        panic!("Expected exactly one pipe in string.")
    }

    fn parse_exn(s: &str) -> usize {
        s.parse().expect("Could not parse as number")
    }

    // Need split_whitespace because some numbers have two spaces
    let winning_numbers = parts[0].split_whitespace().map(parse_exn).collect();
    let our_numbers = parts[1].split_whitespace().map(parse_exn).collect();

    Card {
        winning_numbers,
        our_numbers,
    }
}

fn number_of_winning_numbers(card: &Card) -> usize {
    card.our_numbers
        .iter()
        .copied()
        .filter(|x| card.winning_numbers.contains(x))
        .count()
}

fn score(card: &Card) -> usize {
    let number_of_winning_numbers = number_of_winning_numbers(card);
    if number_of_winning_numbers == 0 {
        return 0usize;
    }
    {
        2usize.pow((number_of_winning_numbers - 1) as u32)
    }
}

fn score_part2(cards: &Vec<Card>) -> usize {
    let mut number_of_cards: Vec<usize> = cards.iter().map(|_card| 1).collect();

    for (index, card) in cards.iter().enumerate() {
        let number_of_this_card = number_of_cards[index];
        let n = number_of_winning_numbers(card);

        // If we have n winning numbers, and M of this card, that gives us M copies of the next n cards.
        // I haven't thought about overflow here - the problem is ambiguous.
        for i in index + 1..=index + n {
            number_of_cards[i] += number_of_this_card
        }
    }

    number_of_cards.iter().sum()
}

fn main() {
    let cards: Vec<Card> = include_str!("out2")
        .trim()
        .lines()
        .map(parse_line)
        .collect();
    let score1: usize = cards.iter().map(score).sum();
    println!("Solution for part 1: {}", score1);

    let score2: usize = score_part2(&cards);
    println!("Solution for part 2: {}", score2);
}
