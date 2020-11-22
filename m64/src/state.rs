use bitflags::bitflags;

const CHANNELS_MAX: u8 = 16;
const LAYERS_MAX: u8 = 4;

const TATUMS_PER_BEAT: u16 = 48;
const TEMPO_SCALE: u16 = TATUMS_PER_BEAT;

// AudioSessionSettings stuff i dont really understand
const FREQUENCY: u32 = 32000;
const SAMPLES_PER_FRAME_TARGET: u32 = FREQUENCY / 60;
const UPDATES_PER_FRAME: u32 = SAMPLES_PER_FRAME_TARGET / 160 + 1;
const TEMPO_INTERNAL_TO_EXTERNAL: u32 = (UPDATES_PER_FRAME as f32 * 2_880_000.0 / TATUMS_PER_BEAT as f32 / 16.713) as u32;

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
                use crate::sequence::SequenceCmd::{self, *};
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
                        state.stack[state.depth-1] = state.pc;
                    }
                    LoopEnd => {
                        state.rem_loop_iters[state.depth - 1] = state.rem_loop_iters[state.depth - 1].wrapping_sub(1);
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

                    _ => unimplemented!("player cmd {:x?}", cmd),
                }
            }
        }

        for i in 0 .. self.channels.len() {
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
    pub fn new() -> Self {
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
            for j in 0 .. self.layers.len() {
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
                use crate::channel::ChannelCmd::{self, *};
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
                    },

                    Delay1 => {
                        break;
                    },
                    Delay(delay) => {
                        self.delay = delay;
                        break;
                    },

                    // ...

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

                    _ => unimplemented!("channel cmd {:x?}", cmd),
                }
            }
        }

        for j in 0 .. self.layers.len() {
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
    transposition: i16,
    //freq_scale: f32,
    velocity_square: f32,
    pan: f32,
    //note_velocity: f32,
    //note_pan: f32,
    //note_freq_scale: f32,
    //short_note_default_play_percentage: i16,
    play_percentage: Option<i16>,
    delay: i16,
    duration: i16,
    delay_unused: i16,
    //note
    //instrument
    //sound
    //seq_channel
    script_state: ScriptState,
    //listItem

    pub pitch: Option<u8>,
}

impl SequenceLayer {
    pub fn new(addr: u16, _channel: &SequenceChannel) -> Self {
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
            use crate::layer::LayerCmd::{self, *};
            let (cmd, size) = LayerCmd::read(&data[self.script_state.pc as usize..], channel.large_notes);
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
                },

                // ...

                Delay(delay) => {
                    self.delay = delay as i16;
                    self.stop_something = true;
                    break;
                },

                Note0 { percentage, duration, velocity, pitch } => {
                    // TODO
                    self.stop_something = false;
                    self.note_duration = duration;
                    self.play_percentage = Some(percentage as i16);
                    self.velocity_square = (velocity as f32).powi(2);
                    self.delay = percentage as i16;
                    self.duration = self.note_duration as i16 * percentage as i16 / 256;

                    // TODO: etc
                    self.pitch = Some(pitch);
                    break;
                },
                Note1 { percentage, velocity, pitch } => {
                    self.note_duration = 0;
                    self.play_percentage = Some(percentage as i16);
                    self.velocity_square = (velocity as f32).powi(2);
                    self.delay = percentage as i16;
                    self.duration = self.note_duration as i16 * percentage as i16 / 256;

                    self.pitch = Some(pitch);
                    break;
                },
                Note2 {  duration, velocity, pitch } => {
                    self.stop_something = false;
                    self.note_duration = duration;
                    self.velocity_square = (velocity as f32).powi(2);
                    let percentage = self.play_percentage.unwrap();
                    self.delay = percentage as i16;
                    self.duration = self.note_duration as i16 * percentage as i16 / 256;

                    self.pitch = Some(pitch);
                    break;
                },

                // ...

                _ => unimplemented!("channel cmd {:x?}", cmd),
            }
        }
    }
}
