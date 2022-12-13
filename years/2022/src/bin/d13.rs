use core::cmp::{Ordering, PartialOrd};

use lib::prelude::*;

// These might need tweaking to get your input to parse.
const ARENA: usize = 1 << 18;
const MAX_SLICE: usize = 5;

// Specified divisors.
const DIV1: Packet = Packet::List(&[Packet::List(&[Packet::Number(2)])]);
const DIV2: Packet = Packet::List(&[Packet::List(&[Packet::Number(6)])]);

#[entry(input = "d13.txt", expect = (4809, 22600))]
fn main(mut input: IStr) -> Result<(u32, usize)> {
    let mut part1 = 0;

    let mut data = [0; ARENA];
    let arena = Arena::new(&mut data);

    let mut n = 0;

    let mut div1 = usize::from(DIV1 > DIV2) + 1;
    let mut div2 = usize::from(DIV2 > DIV1) + 1;

    while let Some(mut a) = input.try_line::<IStr>()? {
        n += 1;
        let mut b = input.line::<IStr>()?;

        let a = parse(&mut a, &arena)?.context("failed to parse a")?;
        let b = parse(&mut b, &arena)?.context("failed to parse b")?;

        if a < b {
            part1 += n;
        }

        div1 += usize::from(DIV1 >= a) + usize::from(DIV1 > b);
        div2 += usize::from(DIV2 >= a) + usize::from(DIV2 > b);

        if input.ws()? == 0 {
            break;
        }
    }

    let part2 = div1 * div2;
    Ok((part1, part2))
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Packet<'a> {
    List(&'a [Packet<'a>]),
    Number(u32),
}

impl PartialOrd for Packet<'_> {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Packet<'_> {
    fn cmp(&self, other: &Self) -> Ordering {
        match (self, other) {
            (Packet::List(a), Packet::List(b)) => a.iter().cmp(b.iter()),
            (Packet::List(a), Packet::Number(b)) => a.iter().cmp([&Packet::Number(*b)]),
            (Packet::Number(a), Packet::List(b)) => [&Packet::Number(*a)].into_iter().cmp(b.iter()),
            (Packet::Number(a), Packet::Number(b)) => a.cmp(b),
        }
    }
}

/// Parse input recursively, storing parsed data in the arena allocator.
fn parse<'a>(input: &mut IStr, arena: &'a Arena<'a>) -> Result<Option<Packet<'a>>> {
    let Some(B(b)) = input.peek::<B>()? else {
        return Ok(None);
    };

    match b {
        b'[' => {
            input.next::<B>()?;

            let mut data = ArrayVec::<_, MAX_SLICE>::new();

            while let Some(item) = parse(input, arena)? {
                data.try_push(item).map_err(|e| anyhow::anyhow!("{e}"))?;
            }

            let mut slice = arena.alloc_iter(data.len())?;

            for item in data {
                slice.write(item)?;
            }

            Ok(Some(Packet::List(slice.finish())))
        }
        b']' => {
            input.next::<B>()?;
            Ok(None)
        }
        b',' => {
            input.next::<B>()?;
            parse(input, arena)
        }
        _ => Ok(Some(Packet::Number(input.next::<Digits<_>>()?.0))),
    }
}
