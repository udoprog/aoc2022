use lib::prelude::*;

use std::iter::from_fn;

#[entry(input = "d02.txt", expect = (321, 386))]
fn main(mut input: IStr) -> Result<(u32, u32)> {
    let mut o1 = 0;
    let mut o2 = 0;

    while let Some(values) = input.try_line::<ArrayVec<u32>>()? {
        if values.is_empty() {
            break;
        }

        ensure!(values.len() > 1, "invalid input");

        let found_a = pairs(values.iter().copied()).position(|(a, b)| !dec(a, b));
        let found_b = pairs(values.iter().copied()).position(|(a, b)| !inc(a, b));

        let (Some(a), Some(b)) = (found_a, found_b) else {
            o1 += 1;
            o2 += 1;
            continue;
        };

        let p2 = 'out: {
            let alts = [(a, dec as fn(_, _) -> bool), (b, inc as fn(_, _) -> bool)];

            for (n, m) in alts {
                // Check that the remaining unredacted values satisfies the condition which failed.
                if !pairs(values[n + 2..].iter().copied()).all(|(a, b)| m(a, b)) {
                    continue;
                }

                // Check whether all possible redactions passes a region of x
                // values.
                //
                // If x is the first value, then we need to check the next 3
                // values, one of which is skipped. Otherwise 4 values are
                // checked.
                for n in n.checked_sub(1).into_iter().chain([n, n + 1]) {
                    let (n, redact, take) =
                        n.checked_sub(1).map(|n| (n, 1, 4)).unwrap_or((0, 0, 3));

                    if pairs(skip(values[n..].iter().take(take).copied(), redact))
                        .all(|(a, b)| m(a, b))
                    {
                        break 'out true;
                    }
                }
            }

            false
        };

        o2 += u32::from(p2);
    }

    Ok((o1, o2))
}

#[inline]
fn dec(a: u32, b: u32) -> bool {
    dist(a, b) && a < b
}

#[inline]
fn inc(a: u32, b: u32) -> bool {
    dist(a, b) && a > b
}

#[inline]
fn dist(a: u32, b: u32) -> bool {
    matches!(a.max(b) - a.min(b), 1..=3)
}

#[inline]
fn pairs(it: impl IntoIterator<Item = u32>) -> impl Iterator<Item = (u32, u32)> {
    let mut it = it.into_iter();
    let mut buf = it.next();

    from_fn(move || {
        let a = buf.take()?;
        let b = it.next()?;
        buf = Some(b);
        Some((a, b))
    })
}

#[inline]
fn skip(it: impl IntoIterator<Item = u32>, redact: usize) -> impl Iterator<Item = u32> {
    it.into_iter()
        .enumerate()
        .filter(move |&(i, _)| redact != i)
        .map(|(_, v)| v)
}
