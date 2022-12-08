use lib::prelude::*;

/// Max grid width / height.
const CAP: usize = 100;

#[entry(input = "d08.txt", expect = (1814, 330786))]
fn main(mut input: IStr) -> Result<(u32, u32)> {
    let mut grid = ArrayVec::<&'static BStr, CAP>::new();
    let mut mask = ArrayVec::<u128, CAP>::new();
    let mut seen = [[[0u8; 4]; CAP]; CAP];

    let mut cols = 0;

    while let Some(line) = input.try_line::<&BStr>()? {
        if line.is_empty() {
            break;
        }

        grid.try_push(line)?;
        mask.try_push(0)?;
        cols = cols.max(line.len());
    }

    let mut set = |x: usize, y: usize, c: &mut u8| {
        if *c < grid[y][x] {
            mask[y].set_bit(x as u32);
            *c = grid[y][x];
            true
        } else {
            false
        }
    };

    let mut c;

    for y in 0..grid.len() {
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

    for x in 0..cols {
        c = 0;

        let mut last = 0;

        for y in 0..grid.len() {
            if set(x, y, &mut c) {
                last = y;
            }
        }

        c = 0;

        for y in (last..grid.len()).rev() {
            set(x, y, &mut c);
        }
    }

    let part1 = mask.iter().map(|b| b.count_ones()).sum::<u32>();

    let set = |x: usize, y: usize, c: &mut u8| {
        if *c < grid[y][x] {
            *c = grid[y][x];
            1
        } else {
            0
        }
    };

    for y in 0..grid.len() {
        for x in 0..cols {
            c = 0;

            for y in (0..y).rev() {
                seen[y][x][0] += set(x, y, &mut c);
            }

            c = 0;

            for y in y + 1..grid.len() {
                seen[y][x][1] += set(x, y, &mut c);
            }

            c = 0;

            for x in (0..x).rev() {
                seen[y][x][2] += set(x, y, &mut c);
            }

            c = 0;

            for x in x + 1..cols {
                seen[y][x][3] += set(x, y, &mut c);
            }
        }
    }

    let mut part2 = 0;

    for row in seen {
        for col in row {
            let score = col.into_iter().map(|n| n as u32).fold(1, |a, b| a * b);
            part2 = part2.max(score);
        }
    }

    Ok((part1, part2))
}
