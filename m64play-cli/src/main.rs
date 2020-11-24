use cpal::traits::*;
use m64play::DecompFiles;

fn list_sequences(files: DecompFiles) {
    println!("playable sequences:");
    for seq in files.sequences() {
        println!("- {}", seq.name);
    }
}

fn play_sequence(files: DecompFiles, seq_name: &str) {
    let mut player = files.new_player(seq_name, 44_100.0);

    let host = cpal::default_host();
    let device = host.default_output_device().unwrap();
    let stream = device.build_output_stream(
        &cpal::StreamConfig {
            channels: 2,
            sample_rate: cpal::SampleRate(44_100),
            buffer_size: cpal::BufferSize::Default,
        },
        move |data: &mut [f32], _| {
            player.fill(data);
        },
        move |err| {
            println!("cpal error: {:?}", err);
        }
    ).unwrap();
    stream.play().unwrap();
    std::thread::sleep(std::time::Duration::from_secs_f32(2.0));
}

fn main() {
    let files = m64play::DecompFiles::load(std::env::var("DECOMP_SOUND_PATH").unwrap().as_ref());
    
    match std::env::args().nth(1) {
        None => list_sequences(files),
        Some(seq_name) => play_sequence(files, &seq_name),
    }
}
