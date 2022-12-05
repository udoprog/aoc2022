#![allow(warnings, unused)]

use lib::prelude::*;

#[entry(input = "d06.txt", expect = (0, 0))]
fn main(mut input: IStr) -> Result<(u32, u32)> {
    let mut part1 = 0;
    let mut part2 = 0;

    while let Some(mut data) = input.try_line::<ArrayVec<char, 16>>()? {
        data.try_push('t').context("push")?;
    }

    Ok((part1, part2))
}
