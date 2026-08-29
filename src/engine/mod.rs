//! Shared machinery every game draws on: the framebuffer, the generator, and
//! the pointer phases the host reports.

pub mod frame;
pub mod rng;

pub use frame::{clamp, fabs, height, lerp, set_size, width, Frame, Rgb};
pub use rng::Rng;

/// What the host is telling us the finger just did.
pub const DOWN: u32 = 0;
pub const MOVE: u32 = 1;
pub const UP: u32 = 2;
