//! Throw Rocks in the Pond.
//!
//! Seen from above, so the whole pond is on screen at once and every ripple is
//! a true circle. Height is what makes it read as three-dimensional: a rock in
//! the air grows and its shadow slides out from under it, and the two come back
//! together at the moment it lands.
//!
//! There is nothing to complete. Rocks that sink stay on the bottom, so the
//! pond slowly fills with everything he has ever thrown, and fresh pebbles wash
//! up on the bank so he can never run out.

use crate::engine::frame::{ellipse, wave, DIRS8};
use crate::engine::{audio, clamp, fabs, sfx, Frame, Rgb, Rng, DOWN, MOVE, UP};
use crate::games::Game;

const ROCKS: usize = 20;
const RIPPLES: usize = 14;
const LILIES: usize = 7;
const FISH: usize = 5;
const REEDS: usize = 26;
const BOULDERS: usize = 6;
const CRITTERS: usize = 15;

// ------------------------------------------------------------------- colours
//
// A rock pool on a shaded forest floor: dark all round, so the water is the
// brightest thing on the screen and the eye goes straight to it.

const FLOOR: Rgb = (44, 54, 52);
const FLOOR_DARK: Rgb = (34, 43, 42);
const FLOOR_LIT: Rgb = (58, 70, 66);

const MOSS: Rgb = (104, 150, 66);
const MOSS_DARK: Rgb = (74, 116, 50);
const MOSS_LIGHT: Rgb = (146, 186, 88);

const RIM: Rgb = (96, 100, 100);
const RIM_DARK: Rgb = (68, 72, 74);
const RIM_LIGHT: Rgb = (128, 132, 130);

const SHALLOW: Rgb = (128, 164, 176);
const MID: Rgb = (96, 136, 154);
const DEEP: Rgb = (70, 106, 128);
const GLINT: Rgb = (186, 216, 224);
const CAUSTIC: Rgb = (158, 198, 212);
const FOAM: Rgb = (238, 248, 250);

const STONE: Rgb = (196, 186, 164);
const STONE_LIGHT: Rgb = (224, 216, 196);
const STONE_DARK: Rgb = (150, 142, 124);

const LEAF: [Rgb; 3] = [(96, 152, 62), (124, 178, 76), (72, 124, 52)];
const BRANCH: Rgb = (78, 58, 44);

const LILY: Rgb = (74, 140, 78);
const LILY_DARK: Rgb = (54, 112, 62);
const BLOOM: Rgb = (246, 186, 206);
const BLOOM_MID: Rgb = (250, 226, 160);
const REED: Rgb = (96, 146, 70);

const FISH_A: Rgb = (232, 138, 60);
const FISH_B: Rgb = (244, 198, 82);
const SHADOW: Rgb = (30, 44, 48);

const FROG: Rgb = (104, 176, 82);
const FROG_DARK: Rgb = (72, 138, 60);
const FROG_EYE: Rgb = (250, 240, 200);
const TADPOLE: Rgb = (54, 62, 66);
const TURTLE: Rgb = (96, 132, 88);
const TURTLE_SHELL: Rgb = (122, 96, 62);
const TURTLE_SHELL_HI: Rgb = (154, 124, 84);
const NEWT: Rgb = (216, 128, 66);
const NEWT_SPOT: Rgb = (66, 46, 36);
const SNAIL: Rgb = (206, 176, 132);
const SNAIL_SHELL: Rgb = (168, 118, 70);
const WING: Rgb = (198, 226, 234);
const DFLY: Rgb = (86, 158, 196);

/// Bright, nameable pebbles. A grey pond would be a sad pond.
const PEBBLES: [(Rgb, Rgb); 8] = [
    ((214, 88, 72), (238, 138, 122)),   // red
    ((236, 166, 60), (250, 206, 122)),  // amber
    ((92, 164, 206), (150, 208, 236)),  // blue
    ((118, 176, 96), (166, 212, 146)),  // green
    ((166, 128, 200), (206, 178, 232)), // violet
    ((236, 226, 208), (252, 246, 236)), // chalk
    ((122, 128, 136), (168, 174, 182)), // slate
    ((238, 132, 168), (252, 186, 208)), // pink
];

// -------------------------------------------------------------------- state

/// Everything living in the pool. Each one wants a different thing, moves at a
/// different speed, and answers a touch in its own way — which is the whole
/// point of putting them here.
#[derive(Clone, Copy, PartialEq)]
enum Critter {
    Frog,      // sits on a pad, croaks, hops when startled
    Dragonfly, // hovers over the water, darts off if touched
    Tadpole,   // wiggles about in the shallows in little groups
    Turtle,    // paddles slowly, pulls in when touched
    Newt,      // crawls around the rim
    Snail,     // barely moves at all, and that is the joke
}

#[derive(Clone, Copy, PartialEq)]
enum Pebble {
    Bank,   // sitting on the grass, waiting to be picked up
    Held,   // in his hand
    Flying, // in the air, with a shadow beneath it
    Sunk,   // on the bottom, seen through the water, there for good
    Landed, // came down on the bank again
}

pub struct Pond {
    x: [f32; ROCKS],
    y: [f32; ROCKS],
    z: [f32; ROCKS], // height above the surface
    vx: [f32; ROCKS],
    vy: [f32; ROCKS],
    vz: [f32; ROCKS],
    kind: [u8; ROCKS],
    shape: [u32; ROCKS],
    size: [f32; ROCKS],
    state: [Pebble; ROCKS],
    sunk_at: [f32; ROCKS],

