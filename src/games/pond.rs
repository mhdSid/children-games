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

use crate::engine::frame::{arc_dashes, ellipse, hash, wave, wave_q, DIRS8};
use crate::engine::{audio, clamp, fabs, sfx, Frame, Rgb, Rng, DOWN, MOVE, UP};
use crate::games::Game;

const ROCKS: usize = 20;
const RIPPLES: usize = 14;
const LILIES: usize = 7;
const FISH: usize = 5;
const REEDS: usize = 26;
const BOULDERS: usize = 14;
const RIM_STONES: usize = 38;
const PROPS: usize = 54;
const CRITTERS: usize = 24;

// ------------------------------------------------------------------- colours
//
// A rock pool on a shaded forest floor: dark all round, so the water is the
// brightest thing on the screen and the eye goes straight to it.

// A terrarium floor, not a void: damp earth with moss growing over it, and
// things living in it. The dark ring round the pool used to be nothing at all,
// which is half the picture wasted.
const FLOOR: Rgb = (68, 78, 58);
const FLOOR_DARK: Rgb = (54, 62, 46);
const FLOOR_LIT: Rgb = (86, 98, 70);
const SOIL: Rgb = (76, 60, 46);
const SOIL_DARK: Rgb = (58, 46, 36);
const MOSS_PATCH: Rgb = (92, 138, 62);
const MOSS_PATCH_HI: Rgb = (124, 168, 82);
const FERN: Rgb = (78, 134, 66);
const FERN_HI: Rgb = (108, 164, 88);
const CAP: Rgb = (206, 82, 64);
const CAP_SPOT: Rgb = (244, 238, 226);
const STEM: Rgb = (232, 224, 206);
const PETAL: Rgb = (244, 226, 246);
const PETAL_GOLD: Rgb = (250, 226, 132);
const LITTER: Rgb = (152, 112, 62);
const LITTER_DARK: Rgb = (118, 86, 50);
const TWIG: Rgb = (96, 74, 52);

const MOSS: Rgb = (104, 150, 66);
const MOSS_DARK: Rgb = (74, 116, 50);
const MOSS_LIGHT: Rgb = (146, 186, 88);

const RIM: Rgb = (96, 100, 100);
const RIM_DARK: Rgb = (68, 72, 74);
const RIM_LIGHT: Rgb = (128, 132, 130);

// Shallow water is bright because you can see the bottom through it; deep
// water is dark because you cannot. Getting that the wrong way round — a lit
// centre and shaded edges — makes a pond look like a painted disc.
const SHOAL: Rgb = (170, 196, 190);   // sand showing through at the very edge
const SHALLOW: Rgb = (128, 170, 178);
const MID: Rgb = (78, 122, 146);
const DEEP: Rgb = (42, 76, 104);
const ABYSS: Rgb = (26, 52, 76);      // the middle, where the bottom is gone
const GLINT: Rgb = (186, 216, 224);
const CAUSTIC: Rgb = (158, 198, 212);
const FOAM: Rgb = (238, 248, 250);
const PICKABLE: Rgb = (250, 246, 224);

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

/// The light the pool is seen in. Applied to the finished frame, so a theme
/// costs one line rather than a second palette for every object in the scene.
/// Multipliers per channel, then a lift.
const THEMES: [(f32, f32, f32, i32, &str); 5] = [
    (1.00, 1.00, 1.00, 0, "fern"),      // daylight under the trees
    (1.12, 0.96, 0.78, 6, "amber"),     // late afternoon, warm and low
    (0.62, 0.72, 1.05, -6, "moonlit"),  // cold, blue, quiet
    (1.10, 0.88, 0.70, 2, "autumn"),    // rust and gold
    (0.84, 1.04, 0.94, 10, "misty")     // pale, washed out, early
];

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
const BEETLE: Rgb = (62, 76, 62);
const LADYBIRD: Rgb = (206, 62, 52);
const WING_A: Rgb = (246, 186, 92);
const WING_B: Rgb = (238, 148, 176);
const WORM: Rgb = (216, 148, 148);
const DFLY: Rgb = (86, 158, 196);

