//! Disassembles the m64 data from stdin.
use std::io::Read;
use std::collections::BTreeSet;
use m64::sequence::SequenceCmd;
use m64::channel::ChannelCmd;
use m64::layer::LayerCmd;

pub fn main() {
    let mut buf = Vec::new();
    std::io::stdin().read_to_end(&mut buf).unwrap();

    let mut channels = BTreeSet::new();
    {
        println!("== SEQUENCE ==");
        let mut data = &buf[..];
        loop {
            let (cmd, size) = SequenceCmd::read(data);
            data = &data[size..];

            println!("{:x?}", cmd);
            if cmd.is_end() {
                break;
            }

            if let SequenceCmd::StartChannel(_, channel_addr) = cmd {
                channels.insert(channel_addr as usize);
            }
        }
        println!();
    }

    let mut layers = BTreeSet::new();
    for channel_addr in &channels {
        println!("== CHANNEL {:x} ==", channel_addr);
        let mut data = &buf[*channel_addr as usize..];
        loop {
            let (cmd, size) = ChannelCmd::read(data);
            data = &data[size..];

            println!("{:x?}", cmd);
            if cmd.is_end() {
                break;
            }

            if let ChannelCmd::SetLayer(_, layer_addr) = cmd {
                layers.insert(layer_addr as usize);
            }
        }
        println!();
    }

    for layer_addr in &layers {
        println!("== LAYER {:x} ==", layer_addr);
        let mut data = &buf[*layer_addr as usize..];
        loop {
            let (cmd, size) = LayerCmd::read(data, true);
            data = &data[size..];

            println!("{:x?}", cmd);
            if cmd.is_end() {
                break;
            }
        }
        println!();
    }
}