    held: i32,
    grab_dx: f32,
    grab_dy: f32,
    // the last two pointer samples, so a release can throw with the flick
    px: f32,
    py: f32,
    fling_x: f32,
    fling_y: f32,

    ripples: [(f32, f32, f32); RIPPLES], // x, y, age
    ripple_i: usize,
    lilies: [(f32, f32, f32); LILIES],   // x, y, phase
    fish: [(f32, f32, f32); FISH],       // x, y, heading (index into DIRS8, as f32)
    reeds: [(f32, f32, f32); REEDS],     // x, y, height
    boulders: [(f32, f32, f32); BOULDERS], // x, y, scale

    // the living things
    ckind: [Critter; CRITTERS],
    cx: [f32; CRITTERS],
    cy: [f32; CRITTERS],
    cvx: [f32; CRITTERS],
    cvy: [f32; CRITTERS],
    cphase: [f32; CRITTERS],
    ctimer: [f32; CRITTERS], // counts down a reaction: a hop, a dart, a hide
    chome: [f32; CRITTERS],  // the pad a frog belongs to, as an index

    t: f32,
    busy: f32, // how much the water is moving, for the ambient sound
    chirp_t: f32,
    thrown: u32,
    accum: f32,
    moved: bool,
    ready: bool,
}

impl Pond {
    pub const fn new() -> Pond {
        Pond {
            x: [0.0; ROCKS],
            y: [0.0; ROCKS],
            z: [0.0; ROCKS],
            vx: [0.0; ROCKS],
            vy: [0.0; ROCKS],
            vz: [0.0; ROCKS],
            kind: [0; ROCKS],
            shape: [0; ROCKS],
            size: [1.0; ROCKS],
            state: [Pebble::Bank; ROCKS],
            sunk_at: [0.0; ROCKS],
            held: -1,
            grab_dx: 0.0,
            grab_dy: 0.0,
            px: 0.0,
            py: 0.0,
            fling_x: 0.0,
            fling_y: 0.0,
            ripples: [(0.0, 0.0, 9.0); RIPPLES],
            ripple_i: 0,
            lilies: [(0.0, 0.0, 0.0); LILIES],
            fish: [(0.0, 0.0, 0.0); FISH],
            reeds: [(0.0, 0.0, 0.0); REEDS],
            boulders: [(0.0, 0.0, 0.0); BOULDERS],
            ckind: [Critter::Tadpole; CRITTERS],
            cx: [0.0; CRITTERS],
            cy: [0.0; CRITTERS],
            cvx: [0.0; CRITTERS],
            cvy: [0.0; CRITTERS],
            cphase: [0.0; CRITTERS],
            ctimer: [0.0; CRITTERS],
            chome: [0.0; CRITTERS],
            t: 0.0,
            busy: 0.0,
            chirp_t: 0.0,
            thrown: 0,
            accum: 0.0,
            moved: false,
            ready: false,
        }
    }

    // ------------------------------------------------------------ the pond

    /// The mossy stone rim around the pool.
    fn rim(&self, w: f32, h: f32) -> (f32, f32, f32, f32) {
        let u = if w < h { w } else { h };
        (w * 0.5, h * 0.5, w * 0.46, h * 0.46 - u * 0.02)
    }

    /// The water. Seen from almost straight above, tilted just enough that it
    /// is an ellipse rather than a circle.
    fn pool(&self, w: f32, h: f32) -> (f32, f32, f32, f32) {
        let (cx, cy, rx, ry) = self.rim(w, h);
        (cx, cy, rx * 0.88, ry * 0.86)
    }

    /// How far inside the water a point is: 1 at the middle, 0 at the edge,
    /// negative on the bank. Cheap, and it is the only test the game needs.
    fn depth(&self, w: f32, h: f32, x: f32, y: f32) -> f32 {
        let (cx, cy, rx, ry) = self.pool(w, h);
        let dx = (x - cx) / rx;
        let dy = (y - cy) / ry;
        1.0 - (dx * dx + dy * dy)
    }

    fn rock_r(&self, w: f32, h: f32, i: usize) -> f32 {
        let u = if w < h { w } else { h };
        u * 0.030 * self.size[i]
    }

    fn ripple(&mut self, x: f32, y: f32) {
        self.ripples[self.ripple_i] = (x, y, 0.0);
        self.ripple_i = (self.ripple_i + 1) % RIPPLES;
    }

    /// Put a pebble back on the bank, somewhere near the bottom edge where his
    /// hand already is.
    fn place_on_bank(&mut self, w: f32, h: f32, i: usize, rng: &mut Rng) {
        let u = if w < h { w } else { h };
        loop {
            let x = u * 0.06 + rng.unit() * (w - u * 0.12);
            let y = u * 0.06 + rng.unit() * (h - u * 0.12);
            if self.depth(w, h, x, y) < -0.06 {
                self.x[i] = x;
                self.y[i] = y;
                break;
            }
        }
        self.z[i] = 0.0;
        self.vx[i] = 0.0;
        self.vy[i] = 0.0;
        self.vz[i] = 0.0;
        self.state[i] = Pebble::Bank;
    }

    fn on_bank_count(&self) -> usize {
        (0..ROCKS)
            .filter(|&i| matches!(self.state[i], Pebble::Bank | Pebble::Landed))
            .count()
    }

