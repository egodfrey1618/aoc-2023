use std::collections::HashMap;

fn string_to_number(s: &str) -> usize {
    let digits: Vec<usize> = s.chars().filter_map(|x| x.to_digit(10)).map(|x| x as usize).collect();

    // The first and last digit - will deal with only one digit in the string.
    10 * digits[0] + digits[digits.len() - 1]
}

fn string_to_number2(s: &str) -> usize {
    // More annoying - we also need to look for digits that are spelled out.

    let digit_map = HashMap::from([
        ("one", 1usize),
        ("two", 2),
        ("three", 3),
        ("four", 4),
        ("five", 5),
        ("six", 6),
        ("seven", 7),
        ("eight", 8),
        ("nine", 9)]);

    // I did this mutably rather than functionally like above.
    let mut position_to_digit = HashMap::<usize, usize>::new();

    for (i, c) in s.chars().enumerate() {
        match c.to_digit(10) {
            Some(x) => { position_to_digit.insert(i, x as usize); }
            None => {
                // See if this maps any of the digits.
                for (&digit_word, digit_value) in digit_map.iter() {
                    if s[i..].starts_with(digit_word) {
                        position_to_digit.insert(i, *digit_value);
                    }
                }
            }
        }
    }

    let keys = position_to_digit.keys();
    let min_key = keys.clone().min().expect("Expected at least one digit in the string");
    let max_key = keys.clone().max().expect("Expected at least one digit in the string");

    10 * position_to_digit[min_key] + position_to_digit[max_key]
}
fn main() {
    let s = include_str!("out");
    let lines = s.trim().split("\n");

    let total: usize = lines.clone().into_iter().map(string_to_number).sum();
    println!("Solution for part 1: {}", total);

    let total: usize = lines.clone().into_iter().map(string_to_number2).sum();

    println!("Solution for part 2: {}", total);
}
