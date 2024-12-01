use lib::prelude::*;

#[entry(input = "d02.txt", expect = (2632, 69629))]
fn main(mut input: IStr) -> Result<(u32, u32)> {
    let base = (12, 13, 14);

    let mut o1 = 0;
    let mut o2 = 0;

    while let Some(value) = input.try_line::<ArrayString<256>>()? {
        let Some((game, rest)) = value.split_once(": ") else {
            continue;
        };

        let (_, id) = game.split_once(" ").context("bad game")?;
        let id = id.parse::<u32>()?;

        let mut possible = true;
        let mut cols2 = (0, 0, 0);

        for part in rest.split("; ") {
            let mut cols = base;

            for pull in part.split(", ") {
                let (count, color) = pull.split_once(" ").context("bad move")?;
                let count = count.parse::<u32>()?;

                let (c1, c2) = match color {
                    "red" => (&mut cols.0, &mut cols2.0),
                    "green" => (&mut cols.1, &mut cols2.1),
                    "blue" => (&mut cols.2, &mut cols2.2),
                    _ => continue,
                };

                if *c1 < count {
                    possible = false;
                } else {
                    *c1 -= count;
                }

                *c2 = (*c2).max(count);
            }
        }

        if possible {
            o1 += id;
        }

        o2 += cols2.0 * cols2.1 * cols2.2;
    }

    Ok((o1, o2))
}
