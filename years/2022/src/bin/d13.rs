use core::cmp::{Ordering, PartialOrd};
use core::fmt;

use lib::prelude::*;

// These might need tweaking to get your input to parse.
const ARENA: usize = 1 << 19;
const MAX_SLICE: usize = 5;
const PACKETS: usize = 512;

// Specified divisors.
const DIV1: Packet = Packet::List(&[Packet::List(&[Packet::Number(2)])]);
const DIV2: Packet = Packet::List(&[Packet::List(&[Packet::Number(6)])]);

#[entry(input = "d13.txt", expect = (4809, 22600))]
fn main(mut input: IStr) -> Result<(u32, usize)> {
    let mut part1 = 0;

    let mut data = [0; ARENA];
    let arena = Arena::new(&mut data);

    let mut all = ArrayVec::<_, PACKETS>::new();

    let mut n = 1;

    all.try_push(DIV1).map_err(|e| anyhow::anyhow!("{e}"))?;
    all.try_push(DIV2).map_err(|e| anyhow::anyhow!("{e}"))?;

    while let Some(mut a) = input.try_line::<IStr>()? {
        let mut b = input.line::<IStr>()?;

        let a = parse(&mut a, &arena)?.context("failed to parse a")?;
        let b = parse(&mut b, &arena)?.context("failed to parse b")?;

        if matches!(a.cmp(&b), Ordering::Less) {
            part1 += n;
        }

        all.try_push(a).map_err(|e| anyhow::anyhow!("{e}"))?;
        all.try_push(b).map_err(|e| anyhow::anyhow!("{e}"))?;

        if input.ws()? == 0 {
            break;
        }

        n += 1;
    }

    all.sort();

    let mut part2 = 1;

    for (n, item) in all.iter().enumerate() {
        match *item {
            DIV1 | DIV2 => {
                part2 *= n + 1;
            }
            _ => {}
        }
    }

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

impl fmt::Debug for Packet<'_> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Packet::List(list) => {
                let mut f = f.debug_list();

                for item in *list {
                    f.entry(item);
                }

                f.finish()
            }
            Packet::Number(number) => number.fmt(f),
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

            let mut slice = arena.alloc_iter(MAX_SLICE)?;

            while let Some(item) = parse(input, arena)? {
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
        _ => Ok(Some(Packet::Number(input.next::<Digits<u32>>()?.0))),
    }
}
