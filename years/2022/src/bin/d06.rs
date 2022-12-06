use lib::prelude::*;

#[entry(input = "d06.txt", expect = (Some(1582), Some(3588)))]
fn main(mut input: IStr) -> Result<(Option<usize>, Option<usize>)> {
    let mut part1 = None;
    let mut part2 = None;

    let mut d = ArrayRingBuffer::<_, 16>::new();
    let mut n = 0;
    let mut set = FixedSet::<[u64; 4]>::empty();

    let mut b = ArrayVec::<u8, 16>::new();

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

        let mut diff2 = |it| {
            b.clear();
            b.extend(it);

            if b.len() != 14 {
                return false;
            }

            let a = &b;
            a[0] != a[1]
                && a[0] != a[2]
                && a[0] != a[3]
                && a[0] != a[4]
                && a[0] != a[5]
                && a[0] != a[6]
                && a[0] != a[7]
                && a[0] != a[8]
                && a[0] != a[9]
                && a[0] != a[10]
                && a[0] != a[11]
                && a[0] != a[12]
                && a[0] != a[13]
                && a[1] != a[2]
                && a[1] != a[3]
                && a[1] != a[4]
                && a[1] != a[5]
                && a[1] != a[6]
                && a[1] != a[7]
                && a[1] != a[8]
                && a[1] != a[9]
                && a[1] != a[10]
                && a[1] != a[11]
                && a[1] != a[12]
                && a[1] != a[13]
                && a[2] != a[3]
                && a[2] != a[4]
                && a[2] != a[5]
                && a[2] != a[6]
                && a[2] != a[7]
                && a[2] != a[8]
                && a[2] != a[9]
                && a[2] != a[10]
                && a[2] != a[11]
                && a[2] != a[12]
                && a[2] != a[13]
                && a[3] != a[4]
                && a[3] != a[5]
                && a[3] != a[6]
                && a[3] != a[7]
                && a[3] != a[8]
                && a[3] != a[9]
                && a[3] != a[10]
                && a[3] != a[11]
                && a[3] != a[12]
                && a[3] != a[13]
                && a[4] != a[5]
                && a[4] != a[6]
                && a[4] != a[7]
                && a[4] != a[8]
                && a[4] != a[9]
                && a[4] != a[10]
                && a[4] != a[11]
                && a[4] != a[12]
                && a[4] != a[13]
                && a[5] != a[6]
                && a[5] != a[7]
                && a[5] != a[8]
                && a[5] != a[9]
                && a[5] != a[10]
                && a[5] != a[11]
                && a[5] != a[12]
                && a[5] != a[13]
                && a[6] != a[7]
                && a[6] != a[8]
                && a[6] != a[9]
                && a[6] != a[10]
                && a[6] != a[11]
                && a[6] != a[12]
                && a[6] != a[13]
                && a[7] != a[8]
                && a[7] != a[9]
                && a[7] != a[10]
                && a[7] != a[11]
                && a[7] != a[12]
                && a[7] != a[13]
                && a[8] != a[9]
                && a[8] != a[10]
                && a[8] != a[11]
                && a[8] != a[12]
                && a[8] != a[13]
                && a[9] != a[10]
                && a[9] != a[11]
                && a[9] != a[12]
                && a[9] != a[13]
                && a[10] != a[11]
                && a[10] != a[12]
                && a[10] != a[13]
                && a[11] != a[12]
                && a[11] != a[13]
                && a[12] != a[13]
        };

        if part1.is_none() && d.len() >= 4 && diff(d.iter().rev().take(4).copied()) {
            part1 = Some(n);
        }

        if part2.is_none() && d.len() >= 14 && diff2(d.iter().rev().take(14).copied()) {
            part2 = Some(n);
        }
    }

    Ok((part1, part2))
}
