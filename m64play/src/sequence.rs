use std::path::Path;

#[derive(Debug)]
pub struct Sequence {
    pub name: String,
    pub sound_banks: Vec<String>,
}

impl Sequence {
    pub fn load_json(base_path: impl AsRef<Path>) -> Vec<Self> {
        use serde_json::Value;

        let path = base_path.as_ref().join("sequences.json");
        let json = std::fs::read_to_string(path).unwrap();

        let v: Value = serde_json::from_str(&json).unwrap();
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

    pub fn read_m64(&self, base_path: &Path) -> Vec<u8> {
        let path = base_path.join(format!("sequences/us/{}.m64", self.name));
        std::fs::read(path).unwrap()
    }
}

