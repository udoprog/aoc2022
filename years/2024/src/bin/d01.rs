use lib::prelude::*;

#[entry(input = "d01.txt", expect = (2086478, 24941624))]
fn main(mut input: IStr) -> Result<(u32, u32)> {
    let mut a = ArrayVec::<u32, 1024>::new();
    let mut b = ArrayVec::<u32, 1024>::new();

    for value in input.iter::<(u32, u32)>() {
        let (left, right) = value?;
        a.try_push(left)?;
        b.try_push(right)?;
    }

    a.sort();
    b.sort();

    let mut o1 = 0;
    let mut o2 = 0;

    for (l, r) in a.iter().zip(b.iter()) {
        o1 += l.max(r) - l.min(r);

        let mut c = 0;

        for r in b.iter() {
            if l == r {
                c += 1;
            }
        }

        o2 += l * c;
    }

    Ok((o1, o2))
}
