use lib::prelude::*;

#[entry(input = "d06.txt", expect = (Some(1582), Some(3588)))]
fn main(input: IStr) -> Result<(Option<usize>, Option<usize>)> {
    let mut part1 = None;
    let mut part2 = None;

    for (n, window) in input.as_bstr().windows(4).enumerate() {
        if diff(window, 4) {
            part1 = Some(n + 4);
            break;
        }
    }

    for (n, window) in input.as_bstr().windows(14).enumerate() {
        if diff(window, 14) {
            part2 = Some(n + 14);
            break;
        }
    }

    Ok((part1, part2))
}

#[inline]
fn diff(window: &[u8], n: u32) -> bool {
    let c = window
        .iter()
        .fold(0u64, |n, d| n | 1 << (*d - b'A') as u64)
        .count_ones();
    c == n
}
