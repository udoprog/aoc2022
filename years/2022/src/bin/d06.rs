use lib::prelude::*;

#[entry(input = "d06.txt", expect = (Some(1582), Some(3588)))]
fn main(mut input: IStr) -> Result<(Option<usize>, Option<usize>)> {
    let mut part1 = None;
    let mut part2 = None;

    let mut d = ArrayRingBuffer::<_, 16>::new();
    let mut n = 0;
    let mut set = FixedSet::<[u64; 4]>::empty();

    while let Some(B(c)) = input.try_next::<B>()? {
        if d.len() == 14 {
            d.dequeue();
        }

        d.push(c);

        n += 1;

        let mut diff = |it| -> bool {
            set.clear();

            for d in it {
                if set.test(d as usize) {
                    return false;
                }

                set.set(d as usize);
            }

            true
        };

        if part1.is_none() && d.len() >= 4 && diff(d.iter().rev().take(4).copied()) {
            part1 = Some(n);
        }

        if part2.is_none() && d.len() >= 14 && diff(d.iter().rev().take(14).copied()) {
            part2 = Some(n);
        }
    }

    Ok((part1, part2))
}
