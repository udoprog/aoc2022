use lib::prelude::*;

const CAP: usize = 800;
const SAND: Point = Point { x: 500, y: 0 };

#[entry(input = "d14.txt", expect = (1072, 24659))]
fn main(mut input: IStr) -> Result<(u32, u32)> {
    let mut grid = [0u128; { CAP / 128 } * CAP];
    let mut floor = 0;

    while let Some(mut line) = input.try_line::<IStr>()? {
        let mut it = line.split(b"->");
        let mut last = it.next::<Point>()?.context("missing first")?;

        floor = floor.max(last.y);

        while let Some(to) = it.next::<Point>()? {
            floor = floor.max(to.y);

            if last.y != to.y {
                for c in (last.y.min(to.y)..=last.y.max(to.y)).map(move |y| Point { x: to.x, y }) {
                    grid.set_bit(index(c));
                }
            } else {
                for c in (last.x.min(to.x)..=last.x.max(to.x)).map(move |x| Point { x, y: to.y }) {
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
fn index(pos: Point) -> u32 {
    pos.y * CAP as u32 + pos.x
}

fn neigh(&Point { x, y }: &Point) -> Result<impl IntoIterator<Item = Point>> {
    anyhow::ensure!(x != 0 && x + 1 < CAP as u32, "x neighbour out-of-bounds");

    Ok([
        Point { x, y: y + 1 },
        Point { x: x - 1, y: y + 1 },
        Point { x: x + 1, y: y + 1 },
    ])
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Point {
    x: u32,
    y: u32,
}

lib::from_input! {
    |W(Split((x, y))): W<Split<',', (u32, u32)>>| -> Point {
        Ok(Point { x, y })
    }
}
