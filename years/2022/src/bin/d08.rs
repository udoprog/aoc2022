use lib::prelude::*;

/// Max grid width / height.
const CAP: usize = 128;

/// Rough description of Part 2, because I'm bound to forget it:
///
/// Scan outwards from each tree to find which other higher non-obstructed tree
/// finds it and keep track of visibility. So if a lower tree sees a higher
/// tree, the higher tree (i.e. where we're building) sees the lower tree as
/// well.
#[entry(input = "d08.txt", expect = (1814, 330786))]
fn main(mut input: IStr) -> Result<(u32, u32)> {
    let mut mask = ArrayVec::<u128, CAP>::new();
    let mut scores = [0u32; { CAP * CAP }];

    let grid = input.as_data();
    let cols = input.line::<&[u8]>()?.len();

    anyhow::ensure!(
        cols <= CAP,
        "unsupported number of columns {cols}, expected at most {CAP}"
    );

    let grid = grid.as_grid_with_stride(cols, 1);

    for _ in 0..grid.rows_len() {
        mask.try_push(0)?;
    }

    let mut scores = scores.as_grid_mut(cols);

    for (y, row) in grid.rows().enumerate() {
        let it = row.into_iter().enumerate().map(|(x, &d)| (x, y, d));
        let back = it.clone();
        let (x, _) = part1_scan(it, &mut mask);
        part1_scan(back.skip(x).rev(), &mut mask);
    }

    for (x, col) in grid.columns().enumerate() {
        let it = col.into_iter().enumerate().map(|(y, &d)| (x, y, d));
        let back = it.clone();
        let (_, y) = part1_scan(it, &mut mask);
        part1_scan(back.skip(y).rev(), &mut mask);
    }

    let part1 = mask.count_ones();

    let set = |x: usize, y: usize, c: &mut u8| {
        let d = *grid.get(y, x);

        if *c < d {
            *c = d;
            1
        } else {
            0
        }
    };

    // TODO: rework to do less work.
    for y in 0..grid.rows_len() {
        for x in 0..grid.columns_len() {
            let mut c = 0;

            for y in (0..y).rev() {
                *scores.get_mut(y, x) += set(x, y, &mut c);
            }

            let mut c = 0;

            for y in y + 1..grid.rows_len() {
                *scores.get_mut(y, x) += set(x, y, &mut c) << 8;
            }

            let mut c = 0;

            for x in (0..x).rev() {
                *scores.get_mut(y, x) += set(x, y, &mut c) << 16;
            }

            let mut c = 0;

            for x in x + 1..grid.columns_len() {
                *scores.get_mut(y, x) += set(x, y, &mut c) << 24;
            }
        }
    }

    let mut part2 = 0;

    for row in scores.rows() {
        for &n in row {
            const M: u32 = 0b11111111;
            let score = ((n >> 24) & M) * ((n >> 16) & M) * ((n >> 8) & M) * (n & M);
            part2 = part2.max(score);
        }
    }

    Ok((part1, part2))
}

/// Scan an iterator over trees, returning the last position that one was seen.
fn part1_scan<I>(iter: I, mask: &mut [u128]) -> (usize, usize)
where
    I: Iterator<Item = (usize, usize, u8)> + Clone,
{
    let mut c = 0;

    let mut last = (0, 0);

    for (x, y, d) in iter {
        if c < d {
            mask[y].set_bit(x as u32);
            c = d;
            last = (x, y);

            // Break on largest possible tree.
            if d == b'9' {
                break;
            }
        }
    }

    last
}
