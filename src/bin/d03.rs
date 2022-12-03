use lib::prelude::*;

fn main() -> Result<()> {
    let mut input = lib::input!("d03.txt");

    while let Some(data) = input.try_line::<ArrayVec<u32>>()? {
        dbg!(data.into_iter());
    }

    Ok(())
}
