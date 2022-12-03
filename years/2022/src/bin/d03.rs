use lib::prelude::*;

fn main() -> Result<()> {
    let mut input = lib::input!("d03.txt");

    let mut part1 = 0;
    let mut part2 = 0;

    while let Some(data) = input.try_line::<&str>()? {
        let (first, second) = data.split_at(data.len() / 2);
        let Set(first) = set(first);
        let Set(second) = set(second);
        part1 += (first & second).trailing_zeros();
    }

    input.reset();

    while let Some(Set(a)) = input.try_line::<Set>()? {
        let Set(b) = input.line::<Set>()?;
        let Set(c) = input.line::<Set>()?;
        part2 += ((a & b) & c).trailing_zeros();
    }

    assert_eq!(part1, 8233);
    assert_eq!(part2, 2821);
    Ok(())
}

fn score(c: char) -> u64 {
    match c {
        'a'..='z' => (c as u64 - 'a' as u64) + 1,
        'A'..='Z' => (c as u64 - 'A' as u64) + 27,
        _ => 0,
    }
}

lib::from_input! {
    |v: &'static str| -> Set(u64) { Ok(set(v)) }
}

fn set(string: &str) -> Set {
    Set(string.chars().fold(0, |n, c| n | 1u64 << score(c)))
}