    fn deal(&mut self, w: f32, h: f32, rng: &mut Rng) {
        for i in 0..ROCKS {
            self.kind[i] = (rng.next() % PEBBLES.len() as u32) as u8;
            self.shape[i] = rng.next();
            self.size[i] = 0.72 + rng.unit() * 0.7;
            self.sunk_at[i] = 0.0;
            self.place_on_bank(w, h, i, rng);
        }
        let (cx, cy, rx, ry) = self.pool(w, h);
        for k in 0..LILIES {
            let a = k as f32 / LILIES as f32;
            self.lilies[k] = (
                cx + wave(a) * rx * 0.62,
                cy + wave(a + 0.27) * ry * 0.62,
                rng.unit(),
            );
        }
        for k in 0..FISH {
            let a = (k as f32 + 0.5) / FISH as f32;
            self.fish[k] = (
                cx + wave(a + 0.13) * rx * 0.5,
                cy + wave(a + 0.61) * ry * 0.5,
                (rng.next() % 8) as f32,
            );
        }
        // Boulders sitting in the shallows and on the rim, the way they do in
        // a real pool: mostly round the edge, a couple out in the middle.
        for k in 0..BOULDERS {
            let a = (k as f32 + 0.35) / BOULDERS as f32;
            let reach = if k % 3 == 0 { 0.42 } else { 0.86 };
            self.boulders[k] = (
                cx + wave(a) * rx * reach,
                cy + wave(a + 0.31) * ry * reach,
                0.7 + rng.unit() * 0.8
            );
        }
        for k in 0..REEDS {
            let a = k as f32 / REEDS as f32;
            let jitter = 0.94 + rng.unit() * 0.14;
            self.reeds[k] = (
                cx + wave(a) * rx * jitter,
                cy + wave(a + 0.25) * ry * jitter,
                0.5 + rng.unit(),
            );
        }
        // Who lives here. Frogs get a lily pad each; everything else finds its
        // own place in or around the water.
        for k in 0..CRITTERS {
            let kind = match k % 6 {
                0 => Critter::Frog,
                1 => Critter::Dragonfly,
                2 => Critter::Tadpole,
                3 => Critter::Turtle,
                4 => Critter::Newt,
                _ => Critter::Snail
            };
            self.ckind[k] = kind;
            self.cphase[k] = rng.unit();
            self.ctimer[k] = 0.0;
            self.cvx[k] = 0.0;
            self.cvy[k] = 0.0;
            let a = (k as f32 + 0.5) / CRITTERS as f32;
            match kind {
                Critter::Frog => {
                    let pad = (rng.next() as usize) % LILIES;
                    self.chome[k] = pad as f32;
                    self.cx[k] = self.lilies[pad].0;
                    self.cy[k] = self.lilies[pad].1;
                }
                Critter::Newt | Critter::Snail => {
                    // out on the rim, where the stones are
                    self.cx[k] = cx + wave(a + 0.11) * rx * 1.06;
                    self.cy[k] = cy + wave(a + 0.42) * ry * 1.06;
                }
                _ => {
                    self.cx[k] = cx + wave(a + 0.19) * rx * 0.62;
                    self.cy[k] = cy + wave(a + 0.53) * ry * 0.62;
                }
            }
        }

        self.held = -1;
        self.ready = true;
    }

    /// Anything living within `reach` of a point, nearest first.
    fn critter_near(&self, x: f32, y: f32, reach: f32) -> Option<usize> {
        let mut best = reach * reach;
        let mut pick = None;
        for k in 0..CRITTERS {
            let dx = x - self.cx[k];
            let dy = y - self.cy[k];
            let d2 = dx * dx + dy * dy;
            if d2 < best {
                best = d2;
                pick = Some(k);
            }
        }
        pick
    }

    /// Startle one: a frog hops, a dragonfly bolts, a turtle pulls in.
    fn startle(&mut self, k: usize, w: f32, h: f32, away_x: f32, away_y: f32, rng: &mut Rng) {
        let u = if w < h { w } else { h };
        let mut dx = self.cx[k] - away_x;
        let mut dy = self.cy[k] - away_y;
        let d = fabs(dx) + fabs(dy);
        if d < 0.001 {
            dx = 1.0;
            dy = 0.0;
        } else {
            dx /= d;
            dy /= d;
        }
        match self.ckind[k] {
            Critter::Frog => {
                let pad = (rng.next() as usize) % LILIES;
                self.chome[k] = pad as f32;
                self.ctimer[k] = 0.55;          // the hop takes this long
                sfx(audio::CROAK, 0.4 + rng.unit() * 0.6);
            }
            Critter::Dragonfly => {
                self.cvx[k] = dx * u * 0.9;
                self.cvy[k] = dy * u * 0.9;
                self.ctimer[k] = 0.9;
                sfx(audio::BUZZ, 1.0);
            }
            Critter::Turtle => {
                self.ctimer[k] = 1.6;           // pulled in, not going anywhere
                sfx(audio::PLIP, 0.3);
            }
            Critter::Tadpole => {
                self.cvx[k] = dx * u * 0.35;
                self.cvy[k] = dy * u * 0.35;
                self.ctimer[k] = 0.5;
                sfx(audio::PLIP, 0.8);
            }
            Critter::Newt => {
                self.cvx[k] = dx * u * 0.30;
                self.cvy[k] = dy * u * 0.30;
                self.ctimer[k] = 0.7;
                sfx(audio::PLIP, 0.5);
            }
            Critter::Snail => {
                self.ctimer[k] = 2.2;           // withdraws, extremely slowly
                sfx(audio::PLIP, 0.15);
            }
        }
    }

    // ---------------------------------------------------------------- step

