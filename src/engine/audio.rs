//! The one thing this module asks of the host.
//!
//! For a child who cannot read, sound is not decoration — it is half the
//! feedback channel. That is worth the module's import-free property, so this
//! is the single import: one function, no audio files, a WebAudio synth on the
//! JavaScript side.

#[link(wasm_import_module = "env")]
extern "C" {
    fn host_sfx(id: u32, param: f32);
}

pub const PICKUP: u32 = 0; // a rock leaves the ground
pub const SEAT: u32 = 1; // a rock settles into the bed
pub const TIP: u32 = 2; // hydraulics, bed rising
pub const LAND: u32 = 3; // a rock hits the ground; param is impact 0..1
pub const HORN: u32 = 4;
pub const ENGINE: u32 = 5; // continuous; param is speed 0..1
pub const REVERSE: u32 = 6; // reversing beeper; param 1 on, 0 off
pub const COUNT: u32 = 7; // the number changed; param is the new count
pub const ROCK_HIT: u32 = 8; // rock landed on rock, not on ground; param impact
pub const TURN: u32 = 9; // the truck is turning around
pub const SET: u32 = 10; // a rock locked into the wall; param is which one
pub const DONE: u32 = 11; // the wall is finished
pub const CRUNCH: u32 = 12; // a rock went into the crusher
pub const MILL: u32 = 13; // continuous; how hard the crusher is working
pub const GEM: u32 = 14; // a gem came out of the chute

/// Fire a sound. Safe wrapper so nothing else in the tree needs `unsafe`.
pub fn sfx(id: u32, param: f32) {
    unsafe { host_sfx(id, param) }
}
