use lib::prelude::*;

#[entry(input = "d01.txt", expect = (70764, 203905))]
fn main(input: IStr) -> Result<(u32, u32)> {
    let mut output = [0; 4];
    let mut calories = 0u32;

    for value in input.iter::<(u32, _)>() {
        let (n, Ws(lines)) = value?;

        calories += n;

        if lines == 2 {
            output[0] = std::mem::take(&mut calories);
            output.sort_unstable();
        }
    }

    if calories != 0 {
        output[0] = calories;
        output.sort_unstable();
    }

    let [_, a, b, c] = output;
    let part1 = c;
    let part2 = a + b + c;
    Ok((part1, part2))
}
