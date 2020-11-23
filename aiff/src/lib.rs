//! Basic support for Audio IFF file reading.

mod error;
pub use error::{AiffError, Result};

pub mod types;
use types::*;

pub mod chunks;
use chunks::*;

use std::collections::HashMap;
use std::convert::TryInto;
use byteorder::{BE, ByteOrder};

#[derive(Debug, Default)]
pub struct AiffReader {
    //pub read_mark: bool,
    //pub read_inst: bool,
    //pub read_midi: bool,
    //pub read_aesd: bool,
    //pub read_appl: bool,
    //pub read_comt: bool,
    pub read_other: bool,
}

impl AiffReader {
    pub fn read<'a>(&self, data: &'a [u8]) -> Result<Aiff<'a>> {
        Aiff::read(data, &self)
    }
}

#[derive(Debug)]
pub struct Aiff<'a> {
    pub comm: CommonChunk,
    pub ssnd: SoundDataChunk<'a>,
    //pub mark: Option<MarkerChunk>,
    //pub inst: Option<InstrumentChunk>,
    //pub midi: Option<MidiDataChunk>,
    //pub aesd: Option<AudioRecordingChunk>,
    //pub appl: Option<ApplicationSpecificChunk>,
    //pub comt: Option<CommentsChunk>,
    pub other_chunks: HashMap<ID, &'a [u8]>,
}

impl<'a> Aiff<'a> {
    pub(crate) fn read(data: &'a [u8], config: &AiffReader) -> Result<Self> {
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
        let mut ssnd = None;
        let mut other_chunks = HashMap::new();

        let mut data = &data[4..];
        while !data.is_empty() {
            let (chunk_id, chunk_data) = read_chunk(&mut data)?;

            match chunk_id.data() {
                b"COMM" => {
                    comm = Some(CommonChunk::read(chunk_data)?);
                }
                b"SSND" => {
                    ssnd = Some(SoundDataChunk::read(chunk_data)?);
                }

                _ => if config.read_other {
                    other_chunks.insert(chunk_id, chunk_data);
                }
            }
        }

        Ok(Self {
            comm: comm.ok_or(AiffError::MissingComm)?,
            ssnd: ssnd.ok_or(AiffError::MissingSsnd)?,
            other_chunks,
        })
    }

    pub fn samples(&self) -> Samples<'a> {
        Samples {
            data: self.ssnd.raw_data(),
            remaining_sample_frames: self.comm.num_sample_frames,
            sample_size: self.comm.sample_size,
        }
    }
}

#[derive(Debug)]
pub struct Samples<'a> {
    data: &'a [u8],
    remaining_sample_frames: u32,
    sample_size: u16,
}

impl<'a> Iterator for Samples<'a> {
    type Item = i32;

    fn next(&mut self) -> Option<i32> {
        let bytes_per_sample = (self.sample_size + 7) / 8;

        if self.data.len() < bytes_per_sample as usize || self.remaining_sample_frames == 0 {
            None
        } else {
            self.remaining_sample_frames -= 1;

            let value = match bytes_per_sample {
                1 => {
                    let v = self.data[0] as i8;
                    (v >> (8 - self.sample_size)) as i32
                }

                2 => {
                    let v = BE::read_i16(self.data);
                    (v >> (16 - self.sample_size)) as i32
                }

                3 => {
                    let v = i32::from_be_bytes([0, self.data[0], self.data[1], self.data[2]]);
                    v >> (24 - self.sample_size)
                }

                4 => {
                    let v = BE::read_i32(self.data);
                    v >> (32 - self.sample_size)
                }

                _ => panic!("invalid sample size {}", self.sample_size),
            };
            self.data = &self.data[bytes_per_sample as usize..];
            Some(value)
        }
    }
}

impl<'a> Samples<'a> {
    pub fn normalize_to_f32(self) -> impl Iterator<Item = f32> + 'a {
        let sample_size = self.sample_size;
        self.map(move |s| s as f32 / (1 << sample_size - 1) as f32)
    }
}
