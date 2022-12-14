use lib::prelude::*;

const CAP: usize = 800;
const SAND: Vec2 = Vec2 { x: 500, y: 0 };

#[entry(input = "d14.txt", expect = (1072, 24659))]
fn main(mut input: IStr) -> Result<(u32, u32)> {
    let mut grid = [0u128; { CAP / 128 } * CAP];
    let mut floor = 0;

    while let Some(Split2(line)) = input.try_line::<Split2<'-', '>', ArrayVec<Vec2, 24>>>()? {
        let mut it = line.into_iter();
        let mut last = it.next().context("missing first")?;

        floor = floor.max(last.y);

        for to in it {
            floor = floor.max(to.y);

            if last.y != to.y {
                for c in (last.y.min(to.y)..=last.y.max(to.y)).map(move |y| Vec2 { x: to.x, y }) {
                    grid.set_bit(index(c));
                }
            } else {
                for c in (last.x.min(to.x)..=last.x.max(to.x)).map(move |x| Vec2 { x, y: to.y }) {
                    grid.set_bit(index(c));
                }
            }

            last = to;
        }

        anyhow::ensure!(floor < CAP as u32, "grid out of capacity");
    }

    let mut part1_done = false;
    let mut part1 = 0;
    let mut part2 = 0;

    loop {
        let mut s = SAND;

        'falling: while s.y <= floor {
            for n in neigh(&s)? {
                if !grid.test_bit(index(n)) {
                    s = n;
                    continue 'falling;
                }
            }

            break;
        }

        part1_done |= s.y >= floor;
        part1 += u32::from(!part1_done);
        part2 += 1;
        grid.set_bit(index(s));

        if s == SAND {
            break;
        }
    }

    Ok((part1, part2))
}

#[inline]
fn index(pos: Vec2) -> u32 {
    pos.y * CAP as u32 + pos.x
}

fn neigh(&Vec2 { x, y }: &Vec2) -> Result<impl IntoIterator<Item = Vec2>> {
    anyhow::ensure!(x != 0 && x + 1 < CAP as u32, "x neighbour out-of-bounds");

    Ok([
        Vec2 { x, y: y + 1 },
        Vec2 { x: x - 1, y: y + 1 },
        Vec2 { x: x + 1, y: y + 1 },
    ])
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Vec2 {
    x: u32,
    y: u32,
}

lib::from_input! {
    |W(Split((x, y))): W<Split<',', (u32, u32)>>| -> Vec2 {
        Ok(Vec2 { x, y })
    }
}
