use lib::prelude::*;

#[entry(input = "d02.txt", expect = (13682, 12881))]
fn main(mut input: Input) -> Result<(i32, i32)> {
    let mut part1 = 0;
    let mut part2 = 0;

    while let Some((Move(a), Move(b))) = input.try_next()? {
        part1 += (2 - (a - b + 1).rem_euclid(3)) * 3 + b + 1;
        part2 += b * 3 + (a + b - 1).rem_euclid(3) + 1;
    }

    Ok((part1, part2))
}

struct Move(i32);

lib::from_input! {
    |v: &'static str| -> Move {
        Ok(Move(match v {
            "X" | "A" => 0,
            "Y" | "B" => 1,
            "Z" | "C" => 2,
            c => bail!(c),
        }))
    }
}