    fn step(&mut self, w: f32, h: f32, s: f32, rng: &mut Rng) {
        let u = if w < h { w } else { h };
        self.t += s;
        self.busy = if self.busy > 0.0 { self.busy - s * 0.5 } else { 0.0 };

        for k in 0..RIPPLES {
            if self.ripples[k].2 < 3.0 {
                self.ripples[k].2 += s;
                self.moved = true;
            }
        }

        for i in 0..ROCKS {
            match self.state[i] {
                Pebble::Flying => {
                    self.vz[i] -= u * 3.2 * s; // gravity, in the vertical
                    self.z[i] += self.vz[i] * s;
                    self.x[i] += self.vx[i] * s;
                    self.y[i] += self.vy[i] * s;
                    // air drag, so a hard throw slows as it goes
                    self.vx[i] *= 1.0 - 0.5 * s;
                    self.vy[i] *= 1.0 - 0.5 * s;

                    if self.z[i] <= 0.0 {
                        self.z[i] = 0.0;
                        let d = self.depth(w, h, self.x[i], self.y[i]);
                        let speed = fabs(self.vz[i]) / (u * 1.6);
                        let hit = if speed > 1.0 { 1.0 } else { speed };
                        if d > 0.0 {
                            self.state[i] = Pebble::Sunk;
                            self.sunk_at[i] = self.t;
                            self.ripple(self.x[i], self.y[i]);
                            self.busy = 1.0;
                            self.thrown += 1;
                            sfx(audio::PLOP, self.size[i] * (0.4 + hit));
                            // everything nearby scatters
                            let (sx2, sy2) = (self.x[i], self.y[i]);
                            for k in 0..CRITTERS {
                                let dx = self.cx[k] - sx2;
                                let dy = self.cy[k] - sy2;
                                if fabs(dx) + fabs(dy) < u * 0.26 {
                                    self.startle(k, w, h, sx2, sy2, rng);
                                }
                            }
                            for k in 0..FISH {
                                let fx = self.fish[k].0 - self.x[i];
                                let fy = self.fish[k].1 - self.y[i];
                                if fabs(fx) + fabs(fy) < u * 0.30 {
                                    self.fish[k].2 = if fx < 0.0 { 4.0 } else { 0.0 };
                                }
                            }
                        } else {
                            self.state[i] = Pebble::Landed;
                            self.thrown += 1;
                            sfx(audio::THUD, self.size[i] * (0.4 + hit));
                        }
                    }
                    self.moved = true;
                }
                Pebble::Sunk => {
                    // it settles, then drifts a hair with the water
                    self.x[i] += wave(self.t * 0.09 + i as f32 * 0.13) * u * 0.004 * s;
                }
                _ => {}
            }
            // never let one leave the picture
            self.x[i] = clamp(self.x[i], u * 0.03, w - u * 0.03);
            self.y[i] = clamp(self.y[i], u * 0.03, h - u * 0.03);
        }

        // fresh pebbles wash up, so the bank is never empty
        if self.on_bank_count() < 5 {
            let mut oldest = -1i32;
            let mut when = self.t + 1.0;
            for i in 0..ROCKS {
                if self.state[i] == Pebble::Sunk && self.sunk_at[i] < when {
                    when = self.sunk_at[i];
                    oldest = i as i32;
                }
            }
            if oldest >= 0 && self.t - when > 4.0 {
                let i = oldest as usize;
                self.place_on_bank(w, h, i, rng);
                self.kind[i] = (rng.next() % PEBBLES.len() as u32) as u8;
                self.moved = true;
            }
        }

        // fish wander, and turn away from the bank
        for k in 0..FISH {
            let dir = DIRS8[(self.fish[k].2 as usize) % 8];
            let sp = u * 0.055;
            let nx = self.fish[k].0 + dir.0 * sp * s;
            let ny = self.fish[k].1 + dir.1 * sp * s;
            if self.depth(w, h, nx, ny) > 0.10 {
                self.fish[k].0 = nx;
                self.fish[k].1 = ny;
                if rng.unit() < 0.4 * s {
                    self.fish[k].2 = (rng.next() % 8) as f32;
                }
            } else {
                self.fish[k].2 = ((self.fish[k].2 as usize + 4) % 8) as f32;
            }
        }

        // ---------------------------------------------------------- the living
        let (pcx, pcy, _prx, _pry) = self.pool(w, h);
        for k in 0..CRITTERS {
            self.cphase[k] += s;
            if self.ctimer[k] > 0.0 {
                self.ctimer[k] -= s;
            }
            match self.ckind[k] {
                Critter::Frog => {
                    // rides its pad, and hops to a new one when startled
                    let pad = (self.chome[k] as usize) % LILIES;
                    let (lx, ly, ph) = self.lilies[pad];
                    let bob = wave(self.t * 0.28 + ph) * u * 0.005;
                    let speed = if self.ctimer[k] > 0.0 { 9.0 } else { 3.0 };
                    self.cx[k] += (lx - self.cx[k]) * speed * s;
                    self.cy[k] += (ly + bob - self.cy[k]) * speed * s;
                    // an unprompted croak now and then
                    if rng.unit() < 0.06 * s {
                        sfx(audio::CROAK, 0.3 + rng.unit() * 0.5);
                    }
                }
                Critter::Dragonfly => {
                    if self.ctimer[k] > 0.0 {
                        self.cx[k] += self.cvx[k] * s;
                        self.cy[k] += self.cvy[k] * s;
                        self.cvx[k] *= 1.0 - 2.0 * s;
                        self.cvy[k] *= 1.0 - 2.0 * s;
                    } else {
                        // hovers, with a small nervous wander
                        self.cx[k] += wave(self.cphase[k] * 0.7) * u * 0.05 * s;
                        self.cy[k] += wave(self.cphase[k] * 0.9 + 0.3) * u * 0.04 * s;
                    }
                    if self.depth(w, h, self.cx[k], self.cy[k]) < 0.05 {
                        self.cx[k] += (pcx - self.cx[k]) * 0.6 * s;
                        self.cy[k] += (pcy - self.cy[k]) * 0.6 * s;
                    }
                }
                Critter::Tadpole => {
                    let wig = wave(self.cphase[k] * 2.6 + k as f32) * u * 0.10;
                    self.cx[k] += (self.cvx[k] + wig * 0.4) * s;
                    self.cy[k] += (self.cvy[k] + wig * 0.2) * s;
                    self.cvx[k] *= 1.0 - 1.4 * s;
                    self.cvy[k] *= 1.0 - 1.4 * s;
                    // stays in the shallows, near the edge
                    let d = self.depth(w, h, self.cx[k], self.cy[k]);
                    if d < 0.06 || d > 0.75 {
                        self.cx[k] += (pcx - self.cx[k]) * 0.5 * s;
                        self.cy[k] += (pcy - self.cy[k]) * 0.5 * s;
                    }
                }
                Critter::Turtle => {
                    if self.ctimer[k] <= 0.0 {
                        let dir = DIRS8[(k * 3 + (self.cphase[k] * 0.15) as usize) % 8];
                        self.cx[k] += dir.0 * u * 0.030 * s;
                        self.cy[k] += dir.1 * u * 0.030 * s;
                        if self.depth(w, h, self.cx[k], self.cy[k]) < 0.10 {
                            self.cx[k] += (pcx - self.cx[k]) * 0.7 * s;
                            self.cy[k] += (pcy - self.cy[k]) * 0.7 * s;
                        }
                    }
                }
                Critter::Newt => {
                    self.cx[k] += self.cvx[k] * s;
                    self.cy[k] += self.cvy[k] * s;
                    self.cvx[k] *= 1.0 - 1.8 * s;
                    self.cvy[k] *= 1.0 - 1.8 * s;
                    // keeps to the rim: neither in the water nor off the edge
                    let d = self.depth(w, h, self.cx[k], self.cy[k]);
                    if d > -0.02 {
                        self.cx[k] += (self.cx[k] - pcx) * 0.8 * s;
                        self.cy[k] += (self.cy[k] - pcy) * 0.8 * s;
                    }
                    if rng.unit() < 0.5 * s {
                        let dir = DIRS8[(rng.next() % 8) as usize];
                        self.cvx[k] = dir.0 * u * 0.06;
                        self.cvy[k] = dir.1 * u * 0.06;
                    }
                }
                Critter::Snail => {
                    self.cx[k] += wave(self.cphase[k] * 0.05) * u * 0.004 * s;
                }
            }
            self.cx[k] = clamp(self.cx[k], u * 0.02, w - u * 0.02);
            self.cy[k] = clamp(self.cy[k], u * 0.02, h - u * 0.02);
        }

        // something calling from the trees, now and then
        self.chirp_t -= s;
        if self.chirp_t <= 0.0 {
            self.chirp_t = 4.0 + rng.unit() * 9.0;
            sfx(audio::CHIRP, rng.unit());
        }

        sfx(audio::LAP, self.busy);
    }
}

