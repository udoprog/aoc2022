use lib::prelude::*;

fn main() -> Result<()> {
    let mut input = lib::input!("d03.txt");

    while let Some((a, b)) = input.try_next::<(u32, u32)>()? {
        dbg!(a, b);
    }

    Ok(())
}
