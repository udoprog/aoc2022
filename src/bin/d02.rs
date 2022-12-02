use anyhow::{bail, Result};
use lib::Input;

fn main() -> Result<()> {
    let mut input = lib::input!("inputs/d02.txt");

    let mut part1 = 0;
    let mut part2 = 0;

    while let Some((Move(a), Move(b))) = input.try_next()? {
        part1 += (2 - (a - b + 1).rem_euclid(3)) * 3 + b + 1;
        part2 += b * 3 + (a + b - 1).rem_euclid(3) + 1;
    }

    assert_eq!(part1, 13682);
    assert_eq!(part2, 12881);
    Ok(())
}

lib::map! {
    |v: &str| -> Move(i32) {
        Ok(Move(match v {
            "X" | "A" => 0,
            "Y" | "B" => 1,
            "Z" | "C" => 2,
            c => bail!("{c}"),
        }))
    }
}
