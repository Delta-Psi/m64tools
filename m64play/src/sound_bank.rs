// TODO: loop points

use std::path::{Path, PathBuf};
use std::collections::HashMap;
use std::sync::Arc;
use serde_json::Value;

const BASE_SAMPLE_RATE: f64 = 32000.0;

type Envelopes<'a> = HashMap<&'a str, Vec<EnvelopeStep>>;

#[derive(Debug, Copy, Clone)]
pub enum EnvelopeStep {
    Stop,
    Hang,
    Goto(u16),
    Restart,
    Next {
        delay: u16,
        volume: u16,
    },
}

fn parse_envelope(value: &Value) -> Vec<EnvelopeStep> {
    value.as_array().unwrap().iter().map(|step|
        match step {
            Value::String(str_) => match str_.as_ref() {
                "stop" => EnvelopeStep::Stop,
                "hang" => EnvelopeStep::Hang,
                "restart" => EnvelopeStep::Restart,
                _ => panic!(),
            },
            Value::Array(vec) => match vec.as_slice() {
                [Value::String(str_), i] if str_.as_str() == "goto" =>
                    EnvelopeStep::Goto(i.as_i64().unwrap() as u16),
                [delay, volume] =>
                    EnvelopeStep::Next {
                        delay: delay.as_i64().unwrap() as u16,
                        volume: volume.as_i64().unwrap() as u16,
                    },
                _ => panic!(),
            },
            _ => panic!(),
        }
    ).collect()
}

#[derive(Debug, Clone)]
pub struct Loop {
    start: u32,
    end: u32,
    count: u32,
    state: [i16; 16],
}

#[derive(Debug, Clone)]
pub struct Sample {
    data: Vec<i16>,
    //loop_: Loop,
}

struct SampleLoader {
    sample_path: PathBuf,
    samples: HashMap<String, (Arc<Sample>, f32)>,
}

impl SampleLoader {
    fn load(&mut self, name: &str) -> (Arc<Sample>, f32) {
        let path = self.sample_path.join(format!("{}.aiff", name));
        self.samples.entry(name.to_string()).or_insert_with(|| {
            let data = std::fs::read(path).unwrap();
            let aiff = aiff::AiffReader {
                read_mark: true,
                read_inst: true,
                ..Default::default()
            }.read(&data).unwrap();

            let tuning = (aiff.comm.sample_rate / BASE_SAMPLE_RATE) as f32;

            //let loop_ = &aiff.inst.as_ref().unwrap().sustain_loop;
            //let loop_start = aiff.mark.as_ref().unwrap().markers[loop_.begin_loop as usize - 1].position;
            //let loop_end = aiff.mark.as_ref().unwrap().markers[loop_.end_loop as usize - 1].position;

            (Arc::new(Sample {
                data: aiff.samples().map(|f| f as i16).collect(),
                //loop_: todo!(),
            }), tuning)
        }).clone()
    }
}

#[derive(Debug, Clone)]
pub struct Sound {
    sample: Arc<Sample>,
    tuning: f32,
}

fn parse_sound(value: &Value, sample_loader: &mut SampleLoader) -> Sound {
    if let Some(sample) = value.as_str() {
        let (sample, tuning) = sample_loader.load(sample);
        Sound {
            sample,
            tuning,
        }
    } else {
        let value = value.as_array().unwrap();
        Sound {
            sample: sample_loader.load(value[0].as_str().unwrap()).0,
            tuning: value[1].as_f64().unwrap() as f32,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Instrument {
    normal_range_lo: u8,
    normal_range_hi: u8,
    release_rate: u8,
    envelope: Vec<EnvelopeStep>,
    low_notes_sound: Option<Sound>,
    normal_notes_sound: Sound,
    high_notes_sound: Option<Sound>,
}

fn parse_instrument(value: &Value, envelopes: &Envelopes, sample_loader: &mut SampleLoader) -> Instrument {
    let instrument = value.as_object().unwrap();

    Instrument {
        normal_range_lo: instrument.get("normal_range_lo").map(|v| v.as_i64().unwrap() as u8).unwrap_or(0),
        normal_range_hi: instrument.get("normal_range_hi").map(|v| v.as_i64().unwrap() as u8).unwrap_or(127),
        release_rate: instrument["release_rate"].as_i64().unwrap() as u8,
        envelope: envelopes[instrument["envelope"].as_str().unwrap()].clone(),
        low_notes_sound: instrument.get("sound_lo").map(|s| parse_sound(s, sample_loader)),
        normal_notes_sound: parse_sound(&instrument["sound"], sample_loader),
        high_notes_sound: instrument.get("sound_hi").map(|s| parse_sound(s, sample_loader)),
    }
}

#[derive(Debug)]
pub struct Drum {
    pub release_rate: u8,
    pub pan: u8,
    sound: Sound,
    envelope: Vec<EnvelopeStep>,
}

fn parse_drum(value: &Value, envelopes: &Envelopes, sample_loader: &mut SampleLoader) -> Drum {
    let drum = value.as_object().unwrap();

    Drum {
        release_rate: drum["release_rate"].as_i64().unwrap() as u8,
        pan: drum["pan"].as_i64().unwrap() as u8,
        sound: parse_sound(&drum["sound"], sample_loader),
        envelope: envelopes[drum["envelope"].as_str().unwrap()].clone(),
    }
}

#[derive(Debug)]
pub struct SoundBank {
    drums: Vec<Drum>,
    instruments: Vec<Option<Instrument>>,
}

impl SoundBank {
    pub(crate) fn load(base_path: &Path, name: &str) -> Self {
        let path = base_path.join(format!("sound_banks/{}.json", name));
        let json = std::fs::read_to_string(path).unwrap();

        let v: Value = serde_json::from_str(&json).unwrap();
        let sound_bank_map = v.as_object().unwrap();

        let envelopes = Self::parse_envelopes(&sound_bank_map["envelopes"]);

        let sample_subdir = sound_bank_map["sample_bank"].as_str().unwrap();
        let mut sample_loader = SampleLoader {
            sample_path: base_path.join(format!("samples/{}", sample_subdir)),
            samples: HashMap::new(),
        };
        let mut instrument_map = HashMap::new();
        let mut drums = Vec::new();
        for (name, instrument) in sound_bank_map["instruments"].as_object().unwrap().iter() {
            if name == "percussion" {
                drums.extend(instrument.as_array().unwrap().iter()
                    .map(|drum| parse_drum(drum, &envelopes, &mut sample_loader))
                );
            } else {
                instrument_map.insert(name.clone(), parse_instrument(instrument, &envelopes, &mut sample_loader));
            }
        }

        let instruments = sound_bank_map["instrument_list"].as_array().unwrap().iter()
            .map(|i| i.as_str())
            .map(|i| match i {
                Some(i) => Some(instrument_map[i].clone()),
                None => None,
            }).collect();

        SoundBank {
            drums,
            instruments,
        }
    }

    fn parse_envelopes(map: &serde_json::Value) -> Envelopes {
        map.as_object().unwrap()
            .iter()
            .map(|(name, envelope)| (
                    name.as_ref(),
                    parse_envelope(envelope),
                )
            ).collect()
    }
}
