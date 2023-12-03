#[derive(Debug)]
struct NumberWithPos {
    number: usize,
    row: usize,
    start_col: usize,
    end_col: usize,
}

#[derive(Debug)]
struct Grid {
    raw_grid: Vec<Vec<char>>,
    numbers: Vec<NumberWithPos>,
    adjacent_to_symbol: Vec<Vec<bool>>,
}

fn parse_grid(raw_grid: Vec<Vec<char>>) -> Grid {
    // Find the numbers in the grid
    let mut numbers = vec![];

    for (row, row_chars) in raw_grid.iter().enumerate() {
        let mut next = 0usize;

        while next < row_chars.len() {
            // Find the next character <= next which is a number
            // Question: Is minus sign part of the number or a symbol?
            next = (next..row_chars.len())
                .map(|i| (i, row_chars[i]))
                .filter(|(_i, c)| c.is_digit(10))
                .map(|(x, _y)| x)
                .next()
                .unwrap_or(row_chars.len());

            if next != row_chars.len() {
                // Find the last character < next which is not a number. Save this down.
                // Then set next to be 1 after that position.

                // TODO: Should really factor this block out...
                let first_not_number = (next..row_chars.len())
                    .map(|i| (i, row_chars[i]))
                    .filter(|(_i, c)| !c.is_digit(10))
                    .map(|(x, _y)| x)
                    .next()
                    .unwrap_or(row_chars.len());

                let number: usize = String::from_iter(row_chars[next..first_not_number].iter())
                    .parse()
                    .expect("BUG: Checked all these were digits.");

                numbers.push(NumberWithPos {
                    number,
                    row,
                    start_col: next,
                    end_col: first_not_number - 1,
                });
                next = first_not_number;
            }
        }
    }

    let mut adjacent_to_symbol: Vec<Vec<bool>> = raw_grid
        .iter()
        .map(|s| s.iter().map(|_x| false).collect())
        .collect();

    // Mark cells in grid which are adjacent to symbol
    for (i, row) in raw_grid.iter().enumerate() {
        for (j, c) in row.iter().copied().enumerate() {
            let looks_like_symbol = !c.is_digit(10) && c != '.';

            if looks_like_symbol {
                let mut neighbouring_rows = vec![i];
                let mut neighbouring_cols = vec![j];
                if i != 0 {
                    neighbouring_rows.push(i - 1);
                }
                if i != raw_grid.len() - 1 {
                    neighbouring_rows.push(i + 1);
                }
                if j != 0 {
                    neighbouring_cols.push(j - 1);
                }
                if j != raw_grid[0].len() - 1 {
                    neighbouring_cols.push(j + 1);
                }

                for r in neighbouring_rows.iter().copied() {
                    for c in neighbouring_cols.iter().copied() {
                        if (r, c) != (i, j) {
                            adjacent_to_symbol[r][c] = true;
                        }
                    }
                }
            }
        }
    }

    Grid {
        raw_grid,
        numbers,
        adjacent_to_symbol,
    }
}

fn is_adjacent_to_symbol(grid: &Grid, number_with_pos: &NumberWithPos) -> bool {
    // For part 1, we need to identify which numbers are adjacent to any symbol.
    // When constructing the grid, we precached which symbols were with which nunber.
    let row = number_with_pos.row;
    for col in number_with_pos.start_col..=number_with_pos.end_col {
        if grid.adjacent_to_symbol[row][col] {
            return true;
        }
    }
    false
}

fn adjacent_numbers(grid: &Grid, row: usize, col: usize) -> Vec<&NumberWithPos> {
    // Given a cell, identify which numbers are adjacent to it.
    // (This is pretty inefficient - I'm just looping over all of them.)

    grid.numbers
        .iter()
        .filter(|number| {
            let cells = (number.start_col..=number.end_col).map(|j| (number.row, j));

            cells
                .map(|(i, j)| {
                    // Are they adjacent? Some extreme jank to do this because of signed v.s. unsigned.
                    fn zero_or_one(x1: usize, x2: usize) -> bool {
                        (x1 == x2) || (x1.saturating_sub(1) == x2) || (x2.saturating_sub(1) == x1)
                    }

                    zero_or_one(i, row) && zero_or_one(j, col)
                })
                .any(|x| x)
        })
        .collect()
}

fn find_gears(g: &Grid) -> Vec<(usize, usize, &NumberWithPos, &NumberWithPos)> {
    let mut result = vec![];

    for (i, row) in g.raw_grid.iter().enumerate() {
        for (j, c) in row.iter().copied().enumerate() {
            if c == '*' {
                let numbers = adjacent_numbers(g, i, j);
                println!("Checking {}, {}, {:?}", i, j, numbers);
                if numbers.len() == 2 {
                    result.push((i, j, numbers[0], numbers[1]));
                }
            }
        }
    }

    result
}

fn main() {
    let s = include_str!("out2");
    let raw_grid: Vec<Vec<char>> = s
        .trim()
        .split("\n")
        .into_iter()
        .map(|s| s.chars().collect())
        .collect();

    let grid = parse_grid(raw_grid);
    println!("{:?}", grid);

    let mut total = 0;
    for number in grid.numbers.iter() {
        if is_adjacent_to_symbol(&grid, number) {
            total += number.number
        }
    }
    println!("Solution for part 1: {}", total);

    let gears = find_gears(&grid);
    println!("Gears: {:?}", gears);

    let solution2: usize = gears
        .iter()
        .map(|(_x, _y, number1, number2)| number1.number * number2.number)
        .sum();
    println!("Solution for part 2: {}", solution2);
}
