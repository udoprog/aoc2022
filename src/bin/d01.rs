use std::path::Path;

use anyhow::{Context, Result};
use thiserror::Error;

#[derive(Debug, Error)]
enum Error {
    #[error("{0}:{1}:{2}: expected more input")]
    Expected(Box<Path>, usize, usize),
    #[error("{0}:{1}:{2}: bad input")]
    BadInput(Box<Path>, usize, usize),
    #[error("missing top entry")]
    MissingTop,
}

struct Elf {
    calories: Vec<u32>,
}

fn main() -> Result<()> {
    let lines = parse("inputs/d01.txt")?;

    let mut totals = Vec::new();

    for elf in lines {
        totals.push(elf.calories.iter().sum());
    }

    totals.sort();

    let top3: u32 = totals.iter().rev().take(3).sum();
    let top1 = *totals.last().ok_or(Error::MissingTop)?;

    assert_eq!(top1, 70764);
    assert_eq!(top3, 203905);
    Ok(())
}

/// Parse input lines.
fn parse<P>(path: P) -> Result<Vec<Elf>> where P: AsRef<Path> {
    let path = path.as_ref();
    let string = std::fs::read_to_string(path)?;

    let mut output = Vec::new();
    let mut calories = Vec::new();

    for (n, line) in string.lines().enumerate() {
        if line.trim().is_empty() {
            output.push(Elf { calories: std::mem::take(&mut calories) });
            continue;
        }

        let mut cols = line.split(' ');
        calories.push(cols.next().ok_or(Error::Expected(path.into(), n + 1, 0))?.parse().context(Error::BadInput(path.into(), n + 1, 0))?);
    }

    if !calories.is_empty() {
        output.push(Elf { calories });
    }

    Ok(output)
}
