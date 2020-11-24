pub mod sequence;
use sequence::Sequence;

pub mod sound_bank;
use sound_bank::SoundBank;

pub mod state;

mod player;
pub use player::Player;

use std::path::{PathBuf, Path};

#[derive(Debug)]
pub struct DecompFiles {
    sound_path: PathBuf,
    sequences: Vec<Sequence>,
}

impl DecompFiles {
    pub fn load(sound_path: &Path) -> Self {
        Self {
            sound_path: sound_path.to_owned(),
            sequences: Sequence::load_json(sound_path),
        }
    }

    pub fn new_player(&self, seq_name: &str, sample_rate: f64) -> Player {
        // find the sequence by name
        let sequence = self.sequences.iter()
            .find(|seq| seq.name == seq_name)
            .unwrap();

        let sound_banks: Vec<_> = sequence.sound_banks.iter()
            .map(|sb| SoundBank::load(&self.sound_path, sb))
            .collect();

        let m64_path = self.sound_path.join(format!("sequences/us/{}.m64", seq_name));
        let m64_data = std::fs::read(m64_path).unwrap();

        Player::new(sound_banks, m64_data, sample_rate)
    }

    pub fn sound_path(&self) -> &Path {
        &self.sound_path
    }
    pub fn sequences(&self) -> &[Sequence] {
        &self.sequences
    }
}