/// Pebbles a river would actually produce: warm and cool stones, each clearly
/// a different one, none of them shouting. Saturated toys sat on top of this
/// scene rather than in it — the outline is what makes them findable, not the
/// colour.
const PEBBLES: [(Rgb, Rgb); 8] = [
    ((196, 148, 112), (226, 186, 152)), // sandstone
    ((208, 178, 130), (238, 214, 174)), // warm tan
    ((146, 158, 166), (186, 198, 204)), // river grey
    ((160, 168, 142), (198, 206, 180)), // olive
    ((186, 144, 138), (220, 184, 178)), // dusty rose
    ((226, 218, 200), (246, 240, 228)), // chalk
    ((118, 126, 134), (158, 166, 174)), // slate
    ((172, 152, 176), (208, 192, 210)), // lilac grey
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
    Beetle,    // trundles over the floor
    Ladybird,  // wanders, and is very red
    Butterfly, // loops about above the moss
    Worm       // half in the soil, going nowhere in particular
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
    cheld: i32, // the creature currently cupped in his hand, or -1
    cvoice: f32,
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
    props: [(f32, f32, u8); PROPS],      // x, y, what it is
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

    theme: usize,
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
            cheld: -1,
            cvoice: 0.0,
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
            props: [(0.0, 0.0, 0); PROPS],
            boulders: [(0.0, 0.0, 0.0); BOULDERS],
            ckind: [Critter::Tadpole; CRITTERS],
            cx: [0.0; CRITTERS],
            cy: [0.0; CRITTERS],
            cvx: [0.0; CRITTERS],
            cvy: [0.0; CRITTERS],
            cphase: [0.0; CRITTERS],
            ctimer: [0.0; CRITTERS],
            chome: [0.0; CRITTERS],
            theme: 0,
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

    /// Big. A stone has to be a comfortable target for a two-year-old's
    /// fingertip, which is far larger than a stone would realistically be —
    /// so the stones are unrealistic and the game works.
    fn rock_r(&self, w: f32, h: f32, i: usize) -> f32 {
        let u = if w < h { w } else { h };
        u * 0.055 * self.size[i]
    }

    fn ripple(&mut self, x: f32, y: f32) {
        self.ripples[self.ripple_i] = (x, y, 0.0);
        self.ripple_i = (self.ripple_i + 1) % RIPPLES;
    }

    /// Put a pebble back on the bank, somewhere near the bottom edge where his
    /// hand already is.
    fn place_on_bank(&mut self, w: f32, h: f32, i: usize, rng: &mut Rng) {
        let u = if w < h { w } else { h };
        // On the bank ring just outside the water, where the stones would
        // really be — not scattered out over the dark floor.
        loop {
            let x = u * 0.04 + rng.unit() * (w - u * 0.08);
            let y = u * 0.04 + rng.unit() * (h - u * 0.08);
            let d = self.depth(w, h, x, y);
            if d < -0.02 && d > -0.55 {
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
            self.size[i] = 0.80 + rng.unit() * 0.45;
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
            // scattered right across the pool, not just round the edge
            let hh = crate::engine::frame::hash(k as u32 * 31 + 7);
            let reach = 0.20 + ((hh & 255) as f32 / 255.0) * 0.72;
            let ang = a + ((hh >> 8) & 255) as f32 / 255.0 * 0.06;
            self.boulders[k] = (
                cx + wave_q(ang) * rx * reach,
                cy + wave(ang) * ry * reach,
                0.55 + ((hh >> 16) & 255) as f32 / 255.0 * 0.85
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
        // Everything growing on the floor around the pool. Placed only where
        // there is no water, and never on top of the rim.
        for k in 0..PROPS {
            let hh = crate::engine::frame::hash(k as u32 * 97 + 13);
            let mut px = w * 0.5;
            let mut py = h * 0.06;
            for tries in 0..40 {
                let a = crate::engine::frame::hash(hh ^ (tries * 7919));
                let b = crate::engine::frame::hash(hh ^ (tries * 104729) ^ 0x5A);
                let qx = (a & 4095) as f32 / 4095.0 * w;
                let qy = (b & 4095) as f32 / 4095.0 * h;
                // dry ground only: outside the water AND clear of the stone rim
                if self.depth(w, h, qx, qy) < -0.38 {
                    px = qx;
                    py = qy;
                    break;
                }
            }
            self.props[k] = (px, py, (hh % 6) as u8);
        }

        // Who lives here. Frogs get a lily pad each; everything else finds its
        // own place in or around the water.
        for k in 0..CRITTERS {
            let kind = match k % 10 {
                0 => Critter::Frog,
                1 => Critter::Dragonfly,
                2 => Critter::Tadpole,
                3 => Critter::Turtle,
                4 => Critter::Newt,
                5 => Critter::Snail,
                6 => Critter::Beetle,
                7 => Critter::Ladybird,
                8 => Critter::Butterfly,
                _ => Critter::Worm
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
                    self.cx[k] = cx + wave_q(a + 0.11) * rx * 1.06;
                    self.cy[k] = cy + wave(a + 0.42) * ry * 1.06;
                }
                Critter::Beetle | Critter::Ladybird | Critter::Butterfly | Critter::Worm => {
                    // out on the floor, in among the moss and the leaves
                    let mut placed = false;
                    for tries in 0..40 {
                        let hh = crate::engine::frame::hash(k as u32 * 331 + tries);
                        let qx = (hh & 4095) as f32 / 4095.0 * w;
                        let qy = ((hh >> 12) & 4095) as f32 / 4095.0 * h;
                        if self.depth(w, h, qx, qy) < -0.40 {
                            self.cx[k] = qx;
                            self.cy[k] = qy;
                            placed = true;
                            break;
                        }
                    }
                    if !placed {
                        self.cx[k] = w * 0.06;
                        self.cy[k] = h * 0.10;
                    }
                }
                _ => {
                    self.cx[k] = cx + wave(a + 0.19) * rx * 0.62;
                    self.cy[k] = cy + wave(a + 0.53) * ry * 0.62;
                }
            }
        }

        self.held = -1;
        self.cheld = -1;
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
            Critter::Beetle | Critter::Ladybird => {
                self.cvx[k] = dx * u * 0.34;
                self.cvy[k] = dy * u * 0.34;
                self.ctimer[k] = 0.8;
                sfx(audio::CLICK, 0.5 + rng.unit() * 0.5);
            }
            Critter::Butterfly => {
                self.cvx[k] = dx * u * 0.55;
                self.cvy[k] = dy * u * 0.55 - u * 0.2;
                self.ctimer[k] = 1.1;
                sfx(audio::FLUTTER, 1.0);
            }
            Critter::Worm => {
                self.ctimer[k] = 1.8;           // pulls down into the soil
                sfx(audio::PLIP, 0.2);
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
                Critter::Beetle | Critter::Ladybird => {
                    self.cx[k] += self.cvx[k] * s;
                    self.cy[k] += self.cvy[k] * s;
                    self.cvx[k] *= 1.0 - 2.2 * s;
                    self.cvy[k] *= 1.0 - 2.2 * s;
                    if rng.unit() < 0.7 * s {
                        let dir = DIRS8[(rng.next() % 8) as usize];
                        self.cvx[k] = dir.0 * u * 0.05;
                        self.cvy[k] = dir.1 * u * 0.05;
                    }
                    // never walks into the pool
                    if self.depth(w, h, self.cx[k], self.cy[k]) > -0.34 {
                        self.cvx[k] = (self.cx[k] - pcx) * 0.5;
                        self.cvy[k] = (self.cy[k] - pcy) * 0.5;
                    }
                    if rng.unit() < 0.10 * s {
                        sfx(audio::CLICK, 0.2 + rng.unit() * 0.3);
                    }
                }
                Critter::Butterfly => {
                    // loops, rather than travels
                    let loop_x = wave(self.cphase[k] * 0.5) * u * 0.10;
                    let loop_y = wave(self.cphase[k] * 0.5 + 0.25) * u * 0.07;
                    self.cx[k] += (self.cvx[k] + loop_x) * s;
                    self.cy[k] += (self.cvy[k] + loop_y) * s;
                    self.cvx[k] *= 1.0 - 1.6 * s;
                    self.cvy[k] *= 1.0 - 1.6 * s;
                    if self.depth(w, h, self.cx[k], self.cy[k]) > -0.30 {
                        self.cvx[k] += (self.cx[k] - pcx) * 0.4 * s;
                        self.cvy[k] += (self.cy[k] - pcy) * 0.4 * s;
                    }
                }
                Critter::Worm => {
                    self.cx[k] += wave(self.cphase[k] * 0.5) * u * 0.010 * s;
                }
            }
            self.cx[k] = clamp(self.cx[k], u * 0.02, w - u * 0.02);
            self.cy[k] = clamp(self.cy[k], u * 0.02, h - u * 0.02);
        }

        // whatever is in his hand keeps wriggling, and keeps talking
        if self.cheld >= 0 {
            let k = self.cheld as usize;
            self.ctimer[k] = 0.4;                       // stays in its startled pose
            self.cvoice -= s;
            if self.cvoice <= 0.0 {
                self.cvoice = 0.55 + rng.unit() * 0.7;
                match self.ckind[k] {
                    Critter::Frog => sfx(audio::CROAK, 0.3 + rng.unit() * 0.6),
                    Critter::Dragonfly | Critter::Butterfly => sfx(audio::FLUTTER, 0.8),
                    Critter::Beetle | Critter::Ladybird => sfx(audio::CLICK, 0.6),
                    _ => sfx(audio::PLIP, 0.4)
                }
            }
            self.moved = true;
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

    fn next_theme(&mut self) {
        self.theme = (self.theme + 1) % THEMES.len();
        sfx(audio::CHIRP, 0.5);
    }

    fn theme_count(&self) -> u32 {
        THEMES.len() as u32
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
                    // Sunk ones too: he should be able to reach in and take
                    // back anything he has already thrown.
                    if !matches!(
                        self.state[i],
                        Pebble::Bank | Pebble::Landed | Pebble::Sunk
                    ) {
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
                    if let Some(k) = self.critter_near(x, y, u * 0.085) {
                        // Cupped in the hand rather than merely poked. Holding
                        // a small animal gently is most of the appeal of a
                        // terrarium, and it costs nothing to allow.
                        self.cheld = k as i32;
                        self.cvoice = 0.0;
                        self.grab_dx = self.cx[k] - x;
                        self.grab_dy = self.cy[k] - y;
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
                    // lifted at once, so it already casts a shadow in his hand
                    self.z[i] = u * 0.05;
                    sfx(audio::PLIP, 0.5);
                } else {
                    // Nothing under the finger — but a touch always answers.
                    // A boulder knocks, a lily bobs, open water rings.
                    let mut hit = false;
                    for k in 0..BOULDERS {
                        let (bx, by, sc) = self.boulders[k];
                        let br = u * 0.055 * sc;
                        if fabs(x - bx) + fabs(y - by) < br * 1.6 {
                            sfx(audio::THUD, 0.5 + sc * 0.3);
                            self.boulders[k].2 = sc; // knocked, but immovable
                            hit = true;
                            break;
                        }
                    }
                    if !hit {
                        for k in 0..LILIES {
                            let (lx, ly, _) = self.lilies[k];
                            if fabs(x - lx) + fabs(y - ly) < u * 0.07 {
                                self.lilies[k].2 += 0.35; // sets it bobbing
                                self.ripple(lx, ly);
                                sfx(audio::PLIP, 0.35);
                                hit = true;
                                break;
                            }
                        }
                    }
                    if !hit {
                        if self.depth(w, h, x, y) > 0.0 {
                            self.ripple(x, y);
                            self.busy = 0.5;
                            sfx(audio::PLIP, 0.6);
                        } else {
                            sfx(audio::THUD, 0.25);
                        }
                    }
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
                if self.cheld >= 0 {
                    let k = self.cheld as usize;
                    self.cx[k] = clamp(x + self.grab_dx, 0.0, w);
                    self.cy[k] = clamp(y + self.grab_dy, 0.0, h);
                }
                self.moved = true;
            }
            UP => {
                if self.cheld >= 0 {
                    // Set down wherever he lets go, and off it goes. A frog
                    // put back in the water gets a ripple for its trouble.
                    let k = self.cheld as usize;
                    self.cheld = -1;
                    self.ctimer[k] = 0.6;
                    if self.depth(w, h, self.cx[k], self.cy[k]) > 0.0 {
                        self.ripple(self.cx[k], self.cy[k]);
                        sfx(audio::PLIP, 0.7);
                    } else {
                        sfx(audio::CLICK, 0.3);
                    }
                    self.moved = true;
                    return;
                }
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

        // ---------------------------------------------------- what grows here
        for k in 0..PROPS {
            let (px, py, kind) = self.props[k];
            let hh = hash(k as u32 * 53 + 3);
            let sc = 0.7 + (hh & 63) as f32 / 63.0 * 0.7;
            let (x, y) = (px as i32, py as i32);
            let sway = wave(self.t * 0.25 + k as f32 * 0.17) * u * 0.006;
            match kind {
                0 => {
                    // a patch of moss, low and spreading
                    let r = (u * 0.055 * sc) as i32;
                    fb.rock(x, y, r, hh, MOSS_PATCH);
                    fb.rock(x - r / 4, y - r / 4, r / 2, hh ^ 0x2B1, MOSS_PATCH_HI);
                }
                1 => {
                    // a fern: a stem with fronds stepping down it
                    let tall = (u * 0.10 * sc) as i32;
                    fb.rect(x, y - tall, 3, tall, FERN);
                    for f in 0..5 {
                        let fy = y - tall + f * tall / 5;
                        let fw = (u * 0.035 * sc * (1.0 - f as f32 * 0.14)) as i32;
                        let c = if f % 2 == 0 { FERN } else { FERN_HI };
                        fb.rect(x - fw + (sway * 0.5) as i32, fy, fw, 3, c);
                        fb.rect(x + (sway * 0.5) as i32, fy + 2, fw, 3, c);
                    }
                }
                2 => {
                    // a toadstool
                    let r = (u * 0.030 * sc) as i32;
                    fb.rect(x - r / 3, y - r, (r * 2 / 3).max(2), r, STEM);
                    fb.fill_ellipse(x, y - r, r, r * 2 / 3, CAP);
                    fb.disc(x - r / 3, y - r - r / 5, r / 5, CAP_SPOT);
                    fb.disc(x + r / 2, y - r, r / 6, CAP_SPOT);
                }
                3 => {
                    // small flowers
                    let r = (u * 0.012 * sc) as i32 + 2;
                    let tall = (u * 0.045 * sc) as i32;
                    fb.rect(x, y - tall, 2, tall, FERN);
                    let c = if hh % 2 == 0 { PETAL } else { PETAL_GOLD };
                    for pdir in 0..4 {
                        let d = DIRS8[pdir * 2];
                        fb.disc(x + (d.0 * r as f32 * 1.2) as i32,
                                y - tall + (d.1 * r as f32 * 1.2) as i32, r, c);
                    }
                    fb.disc(x, y - tall, r, PETAL_GOLD);
                }
                4 => {
                    // a fallen leaf
                    let r = (u * 0.028 * sc) as i32;
                    let c = if hh % 3 == 0 { LITTER } else { LITTER_DARK };
                    fb.fill_ellipse(x, y, r, r / 2, c);
                    fb.rect(x - r, y, r * 2, 1, SOIL_DARK);
                }
                _ => {
                    // a twig, and the soil it lies on
                    let len = (u * 0.06 * sc) as i32;
                    fb.fill_ellipse(x, y, len, (u * 0.014) as i32, SOIL);
                    fb.rect(x - len / 2, y, len, 3, TWIG);
                    fb.rect(x + len / 6, y - 4, len / 3, 3, TWIG);
                }
            }
        }

        // ------------------------------------------------- the mossy stone rim
        // The rim IS boulders. Drawn dense and overlapping so they merge into a
        // bank, rather than a grey band with a few pebbles sitting on it.
        fb.fill_ellipse(rcx as i32, rcy as i32, (rrx + u * 0.05) as i32,
                        (rry + u * 0.05) as i32, RIM_DARK);
        for k in 0..RIM_STONES {
            let t = k as f32 / RIM_STONES as f32;
            let hh = hash(k as u32 * 7 + 11);
            let jitter = 0.97 + ((hh & 63) as f32 / 63.0) * 0.09;
            let sx = rcx + rrx * jitter * wave_q(t);
            let sy = rcy + rry * jitter * wave(t);
            let sr = u * (0.036 + ((hh >> 6) & 31) as f32 / 31.0 * 0.030);
            // a shadow under it, the stone, then a lit cap on top
            fb.rock(sx as i32, sy as i32 + (sr * 0.3) as i32, sr as i32, hh, RIM_DARK);
            fb.rock(sx as i32, sy as i32, sr as i32, hh, RIM);
            fb.rock(sx as i32 - (sr * 0.15) as i32, sy as i32 - (sr * 0.28) as i32,
                    (sr * 0.66) as i32, hh ^ 0x9E37, RIM_LIGHT);
            // moss, always on the upper side
            if hh % 5 != 0 {
                let mc = if hh % 2 == 0 { MOSS } else { MOSS_DARK };
                fb.rock(sx as i32 + (sr * 0.1) as i32, sy as i32 - (sr * 0.45) as i32,
                        (sr * 0.70) as i32, hh ^ 0x51AB, mc);
                fb.rock(sx as i32 - (sr * 0.35) as i32, sy as i32 - (sr * 0.55) as i32,
                        (sr * 0.42) as i32, hh ^ 0x22C7, MOSS_LIGHT);
                // blades sprouting out of it
                for blade in 0..3 {
                    let bx2 = sx + (blade as f32 - 1.0) * sr * 0.45;
                    let bh2 = sr * (0.45 + ((hh >> blade) & 7) as f32 / 7.0 * 0.5);
                    fb.rect(bx2 as i32, (sy - sr * 0.5 - bh2) as i32, 3, bh2 as i32, MOSS_LIGHT);
                }
            }
        }

        // ------------------------------------------------------------- water
        // Bright at the rim where the bottom shows through, falling away fast
        // to something you cannot see into. The steepness of that fall is what
        // reads as depth: a gentle gradient looks like a shallow puddle.
        let bands = 30;
        for b in 0..bands {
            let k = b as f32 / bands as f32;
            let wob = wave(self.t * 0.16 + k * 1.7) * u * 0.004;
            // most of the darkening happens in the first third, the way a
            // bank shelves off
            let d = k * k * (3.0 - 2.0 * k);           // smoothstep
            let c = if d < 0.35 {
                let q = (d / 0.35 * 255.0) as u32;
                (crate::engine::lerp(SHOAL.0, SHALLOW.0, q),
                 crate::engine::lerp(SHOAL.1, SHALLOW.1, q),
                 crate::engine::lerp(SHOAL.2, SHALLOW.2, q))
            } else if d < 0.72 {
                let q = ((d - 0.35) / 0.37 * 255.0) as u32;
                (crate::engine::lerp(SHALLOW.0, MID.0, q),
                 crate::engine::lerp(SHALLOW.1, MID.1, q),
                 crate::engine::lerp(SHALLOW.2, MID.2, q))
            } else {
                let q = ((d - 0.72) / 0.28 * 255.0) as u32;
                (crate::engine::lerp(MID.0, ABYSS.0, q),
                 crate::engine::lerp(MID.1, ABYSS.1, q),
                 crate::engine::lerp(MID.2, ABYSS.2, q))
            };
            let er = rx * (1.0 - k * 0.98) + wob;
            let ey = ry * (1.0 - k * 0.98) + wob;
            if er > 1.0 && ey > 1.0 {
                fb.fill_ellipse(cx as i32, cy as i32, er as i32, ey as i32, c);
            }
        }

        // Grit on the bottom, visible only in the shallows — the clearest
        // possible statement that there IS a bottom, and that it drops away.
        for k in 0..40 {
            let hh = hash(k as u32 * 41 + 17);
            let t = (hh & 1023) as f32 / 1023.0;
            let rr = 0.80 + ((hh >> 10) & 63) as f32 / 63.0 * 0.17;
            let gx = cx + rx * rr * wave_q(t);
            let gy = cy + ry * rr * wave(t);
            fb.rock(gx as i32, gy as i32, (u * 0.010) as i32 + 1, hh, SHOAL);
        }

        // The standing swell: many broken arcs at scattered radii, drifting.
        // This is what the water actually looks like, and one tidy set of
        // concentric circles is what it does not.
        for k in 0..26 {
            let hh = hash(k as u32 * 13 + 3);
            let base = 0.16 + ((hh & 255) as f32 / 255.0) * 0.86;
            let drift = wave(self.t * 0.05 + k as f32 * 0.11) * 0.03;
            let ox = wave(self.t * 0.04 + k as f32 * 0.21) * u * 0.035;
            let oy = wave(self.t * 0.03 + k as f32 * 0.17) * u * 0.025;
            let r = base + drift;
            if r <= 0.03 {
                continue;
            }
            let shade = if hh % 5 == 0 { GLINT } else { CAUSTIC };
            arc_dashes(fb, cx + ox, cy + oy, rx * r, ry * r,
                       hh, if hh % 7 == 0 { 2 } else { 1 }, shade);
        }

        // flecks of foam, gathered toward the near edge
        for k in 0..26 {
            let hh = hash(k as u32 * 29 + 101);
            let t = (hh & 1023) as f32 / 1023.0;
            let rr = 0.80 + ((hh >> 10) & 63) as f32 / 63.0 * 0.18;
            let fx = cx + rx * rr * wave_q(t);
            let fy = cy + ry * rr * wave(t);
            if fy > cy {
                fb.rect(fx as i32, fy as i32, 3 + (hh % 3) as i32, 2, FOAM);
            }
        }

        // ------------------------------------------------ stones in the water
        for k in 0..BOULDERS {
            let (bx, by, sc) = self.boulders[k];
            let hh = hash(k as u32 * 17 + 5);
            let br = (u * 0.058 * sc) as i32;
            if br < 3 {
                continue;
            }
            // A stone in water is three things: the part under the surface, the
            // ring where it breaks through, and the lit part above. Draw only
            // the last and it looks pasted on top of the pond.
            fb.fill_ellipse(bx as i32, by as i32 + br / 3, (br as f32 * 1.15) as i32,
                            br / 2, DEEP);
            arc_dashes(fb, bx, by + br as f32 * 0.30, br as f32 * 1.25,
                       br as f32 * 0.55, hh, 1, CAUSTIC);
            fb.rock(bx as i32, by as i32, br, hh, STONE_DARK);
            fb.rock(bx as i32, by as i32 - br / 6, br, hh, STONE);
            fb.rock(bx as i32 - br / 5, by as i32 - br / 3, (br as f32 * 0.62) as i32,
                    hh ^ 0x4C1D, STONE_LIGHT);
            // a little foam where it meets the water
            fb.rect(bx as i32 - br / 2, by as i32 + br / 4, br / 3, 2, FOAM);
        }

        // ------------------------------------------------------ sunk pebbles
        for i in 0..ROCKS {
            if self.state[i] != Pebble::Sunk {
                continue;
            }
            let r = self.rock_r(w, h, i) * 0.92;
            let (body, _) = PEBBLES[self.kind[i] as usize % PEBBLES.len()];
            // the deeper it lies, the more the water takes it
            let d = self.depth(w, h, self.x[i], self.y[i]);
            let veil = (60.0 + d * 190.0) as u32;
            let sunkc = (
                crate::engine::lerp(body.0, ABYSS.0, veil),
                crate::engine::lerp(body.1, ABYSS.1, veil),
                crate::engine::lerp(body.2, ABYSS.2, veil),
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
        // Drawn large, and built out of the parts that make each one
        // recognisable: a frog is eyes and haunches, a turtle is a patterned
        // shell with a head and four feet, a snail is a spiral. He should be
        // able to name them without being told.
        for k in 0..CRITTERS {
            let inhand = self.cheld == k as i32;
            let lift = if inhand { (u * 0.055) as i32 } else { 0 };
            if inhand {
                // its shadow stays on the ground, so it reads as picked up
                fb.fill_ellipse(self.cx[k] as i32, self.cy[k] as i32,
                                (u * 0.030) as i32, (u * 0.014) as i32, SHADOW);
            }
            let (x, y) = (self.cx[k] as i32, self.cy[k] as i32 - lift);
            let busy = self.ctimer[k] > 0.0;
            match self.ckind[k] {
                Critter::Frog => {
                    let r = (u * 0.055) as i32;
                    let hop = if busy { wave(self.ctimer[k] * 0.9) * u * 0.06 } else { 0.0 };
                    if hop > 0.5 {
                        fb.fill_ellipse(x, y, r, r / 2, SHADOW);
                    }
                    let fy = y - hop as i32;
                    // haunches, then the body over them
                    fb.fill_ellipse(x - r * 3 / 4, fy + r / 3, r / 2, r / 3, FROG_DARK);
                    fb.fill_ellipse(x + r * 3 / 4, fy + r / 3, r / 2, r / 3, FROG_DARK);
                    fb.fill_ellipse(x, fy, r, r * 3 / 4, FROG);
                    // front feet
                    fb.fill_ellipse(x - r / 2, fy + r * 2 / 3, r / 3, r / 5, FROG_DARK);
                    fb.fill_ellipse(x + r / 2, fy + r * 2 / 3, r / 3, r / 5, FROG_DARK);
                    // the eyes sit proud of the head, which is the whole tell
                    for side in [-1, 1] {
                        let ex = x + side * r / 2;
                        let ey = fy - r / 2;
                        fb.disc(ex, ey, r / 3, FROG);
                        fb.disc(ex, ey, r / 4, FROG_EYE);
                        fb.disc(ex, ey, r / 8 + 1, SHADOW);
                    }
                    // a wide mouth
                    fb.rect(x - r / 2, fy + r / 5, r, 2, FROG_DARK);
                }
                Critter::Dragonfly => {
                    let r = (u * 0.014) as i32 + 1;
                    let beat = wave(self.cphase[k] * 14.0);
                    let ww = (u * 0.055 * (0.5 + 0.5 * fabs(beat))) as i32;
                    // four wings, in two pairs
                    fb.fill_ellipse(x - ww / 2, y - 3, ww / 2, 3, WING);
                    fb.fill_ellipse(x + ww / 2, y - 3, ww / 2, 3, WING);
                    fb.fill_ellipse(x - ww / 3, y + 3, ww / 3, 2, WING);
                    fb.fill_ellipse(x + ww / 3, y + 3, ww / 3, 2, WING);
                    // a segmented body and a big head
                    fb.rect(x, y - 1, (u * 0.055) as i32, 3, DFLY);
                    fb.rect(x + (u * 0.020) as i32, y - 1, 3, 3, WING);
                    fb.rect(x + (u * 0.038) as i32, y - 1, 3, 3, WING);
                    fb.disc(x, y, r + 1, DFLY);
                    fb.disc(x - 2, y - 1, 2, FROG_EYE);
                }
                Critter::Tadpole => {
                    let r = (u * 0.018) as i32;
                    let wig = wave(self.cphase[k] * 3.0) * u * 0.014;
                    // a comma: round head, tapering tail
                    for seg in 0..4 {
                        let t = seg as f32 / 4.0;
                        fb.disc(
                            x - (u * 0.014 * (seg as f32 + 1.0)) as i32,
                            y + (wig * t) as i32,
                            ((r as f32) * (0.7 - t * 0.55)) as i32 + 1,
                            TADPOLE
                        );
                    }
                    fb.disc(x, y, r, TADPOLE);
                    fb.disc(x + r / 3, y - r / 3, r / 4, FROG_EYE);
                }
                Critter::Turtle => {
                    let r = (u * 0.055) as i32;
                    let out = if busy { 0 } else { r / 2 };
                    // four feet
                    for (sx2, sy2) in [(-1, -1), (1, -1), (-1, 1), (1, 1)] {
                        fb.fill_ellipse(
                            x + sx2 * r * 3 / 4,
                            y + sy2 * r / 2,
                            r / 3 + out / 3,
                            r / 4,
                            TURTLE
                        );
                    }
                    if out > 0 {
                        fb.fill_ellipse(x + r, y, r / 2, r / 3, TURTLE);   // head
                        fb.disc(x + r + r / 4, y - 2, 2, SHADOW);           // eye
                        fb.rect(x - r - r / 2, y, r / 2, 3, TURTLE);        // tail
                    }
                    // the shell, with plates on it
                    fb.fill_ellipse(x, y, r, r * 3 / 4, TURTLE_SHELL);
                    fb.fill_ellipse(x, y, r * 3 / 4, r / 2, TURTLE_SHELL_HI);
                    for p in 0..5 {
                        let d = DIRS8[p * 2 % 8];
                        fb.rock(
                            x + (d.0 * r as f32 * 0.45) as i32,
                            y + (d.1 * r as f32 * 0.32) as i32,
                            r / 5,
                            0x5AB1_0C33 ^ (p as u32 * 2654435761),
                            TURTLE_SHELL
                        );
                    }
                }
                Critter::Newt => {
                    let r = (u * 0.020) as i32;
                    let wig = wave(self.cphase[k] * 2.2) * u * 0.010;
                    // tail, body, head, and four little legs
                    for seg in 0..4 {
                        let t = seg as f32 / 4.0;
                        fb.fill_ellipse(
                            x - (u * 0.016 * (seg as f32 + 1.0)) as i32,
                            y + (wig * t) as i32,
                            ((r as f32) * (0.8 - t * 0.6)) as i32 + 1,
                            r / 3,
                            NEWT
                        );
                    }
                    for (sx2, sy2) in [(-1, -1), (0, -1), (-1, 1), (0, 1)] {
                        fb.rect(x + sx2 * r, y + sy2 * r / 2, r / 2, 3, NEWT);
                    }
                    fb.fill_ellipse(x, y, r, r * 2 / 3, NEWT);
                    fb.disc(x + r / 2, y - r / 3, 2, SHADOW);
                    for spot in 0..3 {
                        fb.rect(x - spot * r / 2, y - r / 4, 3, 3, NEWT_SPOT);
                    }
                }
                Critter::Beetle => {
                    let r = (u * 0.024) as i32;
                    for leg in 0..3 {
                        let ly = y - r / 2 + leg * r / 2;
                        fb.rect(x - r - r / 2, ly, r / 2, 2, SHADOW);
                        fb.rect(x + r, ly, r / 2, 2, SHADOW);
                    }
                    fb.fill_ellipse(x, y, r, r * 3 / 4, BEETLE);
                    fb.rect(x - 1, y - r * 3 / 4, 2, r * 3 / 2, SHADOW);   // wing case
                    fb.disc(x - r * 3 / 4, y, r / 3, SHADOW);              // head
                    fb.rect(x - r - r / 2, y - r / 2, r / 2, 2, SHADOW);   // antennae
                    fb.rect(x - r - r / 2, y + r / 2, r / 2, 2, SHADOW);
                }
                Critter::Ladybird => {
                    let r = (u * 0.020) as i32;
                    for leg in 0..3 {
                        let ly = y - r / 2 + leg * r / 2;
                        fb.rect(x - r - r / 3, ly, r / 3, 2, SHADOW);
                        fb.rect(x + r, ly, r / 3, 2, SHADOW);
                    }
                    fb.fill_ellipse(x, y, r, r * 4 / 5, LADYBIRD);
                    fb.rect(x - 1, y - r * 4 / 5, 2, r * 8 / 5, SHADOW);
                    fb.disc(x - r * 4 / 5, y, r / 2, SHADOW);
                    fb.disc(x + r / 3, y - r / 3, r / 4, SHADOW);
                    fb.disc(x + r / 3, y + r / 3, r / 4, SHADOW);
                    fb.disc(x - r / 4, y + r / 4, r / 5, SHADOW);
                }
                Critter::Butterfly => {
                    let beat = fabs(wave(self.cphase[k] * 7.0));
                    let ww = (u * 0.030 * (0.35 + 0.65 * beat)) as i32;
                    let c = if k % 2 == 0 { WING_A } else { WING_B };
                    fb.fill_ellipse(x - ww, y - ww / 2, ww, ww * 3 / 4, c);
                    fb.fill_ellipse(x + ww, y - ww / 2, ww, ww * 3 / 4, c);
                    fb.fill_ellipse(x - ww * 3 / 4, y + ww / 2, ww * 2 / 3, ww / 2, WING_B);
                    fb.fill_ellipse(x + ww * 3 / 4, y + ww / 2, ww * 2 / 3, ww / 2, WING_B);
                    fb.rect(x - 1, y - ww / 2, 3, ww, SHADOW);
                    fb.rect(x - 3, y - ww / 2 - 3, 2, 4, SHADOW);
                    fb.rect(x + 2, y - ww / 2 - 3, 2, 4, SHADOW);
                }
                Critter::Worm => {
                    let r = (u * 0.010) as i32 + 1;
                    let hidden = busy;
                    let segs = if hidden { 2 } else { 5 };
                    for seg in 0..segs {
                        let t = seg as f32;
                        let wig = wave(self.cphase[k] * 0.9 + t * 0.22) * u * 0.014;
                        fb.disc(x + (t * u * 0.016) as i32, y + wig as i32, r, WORM);
                    }
                    fb.fill_ellipse(x - r, y + r, (u * 0.022) as i32, r, SOIL_DARK);
                }
                Critter::Snail => {
                    let r = (u * 0.028) as i32;
                    // foot
                    fb.fill_ellipse(x, y + r / 2, r, r / 3, SNAIL);
                    // a spiral shell: rings, each a little smaller and offset
                    fb.disc(x, y, r, SNAIL_SHELL);
                    let mut rr = r;
                    let mut ox = 0;
                    while rr > 2 {
                        fb.disc(x + ox, y, rr, if rr % 4 < 2 { SNAIL_SHELL } else { SNAIL });
                        rr -= r / 4 + 1;
                        ox += r / 6;
                    }
                    if !busy {
                        fb.rect(x + r, y + r / 3, r / 2, 3, SNAIL);          // head
                        fb.rect(x + r + r / 3, y - r / 2, 2, r / 2 + 3, SNAIL);
                        fb.rect(x + r + r / 2 + 2, y - r / 3, 2, r / 3 + 3, SNAIL);
                        fb.disc(x + r + r / 3, y - r / 2, 2, SHADOW);
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
            // A pale outline, so what can be picked up announces itself against
            // stones that cannot. It breathes very slightly, which is enough to
            // catch an eye without ever being loud.
            let pulse = 1.0 + wave(self.t * 0.35 + i as f32 * 0.2) * 0.05;
            fb.rock(self.x[i] as i32, self.y[i] as i32 + 3, r as i32, self.shape[i], SHADOW);
            fb.rock(self.x[i] as i32, self.y[i] as i32, (r * pulse) as i32 + 2,
                    self.shape[i], PICKABLE);
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

        // ---------------------------------------------------------- the light
        let (mr, mg, mb, lift, _name) = THEMES[self.theme % THEMES.len()];
        fb.grade(mr, mg, mb, lift);
    }
}
