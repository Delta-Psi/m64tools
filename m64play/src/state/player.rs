use super::*;
use m64::SequenceCmd;

#[derive(Debug)]
pub struct SequencePlayer {
    pub finished: bool,
    pub muted: bool,
    //pub seq_variation: bool,
    pub state: u8,
    pub note_alloc_policy: u8,
    pub mute_behavior: MuteBehavior,
    //pub seq_id: u8,
    //pub default_bank: u8,
    //pub loading_bank_id: u8,
    //pub loading_bank_num_instruments: u8,
    //pub loading_bank_num_drums: u8,
    pub tempo: u16,
    pub tempo_acc: u16,
    pub fade_timer: u16,
    pub transposition: i16,
    pub delay: u16,
    //pub seq_data
    pub fade_volume: f32,
    pub fade_velocity: f32,
    pub volume: f32,
    pub mute_volume_scale: f32,
    pub channels: [Option<Box<SequenceChannel>>; CHANNELS_MAX as usize],
    pub script_state: ScriptState,
    // short_velocity_table
    // short_note_duration_table
    // note_pool
    // dma things
    // loading_bank
}

impl Default for SequencePlayer {
    fn default() -> Self {
        Self::new()
    }
}

impl SequencePlayer {
    pub fn new() -> Self {
        Self {
            finished: false,
            muted: false,
            delay: 0,
            state: 0,
            fade_timer: 0,
            tempo_acc: 0,
            tempo: 120 * TEMPO_SCALE,
            transposition: 0,
            mute_behavior: MuteBehavior::all(),
            note_alloc_policy: 0,
            //short_note_velocity_table
            //short_note_duration_table
            fade_volume: 1.0,
            fade_velocity: 0.0,
            volume: 0.0,
            mute_volume_scale: 0.5,

            script_state: ScriptState::new(0),
            channels: Default::default(),
        }
    }

    // ported from sequence_player_process_sequence
    pub fn process(&mut self, data: &[u8]) {
        if self.finished {
            return;
        }

        if self.muted && self.mute_behavior.contains(MuteBehavior::STOP_SCRIPT) {
            return;
        }

        // Check if we surpass the number of ticks needed for a tatum, else stop.
        self.tempo_acc += self.tempo;
        if self.tempo_acc < TEMPO_INTERNAL_TO_EXTERNAL as u16 {
            return;
        }
        self.tempo_acc -= TEMPO_INTERNAL_TO_EXTERNAL as u16;

        if self.delay > 1 {
            self.delay -= 1;
        } else {
            let value: Option<i8> = None;

            loop {
                use SequenceCmd::*;
                let (cmd, size) = SequenceCmd::read(&data[self.script_state.pc as usize..]);
                self.script_state.pc += size as u16;
                println!("{:x?}", cmd);

                let state = &mut self.script_state;
                match cmd {
                    End => {
                        if state.depth == 0 {
                            self.finished = true;
                            break;
                        }
                        state.depth -= 1;
                        state.pc = state.stack[state.depth];
                    }

                    Delay(delay) => {
                        self.delay = delay;
                        break;
                    }
                    Delay1 => {
                        self.delay = 1;
                        break;
                    }

                    Call(addr) => {
                        state.stack[state.depth] = state.pc;
                        state.depth += 1;
                        state.pc = addr;
                    }

                    Loop(count) => {
                        state.rem_loop_iters[state.depth] = count;
                        state.depth += 1;
                        state.stack[state.depth - 1] = state.pc;
                    }
                    LoopEnd => {
                        state.rem_loop_iters[state.depth - 1] =
                            state.rem_loop_iters[state.depth - 1].wrapping_sub(1);
                        if state.rem_loop_iters[state.depth - 1] != 0 {
                            state.pc = state.stack[state.depth - 1];
                        } else {
                            state.depth -= 1;
                        }
                    }

                    Jump(addr) => {
                        state.pc = addr;
                    }
                    Beqz(addr) => {
                        if value.unwrap() == 0 {
                            state.pc = addr;
                        }
                    }
                    Bltz(addr) => {
                        if value.unwrap() < 0 {
                            state.pc = addr;
                        }
                    }
                    Bgez(addr) => {
                        if value.unwrap() >= 0 {
                            state.pc = addr;
                        }
                    }

                    ReserveNotes(_amt) => {
                        // TODO
                    }
                    UnReserveNotes => {
                        // TODO
                    }

                    Transpose(transposition) => {
                        self.transposition = transposition as i16;
                    }
                    TransposeRel(transposition) => {
                        self.transposition += transposition as i16;
                    }

                    // ...
                    SetMuteBhv(bhv) => {
                        self.mute_behavior = MuteBehavior::from_bits_truncate(bhv);
                    }

                    InitChannels(mask) => {
                        self.init_channels(mask);
                    }
                    StartChannel(i, addr) => {
                        self.start_channel(i, addr);
                    }
                    DisableChannels(mask) => {
                        self.disable_channels(mask);
                    }

                    SetVol(vol) => {
                        self.volume = vol as f32 / 127.0;
                    }
                    SetTempo(tempo) => {
                        self.tempo = tempo as u16 * TEMPO_SCALE;
                    }

                    // ...
                    _ => todo!("player cmd {:x?}", cmd),
                }
            }
        }

        for i in 0..self.channels.len() {
            if self.channels[i].is_some() {
                // workaround so we can borrow self and the channel at the same time
                let mut channel = std::mem::take(&mut self.channels[i]);
                channel.as_mut().unwrap().process(&self, data);
                self.channels[i] = channel;
            }
        }
    }

    // ported from sequence_player_init_channels
    fn init_channels(&mut self, mask: u16) {
        let mut mask = mask;
        for channel_slot in &mut self.channels {
            if mask & 1 != 0 {
                *channel_slot = Some(Box::new(SequenceChannel::new()));
            }
            mask >>= 1;
        }
    }
    // ported from sequence_channel_enable
    fn start_channel(&mut self, i: u8, addr: u16) {
        if let Some(channel) = self.channels[i as usize].as_mut() {
            channel.enabled = true;
            channel.finished = false;
            channel.script_state.depth = 0;
            channel.script_state.pc = addr;
            channel.delay = 0;
            for layer_slot in &mut channel.layers {
                *layer_slot = None;
            }
        }
    }
    // ported from sequence_player_disable_channels
    fn disable_channels(&mut self, mask: u16) {
        let mut mask = mask;
        for channel_slot in &mut self.channels {
            if mask & 1 != 0 {
                *channel_slot = None;
            }
            mask >>= 1;
        }
    }
}

