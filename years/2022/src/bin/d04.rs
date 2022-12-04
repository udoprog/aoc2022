use lib::prelude::*;

type S = Split<'-', Span>;
type O = Split<',', (S, S)>;

#[entry(input = "d04.txt", expect = (582, 893))]
fn main(mut input: Input) -> Result<(u32, u32)> {
    let mut part1 = 0;
    let mut part2 = 0;

    while let Some(Split((Split(a), Split(b)))) = input.try_line::<O>()? {
        if a.start >= b.start && a.end <= b.end || b.start >= a.start && b.end <= a.end {
            part1 += 1;
        }

        if a.end >= b.start && b.end >= a.start {
            part2 += 1;
        }
    }

    Ok((part1, part2))
}

struct Span {
    start: u32,
    end: u32,
}

lib::from_input_iter! {
    |(start, end): (u32, u32)| -> Span {
        Ok(Span { start, end })
    }
}
