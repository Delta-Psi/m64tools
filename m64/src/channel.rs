use byteorder::{ByteOrder, BE};
use super::read_var;

#[derive(Debug)]
pub enum ChannelCmd {
    /// End of script, loop or function
    End,

    /// Delay for 1 tick
    Delay1,
    /// Delay for X ticks
    Delay(u16),

    /// Call a function
    Call(u16),
    /// Jump to a point in the sequence script
    Jump(u16),
    /// Jump if Q == 0, where Q is an s8 used to hold temporary sequence script state
    Beqz(u16),
    /// Jump if Q < 0
    Bltz(u16),

    /// Repeat until loopend for X iterations (256 if X = 0)
    Loop(u8),
    /// Loop end marker
    LoopEnd,

    /// Decrease loop/function stack depth by 1. Combine with a jump to break out of a loop.
    Break,

    /// Jump if Q >= 0
    Bgez(u16),

    /// Infinite delay.
    Hang,

    /// Reserve X notes for exclusive use by this channel (dropping earlier reservations).
    ReserveNotes(u8),
    /// Drop all earlier reservations made by this channel.
    UnReserveNotes,

    /// If Q != -1, make a function call to position dyntable[Q], where dyntable is a
    /// channel-specific u16 array which can set by setdyntable/dynsetdyntable.
    DynCall,

    /// Set delay until vibrato starts for each note.
    SetVibratoDelay(u8),
    /// Set vibrato extent for each note as a function that
    /// starts at X, goes up linearly to Y over
    /// a time span given by Z, then stays there.
    SetVibratoExtentLinear(u8, u8, u8),
    /// Like above except for vibrato rate.
    SetVibratoRateLinear(u8, u8, u8),

    /// Set scale factor for volume (0…255 = none to twice normal).
    SetVolScale(u8),

    /// Set scale factor #2 for volume (0…255 = none to twice normal). The two
    /// scale factors are separately controlled by outer audio logic, e.g. when
    /// changing between rooms, when playing sound effects, etc. The game almost
    /// exclusively uses setvol, so that should probably be preferred, but
    /// setvolscale shows up in a few locations in the sound effect sequence.
    SetVol(u8),

    /// Pitch bend using a raw frequency multiplier X/2^15.
    FreqScale(u16),

    /// Set the pan for this channel (0…128)
    SetPan(u8),
    /// The pan for each note will be computed as W * channel pan + (1 - W) *
    /// note pan. This sets W (0…128 maps to 0…1). W defaults to 1.0.
    SetPanChanWeight(u8),

    /// Set transposition in semitones.
    Transpose(i8),

    /// Set ADSR envelope. See below for format.
    SetEnvelope(u16),
    /// Set ADSR decay/release rate.
    SetDecayRelease(u8),

    /// Set vibrato extent.
    SetVibratoExtent(u8),
    /// Set vibrato rate.
    SetVibratoRate(u8),

    /// Seems like it was meant to set the number of updates per frame?
    /// But it doesn’t actually do anything.
    SetUpdatesPerFrame(u8),
    
    /// Set amount of reverb (or “dry/wet mix”?).
    SetReverb(u8),

    /// Pitch bend by <= 1 octave in either direction (-127…127).
    PitchBend(i8),

    /// Set ADSR sustain volume.
    SetSustain(u8),

    /// Set note allocation policy for channel. See the description for sequence scripts.
    SetNoteAllocationPolicy(u8),

    /// Enable (1) or disable (0) some pan/volume effects related to
    /// stereo/headset. I don’t know which ones precisely – pan is
    /// still taken into account even with this disabled. With this
    /// disabled, stereo/headset gets the same output.
    StereoHeadsetEffects(u8),

    /// Set Q to a constant.
    SetVal(u8),
    /// Set Q to sequenceData[X + Q].
    ReadSeq(u16),

    /// Set mute behavior for this channel.
    SetMuteBhv(u8),

    /// Bitand Q by a constant.
    BitAnd(u8),
    /// Subtract a constant from Q.
    Subtract(u8),
    /// Overwrite the byte at the given position in the sequence script by (Q + X).
    WriteSeq(u8, u16),

    /// Switch bank within instrument bank sets, to the X’th bank, 0-indexed,
    /// counting from the end.
    SetBank(u8),

    /// If Q != -1, set dyntable to point to position dyntable[Q] in the sequence data.
    DynSetDynTable,

    /// Set notes for this channel to use “large notes” format. SM64 does this for
    /// all channels.  (See note vs. shortnote in the layer section below.)
    LargeNotesOn,
    /// Set notes for this channel to use “short notes” format.
    LargeNotesOff,

    /// Set dyntable to point to a given u16 array from the sequence data.
    SetDynTable(u16),

    /// Set instrument (program?) within bank. 0x80-0x83 set raw waves (sawtooth,
    /// triangle, sine, square), 0x7f is special, otherwise 0-indexed.
    SetInstr(u8),

    /// If Q != -1, start layer N, with layer script given by dyntable[Q].
    DynSetLayer(u8),
    /// Stop layer N.
    FreeLayer(u8),
    /// Start layer N, with layer script starting at address X.
    SetLayer(u8, u16),

    /// Set Q to IO[N], where IO is an array of bytes that can be read/written
    /// by both the game and the channel script. If N < 4, IO[N] is then set to -1.
    IoReadVal(u8),
    /// Set IO[N] to Q.
    IoWriteVal(u8),

