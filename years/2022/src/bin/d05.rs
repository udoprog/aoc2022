use lib::prelude::*;

#[entry(input = "d05.txt", expect = ("RFFFWBPNS", "CQQBBJFCS"))]
fn main(mut input: Input) -> Result<(ArrayString, ArrayString)> {
    let mut stacks1 = ArrayVec::<ArrayVec<_, 128>, 10>::new();

    while let Some(line) = input.try_line::<Input>()?.filter(|s| !s.is_empty()) {
        for (n, chunk) in line.as_bstr().chunks(4).enumerate() {
            if let Some(&d) = chunk.get(1).filter(|d| matches!(d, b'A'..=b'Z')) {
                for _ in stacks1.len()..=n {
                    stacks1
                        .try_push(ArrayVec::new())
                        .ok()
                        .context("stacks capacity")?;
                }

                stacks1
                    .get_mut(n)
                    .and_then(|s| s.try_push(d).ok())
                    .context("capacity")?;
            }
        }
    }

    stacks1.iter_mut().for_each(|s| s.reverse());

    let mut stacks2 = stacks1.clone();

    for line in input.iter::<(W, usize, W, usize, W, usize)>() {
        let (_, c, _, from, _, to) = line?;
        let from = from.checked_sub(1).context("underflow")?;
        let to = to.checked_sub(1).context("underflow")?;

        for _ in 0..c {
            do_move(&mut stacks1, from, to).context("bad move 1")?;
            do_move(&mut stacks2, from, to).context("bad move 2")?;
        }

        do_reverse(&mut stacks2, to, c).context("bad reverse")?;
    }

    let mut part1 = ArrayString::new();
    let mut part2 = ArrayString::new();

    for (s1, s2) in stacks1.into_iter().zip(stacks2.into_iter()) {
        if let Some(d) = s1.last().copied() {
            part1.push(d as char);
        }

        if let Some(d) = s2.last().copied() {
            part2.push(d as char);
        }
    }

    Ok((part1, part2))
}

fn do_move<const N: usize, T>(stacks: &mut [ArrayVec<T, N>], from: usize, to: usize) -> Option<()> {
    let d = stacks.get_mut(from)?.pop()?;
    stacks.get_mut(to)?.try_push(d).ok()?;
    Some(())
}

fn do_reverse<const N: usize, T>(stacks: &mut [ArrayVec<T, N>], i: usize, c: usize) -> Option<()> {
    let stack = stacks.get_mut(i)?;
    let s = stack.len().checked_sub(c)?;
    stack.get_mut(s..)?.reverse();
    Some(())
}
