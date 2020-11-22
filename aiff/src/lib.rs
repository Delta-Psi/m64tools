//! Basic support for Audio IFF file reading.

use byteorder::{BE, ByteOrder};
use std::collections::HashMap;
use std::convert::TryInto;
use std::fmt;

use thiserror::Error;
#[derive(Error, Debug)]
pub enum AiffError {
    #[error("invalid format")]
    InvalidFormat,
    #[error("form type is not AIFF")]
    InvalidFormType,
    #[error("missing common chunk")]
    MissingComm,
}

pub type Result<T> = std::result::Result<T, AiffError>;

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
        for i in 0..4 {
            match value[i] {
                b' ' => {
                    has_spaces = true;
                }
                0x21 ..= 0x7e => {
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
        write!(f, "{}{}{}{}",
            self.0[0] as char,
            self.0[1] as char,
            self.0[2] as char,
            self.0[3] as char,
        )
    }
}

fn read_chunk<'a>(data: &mut &'a [u8]) -> Result<(ID, &'a [u8])> {
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
    if size%2 == 1 {
        *data = &data[1..];
    }

    Ok((id.try_into().unwrap(), chunk_data))
}

fn read_f80(data: &[u8]) -> f64 {
    assert!(data.len() >= 10);
    let exponent = BE::read_u16(&data[0..2]);
    let mantissa = BE::read_u64(&data[2..10]);
    
    println!("{:x?}", data);

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
    } else {
        if (mantissa >> 63) == 0 {
            unimplemented!("unnormal f80")
        } else {
            let exponent = exponent as i32 - 16383;
            if exponent < -1022 || exponent > 1023 {
                unimplemented!("f80 exponent is too large");
            }

            println!("{}", exponent);
            println!("{:b}", mantissa);

            // construct the f64
            let exponent = exponent + 1023;
            let mantissa = mantissa >> (64-52);
            f64::from_bits(
                mantissa as u64 |
                ((exponent as u64) << 52) |
                if sign {
                    1 << 63
                } else {
                    0
                }
            )
        }
    }
}

#[derive(Debug)]
pub struct Aiff<'a> {
    pub comm: CommonChunk,
    //pub ssnd: SoundDataChunk<'a>,
    //pub mark: Option<MarkerChunk>,
    //pub inst: Option<InstrumentChunk>,
    //pub midi: Option<MidiDataChunk>,
    //pub aesd: Option<AudioRecordingChunk>,
    //pub appl: Option<ApplicationSpecificChunk>,
    //pub comt: Option<CommentsChunk>,
    pub other_chunks: HashMap<ID, &'a [u8]>,
}

impl<'a> Aiff<'a> {
    pub fn read(data: &'a [u8]) -> Result<Self> {
        let mut data = data;
        let (form_id, form_data) = read_chunk(&mut data)?;
        if form_id.data() != b"FORM" {
            return Err(AiffError::InvalidFormat);
        }

        let data = form_data;
        let form_type: ID = data[0..4].try_into()?;
        if form_type.data() != b"AIFF" {
            return Err(AiffError::InvalidFormType);
        }

        let mut comm = None;
        let mut other_chunks = HashMap::new();

        let mut data = &data[4..];
        while !data.is_empty() {
            let (chunk_id, chunk_data) = read_chunk(&mut data)?;

            match chunk_id.data() {
                b"COMM" => {
                    comm = Some(CommonChunk::read(chunk_data)?);
                }

                _ => {
                    other_chunks.insert(chunk_id, chunk_data);
                }
            }
        }

        Ok(Self {
            comm: comm.ok_or(AiffError::MissingComm)?,
            other_chunks,
        })
    }

    /*pub fn samples(&self) -> Samples<'a> {
        Samples {
            data: self.ssnd.data,
            sample_size: self.comm.sample_size,
        }
    }*/
}

#[derive(Debug)]
pub struct CommonChunk {
    pub num_channels: u16,
    pub num_sample_frames: u32,
    pub sample_size: u16,
    pub sample_rate: f64,
}

impl CommonChunk {
    fn read(data: &[u8]) -> Result<Self> {
        if data.len() != 18 {
            return Err(AiffError::InvalidFormat);
        }

        let num_channels = BE::read_u16(&data[0..2]);
        let num_sample_frames = BE::read_u32(&data[2..6]);
        let sample_size = BE::read_u16(&data[6..8]);
        let sample_rate = read_f80(&data[8..18]);

        Ok(CommonChunk {
            num_channels,
            num_sample_frames,
            sample_size,
            sample_rate,
        })
    }
}

#[derive(Debug)]
pub struct SoundDataChunk<'a> {
    data: &'a [u8],
}

#[derive(Debug)]
pub struct Samples<'a> {
    data: &'a [u8],
    sample_size: i16,
}

impl<'a> Iterator for Samples<'a> {
    type Item = i32;

    fn next(&mut self) -> Option<i32> {
        if self.data.is_empty() {
            None
        } else {
            //let bytes_per_sample = (self.sample_size + 7) / 8;
            None
        }
    }
}
