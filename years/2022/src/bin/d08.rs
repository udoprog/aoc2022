use lib::prelude::*;

#[entry(input = "d08.txt", expect = (1814, 330786))]
fn main(mut input: IStr) -> Result<(u32, u32)> {
    let grid = input.as_data();
    let cols = input.line::<&[u8]>()?.len();

    let grid = grid.as_grid_with_stride(cols, 1);

    let mut part1 = 0;
    let mut part2 = 0;

    for (x, col) in grid.columns().enumerate() {
        for (y, (d, row)) in grid.rows().flat_map(|r| Some((*r.get(x)?, r))).enumerate() {
            // NB: Since the grid types implement nth, nth_back as appropriate
            // `take` and `skip` should be quite efficient.
            let (a, ea) = scan_edge(col.iter().take(y).rev(), d);
            let (b, eb) = scan_edge(col.iter().skip(y + 1), d);
            let (c, ec) = scan_edge(row.iter().take(x).rev(), d);
            let (d, ed) = scan_edge(row.iter().skip(x + 1), d);

            part2 = part2.max(a * b * c * d);
            part1 += u32::from(ea | eb | ec | ed);
        }
    }

    Ok((part1, part2))
}

/// Scan outwards to find the edge of the forest.
fn scan_edge<'a, I>(iter: I, cur: u8) -> (u32, bool)
where
    I: IntoIterator<Item = &'a u8>,
{
    let mut score = 0;
    let mut bool = true;

    for &d in iter {
        score += 1;

        if d >= cur {
            bool = false;
            break;
        }
    }

    (score, bool)
}
