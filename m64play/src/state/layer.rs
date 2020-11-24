use super::*;
use m64::LayerCmd;

#[derive(Debug)]
pub struct SequenceLayer {
    pub enabled: bool,
    pub finished: bool,
    pub stop_something: bool,
    pub continuous_notes: bool,
    //pub status: u8,
    pub note_duration: u8,
    //pub portamento_target_note: u8,
    //portamento
    //adsr
    //portamento_time: u16,
    pub transposition: i16,
    //freq_scale: f32,
    pub velocity_square: f32,
    pub pan: f32,
    //note_velocity: f32,
    //note_pan: f32,
    //note_freq_scale: f32,
    //short_note_default_play_percentage: i16,
    pub play_percentage: Option<i16>,
    pub delay: i16,
    pub duration: i16,
    pub delay_unused: i16,
    //note
    //instrument
    //sound
    //seq_channel
    pub script_state: ScriptState,
    //listItem
    pub pitch: Option<u8>,
}

impl SequenceLayer {
    pub(crate) fn new(addr: u16, _channel: &SequenceChannel) -> Self {
        Self {
            //adsr: channel.adsr,
            //adsr.release_rate: 0
            enabled: true,
            stop_something: false,
            continuous_notes: false,
            finished: false,
            //portamento.mode: 0
            script_state: ScriptState::new(addr),
            //status: not loaded
            note_duration: 0x80,
            transposition: 0,
            delay: 0,
            duration: 0,
            delay_unused: 0,
            //note: None
            //instrument: NOne
            velocity_square: 0.0,
            pan: 0.0,

            play_percentage: None,
            pitch: None,
        }
    }

    // from seq_channel_layer_process_script
    pub fn process(&mut self, channel: &SequenceChannel, data: &[u8]) {
        if !self.enabled {
            return;
        }

        if self.delay > 1 {
            self.delay -= 1;
            if !self.stop_something && self.delay <= self.duration {
                // seq_channel_layer_note_decay
                self.stop_something = true;

                self.pitch = None;
            }
            return;
        }

        if !self.continuous_notes {
            // seq_channel_layer_note_decay
        }

        // TODO: check portamento
        loop {
            use LayerCmd::*;
            let (cmd, size) =
                LayerCmd::read(&data[self.script_state.pc as usize..], channel.large_notes);
            self.script_state.pc += size as u16;
            println!("layer: {:x?}", cmd);

            let state = &mut self.script_state;
            match cmd {
                End => {
                    if state.depth == 0 {
                        self.enabled = false;
                        return;
                    }
                    state.depth -= 1;
                    state.pc = state.stack[state.depth];
                }

                // ...
                Delay(delay) => {
                    self.delay = delay as i16;
                    self.stop_something = true;
                    break;
                }

                Call(addr) => {
                    state.stack[state.depth] = state.pc;
                    state.depth += 1;
                    state.pc = addr;
                }

                Transpose(trans) => {
                    self.transposition = trans as i16;
                }

                Note0 {
                    percentage,
                    duration,
                    velocity,
                    pitch,
                } => {
                    // TODO
                    self.stop_something = false;
                    self.note_duration = duration;
                    self.play_percentage = Some(percentage as i16);
                    self.velocity_square = (velocity as f32).powi(2);
                    self.delay = percentage as i16;
                    self.duration = (self.note_duration as u32 * percentage as u32 / 256) as i16;

                    // TODO: etc
                    self.pitch = Some(pitch);
                    break;
                }
                Note1 {
                    percentage,
                    velocity,
                    pitch,
                } => {
                    self.note_duration = 0;
                    self.play_percentage = Some(percentage as i16);
                    self.velocity_square = (velocity as f32).powi(2);
                    self.delay = percentage as i16;
                    self.duration = (self.note_duration as u32 * percentage as u32 / 256) as i16;

                    self.pitch = Some(pitch);
                    break;
                }
                Note2 {
                    duration,
                    velocity,
                    pitch,
                } => {
                    self.stop_something = false;
                    self.note_duration = duration;
                    self.velocity_square = (velocity as f32).powi(2);
                    let percentage = self.play_percentage.unwrap();
                    self.delay = percentage as i16;
                    self.duration = (self.note_duration as u32 * percentage as u32 / 256) as i16;

                    self.pitch = Some(pitch);
                    break;
                }

                // ...
                _ => todo!("layer cmd {:x?}", cmd),
            }
        }
    }
}
