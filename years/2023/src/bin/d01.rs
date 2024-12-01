use lib::prelude::*;

#[entry(input = "d01.txt", expect = (47954, 54770))]
fn main(mut input: IStr) -> Result<(u32, u32)> {
    let mut o1 = 0;
    let mut o2 = 0;

    while let Some(value) = input.try_line::<ArrayString<64>>()? {
        let mut f1 = 0;
        let mut l1 = 0;
        let mut f2 = 0;
        let mut l2 = 0;

        'outer: for (i, c) in value.char_indices() {
            let (p1, d) = 'out: {
                let Some(d) = c.to_digit(10) else {
                    let d = match value[i..].as_bytes() {
                        [b'o', b'n', b'e', ..] => 1,
                        [b't', b'w', b'o', ..] => 2,
                        [b't', b'h', b'r', b'e', b'e', ..] => 3,
                        [b'f', b'o', b'u', b'r', ..] => 4,
                        [b'f', b'i', b'v', b'e', ..] => 5,
                        [b's', b'i', b'x', ..] => 6,
                        [b's', b'e', b'v', b'e', b'n', ..] => 7,
                        [b'e', b'i', b'g', b'h', b't', ..] => 8,
                        [b'n', b'i', b'n', b'e', ..] => 9,
                        _ => continue 'outer,
                    };

                    break 'out (true, d);
                };

                (false, d)
            };

            if f1 == 0 && p1 {
                f1 = d;
            }

            if p1 {
                l1 = d;
            }

            if f2 == 0 {
                f2 = d;
            }

            l2 = d;
        }

        o1 += f1 * 10 + l1;
        o2 += f2 * 10 + l2;
    }

    Ok((o1, o2))
}
