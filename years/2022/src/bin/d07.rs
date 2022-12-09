use lib::prelude::*;

/// NB: We're assuming income comes in the form of a DFS, so we can avoid
/// keeping track of everything.
///
/// We also have to parse twice - once to sum totals, once to solve part 2.
#[entry(input = "d07.txt", expect = (1444896, 404395))]
fn main(input: IStr) -> Result<(u64, u64)> {
    let mut part1 = 0;
    let mut part2 = u64::MAX;

    let total = visit::<16, _>(input, |size| {
        if size < 100000 {
            part1 += size;
        }
    })?;

    let rem = 70000000u64.saturating_sub(total);
    let needed = 30000000u64.saturating_sub(rem);

    visit::<16, _>(input, |size| {
        if size >= needed {
            part2 = part2.min(size);
        }
    })?;

    Ok((part1, part2))
}

fn visit<const S: usize, T>(mut input: IStr, mut v: T) -> Result<u64>
where
    T: FnMut(u64),
{
    let mut stack = ArrayVec::<u64, S>::new();
    stack.push(0u64);

    while let Some(line) = input.try_line::<&str>()? {
        let (a, rest) = line.split_once(' ').context("first")?;

        match (a, rest) {
            ("$", "ls") => {}
            ("$", rest) => {
                let (a, b) = rest.split_once(' ').context("command")?;

                match (a, b) {
                    ("cd", "/") => {}
                    ("cd", "..") => {
                        let last = stack.pop().context("missing last")?;
                        v(last);
                        *stack.last_mut().context("missing parent")? += last;
                    }
                    ("cd", _) => {
                        stack.push(0);
                    }
                    (a, _) => {
                        bail!(a)
                    }
                }
            }
            ("dir", _) => {}
            (n, _) => {
                *stack.last_mut().context("missing last")? += n.parse::<u64>()?;
            }
        }
    }

    Ok(stack.into_iter().sum())
}
