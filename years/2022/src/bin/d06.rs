use lib::prelude::*;

#[entry(input = "d06.txt", expect = (Some(1582), Some(3588)))]
fn main(input: IStr) -> Result<(Option<usize>, Option<usize>)> {
    let mut part1 = None;
    let mut part2 = None;

    for (n, window) in input.as_bstr().windows(4).enumerate() {
        if diff::<u32>(window, 4) {
            part1 = Some(n + 4);
            break;
        }
    }

    for (n, window) in input.as_bstr().windows(14).enumerate() {
        if diff::<u32>(window, 14) {
            part2 = Some(n + 14);
            break;
        }
    }

    Ok((part1, part2))
}

#[inline]
fn diff<T>(window: &[u8], n: u32) -> bool
where
    T: OwnedBits,
{
    window
        .iter()
        .fold(T::zeros(), |n, d| n.with_bit((*d - b'A') as u32))
        .bits_len()
        == n
}
