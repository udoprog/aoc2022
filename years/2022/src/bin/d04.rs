use lib::prelude::*;

type S = Split<b'-', u32, u32>;
type O = Split<b',', S, S>;

#[entry(input = "d04.txt", expect = (582, 893))]
fn main(mut input: Input) -> Result<(u32, u32)> {
    let mut part1 = 0;
    let mut part2 = 0;

    while let Some(Split(a, b)) = input.try_line::<O>()? {
        if a.0 >= b.0 && a.1 <= b.1 || b.0 >= a.0 && b.1 <= a.1 {
            part1 += 1;
        }

        if !(a.1 < b.0) && !(b.1 < a.0) {
            part2 += 1;
        }
    }

    Ok((part1, part2))
}
