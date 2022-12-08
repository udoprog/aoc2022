use lib::prelude::*;

/// Max grid width / height.
const CAP: usize = 100;

/// Rough description of Part 2, because I'm bound to forget it:
///
/// Scan outwards from each tree to find which other higher non-obstructed tree
/// finds it and keep track of visibility. So if a lower tree sees a higher
/// tree, the higher tree (i.e. where we're building) sees the lower tree as
/// well.
#[entry(input = "d08.txt", expect = (1814, 330786))]
fn main(mut input: IStr) -> Result<(u32, u32)> {
    let mut grid = ArrayVec::<u8, { CAP * CAP }>::new();
    let mut mask = ArrayVec::<u128, CAP>::new();
    let mut seen = [0u32; { CAP * CAP }];

    let mut cols = 0;

    while let Some(line) = input.try_line::<&[u8]>()? {
        if line.is_empty() {
            break;
        }

        grid.try_extend_from_slice(line)?;
        mask.try_push(0)?;
        cols = cols.max(line.len());
    }

    let grid = grid.as_grid(cols);
    let mut seen = seen.as_grid_mut(cols);

    let mut set = |x: usize, y: usize, c: &mut u8| {
        let d = *grid.get(y, x);

        if *c < d {
            mask[y].set_bit(x as u32);
            *c = d;
            true
        } else {
            false
        }
    };

    let mut c;

    for y in 0..grid.rows_len() {
        c = 0;

        let mut last = 0;

        for x in 0..cols {
            if set(x, y, &mut c) {
                last = x;
            }
        }

        c = 0;

        for x in (last..cols).rev() {
            set(x, y, &mut c);
        }
    }

    for x in 0..grid.columns_len() {
        c = 0;

        let mut last = 0;

        for y in 0..grid.rows_len() {
            if set(x, y, &mut c) {
                last = y;
            }
        }

        c = 0;

        for y in (last..grid.rows_len()).rev() {
            set(x, y, &mut c);
        }
    }

    let part1 = mask.iter().map(|b| b.count_ones()).sum::<u32>();

    let set = |x: usize, y: usize, c: &mut u8| {
        let d = *grid.get(y, x);

        if *c < d {
            *c = d;
            1
        } else {
            0
        }
    };

    for y in 0..grid.rows_len() {
        for x in 0..grid.columns_len() {
            c = 0;

            for y in (0..y).rev() {
                *seen.get_mut(y, x) += set(x, y, &mut c);
            }

            c = 0;

            for y in y + 1..grid.rows_len() {
                *seen.get_mut(y, x) += set(x, y, &mut c) << 8;
            }

            c = 0;

            for x in (0..x).rev() {
                *seen.get_mut(y, x) += set(x, y, &mut c) << 16;
            }

            c = 0;

            for x in x + 1..grid.columns_len() {
                *seen.get_mut(y, x) += set(x, y, &mut c) << 24;
            }
        }
    }

    let mut part2 = 0;

    for row in seen.rows() {
        for n in row {
            const M: u32 = 0b11111111;
            let score = [(n >> 24) & M, (n >> 16) & M, (n >> 8) & M, n & M]
                .into_iter()
                .product::<u32>();
            part2 = part2.max(score);
        }
    }

    Ok((part1, part2))
}
