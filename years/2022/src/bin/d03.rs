use lib::prelude::*;

#[entry(input = "d03.txt", expect = (8233, 2821))]
fn main(mut input: Input) -> Result<(u32, u32)> {
    let mut part1 = 0;
    let mut part2 = 0;

    while let Some(data) = input.try_line::<&[u8]>()? {
        let (first, second) = data.split_at(data.len() / 2);
        part1 += (set(first) & set(second)).trailing_zeros();
    }

    input.reset();

    while let Some((S(a), S(b), S(c))) = input.try_next::<(S, S, S)>()? {
        part2 += (a & b & c).trailing_zeros();
    }

    Ok((part1, part2))
}

fn score(c: u8) -> u64 {
    match c {
        b'a'..=b'z' => (c as u64 - 'a' as u64) + 1,
        b'A'..=b'Z' => (c as u64 - 'A' as u64) + 27,
        _ => 0,
    }
}

struct S(u64);

lib::from_input! {
    |v: &BStr| -> S { Ok(S(set(v))) }
}

fn set(string: &[u8]) -> u64 {
    string.bytes().fold(0, |n, c| n | 1u64 << score(c))
}
