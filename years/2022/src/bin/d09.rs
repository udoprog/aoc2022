use lib::prelude::*;

const CAP: usize = 1024;
const MID: i32 = (CAP as i32) / 2;

#[entry(input = "d09.txt", expect = (5907, 2303))]
fn main(mut input: IStr) -> Result<(u32, u32)> {
    let mut part1 = [0u128; { CAP * (CAP / 128) }];
    let mut part2 = [0u128; { CAP * (CAP / 128) }];

    let mut knots = [(0, 0); 10];

    part1.set_bit(pos_to_index(knots[0]));
    part2.set_bit(pos_to_index(knots[0]));

    for line in input.iter::<Nl<(Move, usize)>>() {
        let Nl((m, b)) = line?;

        for _ in 0..b {
            knots[0].0 += m.0;
            knots[0].1 += m.1;

            for n in 1..knots.len() {
                let h = knots[n - 1];
                let t = &mut knots[n];

                if let Some(n) = move_for(&h, t) {
                    *t = n;
                }
            }

            let [h, p1, .., p2] = knots;
            anyhow::ensure!(h.0.abs() < MID && h.1.abs() < MID, "oob: {h:?}");
            part1.set_bit(pos_to_index(p1));
            part2.set_bit(pos_to_index(p2));
        }
    }

    let part1 = part1.count_ones();
    let part2 = part2.count_ones();
    Ok((part1, part2))
}

#[derive(Debug, Clone, Copy)]
struct Move(i32, i32);

lib::from_input! {
    |B(v): B| -> Move {
        Ok(match v {
            b'R' => Move(1, 0),
            b'U' => Move(0, -1),
            b'L' => Move(-1, 0),
            b'D' => Move(0, 1),
            c => bail!(c as char),
        })
    }
}

/// Calculate move.
#[inline]
fn move_for(h: &(i32, i32), t: &(i32, i32)) -> Option<(i32, i32)> {
    let x = h.0 - t.0;
    let y = h.1 - t.1;

    if x.abs() < 2 && y.abs() < 2 {
        return None;
    }

    Some((t.0 + x.signum(), t.1 + y.signum()))
}

/// Translate position to bitset index.
#[inline]
fn pos_to_index(p: (i32, i32)) -> u32 {
    (p.0 + MID) as u32 * CAP as u32 + (p.1 + MID) as u32
}
