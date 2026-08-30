//! A small arcade cabinet in WebAssembly.
//!
//! Several games share one `no_std` module: one framebuffer, one generator, one
//! set of exports. JavaScript reads the pixels straight out of linear memory and
//! blits them with `putImageData`; switching games is a single call rather than
//! another fetch, which matters when the player is two.
//!
//! Nothing is imported from the host. The entire boundary is the functions
//! below plus one pointer.

#![no_std]

use core::panic::PanicInfo;

mod engine;
mod games;

use engine::{set_size, Frame, Rng};

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

static mut RNG: Rng = Rng::new();
static mut CURRENT: u32 = 0;

fn rng() -> &'static mut Rng {
    // Single-threaded by construction: wasm calls in one at a time.
    unsafe { &mut *core::ptr::addr_of_mut!(RNG) }
}

fn current_id() -> u32 {
    unsafe { core::ptr::read(core::ptr::addr_of!(CURRENT)) }
}

// ------------------------------------------------------------------- exports

/// Seed the generator and deal the first game. Call once, before the loop.
#[no_mangle]
pub extern "C" fn init(seed: u32) {
    rng().seed(seed);
    unsafe { core::ptr::write(core::ptr::addr_of_mut!(CURRENT), 0) };
    let g = games::get(0);
    g.reset(rng());
    g.draw(&mut Frame::new());
}

/// How many games this module carries.
#[no_mangle]
pub extern "C" fn game_count() -> u32 {
    games::COUNT
}

/// Which one is on screen.
#[no_mangle]
pub extern "C" fn current() -> u32 {
    current_id()
}

/// Switch games. Deals the incoming one fresh; no fetch, no reinstantiate.
#[no_mangle]
pub extern "C" fn select(id: u32) {
    let id = if id < games::COUNT { id } else { 0 };
    unsafe { core::ptr::write(core::ptr::addr_of_mut!(CURRENT), id) };
    let g = games::get(id);
    g.reset(rng());
    g.draw(&mut Frame::new());
}

/// Deal the current game again.
#[no_mangle]
pub extern "C" fn restart() {
    let g = games::get(current_id());
    g.reset(rng());
    g.draw(&mut Frame::new());
}

/// 0 up, 1 right, 2 down, 3 left.
#[no_mangle]
pub extern "C" fn turn(dir: u32) {
    games::get(current_id()).key(dir);
}

/// Turn the current game's vehicle around. No-op where nothing faces a way.
#[no_mangle]
pub extern "C" fn flip() {
    games::get(current_id()).flip();
}

/// 1 if this game wants a turn-around button on the page.
#[no_mangle]
pub extern "C" fn can_flip() -> u32 {
    if games::get(current_id()).can_flip() {
        1
    } else {
        0
    }
}

/// Which way the vehicle faces: 1 right, -1 left, 0 if it has no facing.
#[no_mangle]
pub extern "C" fn facing() -> i32 {
    games::get(current_id()).facing()
}

/// Finger position in framebuffer space. Phase: 0 down, 1 move, 2 up.
#[no_mangle]
pub extern "C" fn pointer(x: f32, y: f32, phase: u32) {
    games::get(current_id()).pointer(x, y, phase, rng());
}

/// Advance by `dt` milliseconds and redraw. Returns 1 if anything moved.
#[no_mangle]
pub extern "C" fn tick(dt: f32) -> u32 {
    let g = games::get(current_id());
    let moved = g.update(dt, rng());
    g.draw(&mut Frame::new());
    if moved {
        1
    } else {
        0
    }
}

/// Offset of the RGBA framebuffer inside linear memory.
#[no_mangle]
pub extern "C" fn frame_ptr() -> *const u8 {
    Frame::ptr()
}

#[no_mangle]
pub extern "C" fn frame_w() -> u32 {
    engine::width() as u32
}

#[no_mangle]
pub extern "C" fn frame_h() -> u32 {
    engine::height() as u32
}

/// Resize the framebuffer to match the space the page has for it. Games that
/// can carry their state across a change of shape do so; the rest deal again.
#[no_mangle]
pub extern "C" fn resize(w: u32, h: u32) {
    set_size(w as usize, h as usize);
    let g = games::get(current_id());
    g.relayout(rng());
    g.draw(&mut Frame::new());
}

#[no_mangle]
pub extern "C" fn score() -> u32 {
    games::get(current_id()).score()
}

#[no_mangle]
pub extern "C" fn best() -> u32 {
    games::get(current_id()).best()
}

/// 0 waiting for first input, 1 running, 2 over. Games with no fail state
/// never report 2.
#[no_mangle]
pub extern "C" fn status() -> u32 {
    games::get(current_id()).status()
}
