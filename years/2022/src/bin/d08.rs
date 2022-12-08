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
    let mut scores = [[0u8; 4]; { CAP * CAP }];

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

    let mut s = scores.as_grid_mut(cols);

    for (y, r) in grid.rows().enumerate() {
        for (x, c) in grid.columns().enumerate() {
            let r = r.clone().into_iter().enumerate();
            let c = c.into_iter().enumerate();

            part2(c.clone().take(y).rev().map(|(y, &d)| (x, y, d)), &mut s, 0);
            part2(r.clone().take(x).rev().map(|(x, &d)| (x, y, d)), &mut s, 1);
            part2(c.skip(y + 1).map(|(y, &d)| (x, y, d)), &mut s, 2);
            part2(r.skip(x + 1).map(|(x, &d)| (x, y, d)), &mut s, 3);
        }
    }

    let mut part2 = 0;

    for row in s.rows() {
        for &[a, b, c, d] in row {
            part2 = part2.max(a as u32 * b as u32 * c as u32 * d as u32);
        }
    }

    Ok((part1, part2))
}

/// Score part two.
fn part2<I, S>(iter: I, scores: &mut S, n: usize)
where
    I: IntoIterator<Item = (usize, usize, u8)>,
    S: GridMut<[u8; 4]>,
{
    let mut c = 0;

    for (x, y, d) in iter {
        if c < d {
            scores.get_mut(y, x)[n] += 1;
            c = d;
        }

        if d == b'9' {
            break;
        }
    }
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
