use crate::error::*;
use byteorder::{ByteOrder, BE};
use std::convert::TryInto;
use std::fmt;

#[derive(Hash, PartialEq, Eq)]
pub struct ID([u8; 4]);

impl ID {
    pub fn data(&self) -> &[u8] {
        &self.0
    }
}

impl<'a> std::convert::TryFrom<&'a [u8]> for ID {
    type Error = AiffError;

    fn try_from(value: &'a [u8]) -> Result<Self> {
        assert_eq!(value.len(), 4);

        let mut has_spaces = false;
        for b in value.iter() {
            match b {
                b' ' => {
                    has_spaces = true;
                }
                0x21..=0x7e => {
                    if has_spaces {
                        return Err(AiffError::InvalidFormat);
                    }
                }

                _ => return Err(AiffError::InvalidFormat),
            }
        }

        Ok(Self(value.try_into().unwrap()))
    }
}

impl fmt::Debug for ID {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ID(")?;
        fmt::Display::fmt(&self, f)?;
        write!(f, ")")
    }
}

impl fmt::Display for ID {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}{}{}{}",
            self.0[0] as char, self.0[1] as char, self.0[2] as char, self.0[3] as char,
        )
    }
}

pub fn read_chunk<'a>(data: &mut &'a [u8]) -> Result<(ID, &'a [u8])> {
    if data.len() < 8 {
        return Err(AiffError::InvalidFormat);
    }
    let id = &data[0..4];
    let size = BE::read_u32(&data[4..8]);

    if data.len() < 8 + size as usize {
        return Err(AiffError::InvalidFormat);
    }
    let chunk_data = &data[8..][..size as usize];

    *data = &data[8..][size as usize..];
    if size % 2 == 1 {
        *data = &data[1..];
    }

    Ok((id.try_into().unwrap(), chunk_data))
}

pub fn read_f80(data: &[u8]) -> f64 {
    assert!(data.len() >= 10);
    let exponent = BE::read_u16(&data[0..2]);
    let mantissa = BE::read_u64(&data[2..10]);

    let sign = (exponent >> 15) != 0;
    let exponent = exponent & 0b0111_1111_1111_1111;

    if exponent == 0 {
        if (mantissa >> 63) == 0 {
            if mantissa == 0 {
                if !sign {
                    0.0
                } else {
                    -0.0
                }
            } else {
                unimplemented!("denormal f80")
            }
        } else {
            unimplemented!("pseudo-denormal f80")
        }
    } else if exponent == (1 << 15) - 1 {
        unimplemented!("infinity/nan f80")
    } else if (mantissa >> 63) == 0 {
        unimplemented!("unnormal f80")
    } else {
        let exponent = exponent as i32 - 16383;
        if exponent < -1022 || exponent > 1023 {
            unimplemented!("f80 exponent is too large");
        }

        // construct the f64
        let exponent = exponent + 1023;
        let mantissa = mantissa >> (64 - 52);
        f64::from_bits(
            mantissa as u64 | ((exponent as u64) << 52) | if sign { 1 << 63 } else { 0 },
        )
    }
}

pub fn read_pstring(data: &mut &[u8]) -> Result<String> {
    if data.is_empty() {
        return Err(AiffError::InvalidFormat);
    }
    let len = data[0] as usize;
    *data = &data[1..];

    if data.len() < len {
        return Err(AiffError::InvalidFormat);
    }
    let string_data = &data[..len];
    *data = &data[len..];

    if len % 2 == 0 {
        *data = &data[1..];
    }
    Ok(String::from_utf8_lossy(string_data).into_owned())
}
