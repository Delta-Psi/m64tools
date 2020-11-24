pub mod sequence;
use sequence::Sequence;

pub mod sound_bank;
//use sound_bank::SoundBank;

pub mod state;

use std::path::{PathBuf, Path};

#[derive(Debug)]
pub struct DecompFiles {
    sound_path: PathBuf,
    sequences: Vec<Sequence>,
}

impl DecompFiles {
    pub fn load(sound_path: impl AsRef<Path>) -> Self {
        Self {
            sound_path: sound_path.as_ref().to_owned(),
            sequences: Sequence::load_json(sound_path),
        }
    }
}
