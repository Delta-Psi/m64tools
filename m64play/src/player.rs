use crate::state::player::SequencePlayer;
use dasp::Signal;

#[derive(Debug)]
pub struct Player {
    //pub sound_bank: SoundBank,
    pub m64_data: Vec<u8>,
    pub state: SequencePlayer,
}

impl Player {
    /*pub fn new(sample_rate: f64) -> Self {
        Self {
        }
    }*/

    pub fn fill(&mut self, data: &mut [f32]) {
        for i in 0 .. data.len() / 2 {
            let f = self.next();
            data[i*2] = f[0];
            data[i*2 + 1] = f[1];
        }
    }
}

impl Signal for Player {
    type Frame = [f32; 2];

    fn next(&mut self) -> Self::Frame {
        [self.noise.next() as f32, self.noise.next() as f32]
    }
}
