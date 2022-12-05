use lib::{input::IStrError, prelude::*};

#[entry(input = "d06.txt", expect = true)]
fn main(input: &mut IStr) -> Result<bool, IStrError> {
    while let Some(line) = input.try_line::<IStr>()?.filter(|s| !s.is_empty()) {
        for (n, chunk) in line.as_bstr().chunks(4).enumerate() {
            if let Some(&d) = chunk.get(1).filter(|d| matches!(d, b'A'..=b'Z')) {}
        }
    }

    for line in input.iter::<(W, usize, W, usize, W, usize)>() {
        let _ = line?;
    }

    Ok(true)
}
