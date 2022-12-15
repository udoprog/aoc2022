use lib::prelude::*;

type Line = Split2<':', ' ', (([W; 2], Point), ([W; 4], Point))>;

const QUERY: i64 = 2_000_000;
const WIDTH: i64 = 4_000_000;

#[entry(input = "d15.txt", expect = (4873353, 11600823139120))]
fn main(mut input: IStr) -> Result<(i64, i64)> {
    let mut part2 = 0;

    // Rows of beacons.
    let mut rows = ArrayVec::<_, 50>::new();
    // Spans of beacons relative to current row.
    let mut buf = ArrayVec::<_, 50>::new();

    while let Some(Split2(((_, a), (_, b)))) = input.try_line::<Line>()? {
        rows.push((a, b));
    }

    let mut part1 = 0;
    let mut x = i64::MIN;

    for &(s, e) in spans(QUERY, &mut buf, &rows) {
        part1 += (e - x.max(s)).max(0);
        x = e.max(x);
    }

    for y in 0..=(WIDTH as i64) {
        let mut x = 0i64;

        for &(s, e) in spans(y, &mut buf, &rows) {
            if (s..=e).contains(&x) {
                x = e + 1;
            }
        }

        if x <= WIDTH {
            part2 = x * 4000000 + y;
            break;
        }
    }

    Ok((part1, part2))
}

/// Build a collection of covered spans for the given row `y`, and sort by
/// starting position.
fn spans<'a, const N: usize>(
    y: i64,
    buf: &'a mut ArrayVec<(i64, i64), N>,
    rows: &[(Point, Point)],
) -> &'a [(i64, i64)] {
    buf.clear();

    for (a, b) in rows.iter() {
        let d0 = dist(a, &Point { x: a.x, y });
        let d1 = dist(a, b);

        if d0 <= d1 {
            let d = d1 - d0;
            buf.push((a.x - d, a.x + d));
        }
    }

    buf.sort_unstable_by(|a, b| a.0.cmp(&b.0));
    &buf[..]
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct Point {
    x: i64,
    y: i64,
}

type Coord = Split<'=', (Skip, i64)>;

lib::from_input! {
    |Split((Split((_, x)), Split((_, y)))): Split<',', (Coord, Coord)>| -> Point {
        Ok(Point { x, y })
    }
}

#[inline]
fn dist(a: &Point, b: &Point) -> i64 {
    (b.x - a.x).abs() + (b.y - a.y).abs()
}
