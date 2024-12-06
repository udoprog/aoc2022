use lib::prelude::*;

#[entry(input = "d05.txt", expect = (6242, 0))]
fn main(mut input: IStr) -> Result<(u32, u32)> {
    let mut o1 = 0;
    let o2 = 0;

    let mut forward = HashMap::<u32, HashSet<u32>>::new();

    while let Some(Split((a, b))) = input.line::<Option<Split<'|', (u32, u32)>>>()? {
        forward.entry(a).or_default().insert(b);
    }

    while let Some(line) = input.line::<Option<IStr>>()? {
        if line.is_empty() {
            break;
        }

        let mut last = None;

        let mut page = ArrayVec::<u32, 32>::new();
        let mut ok = true;

        for digit in line.split(",").iter::<u32>() {
            let d = digit?;

            if let Some(last) = last {
                if let Some(expected) = forward.get(&last) {
                    ok &= expected.contains(&d);
                } else {
                    ok = false;
                }
            }

            last = Some(d);
            page.try_push(d)?;
        }

        if ok {
            o1 += page.get(page.len() / 2).context("missing middle value")?;
        }
    }

    Ok((o1, o2))
}
