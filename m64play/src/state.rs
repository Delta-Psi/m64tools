pub mod player;
use player::SequencePlayer;
pub mod channel;
use channel::SequenceChannel;
pub mod layer;
use layer::SequenceLayer;

use bitflags::bitflags;

const CHANNELS_MAX: u8 = 16;
const LAYERS_MAX: u8 = 4;

const TATUMS_PER_BEAT: u16 = 48;
const TEMPO_SCALE: u16 = TATUMS_PER_BEAT;

// AudioSessionSettings stuff i dont really understand
const FREQUENCY: u32 = 32000;
const SAMPLES_PER_FRAME_TARGET: u32 = FREQUENCY / 60;
const UPDATES_PER_FRAME: u32 = SAMPLES_PER_FRAME_TARGET / 160 + 1;
const TEMPO_INTERNAL_TO_EXTERNAL: u32 =
    (UPDATES_PER_FRAME as f32 * 2_880_000.0 / TATUMS_PER_BEAT as f32 / 16.713) as u32;

bitflags! {
    pub struct MuteBehavior: u8 {
        const STOP_SCRIPT = 0x80;
        const STOP_NOTES = 0x40;
        const SOFTEN = 0x20;
    }
}

//const NOTE_PRIORITY_DISABLED: u8 = 0;
//const NOTE_PRIORITY_STOPPING: u8 = 1;
//const NOTE_PRIORITY_MIN: u8 = 2;
const NOTE_PRIORITY_DEFAULT: u8 = 3;

#[derive(Debug)]
pub struct ScriptState {
    pub pc: u16,
    pub stack: [u16; 4],
    pub rem_loop_iters: [u8; 4],
    pub depth: usize,
}

impl ScriptState {
    pub fn new(pc: u16) -> Self {
        Self {
            pc,
            stack: Default::default(),
            rem_loop_iters: Default::default(),
            depth: 0,
        }
    }
}

