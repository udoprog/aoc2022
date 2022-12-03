use lib::prelude::*;

#[entry(input = "d01.txt")]
fn main(mut input: Input) -> Result<()> {
    let mut output = [0; 4];
    let mut calories = 0u32;

    while let Some((n, Ws(lines))) = input.try_next::<(u32, _)>()? {
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

    assert_eq!(part1, 70764);
    assert_eq!(part2, 203905);
    Ok(())
}
