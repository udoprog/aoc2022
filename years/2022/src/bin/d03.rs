use lib::prelude::*;

#[entry(input = "d03.txt", expect = (8233, 2821))]
fn main(mut input: IStr) -> Result<(u32, u32)> {
    let mut input2 = input;

    let mut part1 = 0;
    let mut part2 = 0;

    while let Some(W(data)) = input.try_line::<W<&[u8]>>()? {
        let (first, second) = data.split_at(data.len() / 2);
        part1 += (set(first) & set(second)).trailing_zeros();
    }

    while let Some((S(a), S(b), S(c))) = input2.try_next::<(S, S, S)>()? {
        part2 += (a & b & c).trailing_zeros();
    }

    Ok((part1, part2))
}

fn score(c: u8) -> u32 {
    match c {
        b'a'..=b'z' => (c as u32 - 'a' as u32) + 1,
        b'A'..=b'Z' => (c as u32 - 'A' as u32) + 27,
        _ => 0,
    }
}

struct S(u64);

lib::from_input! {
    |W(v): W<&'static [u8]>| -> S {
        Ok(S(set(v)))
    }
}

#[inline]
fn set(string: &[u8]) -> u64 {
    string.bytes().fold(0, |n, c| n.with_bit(score(c)))
}
