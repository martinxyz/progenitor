// Trait for something the user can step forward/backward (e.g. in the web UI).
pub trait Simulation {
    fn step(&mut self);

    // useful when called via trait object (allows loop unrolling)
    fn steps(&mut self, count: usize) {
        for _ in 0..count {
            self.step()
        }
    }

    /// Save the simulation's state. Does not need to include the simulation's
    /// parameters, only enough information to undo a `.step()`.
    fn save_state(&self) -> Vec<u8>;
    fn load_state(&mut self, data: &[u8]);
}
