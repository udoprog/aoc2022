use lib::prelude::*;

fn main() -> Result<()> {
    let mut input = lib::input!("d03.txt");

    let mut total = 0;
    let mut part2 = 0;

    while let Some(data) = input.try_line::<&str>()? {
        let half = data.len() / 2;
        let (first, second) = data.split_at(half);

        for c in first.chars() {
            if second.contains(c) {
                total += score(c);
                break;
            }
        }
    }

    input.reset();

    while let Some(Set(a)) = input.try_line::<Set>()? {
        let Set(b) = input.line::<Set>()?;
        let Set(c) = input.line::<Set>()?;

        let out = (a & b) & c;
        part2 += out.trailing_zeros();
    }

    assert_eq!(total, 8233);
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
    |v: &'static str| -> Set(u64) {
        let mut n = 0u64;

        for c in v.chars() {
            n |= 1u64 << (score(c) as u64);
        }

        Ok(Set(n))
    }
}
