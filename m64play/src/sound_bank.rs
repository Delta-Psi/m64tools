#[derive(Debug)]
pub struct Envelope {
    delay: i16,
    arg: i16,
}

#[derive(Debug)]
pub struct Loop {
    start: u32,
    end: u32,
    count: u32,
}

#[derive(Debug)]
pub struct Sample {
    data: Vec<f32>,
    loop_: Loop,
}

#[derive(Debug)]
pub struct Sound {
    sample: Sample,
    tuning: f32,
}

#[derive(Debug)]
pub struct Instrument {
    normal_range_lo: u8,
    normal_range_hi: u8,
    release_rate: u8,
    envelope: Envelope,
    low_notes_sound: Sound,
    normal_notes_sound: Sound,
    high_notes_sound: Sound,
}

#[derive(Debug)]
pub struct Drum {
    pub release_rate: u8,
    pub pan: u8,
    sound: Sound,
    envelope: AdsrEnvelope,
}

#[derive(Debug)]
pub struct SoundBank {
    drums: _<Drum>,
    instruments: _<Instrument>,
}

impl SoundBank {
    pub(crate) fn load(base_path: &Path, name: &str) -> Self {
    }
}
