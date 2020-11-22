//! Basic support for Audio IFF file reading.

mod error;
pub use error::{AiffError, Result};

pub mod types;
use types::*;

pub mod chunks;
use chunks::*;

use std::collections::HashMap;
use std::convert::TryInto;

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
