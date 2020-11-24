use crate::sound_bank::SoundBank;
use crate::state::player::SequencePlayer;
use dasp::Signal;

#[derive(Debug)]
pub struct Player {
    sound_banks: Vec<SoundBank>,
    m64_data: Vec<u8>,
    sample_rate: f64,
    state: SequencePlayer,
}

impl Player {
    pub(crate) fn new(
        sound_banks: Vec<SoundBank>,
        m64_data: Vec<u8>,
        sample_rate: f64
    ) -> Self {
        Self {
            sound_banks,
            m64_data,
            sample_rate,
            state: SequencePlayer::new(),
        }
    }

    pub fn fill(&mut self, data: &mut [f32]) {
        for i in 0 .. data.len() / 2 {
            let f = self.next();
            data[i*2] = f[0];
            data[i*2 + 1] = f[1];
        }
    }

    /// Call this 240 times per second.
    pub fn process(&mut self) {
        self.state.process(&self.m64_data);
    }

    pub fn finished(&self) -> bool {
        self.state.finished
    }
}

impl Signal for Player {
    type Frame = [f32; 2];

    fn next(&mut self) -> Self::Frame {
        [0.0, 0.0]
    }
}
