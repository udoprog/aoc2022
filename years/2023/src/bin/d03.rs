use lib::prelude::*;

#[entry(input = "d03.txt", expect = (559022, 0))]
fn main(input: IStr) -> Result<(u32, u32)> {
    let mut o1 = 0;
    let o2 = 0;

    let cols = input.clone().line::<&[u8]>()?.len();
    let grid = input.as_data().as_grid_with_stride(cols, 1);

    let mut y = 0;

    loop {
        let Some(row) = grid.row(y) else {
            break;
        };

        let mut x = 0;

        while x < row.len() {
            let Some(d) = row.get(x) else {
                break;
            };

            if !d.is_ascii_digit() {
                x += 1;
                continue;
            }

            let mut v = (*d - b'0') as u32;
            let s = x;

            while x < row.len() {
                x += 1;

                let Some(d) = row.get(x) else {
                    break;
                };

                if !d.is_ascii_digit() {
                    break;
                }

                v = v * 10 + (*d - b'0') as u32;
            }

            let m = 'outer: {
                for iy in y.saturating_sub(1)..y.saturating_add(2).min(grid.rows_len()) {
                    for ix in s.saturating_sub(1)..x.saturating_add(1).min(grid.columns_len()) {
                        if (s..=x).contains(&ix) && iy == y {
                            continue;
                        }

                        let m = match grid.get(iy, ix) {
                            b'.' => false,
                            d if d.is_ascii_digit() => false,
                            d => {
                                ensure!(!d.is_ascii_digit(), "digit ad {iy}.{ix}");
                                true
                            }
                        };

                        if m {
                            break 'outer true;
                        }
                    }
                }

                false
            };

            if m {
                o1 += v;
            }

            x += 1;
        }

        y += 1;
    }

    Ok((o1, o2))
}
