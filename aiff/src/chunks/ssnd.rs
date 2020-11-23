use crate::error::*;
use byteorder::{ByteOrder, BE};

#[derive(Debug)]
pub struct SoundDataChunk<'a> {
    data: &'a [u8],
}

impl<'a> SoundDataChunk<'a> {
    pub(crate) fn read(data: &'a [u8]) -> Result<Self> {
        if data.len() < 8 {
            return Err(AiffError::InvalidFormat);
        }

        let offset = BE::read_u32(&data[0..4]);
        if offset != 0 {
            unimplemented!("nonzero sound data offset");
        }
        let block_size = BE::read_u32(&data[4..8]);
        if block_size != 0 {
            unimplemented!("nonzero sound data block size");
        }

        let data = &data[8..];
        Ok(Self { data })
    }

    pub fn raw_data(&self) -> &'a [u8] {
        self.data
    }
}
