//! xorshift32 — deterministic, tiny, no dependencies. Same seed, same board.

pub struct Rng {
    state: u32,
}

impl Rng {
    pub const fn new() -> Rng {
        Rng {
            state: 0x2545_F491,
        }
    }

    pub fn seed(&mut self, seed: u32) {
        self.state = if seed == 0 { 0x2545_F491 } else { seed };
    }

    pub fn next(&mut self) -> u32 {
        let mut x = self.state;
        x ^= x << 13;
        x ^= x >> 17;
        x ^= x << 5;
        self.state = x;
        x
    }

    /// Uniform in 0.0..1.0
    pub fn unit(&mut self) -> f32 {
        (self.next() >> 8) as f32 / 16_777_216.0
    }
}
