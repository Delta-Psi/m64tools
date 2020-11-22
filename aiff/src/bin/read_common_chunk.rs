use std::io::Read;

fn main() {
    let mut data = Vec::new();
    std::io::stdin().read_to_end(&mut data).unwrap();

    let aiff = aiff::Aiff::read(&data).unwrap();
    println!("{:#?}", aiff.comm);
    println!("audio length: {:?}", aiff.comm.audio_length());
}
