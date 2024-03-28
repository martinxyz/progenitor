use bincode::{DefaultOptions, Options};

use super::wall_sim;

#[allow(dead_code)]
static PARAMS: &[u8] = include_bytes!("./optimized.params.bin");

pub fn load() -> Option<wall_sim::Params> {
    // None
    Some(
        DefaultOptions::new()
            .with_fixint_encoding() // this is the default when encoding, see: https://docs.rs/bincode/latest/bincode/config/index.html#options-struct-vs-bincode-functions
            .reject_trailing_bytes()
            .deserialize(PARAMS)
            .expect("bad optimized.params.bin file"),
    )
}
