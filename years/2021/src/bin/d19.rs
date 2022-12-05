use std::ops::Sub;

use lib::prelude::*;

#[entry(input = "d19.txt", expect = (0, 0))]
fn main(input: &mut IStr) -> Result<(u32, u32)> {
    let part1 = 0;
    let part2 = 0;

    while let Some(title) = input.try_line::<&str>()? {
        println!("{title}");
        let first = input.line::<Coord>()?;

        while let Some(c) = input.try_line::<Coord>()? {
            let c = c - first;
            println!("{},{},{}", c.x, c.y, c.z);
        }

        input.ws()?;
    }

    Ok((part1, part2))
}

#[derive(Debug, Clone, Copy)]
struct Coord {
    x: i32,
    y: i32,
    z: i32,
}

impl Sub for Coord {
    type Output = Self;

    #[inline]
    fn sub(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
            z: self.z - rhs.z,
        }
    }
}

lib::from_input! {
    |Split((x, y, z)): Split<',', (i32, i32, i32)>| -> Coord { Ok(Coord { x, y, z }) }
}
