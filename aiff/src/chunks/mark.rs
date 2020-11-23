use crate::error::*;
use crate::types::*;
use byteorder::{ByteOrder, BE};

#[derive(Debug)]
pub struct Marker {
    pub id: u16,
    pub position: u32,
    pub name: String,
}

impl Marker {
    fn read(data: &mut &[u8]) -> Result<Self> {
        if data.len() < 6 {
            return Err(AiffError::InvalidFormat);
        }

        let id = BE::read_u16(&data[0..2]);
        let position = BE::read_u32(&data[2..6]);
        *data = &data[6..];
        let name = read_pstring(data)?;

        Ok(Marker { id, position, name })
    }
}

#[derive(Debug)]
pub struct MarkerChunk {
    pub markers: Vec<Marker>,
}

impl MarkerChunk {
    pub(crate) fn read(data: &[u8]) -> Result<Self> {
        if data.len() < 2 {
            return Err(AiffError::InvalidFormat);
        }

        let num_markers = BE::read_u16(&data[0..2]);

        let mut markers = Vec::with_capacity(num_markers as usize);
        let mut data = &data[2..];
        for _ in 0..num_markers {
            markers.push(Marker::read(&mut data)?);
        }

        Ok(Self { markers })
    }
}
