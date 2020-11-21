use std::io::Read;
use std::time::{Instant, Duration};
use m64::state::SequencePlayer;

const TICKS_PER_SECOND: f32 = 240.0;

fn main() {
    let data = {
        let mut buf = Vec::new();
        std::io::stdin().read_to_end(&mut buf).unwrap();
        buf
    };

    let mut player = SequencePlayer::new();

    let tick_interval = Duration::from_secs_f32(1.0 / TICKS_PER_SECOND);
    let mut last_tick = Instant::now();
    loop {
        player.process(&data);
        if player.finished {
            break;
        }

        let now = Instant::now();
        let delta = now - last_tick;
        if delta < tick_interval {
            std::thread::sleep(tick_interval - delta);
        }
        last_tick = Instant::now();
    }
}
