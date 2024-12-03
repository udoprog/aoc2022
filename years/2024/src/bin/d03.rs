use lib::prelude::*;

#[entry(input = "d03.txt", expect = (164730528, 70478672))]
fn main(mut input: IStr) -> Result<(u32, u32)> {
    let mut o1 = 0;
    let mut o2 = 0;

    let mut enabled = true;

    while !input.is_empty() {
        if input.eat(b"do()") {
            enabled = true;
            continue;
        }

        if input.eat(b"don't()") {
            enabled = false;
            continue;
        }

        if input.eat(b"mul(") {
            let Some(a) = input.try_next::<u32>()? else {
                continue;
            };

            if !input.eat(b",") {
                continue;
            }

            let Some(b) = input.try_next::<u32>()? else {
                continue;
            };

            if !input.eat(b")") {
                continue;
            }

            ensure!(a < 1000 && b < 1000, "invalid arguments");

            o1 += a * b;

            if enabled {
                o2 += a * b;
            }

            continue;
        }

        input.advance(1);
    }

    Ok((o1, o2))
}
