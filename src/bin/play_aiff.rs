use std::io::Read;
use std::sync::mpsc;
use cpal::traits::*;

const SAMPLE_RATE: u32 = 44_100;

fn main() {
    let mut data = Vec::new();
    std::io::stdin().read_to_end(&mut data).unwrap();
    let aiff = aiff::AiffReader {
        ..Default::default()
    }.read(&data).unwrap();

    let (finished_send, finished_recv) = mpsc::sync_channel::<()>(0);

    let host = cpal::default_host();
    let device = host.default_output_device().unwrap();
    let stream = {
        use dasp::signal::Signal;

        let mut samples = dasp::signal::from_iter(
            aiff.samples().normalize_to_f32()
        );
        let interp = dasp::interpolate::linear::Linear::new(
            samples.next(),
            samples.next(),
        );
        let samples: Vec<_> = samples
            .from_hz_to_hz(
                interp,
                aiff.comm.sample_rate as f64,
                SAMPLE_RATE as f64,
            )
            .until_exhausted()
            .collect();
        let mut i = 0;

        device.build_output_stream(
            &cpal::StreamConfig {
                channels: 1,
                sample_rate: cpal::SampleRate(SAMPLE_RATE),
                buffer_size: cpal::BufferSize::Default,
            },
            move |data: &mut [f32], _| {
                for sample in data.iter_mut() {
                    if i >= samples.len() {
                        finished_send.send(()).unwrap();
                        *sample = 0.0;
                    } else {
                        *sample = samples[i];
                        i += 1;
                    }
                }
            },
            move |_err| {
            },
        ).unwrap()
    };
    stream.play().unwrap();

    finished_recv.recv().unwrap();
    std::process::exit(0);
}
