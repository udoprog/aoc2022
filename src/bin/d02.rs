use anyhow::Result;
use lib::Input;
use std::fs::File;

fn main() -> Result<()> {
    let input = parse("inputs/d02.txt")?;

    let mut part1 = 0;
    let mut part2 = 0;

    for (a, b) in &input {
        let b1 = match (a, b) {
            ('A', 'Z') | ('B', 'X') | ('C', 'Y') => 0,
            ('A', 'X') | ('B', 'Y') | ('C', 'Z') => 3,
            ('A', 'Y') | ('B', 'Z') | ('C', 'X') | _ => 6,
        };

        let s1 = match b {
            'X' => 1,
            'Y' => 2,
            'Z' | _ => 3,
        };

        let b2 = (s1 - 1) * 3;

        let s2 = match (a, b) {
            ('A', 'Y') | ('B', 'X') | ('C', 'Z') => 1,
            ('A', 'Z') | ('B', 'Y') | ('C', 'X') => 2,
            ('A', 'X') | ('B', 'Z') | ('C', 'Y') | _ => 3,
        };

        part1 += b1 + s1;
        part2 += b2 + s2;
    }

    assert_eq!(part1, 13682);
    assert_eq!(part2, 12881);
    Ok(())
}

/// Parse input lines.
fn parse(path: &str) -> Result<Vec<(char, char)>> {
    let path = path.as_ref();
    let reader = File::open(path)?;
    let mut input = Input::new(path, reader);

    let mut output = Vec::new();

    while let Some(a) = input.try_next::<u8>()? {
        let b = input.next::<u8>()?;
        output.push((a as char, b as char));
    }

    Ok(output)
}
