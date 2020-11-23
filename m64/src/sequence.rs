use super::read_var;
use byteorder::{ByteOrder, BE};

#[derive(Debug)]
pub enum SequenceCmd {
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

    /// Jump if Q >= 0
    Bgez(u16),

    /// Reserve X notes for exclusive use by this sequence
    /// (dropping earlier reservations). A limited number of
    /// notes can be played at once (configurable per level,
    /// usually 16); if more than that are played, an existing
    /// one will be stopped and reused. This command can prevent
    /// that from happening across sequences.
    ReserveNotes(u8),
    /// Drop all earlier reservations made by this sequence.
    UnReserveNotes,

    /// Set transposition in semitones.
    Transpose(i8),
    /// Change transposition by some delta.
    TransposeRel(i8),

    /// Set tempo in BPM.
    SetTempo(u8),
    /// Change tempo by some delta.
    AddTempo(i8),

    /// Set volume scale (0...255 = none to twice normal).
    SetVol(u8),
    /// Change volume scale by some delta.
    ChangeVol(i8),

    /// Initialize channels (bitmask). This copies mute behavior and note allocation policy.
    InitChannels(u16),
    /// Uninitialize channels (bitmask).
    DisableChannels(u16),

    /// Set volume multiplier for muted mode (0...127).
    SetMuteScale(i8),
    /// Set sequence to muted mode. Depending on what behavior
    /// has been set this will do different things; from
    /// lowering volume to not running scripts.
    Mute,
    /// Set mute behavior (bitmask). Bit 0x20 lowers volume,
    /// bit 0x40 does something to layers, and bit 0x80 pauses
    /// the sequence script. Default mute behavior is to have
    /// all of those bits set.
    SetMuteBhv(u8),
    /// Set velocity table for short notes (see further down).
    /// Target should be an array of bytes in the sequence data of length 16.
    SetShortNoteVelocityTable(u16),
    /// Set duration table for short notes (see further down). Target should be an array of bytes
    /// in the sequence data of length 16.
    SetShortNoteDurationTable(u16),
    /// Set note allocation policy (bitmask). If bit 0x1 is set, it will try to steal notes back
    /// from what was previously played on the same layer. If bit 0x2 is set, notes will be
    /// allocated exclusively from the channel’s reserved notes, else if 0x4 is set, from the
    /// channel’s or sequence’s reserved notes, else if 0x8 is set, from unreserved notes,
    /// otherwise from all sources. Not used by the game.
    SetNoteAllocationPolicy(u8),

    /// Set Q to a constant.
    SetVal(u8),
    /// Bitand Q by a constant.
    BitAnd(u8),
    /// Subtract a constant from Q.
    Subtract(u8),

    /// Start channel N, with a given channel script.
    StartChannel(u8, u16),

    /// Set Q to the variation bit, which is initially either 0 or 0x80 (i.e. -128, since Q is s8).
    /// This bit comes from the sequence load command. For instance, to play the Koopa shell music,
    /// the game plays sequence 0x8E, which is the same as sequence 0xE but with the variation bit
    /// set.
    GetVariation,
    /// Set the variation bit to Q.
    SetVariation,
    /// Subtract the variation bit from Q.
    SubVariation,

    /// Set Q to 1 or 0, depending on whether channel N has been disabled by channel script.
    TestChDisabled(u8),
}

impl SequenceCmd {
    /// Returns both the command and the amount of bytes read.
    pub fn read(data: &[u8]) -> (Self, usize) {
        use SequenceCmd::*;

        match data[0] {
            0xff => (End, 1),

            0xfe => (Delay1, 1),
            0xfd => {
                let (var, size) = read_var(&data[1..3]);
                (Delay(var), 1 + size)
            }

            0xfc => (Call(BE::read_u16(&data[1..3])), 3),
            0xfb => (Jump(BE::read_u16(&data[1..3])), 3),
            0xfa => (Beqz(BE::read_u16(&data[1..3])), 3),
            0xf9 => (Bltz(BE::read_u16(&data[1..3])), 3),

            0xf8 => (Loop(data[1]), 2),
            0xf7 => (LoopEnd, 1),

            0xf5 => (Bgez(BE::read_u16(&data[1..3])), 3),

            0xf2 => (ReserveNotes(data[1]), 2),
            0xf1 => (UnReserveNotes, 1),

            0xdf => (Transpose(data[1] as i8), 2),
            0xde => (TransposeRel(data[1] as i8), 2),

            0xdd => (SetTempo(data[1]), 2),
            0xdc => (AddTempo(data[1] as i8), 2),

            0xdb => (SetVol(data[1]), 2),
            0xda => (ChangeVol(data[1] as i8), 2),

            0xd7 => (InitChannels(BE::read_u16(&data[1..3])), 3),
            0xd6 => (DisableChannels(BE::read_u16(&data[1..3])), 3),

            0xd5 => (SetMuteScale(data[1] as i8), 2),
            0xd4 => (Mute, 1),

            0xd3 => (SetMuteBhv(data[1]), 2),
            0xd2 => (SetShortNoteVelocityTable(BE::read_u16(&data[1..3])), 3),
            0xd1 => (SetShortNoteDurationTable(BE::read_u16(&data[1..3])), 3),
            0xd0 => (SetNoteAllocationPolicy(data[1]), 2),

            0xcc => (SetVal(data[1]), 2),
            0xc9 => (BitAnd(data[1]), 2),
            0xc8 => (Subtract(data[1]), 2),

            0x90..=0x9f => (StartChannel(data[0] & 0x0f, BE::read_u16(&data[1..3])), 3),

            0x80..=0x8f => (GetVariation, 1),
            0x70..=0x7f => (SetVariation, 1),
            0x50..=0x5f => (SubVariation, 1),
            0x00..=0x0f => (TestChDisabled(data[0] & 0x0f), 1),

            _ => todo!("sequence command 0x{:02x}", data[0]),
        }
    }

    pub fn is_end(&self) -> bool {
        matches!(self, SequenceCmd::End)
    }
}
