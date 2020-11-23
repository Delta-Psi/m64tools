use cpal::traits::*;
use m64::state::SequencePlayer;
use std::collections::HashMap;
use std::io::Read;
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};

const TICKS_PER_SECOND: f32 = 240.0;

const FREQUENCIES: [f32; 128] = [
    0.105112, 0.111362, 0.117984, 0.125, 0.132433, 0.140308, 0.148651, 0.15749, 0.166855, 0.176777,
    0.187288, 0.198425, 0.210224, 0.222725, 0.235969, 0.25, 0.264866, 0.280616, 0.297302, 0.31498,
    0.33371, 0.353553, 0.374577, 0.39685, 0.420448, 0.445449, 0.471937, 0.5, 0.529732, 0.561231,
    0.594604, 0.629961, 0.66742, 0.707107, 0.749154, 0.793701, 0.840897, 0.890899, 0.943875, 1.0,
    1.059463, 1.122462, 1.189207, 1.259921, 1.33484, 1.414214, 1.498307, 1.587401, 1.681793,
    1.781798, 1.887749, 2.0, 2.118926, 2.244924, 2.378414, 2.519842, 2.66968, 2.828428, 2.996615,
    3.174803, 3.363586, 3.563596, 3.775498, 4.0, 4.237853, 4.489849, 4.756829, 5.039685, 5.33936,
    5.656855, 5.993229, 6.349606, 6.727173, 7.127192, 7.550996, 8.0, 8.475705, 8.979697, 9.513658,
    10.07937, 10.67872, 11.31371, 11.986459, 12.699211, 13.454346, 14.254383, 15.101993, 16.0,
    16.95141, 17.959394, 19.027315, 20.15874, 21.35744, 22.62742, 23.972918, 25.398422, 26.908691,
    28.508766, 30.203985, 32.0, 33.90282, 35.91879, 38.05463, 40.31748, 42.71488, 45.25484,
    47.945835, 50.796844, 53.817383, 57.017532, 60.40797, 64.0, 67.80564, 71.83758, 76.10926,
    80.63496, 85.42976, 45.25484, 47.945835, 50.796844, 53.817383, 57.017532, 60.40797, 64.0,
    67.80564, 71.83758, 76.10926, 80.63496,
];
fn frequency(pitch: u8) -> f32 {
    256.0 * FREQUENCIES[pitch as usize]
}
const SAMPLE_RATE: u32 = 41_000;

#[derive(Default)]
struct AudioState {
    pub layers: HashMap<(u8, u8), LayerState>,
}

#[derive(Default)]
struct LayerState {
    pub phase: f32,
}

fn main() {
    let player = SequencePlayer::new();
    let player = Arc::new(RwLock::new(player));

    // init audio
    let host = cpal::default_host();
    let device = host.default_output_device().unwrap();
    let stream = {
        let player = player.clone();
        let mut audio_state = AudioState::default();

        device
            .build_output_stream(
                &cpal::StreamConfig {
                    channels: 1,
                    sample_rate: cpal::SampleRate(SAMPLE_RATE),
                    buffer_size: cpal::BufferSize::Default,
                },
                move |data: &mut [f32], _| {
                    for sample in data.iter_mut() {
                        *sample = 0.0;
                    }

                    let player = player.read().unwrap();
                    for (i, channel) in player
                        .channels
                        .iter()
                        .enumerate()
                        .filter_map(|(i, c)| c.as_ref().map(|c| (i, c)))
                    {
                        for (j, layer) in channel
                            .layers
                            .iter()
                            .enumerate()
                            .filter_map(|(j, l)| l.as_ref().map(|l| (j, l)))
                        {
                            if let Some(pitch) = layer.pitch {
                                let freq = frequency(
                                    (player.transposition
                                        + channel.transposition
                                        + layer.transposition
                                        + pitch as i16) as u8,
                                );

                                let layer_state = audio_state
                                    .layers
                                    .entry((i as u8, j as u8))
                                    .or_insert(Default::default());
                                for sample in data.iter_mut() {
                                    *sample +=
                                        0.1 * (layer_state.phase * std::f32::consts::TAU).sin();
                                    layer_state.phase += freq / SAMPLE_RATE as f32;
                                }
                            }
                        }
                    }
                },
                move |_err| {},
            )
            .unwrap()
    };
    stream.play().unwrap();

    let data = {
        let mut buf = Vec::new();
        std::io::stdin().read_to_end(&mut buf).unwrap();
        buf
    };

    let tick_interval = Duration::from_secs_f32(1.0 / TICKS_PER_SECOND);
    let mut last_tick = Instant::now();
    loop {
        {
            let mut player = player.write().unwrap();
            player.process(&data);
            if player.finished {
                break;
            }
        }

        let now = Instant::now();
        let delta = now - last_tick;
        if delta < tick_interval {
            std::thread::sleep(tick_interval - delta);
        }
        last_tick = Instant::now();
    }
}
