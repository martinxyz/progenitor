use super::wall_sim;

#[allow(dead_code)]
static PARAMS: &[u8] = include_bytes!("./optimized.params.bin");

pub fn load() -> Option<wall_sim::Params> {
    // None
    Some(bincode::deserialize(PARAMS).expect("bad optimized.params.bin file"))
}
