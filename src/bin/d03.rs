use lib::prelude::*;

fn main() -> Result<()> {
    let mut input = lib::input!("d03.txt");

    let mut part1 = 0;
    let mut part2 = 0;

    while let Some(data) = input.try_line::<&str>()? {
        let (first, second) = data.split_at(data.len() / 2);
        let first = Set::from_string(first).0;
        let second = Set::from_string(second).0;
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

fn score(c: char) -> u32 {
    match c {
        'a'..='z' => (c as u32 - 'a' as u32) + 1,
        'A'..='Z' => (c as u32 - 'A' as u32) + 27,
        c => panic!("{c}"),
    }
}

lib::from_input! {
    |v: &'static str| -> Set(u64) { Ok(Set::from_string(v)) }
}

impl Set {
    fn from_string(string: &str) -> Self {
        let mut n = 0u64;

        for c in string.chars() {
            n |= 1u64 << (score(c) as u64);
        }

        Self(n)
    }
}