    /// Set note priority for later notes played on this channel to N. When the
    /// game wants to play a note and there aren’t any free notes available,
    /// a note with smaller or equal priority is discarded. Should be >= 2; 0
    /// and 1 have internal meaning. Defaults to 3.
    SetNotePriority(u8),

    /// Decrease Q by IO[N].
    IoReadValSub(u8),
    /// Set Q to IO2[X], where IO2 is the IO array for channel N.
    IoReadVal2(u8, u8),
    /// Set IO2[X] to Q, where IO2 is the IO array for channel N.
    IoWriteVal2(u8, u8),

    /// Disable channel N for the parent sequence.
    DisableChannel(u8),
    /// Start channel N for the parent sequence, with channel script starting at
    /// address X.
    StartChannel(u8, u16),

    /// Set Q to 1 or 0, depending on whether layer N has been disabled (either
    /// forcibly or by finishing its script).
    TestLayerFinished(u8),
}

impl ChannelCmd {
    /// Returns both the command and the amount of bytes read.
    pub fn read(data: &[u8]) -> (Self, usize) {
        use ChannelCmd::*;

        match data[0] {
            0xff => (End, 1),

            0xfe => (Delay1, 1),
            0xfd => {
                let (var, size) = read_var(&data[1..3]);
                (Delay(var), 1+size)
            },

            0xfc => (Call(BE::read_u16(&data[1..3])), 3),
            0xfb => (Jump(BE::read_u16(&data[1..3])), 3),
            0xfa => (Beqz(BE::read_u16(&data[1..3])), 3),
            0xf9 => (Bltz(BE::read_u16(&data[1..3])), 3),

            0xf8 => (Loop(data[1]), 2),
            0xf7 => (LoopEnd, 1),

            0xf6 => (Break, 1),

            0xf5 => (Bgez(BE::read_u16(&data[1..3])), 3),

            0xf3 => (Hang, 1),

            0xf2 => (ReserveNotes(data[1]), 2),
            0xf1 => (UnReserveNotes, 1),

            0xe4 => (DynCall, 1),

            0xe3 => (SetVibratoDelay(data[1]), 1),
            0xe2 => (SetVibratoExtentLinear(data[1], data[2], data[3]), 4),
            0xe1 => (SetVibratoRateLinear(data[1], data[2], data[3]), 4),
            0xe0 => (SetVolScale(data[1]), 2),

            0xdf => (SetVol(data[1]), 2),
            0xde => (FreqScale(BE::read_u16(&data[1..3])), 4),
            0xdd => (SetPan(data[1]), 2),
            0xdc => (SetPanChanWeight(data[1]), 2),

            0xdb => (Transpose(data[1] as i8), 2),
            0xda => (SetEnvelope(BE::read_u16(&data[1..3])), 3),
            0xd9 => (SetDecayRelease(data[1]), 2),
            0xd8 => (SetVibratoExtent(data[1]), 2),
            0xd7 => (SetVibratoRate(data[1]), 2),

            0xd6 => (SetUpdatesPerFrame(data[1]), 2),

            0xd4 => (SetReverb(data[1]), 2),

            0xd3 => (PitchBend(data[1] as i8), 2),
            0xd2 => (SetSustain(data[1]), 2),

            0xd1 => (SetNoteAllocationPolicy(data[1]), 2),
            0xd0 => (StereoHeadsetEffects(data[1]), 2),

            0xcc => (SetVal(data[1]), 2),
            0xcb => (ReadSeq(BE::read_u16(&data[1..3])), 3),

            0xca => (SetMuteBhv(data[1]), 2),

            0xc9 => (BitAnd(data[1]), 2),
            0xc8 => (Subtract(data[1]), 2),
            0xc7 => (WriteSeq(data[1], BE::read_u16(&data[2..4])), 4),
            0xc6 => (SetBank(data[1]), 2),
            0xc5 => (DynSetDynTable, 1),
            0xc4 => (LargeNotesOn, 1),
            0xc3 => (LargeNotesOff, 1),

            0xc2 => (SetDynTable(BE::read_u16(&data[1..3])), 3),
            0xc1 => (SetInstr(data[1]), 2),

            0xb0 ..= 0xbf => (DynSetLayer(data[0] & 0x0f), 1),
            0xa0 ..= 0xaf => (FreeLayer(data[0] & 0x0f), 1),
            0x90 ..= 0x9f => (SetLayer(data[0] & 0x0f, BE::read_u16(&data[1..3])), 3),

            0x80 ..= 0x8f => (IoReadVal(data[0] & 0x0f), 1),
            0x70 ..= 0x7f => (IoWriteVal(data[0] & 0x0f), 1),

            0x60 ..= 0x6f => (SetNotePriority(data[0] & 0x0f), 1),

            0x50 ..= 0x5f => (IoReadValSub(data[0] & 0x0f), 1),
            0x40 ..= 0x4f => (IoReadVal2(data[0] & 0x0f, data[1]), 2),
            0x30 ..= 0x3f => (IoWriteVal2(data[0] & 0x0f, data[1]), 2),
            0x20 ..= 0x2f => (DisableChannel(data[0] & 0x0f), 1),
            0x10 ..= 0x1f => (StartChannel(data[0] & 0x0f, BE::read_u16(&data[1..3])), 3),
            0x00 ..= 0x0f => (TestLayerFinished(data[0] & 0x0f), 1),


            _ => unimplemented!("sequence command 0x{:02x}", data[0]),
        }
    }

    pub fn is_end(&self) -> bool {
        if let ChannelCmd::End = self {
            true
        } else {
            false
        }
    }
}
