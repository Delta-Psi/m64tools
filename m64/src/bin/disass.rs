//! Disassembles the m64 data from stdin.
use std::io::Read;
use m64::SequenceCmd;

pub fn main() {
    let mut buf = Vec::new();
    std::io::stdin().read_to_end(&mut buf).unwrap();

    let mut buf = buf.as_ref();
    loop {
        let (cmd, size) = SequenceCmd::read(buf);
        buf = &buf[size..];

        println!("{:?}", cmd);
        if cmd.is_end() {
            break;
        }
    }
}
