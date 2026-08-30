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

    /// The framebuffer changed shape. The default is to deal again, which is
    /// the only honest answer for a game whose board geometry is derived from
    /// the frame (snake's grid cannot be reinterpreted at a new aspect). A game
    /// holding state the player built should override this and rescale it —
    /// losing his work because he turned the tablet over is not acceptable.
    fn relayout(&mut self, rng: &mut Rng) {
        self.reset(rng);
    }

    /// Framebuffer-space finger position and what it just did.
    fn pointer(&mut self, _x: f32, _y: f32, _phase: u32, _rng: &mut Rng) {}

    /// 0 up, 1 right, 2 down, 3 left. Only the keyboard games care.
    fn key(&mut self, _dir: u32) {}

    /// Turn around. Only games with something that faces a direction care.
    fn flip(&mut self) {}

    /// Whether the page should offer a turn-around button for this game.
    fn can_flip(&self) -> bool {
        false
    }

    /// +1 facing right, -1 facing left, 0 for games with no facing.
    fn facing(&self) -> i32 {
        0
    }

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
