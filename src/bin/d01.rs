use anyhow::{anyhow, Result};
use lib::Input;

fn main() -> Result<()> {
    let mut totals = parse("inputs/d01.txt")?;
    totals.sort();

    let top3: u32 = totals.iter().rev().take(3).sum();
    let top1 = *totals.last().ok_or_else(|| anyhow!("missing top"))?;

    assert_eq!(top1, 70764);
    assert_eq!(top3, 203905);
    Ok(())
}

/// Parse input lines.
fn parse(path: &str) -> Result<[u32; 4]> {
    let mut input = Input::new(path)?;
    input.set_whitespace(true);

    let mut output = [0; 4];
    let mut calories = 0u32;

    while let Some(n) = input.try_next::<u32>()? {
        calories += n;

        if input.skip_whitespace()? == 2 {
            output[0] = std::mem::take(&mut calories);
            output.sort();
        }
    }

    if calories != 0 {
        output[0] = calories;
        output.sort();
    }

    Ok(output)
}
