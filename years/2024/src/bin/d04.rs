use lib::prelude::*;

use core::iter::repeat;

#[entry(input = "d04.txt", expect = (2642, 1974))]
fn main(mut input: IStr) -> Result<(u32, u32)> {
    let mut o1 = 0;
    let mut o2 = 0;

    let grid = input.as_data();
    let cols = input.line::<&[u8]>()?.len();
    let g = grid.as_grid_with_stride(cols, 1);

    for (y, row) in g.rows().enumerate() {
        for (x, d) in row.iter().enumerate() {
            // Check for the XMAS instances.
            if *d == b'X' {
                let mut values = ArrayVec::<ArrayVec<u8, 4>>::new();

                if let Some(f) = y.checked_sub(3) {
                    values.try_push(g.collect(repeat(x).zip((f..=y).rev())))?;
                    values.try_push(g.collect((x..).zip((f..=f + 3).rev())))?;
                }

                if let Some(f) = x.checked_sub(3) {
                    values.try_push(g.collect((f..=x).rev().zip(repeat(y))))?;
                    values.try_push(g.collect((f..=f + 3).rev().zip(y..)))?;
                }

                if let (Some(x), Some(y)) = (x.checked_sub(3), y.checked_sub(3)) {
                    values.try_push(g.collect((x..=x + 3).rev().zip((y..=y + 3).rev())))?;
                }

                values.try_push(g.collect((x..).zip(repeat(y))))?;
                values.try_push(g.collect(repeat(x).zip(y..)))?;
                values.try_push(g.collect((x..).zip(y..)))?;

                for value in values {
                    o1 += u32::from(&value[..] == b"XMAS");
                }
            }

            // Check for the X-MAS.
            if *d == b'A' {
                let (Some(x), Some(y)) = (x.checked_sub(1), y.checked_sub(1)) else {
                    continue;
                };

                let a = g.collect::<3>((x..).zip(y..));
                let b = g.collect::<3>((x..=x + 2).rev().zip(y..));

                o2 += u32::from(matches!(
                    (&a[..], &b[..]),
                    (b"MAS" | b"SAM", b"MAS" | b"SAM")
                ));
            }
        }
    }

    Ok((o1, o2))
}
