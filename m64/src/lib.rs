//! Referenced from https://hackmd.io/opEB-OmxRa26P8h8pA-x7w.

pub mod sequence;
pub mod channel;
pub mod layer;

fn read_var(data: &[u8]) -> (u16, usize) {
    // check top bit of data[0]
    if data[0] & 0b1000_0000 == 0 {
        // it's a u8
        (data[0] as u16, 1)
    } else {
        // it's an encoded u16
        let value = u16::from_be_bytes([data[0] & 0b0111_1111, data[1]]);
        (value, 2)
    }
}
