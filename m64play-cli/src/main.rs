use cpal::traits::*;
use m64play::DecompFiles;
use std::time::{Duration, Instant};
use std::sync::{Arc, Mutex};

fn list_sequences(files: DecompFiles) {
    println!("playable sequences:");
    for seq in files.sequences() {
        println!("- {}", seq.name);
    }
}

fn play_sequence(files: DecompFiles, seq_name: &str) {
    let player = Arc::new(Mutex::new(files.new_player(seq_name, 44_100.0)));

    let host = cpal::default_host();
    let device = host.default_output_device().unwrap();
    let stream = {
        let player = player.clone();

        let stream = device.build_output_stream(
            &cpal::StreamConfig {
                channels: 2,
                sample_rate: cpal::SampleRate(44_100),
                buffer_size: cpal::BufferSize::Default,
            },
            move |data: &mut [f32], _| {
                let mut player = player.lock().unwrap();
                player.fill(data);
            },
            move |err| {
                println!("cpal error: {:?}", err);
            }
        ).unwrap();
        stream
    };
    stream.play().unwrap();

    let period = Duration::from_secs_f32(1.0 / 240.0);
    let mut last_update = Instant::now();
    loop {
        {
            let mut player = player.lock().unwrap();
            player.process();
            if player.finished() {
                break;
            }
        }

        let now = Instant::now();
        if let Some(sleep_for) = period.checked_sub(now - last_update) {
            std::thread::sleep(sleep_for);
        }
        last_update = now;
    }
}

fn main() {
    let files = m64play::DecompFiles::load(std::env::var("DECOMP_SOUND_PATH").unwrap().as_ref());
    
    match std::env::args().nth(1) {
        None => list_sequences(files),
        Some(seq_name) => play_sequence(files, &seq_name),
    }
}
