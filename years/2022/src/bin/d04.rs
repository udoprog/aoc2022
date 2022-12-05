use lib::input::muck::Muck2;
use lib::prelude::*;

#[entry(input = "d04.txt", expect = (582, 893))]
fn main(input: &mut Input) -> Result<(u32, u32)> {
    let mut part1 = 0;
    let mut part2 = 0;

    while let Some(Split((a, b))) = input.try_line::<Split<',', (Span, Span)>>()? {
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
    start: u8,
    end: u8,
}

lib::from_input! {
    |Split((Muck2(start), Muck2(end))): Split<'-', (Muck2, Muck2)>| -> Span {
        Ok(Span { start, end })
    }
}
