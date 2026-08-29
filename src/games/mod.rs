//! The game registry.
//!
//! Every game is one static and one `impl Game`. Dispatch is a `&mut dyn Game`
//! handed back from `get()` — vtables cost a few bytes and no allocator, which
//! is what lets several games share one module and switch with no fetch.

pub mod dumptruck;
pub mod snake;

use crate::engine::{Frame, Rng};

pub trait Game {
    /// Deal a fresh board. Called on select and on restart.
    fn reset(&mut self, rng: &mut Rng);

    /// Advance by `dt` milliseconds. Returns true if anything actually moved.
    fn update(&mut self, dt: f32, rng: &mut Rng) -> bool;

    fn draw(&mut self, fb: &mut Frame);

    /// Framebuffer-space finger position and what it just did.
    fn pointer(&mut self, _x: f32, _y: f32, _phase: u32, _rng: &mut Rng) {}

    /// 0 up, 1 right, 2 down, 3 left. Only the keyboard games care.
    fn key(&mut self, _dir: u32) {}

    fn score(&self) -> u32 {
        0
    }
    fn best(&self) -> u32 {
        0
    }
    /// 0 waiting, 1 running, 2 over. Games without a fail state stay at 1.
    fn status(&self) -> u32 {
        1
    }
}

static mut SNAKE: snake::Snake = snake::Snake::new();
static mut DUMPTRUCK: dumptruck::DumpTruck = dumptruck::DumpTruck::new();

pub const COUNT: u32 = 2;

pub const DUMPTRUCK_ID: u32 = 1;

pub fn get(id: u32) -> &'static mut dyn Game {
    unsafe {
        match id {
            DUMPTRUCK_ID => &mut *core::ptr::addr_of_mut!(DUMPTRUCK) as &mut dyn Game,
            _ => &mut *core::ptr::addr_of_mut!(SNAKE) as &mut dyn Game,
        }
    }
}
