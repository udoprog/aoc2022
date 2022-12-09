use lib::prelude::*;

const CAP: usize = 1024;
const MID: i32 = (CAP as i32) / 2;

#[entry(input = "d09.txt", expect = (5907, 2303))]
fn main(mut input: IStr) -> Result<(u32, u32)> {
    let mut part1 = [0u128; { CAP * (CAP / 128) }];
    let mut part2 = [0u128; { CAP * (CAP / 128) }];
    let mut knots = [(0, 0); 10];

    // Translate position to bitset index.
    let pos = |p: (i32, i32)| (p.0 + MID) as u32 * CAP as u32 + (p.1 + MID) as u32;

    part1.set_bit(pos(knots[0]));
    part2.set_bit(pos(knots[0]));

    while let Some((B(m), b)) = input.try_line::<(B, usize)>()? {
        let m = match m {
            b'R' => (1, 0),
            b'U' => (0, -1),
            b'L' => (-1, 0),
            b'D' => (0, 1),
            c => bail!(c as char),
        };

        for _ in 0..b {
            knots[0].0 += m.0;
            knots[0].1 += m.1;

            let mut n = 0..;

            while let Some([h, t]) = n.next().and_then(|n| knots.get_mut(n..=n + 1)) {
                let (x, y) = (h.0 - t.0, h.1 - t.1);

                if x.abs() >= 2 || y.abs() >= 2 {
                    *t = (t.0 + x.signum(), t.1 + y.signum());
                }
            }

            let [h, p1, .., p2] = knots;
            anyhow::ensure!(h.0.abs() < MID && h.1.abs() < MID, "oob: {h:?}");
            part1.set_bit(pos(p1));
            part2.set_bit(pos(p2));
        }
    }

    let part1 = part1.count_ones();
    let part2 = part2.count_ones();
    Ok((part1, part2))
}
