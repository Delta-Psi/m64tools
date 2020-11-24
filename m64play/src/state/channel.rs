use super::*;
use m64::ChannelCmd;

#[derive(Debug)]
pub struct SequenceChannel {
    pub enabled: bool,
    pub finished: bool,
    pub stop_script: bool,
    //pub stop_something2: bool,
    pub has_instrument: bool,
    pub stereo_headset_effects: bool,
    pub large_notes: bool,
    //pub note_allocation_policy: u8,
    //pub mute_behavior: MuteBehavior,
    pub reverb: u8,
    pub note_priority: u8,
    //pub bank_id: u8,
    //pub updates_per_frame_unused,
    pub vibrato_rate_start: u16,
    pub vibrato_extent_start: u16,
    pub vibrato_rate_target: u16,
    pub vibrato_extent_target: u16,
    pub vibrato_rate_change_delay: u16,
    pub vibrato_extent_change_delay: u16,
    pub vibrato_delay: u16,
    pub delay: u16,
    //pub instr_or_wave: i16,
    pub transposition: i16,
    pub volume_scale: f32,
    pub volume: f32,
    pub pan: f32,
    pub pan_channel_weight: f32,
    pub freq_scale: f32,
    //pub dyn_table_addr: u16,
    // note_unused
    // layer_unused
    // instrument
    pub layers: [Option<Box<SequenceLayer>>; LAYERS_MAX as usize],
    pub sound_script_io: [i8; 8],
    pub script_state: ScriptState,
    // asdr
    // note_pool
}

impl SequenceChannel {
    pub(crate) fn new() -> Self {
        Self {
            enabled: false,
            finished: false,
            stop_script: false,
            has_instrument: false,
            stereo_headset_effects: false,
            transposition: 0,
            large_notes: false,
            script_state: ScriptState::new(0),
            volume: 1.0,
            volume_scale: 1.0,
            freq_scale: 1.0,
            pan: 0.5,
            pan_channel_weight: 1.0,
            reverb: 0,
            note_priority: NOTE_PRIORITY_DEFAULT,
            delay: 0,
            //adsr:
            vibrato_rate_target: 0x800,
            vibrato_rate_start: 0x800,
            vibrato_extent_target: 0,
            vibrato_extent_start: 0,
            vibrato_rate_change_delay: 0,
            vibrato_extent_change_delay: 0,
            vibrato_delay: 0,
            sound_script_io: [-1; 8],

            layers: unsafe {
                // see above use of unsafe
                std::mem::zeroed()
            },
        }
    }

    // ported from sequence_channel_process_script
    pub fn process(&mut self, player: &SequencePlayer, data: &[u8]) {
        if !self.enabled {
            return;
        }

        if self.stop_script {
            for j in 0..self.layers.len() {
                self.process_layer(j, data);
            }
            return;
        }

        if player.muted && player.mute_behavior.contains(MuteBehavior::STOP_SCRIPT) {
            return;
        }

        if self.delay != 0 {
            self.delay -= 1;
        }

        if self.delay == 0 {
            loop {
                use ChannelCmd::*;
                let (cmd, size) = ChannelCmd::read(&data[self.script_state.pc as usize..]);
                self.script_state.pc += size as u16;
                println!("channel: {:x?}", cmd);

                let state = &mut self.script_state;
                match cmd {
                    End => {
                        if state.depth == 0 {
                            self.enabled = false;
                            self.finished = true;
                            break;
                        }
                        state.depth -= 1;
                        state.pc = state.stack[state.depth];
                    }

                    Delay1 => {
                        break;
                    }
                    Delay(delay) => {
                        self.delay = delay;
                        break;
                    }

                    // ...
                    Transpose(trans) => {
                        self.transposition = trans as i16;
                    }

                    LargeNotesOn => {
                        self.large_notes = true;
                    }
                    SetLayer(j, addr) => {
                        // from seq_channel_set_layer
                        // NOTE: does not check for the global layer limit
                        if self.layers[j as usize].is_some() {
                            // TODO: seq_channel_layer_note_decay
                        }

                        let layer = SequenceLayer::new(addr, &self);
                        self.layers[j as usize] = Some(Box::new(layer));
                    }
                    SetVol(vol) => {
                        self.volume = vol as f32 / 127.0;
                    }
                    SetNotePriority(np) => {
                        self.note_priority = np;
                    }
                    SetPan(pan) => {
                        self.pan = pan as f32 / 128.0;
                    }
                    SetReverb(reverb) => {
                        self.reverb = reverb;
                    }
                    SetInstr(_) => {
                        // big fat TODO
                    }

                    _ => todo!("channel cmd {:x?}", cmd),
                }
            }
        }

        for j in 0..self.layers.len() {
            self.process_layer(j, data);
        }
    }

    fn process_layer(&mut self, j: usize, data: &[u8]) {
        if self.layers[j].is_some() {
            // same as above
            let mut layer = std::mem::take(&mut self.layers[j]);
            layer.as_mut().unwrap().process(&self, data);
            self.layers[j] = layer;
        }
    }
}

