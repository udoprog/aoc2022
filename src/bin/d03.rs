use lib::prelude::*;

fn main() -> Result<()> {
    let mut input = lib::input!("d03.txt");

    let mut total = 0;
    let mut part2 = 0;

    while let Some(data) = input.try_line::<&str>()? {
        let half = data.len() / 2;
        let (first, second) = data.split_at(half);

        for c in first.chars() {
            if second.contains(c) {
                total += score(c);
                break;
            }
        }
    }

    input.reset();

    while let Some(a) = input.try_line::<&str>()? {
        let b = input.line::<&str>()?;
        let c = input.line::<&str>()?;

        for d in a.chars() {
            if b.contains(d) && c.contains(d) {
                part2 += score(d);
                break;
            }
        }
    }

    assert_eq!(total, 8233);
    assert_eq!(part2, 2821);
    Ok(())
}

fn score(c: char) -> u32 {
    match c {
        'a'..='z' => (c as u32 - 'a' as u32) + 1,
        'A'..='Z' => (c as u32 - 'A' as u32) + 27,
        c => panic!("{c}"),
    }
}
