use std::io::Read;
use std::time::{Instant, Duration};
use m64::sequence::SequenceCmd;
use m64::channel::ChannelCmd;

const REFRESH_RATE: f32 = 30.0/1.001;

#[derive(Debug)]
struct SequenceState {
    pub addr: u16,
    pub delay: u16,

    pub channels: [Option<Box<ChannelState>>; 16],
}

impl SequenceState {
    pub fn tick(&mut self, data: &[u8]) -> bool {
        for (i, channel_slot) in self.channels.iter_mut().enumerate() {
            if let Some(channel) = channel_slot {
                if !channel.tick(data, i) {
                    *channel_slot = None;
                }
            }
        }
    
        if self.delay == 0 {
            loop {
                let (cmd, size) = SequenceCmd::read(&data[self.addr as usize..]);
                self.addr += size as u16;

                println!("{:x?}", cmd);
                match cmd {
                    SequenceCmd::End => return false,

                    SequenceCmd::Delay1 => {
                        self.delay = 0;
                        break;
                    },
                    SequenceCmd::Delay(amt) => {
                        self.delay = amt - 1;
                        break;
                    },

                    SequenceCmd::StartChannel(i, addr) => {
                        self.channels[i as usize] = Some(Box::new(ChannelState {
                            addr,
                            delay: 0,
                        }));
                    }

                    _ => (),
                }
            }
        } else {
            self.delay -= 1;
        }

        true
    }
}

#[derive(Debug)]
struct ChannelState {
    pub addr: u16,
    pub delay: u16,
}

impl ChannelState {
    pub fn tick(&mut self, data: &[u8], i: usize) -> bool {
        if self.delay == 0 {
            loop {
                let (cmd, size) = ChannelCmd::read(&data[self.addr as usize..]);
                self.addr += size as u16;

                println!("channel {}: {:x?}", i, cmd);
                match cmd {
                    ChannelCmd::End => return false,

                    ChannelCmd::Delay1 => {
                        self.delay = 0;
                        break;
                    },
                    ChannelCmd::Delay(amt) => {
                        self.delay = amt - 1;
                        break;
                    },

                    _ => (),
                }
            }
        } else {
            self.delay -= 1;
        }

        true
    }
}

fn main() {
    let data = {
        let mut buf = Vec::new();
        std::io::stdin().read_to_end(&mut buf).unwrap();
        buf
    };

    let mut sequence_state = SequenceState {
        addr: 0,
        delay: 0,

        channels: Default::default(),
    };

    let refresh_interval = Duration::from_secs_f32(1.0 / REFRESH_RATE);
    let mut last_tick = Instant::now();
    loop {
        sequence_state.tick(&data);

        let now = Instant::now();
        let delta = now - last_tick;
        if delta < refresh_interval {
            std::thread::sleep(refresh_interval - delta);
        }
        last_tick = Instant::now();
    }
}
