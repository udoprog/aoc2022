use lib::prelude::*;

const CAP: usize = 800;
const SAND: Point = Point { x: 500, y: 0 };
const STACK_CAP: usize = 1024;

#[entry(input = "d14.txt", expect = (1072, 24659))]
fn main(mut input: IStr) -> Result<(u32, u32)> {
    // Bitset keeping track of blocked tiles.
    let mut grid = [0u128; { CAP / 128 } * CAP];

    // Calculate floor, whether it be for part 2 or what counts as the infinite
    // limit.
    let mut floor = 0;

    // Stack that keeps track of the last drop position we deviated from so that
    // we can backtrack without having to re-run the simulation.
    let mut stack = ArrayVec::<Point, STACK_CAP>::new();

    while let Some(line) = input.line::<Option<IStr>>()? {
        let mut it = line.split(b"->");
        let mut last = it.next::<Point>()?.context("missing first")?;

        floor = floor.max(last.y);

        while let Some(to) = it.next::<Point>()? {
            floor = floor.max(to.y);

            for x in last.x.min(to.x)..=last.x.max(to.x) {
                for y in last.y.min(to.y)..=last.y.max(to.y) {
                    grid.set_bit(index(Point { x, y }));
                }
            }

            last = to;
        }

        anyhow::ensure!(floor < CAP as u32, "grid out of capacity");
    }

    let mut part1_done = false;
    let mut part1 = 0;
    let mut part2 = 0;

    stack.push(SAND);

    while let Some(mut s) = stack.pop() {
        'falling: while s.y <= floor {
            for n in neigh(&s)? {
                if !grid.test_bit(index(n)) {
                    stack.push(s);
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
