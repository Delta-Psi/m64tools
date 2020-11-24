pub mod sequence;
use sequence::Sequence;

//pub mod sound_bank;
//use sound_bank::SoundBank;

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

    /*
    pub fn load_sound_bank(&self, sound_bank: &str) -> SoundBank {
        SoundBank::load(&self.sound_path, sound_bank)
    }
    */
}
