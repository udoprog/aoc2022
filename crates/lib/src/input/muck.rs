use crate::input::{FromInput, Input, InputError, Result};

/// A byte-mucked number.
pub struct Muck2(pub u8);

impl FromInput for Muck2 {
    #[inline]
    fn try_from_input(p: &mut Input) -> Result<Option<Self>> {
        let a = p.at(p.index);

        if !matches!(a, Some(b'0'..=b'9')) {
            return Err(InputError::NotByteMuck);
        }

        let b = p.index.checked_add(1).and_then(|d| p.at(d));
        let c = p.index.checked_add(2).and_then(|d| p.at(d));

        let (Some(a), b, c) = (a, b, c) else {
            return Ok(None);
        };

        if matches!(c, Some(b'0'..=b'9')) {
            return Err(InputError::NotByteMuck);
        }

        let mut muck = a - b'0';

        if let Some(b @ b'0'..=b'9') = b {
            muck = muck * 10 + (b - b'0');
            p.index += 2;
        } else {
            p.index += 1;
        }

        Ok(Some(Self(muck)))
    }
}