impl Game for Pond {
    fn reset(&mut self, rng: &mut Rng) {
        let (w, h) = (crate::engine::width() as f32, crate::engine::height() as f32);
        self.t = 0.0;
        self.busy = 0.0;
        self.thrown = 0;
        self.accum = 0.0;
        self.ripples = [(0.0, 0.0, 9.0); RIPPLES];
        self.deal(w, h, rng);
    }

    fn update(&mut self, dt: f32, rng: &mut Rng) -> bool {
        let (w, h) = (crate::engine::width() as f32, crate::engine::height() as f32);
        if !self.ready {
            self.deal(w, h, rng);
        }
        self.moved = false;
        self.accum += if dt > 100.0 { 100.0 } else { dt };
        while self.accum >= 16.0 {
            self.accum -= 16.0;
            self.step(w, h, 0.016, rng);
        }
        self.moved
    }

    fn score(&self) -> u32 {
        self.thrown
    }

    fn pointer(&mut self, x: f32, y: f32, phase: u32, rng: &mut Rng) {
        let (w, h) = (crate::engine::width() as f32, crate::engine::height() as f32);
        let u = if w < h { w } else { h };

        match phase {
            DOWN => {
                if self.held >= 0 {
                    return; // one hand at a time
                }
                let mut pick = -1i32;
                let mut best = (u * 0.14) * (u * 0.14); // a generous reach
                for i in 0..ROCKS {
                    if !matches!(self.state[i], Pebble::Bank | Pebble::Landed) {
                        continue;
                    }
                    let dx = x - self.x[i];
                    let dy = y - self.y[i];
                    let d2 = dx * dx + dy * dy;
                    if d2 < best {
                        best = d2;
                        pick = i as i32;
                    }
                }
                // A creature only answers a touch that was not already a reach
                // for a stone. Throwing is the main verb; the frogs are what he
                // finds when he stops throwing.
                if pick < 0 {
                    if let Some(k) = self.critter_near(x, y, u * 0.075) {
                        self.startle(k, w, h, x, y, rng);
                        self.moved = true;
                        return;
                    }
                }

                if pick >= 0 {
                    let i = pick as usize;
                    self.state[i] = Pebble::Held;
                    self.held = pick;
                    self.grab_dx = self.x[i] - x;
                    self.grab_dy = self.y[i] - y;
                    self.z[i] = u * 0.05; // lifted, so it casts a shadow at once
                }
                self.px = x;
                self.py = y;
                self.fling_x = 0.0;
                self.fling_y = 0.0;
                self.moved = true;
            }
            MOVE => {
                // Remember the flick. Smoothed, or one jittery sample decides
                // the whole throw.
                self.fling_x += ((x - self.px) * 26.0 - self.fling_x) * 0.35;
                self.fling_y += ((y - self.py) * 26.0 - self.fling_y) * 0.35;
                self.px = x;
                self.py = y;
                if self.held >= 0 {
                    let i = self.held as usize;
                    self.x[i] = clamp(x + self.grab_dx, 0.0, w);
                    self.y[i] = clamp(y + self.grab_dy, 0.0, h);
                }
                self.moved = true;
            }
            UP => {
                if self.held >= 0 {
                    let i = self.held as usize;
                    self.state[i] = Pebble::Flying;
                    self.vx[i] = self.fling_x;
                    self.vy[i] = self.fling_y;
                    // Always some loft, and more the harder it is thrown, so a
                    // gentle push still arcs instead of scuffing along.
                    let speed = (fabs(self.fling_x) + fabs(self.fling_y)) / (u * 2.0);
                    let lift = if speed > 1.0 { 1.0 } else { speed };
                    self.vz[i] = u * (0.55 + lift * 0.75);
                    self.held = -1;
                }
                self.moved = true;
            }
            _ => {}
        }
    }

