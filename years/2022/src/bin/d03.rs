use lib::prelude::*;

fn main() -> Result<()> {
    let mut input = lib::input!("d03.txt");

    let mut part1 = 0;
    let mut part2 = 0;

    while let Some(data) = input.try_line::<&[u8]>()? {
        let (first, second) = data.split_at(data.len() / 2);
        let first = set(first);
        let second = set(second);
        part1 += (first & second).trailing_zeros();
    }

    input.reset();

    while let Some((Set(a), Set(b), Set(c))) = input.try_next::<(Set, Set, Set)>()? {
        part2 += (a & b & c).trailing_zeros();
    }

    assert_eq!(part1, 8233);
    assert_eq!(part2, 2821);
    Ok(())
}

fn score(c: u8) -> u64 {
    match c {
        b'a'..=b'z' => (c as u64 - 'a' as u64) + 1,
        b'A'..=b'Z' => (c as u64 - 'A' as u64) + 27,
        _ => 0,
    }
}

struct Set(u64);

lib::from_input! {
    |v: &[u8]| -> Set { Ok(Set(set(v))) }
}

fn set(string: &[u8]) -> u64 {
    string.bytes().fold(0, |n, c| n | 1u64 << score(c))
}
