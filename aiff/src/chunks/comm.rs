use crate::error::*;
use crate::types::*;
use byteorder::{ByteOrder, BE};
use std::time::Duration;

#[derive(Debug)]
pub struct CommonChunk {
    pub num_channels: u16,
    pub num_sample_frames: u32,
    pub sample_size: u16,
    pub sample_rate: f64,
}

impl CommonChunk {
    pub(crate) fn read(data: &[u8]) -> Result<Self> {
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

    pub fn audio_length(&self) -> Duration {
        Duration::from_secs_f64(self.num_sample_frames as f64 / self.sample_rate)
    }
}
