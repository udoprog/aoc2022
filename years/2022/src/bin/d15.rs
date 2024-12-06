use lib::prelude::*;

type Line = Split2<':', ' ', (([W; 2], Point), ([W; 4], Point))>;

const QUERY: i32 = 2_000_000;
const WIDTH: i32 = 4_000_000;
const BEACONS: usize = 64;

#[entry(input = "d15.txt", expect = (4873353, 11600823139120))]
fn main(mut input: IStr) -> Result<(u32, u64)> {
    // Spans of beacons relative to current row.
    let mut buf = [(i32::MIN, i32::MAX); BEACONS];
    // Computed distances which gives an idea of the covering span of a beacon.
    let mut computed = ArrayVec::<_, BEACONS>::new();

    while let Some(Split2(((_, a), (_, b)))) = input.line::<Option<Line>>()? {
        computed.push((a, (b.x - a.x).abs() + (b.y - a.y).abs()));
    }

    // NB: Sorting heuristic. Reduces the amount of sorting done for each span.
    computed.sort_by(|a, b| {
        let a = (a.0.x - a.1, a.0.y - a.1);
        let b = (b.0.x - b.1, b.0.y - b.1);
        a.cmp(&b)
    });

    let mut part1 = 0;
    let mut x = i32::MIN;

    for &(s, e) in spans(QUERY, &mut buf, &computed)? {
        part1 += (e - x.max(s)).max(0) as u32;
        x = e.max(x);
    }

    let mut part2 = 0;

    for y in 0..=(WIDTH as i32) {
        let mut x = 0;

        for &(s, e) in spans(y, &mut buf, &computed)? {
            if (s..=e).contains(&x) {
                x = e + 1;
            }
        }

        if x <= WIDTH {
            part2 = x as u64 * 4000000 + y as u64;
            break;
        }
    }

    Ok((part1, part2))
}

/// Build a collection of covered spans for the given row `y`, and sort by
/// starting position.
#[inline]
fn spans<'a>(
    y: i32,
    buf: &'a mut [(i32, i32)],
    computed: &[(Point, i32)],
) -> Result<&'a [(i32, i32)]> {
    let mut len = 0;

    for (a, d) in computed {
        let w = d - (y - a.y).abs();

        if w >= 0 {
            let o = buf.get_mut(len).context("missing index")?;
            *o = (a.x - w, a.x + w);
            len += 1;
        }
    }

    buf[..len].sort_unstable_by(|a, b| a.0.cmp(&b.0));
    Ok(&buf[..len])
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct Point {
    x: i32,
    y: i32,
}

type Coord = Split<'=', (Skip, i32)>;

lib::from_input! {
    |Split((Split((_, x)), Split((_, y)))): Split<',', (Coord, Coord)>| -> Point {
        Ok(Point { x, y })
    }
}
