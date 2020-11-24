fn main() {
    let files = m64play::DecompFiles::load(std::env::var("DECOMP_SOUND_PATH").unwrap());
    println!("{:#?}", files);
}
