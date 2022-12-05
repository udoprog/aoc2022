use crate::input::{FromInput, IStr, IStrError, Result};

/// A byte-mucked number.
pub struct Muck2(pub u8);

impl FromInput for Muck2 {
    #[inline]
    fn try_from_input(p: &mut IStr) -> Result<Option<Self>> {
        let a = p.at(0);

        if !matches!(a, Some(b'0'..=b'9')) {
            return Err(IStrError::NotByteMuck);
        }

        let b = p.at(1);
        let c = p.at(2);

        let (Some(a), b, c) = (a, b, c) else {
            return Ok(None);
        };

        if matches!(c, Some(b'0'..=b'9')) {
            return Err(IStrError::NotByteMuck);
        }

        let mut muck = a - b'0';

        if let Some(b @ b'0'..=b'9') = b {
            muck = muck * 10 + (b - b'0');
            p.data = p.data.get(2..).unwrap_or_default();
        } else {
            p.data = p.data.get(1..).unwrap_or_default();
        }

        Ok(Some(Self(muck)))
    }
}
