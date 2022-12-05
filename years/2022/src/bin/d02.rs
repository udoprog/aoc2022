use lib::prelude::*;

#[entry(input = "d02.txt", expect = (13682, 12881))]
fn main(input: &mut IStr) -> Result<(i32, i32)> {
    let mut part1 = 0;
    let mut part2 = 0;

    for value in input.iter() {
        let (Move(a), Move(b)) = value?;
        part1 += (2 - (a - b + 1).rem_euclid(3)) * 3 + b + 1;
        part2 += b * 3 + (a + b - 1).rem_euclid(3) + 1;
    }

    Ok((part1, part2))
}

struct Move(i32);

lib::from_input! {
    |W(v): W<&'static str>| -> Move {
        Ok(Move(match v {
            "X" | "A" => 0,
            "Y" | "B" => 1,
            "Z" | "C" => 2,
            c => bail!(c),
        }))
    }
}
