use lib::prelude::*;

#[entry(input = "todo.txt", expect = (2642, 1974))]
fn main(mut input: IStr) -> Result<(u32, u32)> {
    let o1 = 0;
    let o2 = 0;

    while let Some(Split((a, b))) = input.line::<Option<Split<'|', (u32, u32)>>>()? {
        dbg!(a, b);
    }

    while let Some(line) = input.line::<Option<IStr>>()? {
        for digit in line.split(",").iter::<u32>() {
            dbg!(digit?);
        }
    }

    Ok((o1, o2))
}
