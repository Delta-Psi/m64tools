use std::path::PathBuf;

#[derive(Debug)]
pub struct Sequence {
    pub name: String,
    pub sound_banks: Vec<String>,
}

impl Sequence {
    pub fn read_sequences_json(json: &str) -> Vec<Self> {
        use serde_json::Value;

        let v: Value = serde_json::from_str(json).unwrap();
        let sequence_map = v.as_object().unwrap();

        sequence_map.iter().filter_map(|(k, v)| {
            if k == "comment" {
                return None;
            }

            let sound_banks = match v {
                Value::Array(sound_banks) => sound_banks,
                Value::Object(obj) => {
                    if obj["ifdef"].as_array().unwrap().contains(&Value::String("VERSION_US".to_owned())) {
                        obj["banks"].as_array().unwrap()
                    } else {
                        return None;
                    }
                },
                _ => panic!(),
            }.iter().map(|bank| bank.as_str().unwrap().to_owned()).collect();

            Some(Sequence {
                name: k.to_owned(),
                sound_banks,
            })
        }).collect()
    }
}

fn main() {
    let decomp_sound_path: PathBuf = std::env::var("DECOMP_SOUND_PATH").unwrap().into();
    let sequences_path = decomp_sound_path.join("sequences.json");
    let sequences_json = std::fs::read_to_string(sequences_path).unwrap();
    let sequences = Sequence::read_sequences_json(&sequences_json);
    println!("{:#?}", sequences);
}