    fn draw(&mut self, fb: &mut Frame) {
        let (w, h) = (crate::engine::width() as f32, crate::engine::height() as f32);
        let (wi, hi) = (w as i32, h as i32);
        let u = if w < h { w } else { h };
        let (rcx, rcy, rrx, rry) = self.rim(w, h);
        let (cx, cy, rx, ry) = self.pool(w, h);

        // --------------------------------------------------- the forest floor
        fb.fill(FLOOR);
        let mut g = 0;
        while g < wi {
            let k = (g * 37) % hi;
            fb.rect(g, k, (u * 0.05) as i32, 3, FLOOR_DARK);
            fb.rect(g + 17, (k * 3) % hi, (u * 0.03) as i32, 2, FLOOR_LIT);
            g += 23;
        }

        // ------------------------------------------------- the mossy stone rim
        // Stones first, then moss over the top of them, which is the order they
        // ended up in.
        fb.fill_ellipse(rcx as i32, rcy as i32, (rrx + u * 0.03) as i32,
                        (rry + u * 0.03) as i32, RIM_DARK);
        fb.fill_ellipse(rcx as i32, rcy as i32, rrx as i32, rry as i32, RIM);
        for k in 0..34 {
            let a = k as f32 / 34.0;
            let sx = rcx + wave(a) * rrx * 0.99;
            let sy = rcy + wave(a + 0.25) * rry * 0.99;
            let sr = (u * (0.020 + (k % 3) as f32 * 0.006)) as i32;
            fb.rock(sx as i32, sy as i32, sr, 0x51A3_7C0D ^ (k as u32 * 2654435761), RIM_LIGHT);
            // moss tufts, sitting on the stones
            if k % 2 == 0 {
                let c = if k % 4 == 0 { MOSS } else { MOSS_DARK };
                fb.rock(sx as i32, sy as i32 - sr / 2, (sr as f32 * 0.8) as i32,
                        0x2C7B_91E5 ^ (k as u32 * 40503), c);
                fb.rect(sx as i32 - sr / 3, sy as i32 - sr, 3, sr, MOSS_LIGHT);
            }
        }

        // ------------------------------------------------------------- water
        // Bands from the rim inward, so it deepens toward the middle.
        let bands = 16;
        for b in 0..bands {
            let k = b as f32 / bands as f32;
            let wob = wave(self.t * 0.16 + k * 1.7) * u * 0.004;
            let t255 = (k * 255.0) as u32;
            let c = if k < 0.4 {
                let q = (k / 0.4 * 255.0) as u32;
                (crate::engine::lerp(SHALLOW.0, MID.0, q),
                 crate::engine::lerp(SHALLOW.1, MID.1, q),
                 crate::engine::lerp(SHALLOW.2, MID.2, q))
            } else {
                let q = ((k - 0.4) / 0.6 * 255.0) as u32;
                (crate::engine::lerp(MID.0, DEEP.0, q),
                 crate::engine::lerp(MID.1, DEEP.1, q),
                 crate::engine::lerp(MID.2, DEEP.2, q))
            };
            let _ = t255;
            let er = rx * (1.0 - k) + wob;
            let ey = ry * (1.0 - k) + wob;
            if er > 1.0 && ey > 1.0 {
                fb.fill_ellipse(cx as i32, cy as i32, er as i32, ey as i32, c);
            }
        }

        // Standing ripples: fine concentric lines that breathe. The pool is
        // never still, even before he has thrown anything into it.
        for k in 0..7 {
            let a = k as f32 / 7.0;
            let breathe = 0.30 + a * 0.66 + wave(self.t * 0.13 + a * 0.8) * 0.02;
            let ox = wave(self.t * 0.07 + a) * u * 0.02;
            let oy = wave(self.t * 0.05 + a + 0.3) * u * 0.015;
            ellipse(fb, (cx + ox) as i32, (cy + oy) as i32,
                    (rx * breathe) as i32, (ry * breathe) as i32, 1, CAUSTIC);
        }
        for k in 0..11 {
            let a = k as f32 * 0.09;
            let gx = cx + wave(self.t * 0.09 + a) * rx * 0.7;
            let gy = cy + wave(self.t * 0.07 + a + 0.4) * ry * 0.7;
            fb.rect(gx as i32, gy as i32, (u * 0.03) as i32, 2, GLINT);
        }

        // ------------------------------------------------ stones in the water
        for k in 0..BOULDERS {
            let (bx, by, sc) = self.boulders[k];
            let br = (u * 0.045 * sc) as i32;
            fb.rock(bx as i32, by as i32 + br / 3, br, 0x77C1_2A5B ^ (k as u32 * 22695477), SHADOW);
            fb.rock(bx as i32, by as i32, br, 0x77C1_2A5B ^ (k as u32 * 22695477), STONE);
            fb.rock(bx as i32 - br / 4, by as i32 - br / 4, br / 2,
                    0x11D3_4E77 ^ (k as u32 * 69069), STONE_LIGHT);
        }

        // ------------------------------------------------------ sunk pebbles
        for i in 0..ROCKS {
            if self.state[i] != Pebble::Sunk {
                continue;
            }
            let r = self.rock_r(w, h, i) * 0.92;
            let (body, _) = PEBBLES[self.kind[i] as usize % PEBBLES.len()];
            let sunkc = (
                crate::engine::lerp(body.0, DEEP.0, 120),
                crate::engine::lerp(body.1, DEEP.1, 120),
                crate::engine::lerp(body.2, DEEP.2, 120),
            );
            fb.rock(self.x[i] as i32, self.y[i] as i32, r as i32, self.shape[i], sunkc);
        }

        // ------------------------------------------------------ his ripples
        for k in 0..RIPPLES {
            let (px, py, age) = self.ripples[k];
            if age >= 3.0 {
                continue;
            }
            let grow = age / 3.0;
            let r = u * (0.02 + grow * 0.32);
            let th = ((1.0 - grow) * u * 0.010) as i32 + 1;
            let fade = ((1.0 - grow) * 255.0) as u32;
            let c = (
                crate::engine::lerp(MID.0, FOAM.0, fade),
                crate::engine::lerp(MID.1, FOAM.1, fade),
                crate::engine::lerp(MID.2, FOAM.2, fade),
            );
            ellipse(fb, px as i32, py as i32, r as i32, (r * 0.94) as i32, th, c);
            if grow < 0.25 {
                // a burst of foam right at the impact
                for f in 0..6 {
                    let d = DIRS8[f];
                    let fr = r * (0.5 + grow);
                    fb.rect((px + d.0 * fr) as i32, (py + d.1 * fr * 0.94) as i32, 3, 3, FOAM);
                }
            }
        }

        // ------------------------------------------------------------- fish
        for k in 0..FISH {
            let (fx, fy, dirk) = self.fish[k];
            let d = DIRS8[(dirk as usize) % 8];
            let fr = (u * 0.014) as i32;
            let c = if k % 2 == 0 { FISH_A } else { FISH_B };
            fb.disc(fx as i32, fy as i32, fr, c);
            fb.rect((fx - d.0 * u * 0.026) as i32, (fy - d.1 * u * 0.026) as i32, fr, fr, c);
        }

        // ------------------------------------------------------------ lilies
        for k in 0..LILIES {
            let (lx, ly, ph) = self.lilies[k];
            let bob = wave(self.t * 0.28 + ph) * u * 0.005;
            let r = (u * 0.040) as i32;
            fb.disc(lx as i32, (ly + bob) as i32, r, LILY);
            fb.rect(lx as i32, (ly + bob) as i32 - 1, r, 3, LILY_DARK);
            if k % 3 == 0 {
                fb.disc(lx as i32, (ly + bob) as i32 - r / 3, (u * 0.014) as i32, BLOOM);
                fb.disc(lx as i32, (ly + bob) as i32 - r / 3, (u * 0.006) as i32, BLOOM_MID);
            }
        }

        // --------------------------------------------------------- the living
        for k in 0..CRITTERS {
            let (x, y) = (self.cx[k] as i32, self.cy[k] as i32);
            let hidden = self.ctimer[k] > 0.0;
            match self.ckind[k] {
                Critter::Frog => {
                    let r = (u * 0.026) as i32;
                    // mid-hop it is off the water, so it gets a shadow
                    let hop = if hidden { wave(self.ctimer[k] * 0.9) * u * 0.05 } else { 0.0 };
                    if hop > 0.5 {
                        fb.disc(x, y, r, SHADOW);
                    }
                    let fy = y - hop as i32;
                    fb.disc(x, fy, r, FROG);
                    fb.disc(x - r / 2, fy - r / 2, r / 2, FROG_DARK);
                    fb.disc(x + r / 2, fy - r / 2, r / 2, FROG_DARK);
                    fb.disc(x - r / 2, fy - r / 2, r / 3, FROG_EYE);
                    fb.disc(x + r / 2, fy - r / 2, r / 3, FROG_EYE);
                    fb.rect(x - r / 3, fy + r / 3, r * 2 / 3, 2, FROG_DARK);
                }
                Critter::Dragonfly => {
                    let r = (u * 0.012) as i32;
                    let beat = wave(self.cphase[k] * 14.0);
                    let ww = (u * 0.030 * (0.55 + 0.45 * fabs(beat))) as i32;
                    fb.rect(x - ww, y - 2, ww * 2, 2, WING);
                    fb.rect(x - ww / 2, y + 1, ww, 2, WING);
                    fb.disc(x, y, r, DFLY);
                    fb.rect(x, y, (u * 0.030) as i32, 2, DFLY);
                }
                Critter::Tadpole => {
                    let r = (u * 0.009) as i32 + 1;
                    let wig = wave(self.cphase[k] * 3.0) * u * 0.012;
                    fb.disc(x, y, r, TADPOLE);
                    fb.rect(x - (u * 0.018) as i32, y + wig as i32, (u * 0.018) as i32, 2, TADPOLE);
                }
                Critter::Turtle => {
                    let r = (u * 0.028) as i32;
                    let out = if hidden { 0 } else { r / 2 };
                    fb.disc(x, y, r, TURTLE_SHELL);
                    fb.disc(x - r / 3, y - r / 3, r / 2, TURTLE_SHELL_HI);
                    if out > 0 {
                        fb.disc(x + r, y, r / 2, TURTLE);          // head
                        fb.rect(x - r, y + r / 2, r / 2, r / 3, TURTLE);
                        fb.rect(x + r / 2, y + r / 2, r / 2, r / 3, TURTLE);
                    }
                }
                Critter::Newt => {
                    let r = (u * 0.011) as i32 + 1;
                    let wig = wave(self.cphase[k] * 2.2) * u * 0.008;
                    fb.disc(x, y, r, NEWT);
                    fb.rect(x - (u * 0.026) as i32, y + wig as i32, (u * 0.026) as i32, 3, NEWT);
                    fb.rect(x - r, y - r, 2, 2, NEWT_SPOT);
                }
                Critter::Snail => {
                    let r = (u * 0.013) as i32 + 1;
                    fb.rect(x - r, y + r / 2, r * 2, 3, SNAIL);
                    fb.disc(x, y, r, SNAIL_SHELL);
                    fb.disc(x, y, r / 2, SNAIL);
                    if !hidden {
                        fb.rect(x + r, y - r, 2, r, SNAIL);        // eye stalks
                        fb.rect(x + r + 3, y - r, 2, r, SNAIL);
                    }
                }
            }
        }

        // ------------------------------------------------------------- reeds
        for k in 0..REEDS {
            let (rx0, ry0, tall) = self.reeds[k];
            let sway = wave(self.t * 0.35 + k as f32 * 0.21) * u * 0.012;
            let hgt = (u * 0.07 * tall) as i32;
            fb.rect(rx0 as i32, ry0 as i32 - hgt, 3, hgt, REED);
            fb.rect((rx0 + sway) as i32, ry0 as i32 - hgt - 5, 3, 7, MOSS_LIGHT);
        }

        // --------------------------------------------- pebbles waiting to go
        for i in 0..ROCKS {
            if !matches!(self.state[i], Pebble::Bank | Pebble::Landed) {
                continue;
            }
            let r = self.rock_r(w, h, i);
            let (body, light) = PEBBLES[self.kind[i] as usize % PEBBLES.len()];
            fb.rock(self.x[i] as i32, self.y[i] as i32 + 2, r as i32, self.shape[i], SHADOW);
            fb.rock(self.x[i] as i32, self.y[i] as i32, r as i32, self.shape[i], body);
            fb.rect(self.x[i] as i32 - (r * 0.25) as i32, self.y[i] as i32 - (r * 0.45) as i32,
                    (r * 0.5) as i32, (r * 0.3) as i32, light);
        }

        // -------------------------------------------- in the air, and in hand
        for i in 0..ROCKS {
            if !matches!(self.state[i], Pebble::Flying | Pebble::Held) {
                continue;
            }
            let r = self.rock_r(w, h, i);
            let (body, light) = PEBBLES[self.kind[i] as usize % PEBBLES.len()];
            let lift = self.z[i];
            // Height is the whole third dimension: the shadow stays down on the
            // water, the stone rises off it and swells.
            let grow = 1.0 + lift / (u * 0.55);
            let shrink = 1.0 - lift / (u * 1.6);
            let sr = (r * if shrink < 0.35 { 0.35 } else { shrink }) as i32;
            fb.rock(self.x[i] as i32, self.y[i] as i32, if sr < 2 { 2 } else { sr },
                    self.shape[i], SHADOW);
            let ry2 = self.y[i] - lift;
            fb.rock(self.x[i] as i32, ry2 as i32, (r * grow) as i32, self.shape[i], body);
            fb.rect(self.x[i] as i32 - (r * grow * 0.25) as i32,
                    ry2 as i32 - (r * grow * 0.45) as i32,
                    (r * grow * 0.5) as i32, (r * grow * 0.3) as i32, light);
        }

        // ------------------------------------------- foliage over the corners
        // Leaves hanging into frame, so the pool feels like somewhere shaded
        // rather than a shape floating on a background.
        for corner in 0..2 {
            let bx = if corner == 0 { 0.0 } else { w };
            let dir = if corner == 0 { 1.0 } else { -1.0 };
            fb.rect(bx as i32 - if corner == 1 { (u * 0.34) as i32 } else { 0 },
                    (u * 0.05) as i32, (u * 0.34) as i32, 4, BRANCH);
            for k in 0..7 {
                let a = k as f32 / 7.0;
                let lx = bx + dir * (u * 0.03 + a * u * 0.30);
                let ly = u * (0.03 + wave(a * 0.5 + corner as f32 * 0.3) * 0.06 + a * 0.05);
                let lr = (u * (0.045 - a * 0.02)) as i32;
                let sway = wave(self.t * 0.18 + a + corner as f32) * u * 0.008;
                fb.rock((lx + sway) as i32, ly as i32, lr,
                        0x3F5A_11C7 ^ (k as u32 * 2246822519), LEAF[k % 3]);
            }
        }
    }
}
