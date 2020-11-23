use super::read_var;
use byteorder::{ByteOrder, BE};

#[derive(Debug)]
pub enum LayerCmd {
    /// End of script, loop or function
    End,

    /// Call a function
    Call(u16),
    /// Jump to a point in the layer script
    Jump(u16),

    /// Repeat until loopend for X iterations (256 if X = 0)
    Loop(u8),
    /// Loop end marker
    LoopEnd,

    /// Set the duration for future short notes to table[N],
    /// where table defaults to {229, 203, 177, 151, 139,
    /// 126, 113, 100, 87, 74, 61, 48, 36, 23, 10, 0} and
    /// can be set by setshortnotedurationtable.
    SetShortNoteDurationFromTable(u8),
    /// Set the velocity for future short notes to table[N],
    /// where table defaults to {12, 25, 38, 51, 57, 64, 71,
    /// 76, 83, 89, 96, 102, 109, 115, 121, 127} and can be
    /// set by setshortnotevelocitytable.
    SetShortNoteVelocityFromTable(u8),

    /// Set the pan for this layer (0…128).
    SetPan(u8),

    /// Set the duration for future short notes.
    SetShortNoteDuration(u8),

    /// Disable portamento for later notes.
    DisablePortamento,
    /// Enable portamento (aka glissando; continuously sliding
    /// pitch). X describes the mode of operation somehow using
    /// its 0x80 bit and a value 0-5 in its lower-order bits.
    /// Y describes the target pitch. Z gives the duration, and
    /// is a u8 if the 0x80 bit is set, otherwise a var.
    Portamento(u8, u8, u16),

    /// Set instrument/program. Similar to the channel command,
    /// except arguments 0x7f and above are not supported.
    SetInstr(u8),

    /// Turn off some bool related to how notes decay somehow?
    /// (This is the default.)
    SomethingOff,
    /// Turn on the same thing.
    SomethingOn,

    /// Set the default play percentage for short notes (see below).
    SetShortNoteDefaultPlayPercentage(u16),

    /// Set transposition in semitones.
    Transpose(u8),

    /// Set velocity for future short notes.
    SetShortNoteVelocity(u8),

    /// Delay for N ticks
    Delay(u16),

    /// Play note with play percentage X, velocity Y, duration Z and
    /// pitch N. (Is “play percentage” the right term?). Only valid
    /// if channel is set to “large notes”.
    Note0 {
        pitch: u8,
        percentage: u16,
        velocity: u8,
        duration: u8,
    },
    /// Play note with play percentage X, velocity Y, duration 0 and
    /// pitch N. Only valid if channel is set to “large notes”.
    Note1 {
        pitch: u8,
        percentage: u16,
        velocity: u8,
    },
    /// Play note with velocity X, duration Y, pitch N and the last
    /// used play percentage. Only valid if channel is set to “large notes”.
    Note2 {
        pitch: u8,
        velocity: u8,
        duration: u8,
    },

    /// Play note with play percentage X and pitch N, with velocity
    /// and duration taken from the short notes settings set by
    /// setshortnote{velocity,duration} (or the fromtable variants),
    /// defaulting to 0 and 0x80 respectively if not set. Only valid
    /// if channel is set to “small notes”.
    SmallNote0 { pitch: u8, percentage: u16 },
    /// Play note with pitch N and the default play percentage, as
    /// set by setshortnotedefaultplaypercentage (there is no built-in
    /// default; using it without that command will read uninitialized
    /// memory). Velocity and duration is set like in smallnote0.  Only
    /// valid if channel is set to “small notes”.
    SmallNote1 { pitch: u8 },
    /// Play note with pitch N and the previous play percentage. Velocity
    /// and duration is set like in smallnote0. Only valid if channel
    /// is set to “small notes”.
    SmallNote2 { pitch: u8 },
}

impl LayerCmd {
    /// Returns both the command and the amount of bytes read.
    pub fn read(data: &[u8], large_notes: bool) -> (Self, usize) {
        use LayerCmd::*;

        match data[0] {
            0xff => (End, 1),

            0xfc => (Call(BE::read_u16(&data[1..3])), 3),
            0xfb => (Jump(BE::read_u16(&data[1..3])), 3),
            0xf8 => (Loop(data[1]), 2),
            0xf7 => (LoopEnd, 1),

            0xe0..=0xef => (SetShortNoteDurationFromTable(data[0] & 0x0f), 1),
            0xd0..=0xdf => (SetShortNoteVelocityFromTable(data[0] & 0x0f), 1),

            0xca => (SetPan(data[1]), 2),
            0xc9 => (SetShortNoteDuration(data[1]), 2),

            0xc8 => (DisablePortamento, 1),
            0xc7 => {
                let x = data[1];
                let y = data[2];
                let z = data[3];

                let (z, z_size) = if z & 0x80 == 0 {
                    read_var(&data[3..])
                } else {
                    (z as u16, 1)
                };

                (Portamento(x, y, z), 3 + z_size)
            }

            0xc6 => (SetInstr(data[1]), 2),

            0xc5 => (SomethingOff, 1),
            0xc4 => (SomethingOn, 1),

            0xc3 => {
                let (var, size) = read_var(&data[1..3]);
                (SetShortNoteDefaultPlayPercentage(var), 1 + size)
            }

            0xc2 => (Transpose(data[1]), 2),

            0xc1 => (SetShortNoteVelocity(data[1]), 2),

            0xc0 => {
                let (var, size) = read_var(&data[1..3]);
                (Delay(var), 1 + size)
            }

            _ => {
                if large_notes {
                    match data[0] {
                        0x00..=0x3f => {
                            let (var, size) = read_var(&data[1..3]);
                            (
                                Note0 {
                                    pitch: data[0],
                                    percentage: var,
                                    velocity: data[1 + size],
                                    duration: data[1 + size + 1],
                                },
                                1 + size + 2,
                            )
                        }
                        0x40..=0x7f => {
                            let (var, size) = read_var(&data[1..3]);
                            (
                                Note1 {
                                    pitch: data[0] - 0x40,
                                    percentage: var,
                                    velocity: data[1 + size],
                                },
                                1 + size + 1,
                            )
                        }
                        0x80..=0xbf => (
                            Note2 {
                                pitch: data[0] - 0x80,
                                velocity: data[1],
                                duration: data[2],
                            },
                            3,
                        ),

                        _ => todo!("layer command 0x{:02x}", data[0]),
                    }
                } else {
                    match data[1] {
                        0x00..=0x3f => {
                            let (var, size) = read_var(&data[1..3]);
                            (
                                SmallNote0 {
                                    pitch: data[0],
                                    percentage: var,
                                },
                                1 + size,
                            )
                        }
                        0x40..=0x7f => (
                            SmallNote1 {
                                pitch: data[0] - 0x40,
                            },
                            1,
                        ),
                        0x80..=0xbf => (
                            SmallNote2 {
                                pitch: data[0] - 0x80,
                            },
                            1,
                        ),

                        _ => todo!("layer command 0x{:02x}", data[0]),
                    }
                }
            }
        }
    }

    pub fn is_end(&self) -> bool {
        matches!(self, LayerCmd::End)
    }
}
