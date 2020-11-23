use crate::error::*;
use byteorder::{ByteOrder, BE};
use std::convert::TryInto;

#[derive(Debug)]
pub enum PlayMode {
    NoLooping,
    ForwardLooping,
    ForwardBackwardLooping,
}

impl std::convert::TryFrom<u16> for PlayMode {
    type Error = AiffError;

    fn try_from(value: u16) -> Result<Self> {
        match value {
            0 => Ok(PlayMode::NoLooping),
            1 => Ok(PlayMode::ForwardLooping),
            2 => Ok(PlayMode::ForwardBackwardLooping),
            _ => Err(AiffError::InvalidPlayMode(value)),
        }
    }
}

#[derive(Debug)]
pub struct Loop {
    pub play_mode: PlayMode,
    pub begin_loop: u16,
    pub end_loop: u16,
}

impl Loop {
    const SIZE: usize = 6;

    fn read(data: &[u8]) -> Result<Self> {
        if data.len() < Self::SIZE {
            return Err(AiffError::InvalidFormat);
        }

        Ok(Self {
            play_mode: BE::read_u16(&data[0..2]).try_into()?,
            begin_loop: BE::read_u16(&data[2..4]),
            end_loop: BE::read_u16(&data[4..6]),
        })
    }
}

#[derive(Debug)]
pub struct InstrumentChunk {
    pub base_note: i8,
    pub detune: i8,
    pub low_note: i8,
    pub high_note: i8,
    pub low_velocity: i8,
    pub high_velocity: i8,
    pub gain: i16,
    pub sustain_loop: Loop,
    pub release_loop: Loop,
}

impl InstrumentChunk {
    const SIZE: usize = 20;

    pub(crate) fn read(data: &[u8]) -> Result<Self> {
        if data.len() < Self::SIZE {
            return Err(AiffError::InvalidFormat);
        }

        Ok(Self {
            base_note: data[0] as i8,
            detune: data[1] as i8,
            low_note: data[2] as i8,
            high_note: data[3] as i8,
            low_velocity: data[4] as i8,
            high_velocity: data[5] as i8,
            gain: BE::read_i16(&data[6..8]) as i16,
            sustain_loop: Loop::read(&data[8..14])?,
            release_loop: Loop::read(&data[14..20])?,
        })
    }
}
