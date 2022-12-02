use anyhow::Result;
use lib::Input;
use std::fs::File;

fn main() -> Result<()> {
    let input = parse("inputs/d02.txt")?;

    let mut part1 = 0;
    let mut part2 = 0;

    for (a, b) in input {
        // Discovered numerical relationship after mangling the match statements
        // a bit:
        part1 += (2 - (a - b + 1).rem_euclid(3)) * 3 + b + 1;
        part2 += b * 3 + (a + b - 1).rem_euclid(3) + 1;
    }

    assert_eq!(part1, 13682);
    assert_eq!(part2, 12881);
    Ok(())
}

/// Parse input lines.
fn parse(path: &str) -> Result<Vec<(i32, i32)>> {
    let path = path.as_ref();
    let reader = File::open(path)?;
    let mut input = Input::new(path, reader);

    let mut output = Vec::new();

    while let Some(a) = input.try_next::<u8>()? {
        let b = input.next::<u8>()?;
        output.push((to_move(a), to_move(b)));
    }

    Ok(output)
}

fn to_move(v: u8) -> i32 {
    match v {
        b'X' | b'A' => 0,
        b'Y' | b'B' => 1,
        _ => 2,
    }
}
