//! Load the Dump Truck.
//!
//! A sandbox, not a task. The truck drives, the world is wider than the
//! screen, and rocks stay wherever they are dumped — so hauling twelve rocks
//! from the quarry to a spot of his choosing builds a pile that is still there
//! on the next trip. Nothing completes, nothing is scored, nothing is lost.
//!
//! Design rules held throughout: no timer, no fail state, no error feedback,
//! and every touch returns something. A rock released anywhere is simply on
//! the ground there.

use crate::engine::audio;
use crate::engine::frame::DIRS8 as SPIN;
use crate::engine::{clamp, fabs, height, lerp, sfx, width, Frame, Rgb, Rng, DOWN, MOVE, UP};
use crate::games::Game;

const ROCKS: usize = 12;
/// Slots in the wall he is building. Bottom row of four, top row of three,
/// staggered — it reads as brickwork and needs seven of the twelve rocks.
const MILLS: usize = 2; // one at each end of the world
const SLOTS: usize = 7;
const SLOT_GRID: [(f32, f32); SLOTS] = [
    (0.0, 0.0),
    (1.0, 0.0),
    (2.0, 0.0),
    (3.0, 0.0),
    (0.5, 1.0),
    (1.5, 1.0),
    (2.5, 1.0),
];
const BED_SLOTS: usize = 6;
const WORLD_SCALE: f32 = 3.8; // world is this many screens wide
// quarry on the left, building site past it, and open ground beyond that —
// somewhere to tip a load that is NOT the wall
const TIP_MAX: f32 = 0.62;

/// How far the truck actually stands above the ground line, as a fraction of
/// its width. Derived from the cab and chassis constants in `layout()`; if you
/// change those, re-derive this.
const TRUCK_ABOVE_GROUND: f32 = 0.41;
/// Sky left above the cab. When terrain lands, the hills' amplitude has to come
/// out of this too, or the truck clips the top of the screen driving uphill.
const SKY_HEADROOM: f32 = 0.22;


// ------------------------------------------------------------------- colours

const SKY: Rgb = (150, 190, 194);
const SKY_BAND: Rgb = (163, 201, 203);
const CLOUD: Rgb = (191, 216, 216);
const HILL_FAR: Rgb = (126, 160, 152);
const HILL: Rgb = (108, 146, 136);
const DIRT: Rgb = (134, 106, 68);
const DIRT_TOP: Rgb = (158, 128, 84);
const DIRT_SPECK: Rgb = (118, 92, 58);
const QUARRY: Rgb = (112, 88, 58);
const QUARRY_FACE: Rgb = (146, 118, 80);

const BODY: Rgb = (201, 154, 46); // brass, straight from snake
const BODY_DARK: Rgb = (158, 116, 26);
const BODY_LIGHT: Rgb = (226, 186, 88);
const CAB_C: Rgb = (200, 71, 31); // oxide
const CAB_DARK: Rgb = (156, 50, 20);
const GLASS: Rgb = (172, 208, 210);
const GLASS_DARK: Rgb = (128, 168, 172);
const LAMP: Rgb = (250, 232, 150);
const LAMP_OFF: Rgb = (198, 168, 96);

const TIRE: Rgb = (38, 44, 48);
const HUB: Rgb = (186, 190, 194);
const LUG: Rgb = (120, 124, 128);

const ROCK_C: Rgb = (129, 133, 137);
const ROCK_LIGHT: Rgb = (166, 170, 174);
const SAND_C: Rgb = (176, 148, 104);
const SAND_LIGHT: Rgb = (206, 182, 142);
const BASALT_C: Rgb = (86, 90, 98);
const BASALT_LIGHT: Rgb = (120, 126, 136);
const ROCK_HELD: Rgb = (233, 200, 120);
const GEM: Rgb = (86, 160, 150);
const GEM_LIGHT: Rgb = (140, 205, 194);

const SMOKE: Rgb = (176, 186, 186);
const DUST: Rgb = (186, 164, 130);
const BIRD: Rgb = (66, 84, 90);
const BUSH: Rgb = (86, 122, 92);
const BUSH_DARK: Rgb = (66, 98, 72);
const TRACK: Rgb = (120, 94, 60);
const CHALK: Rgb = (222, 214, 196);
const STEEL: Rgb = (96, 104, 112);
const STEEL_DARK: Rgb = (68, 76, 84);
const STEEL_LIGHT: Rgb = (132, 142, 152);
const HAZARD: Rgb = (238, 201, 74); // its own yellow, not the truck's brass
const MOUTH: Rgb = (34, 38, 44);
const GRAVEL: Rgb = (146, 142, 134);
const GRAVEL_DARK: Rgb = (118, 114, 108);
const MORTAR: Rgb = (104, 92, 78);
const POLE: Rgb = (108, 92, 70);
const FLAG_C: Rgb = (200, 71, 31);

const NUMERAL: Rgb = (36, 46, 52);
const PIP_FULL: Rgb = (201, 154, 46);
const PIP_EMPTY: Rgb = (120, 152, 156);

// -------------------------------------------------------------------- layout

struct L {
    w: f32,
    h: f32,
    u: f32, // short side; every vertical gap is a fraction of this
    world_w: f32,
    ground: f32,     // y where the wheels meet the dirt
    rock_line: f32,  // rocks rest nearer the camera, below the wheel line
    horizon: f32,
    truck_w: f32,
    bed_w: f32,
    bed_h: f32,
    cab_w: f32,
    cab_h: f32,
    wall: f32,
    wheel_r: f32,
    rock_r: f32,
}

fn layout() -> L {
    let w = width() as f32;
    let h = height() as f32;
    let u = if w < h { w } else { h };

    // The horizon sits at a fraction of the height so sky and ground stay in
    // proportion on any shape of screen. Anchoring it to the bottom instead
    // crams the whole scene into a strip on a tall phone.
    // Ground sits a fixed slice above the bottom rather than at a fraction of
    // the height: on a tall phone a proportional split leaves a huge slab of
    // empty brown, and sky with clouds in it reads far better than dirt. The
    // slice is a third of the short side — 0.44 put the horizon almost halfway
    // up a landscape screen, which read as standing in a field of mud.
    let ground = h - u * 0.30;
    let horizon = ground - u * 0.09;

    // The world is WORLD_SCALE screens wide, so the truck does not have to be
    // small to leave room to drive — it only has to fit above the horizon.
    //
    // The height budget is the truck's ACTUAL extent above the ground, which
    // works out at 0.41 * truck_w from the constants below (cab 0.351, plus
    // 0.058 of chassis lift). Budgeting the truck's *width* against the whole
    // depth to the wheel line, as this used to, made it far too small in
    // landscape — the aspect where the game reads best.
    let truck_w = {
        // A tall frame leaves the scene marooned in a band at the bottom, so
        // the truck is allowed to take more of the width there. Landscape does
        // not need it — the height budget below is the binding constraint.
        let by_w = w * if h > w * 1.4 { 0.46 } else { 0.34 };
        let by_h = (ground - u * SKY_HEADROOM) / TRUCK_ABOVE_GROUND;
        if by_w < by_h {
            by_w
        } else {
            by_h
        }
    };

    L {
        w,
        h,
        u,
        world_w: w * WORLD_SCALE,
        ground,
        // Rocks live slightly in front of the truck. On the same line they
        // disappear behind the wheels and read as buried in the dirt.
        rock_line: ground + truck_w * 0.125 + u * 0.03,
        horizon,
        truck_w,
        bed_w: truck_w * 0.680,
        bed_h: truck_w * 0.274,
        cab_w: truck_w * 0.292,
        cab_h: truck_w * 0.330,
        wall: truck_w * 0.0357,
        wheel_r: truck_w * 0.101,
        rock_r: truck_w * 0.082,
    }
}

// -------------------------------------------------------------------- state

/// Three sizes he can name, rather than a continuous smear. Categories are
/// learnable; a spectrum is not.
const PEBBLE: f32 = 0.70;
const STONE: f32 = 1.00;
const BOULDER: f32 = 1.35;

#[derive(Clone, Copy, PartialEq)]
enum Kind {
    Granite,
    Sandstone,
    Basalt,
    Gem,
}

impl Kind {
    fn colors(self) -> (Rgb, Rgb) {
        match self {
            Kind::Granite => (ROCK_C, ROCK_LIGHT),
            Kind::Sandstone => (SAND_C, SAND_LIGHT),
            Kind::Basalt => (BASALT_C, BASALT_LIGHT),
            Kind::Gem => (GEM, GEM_LIGHT),
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
enum Rock {
    Ground,
    Feeding, // dropping into the crusher's hopper, shrinking as it goes
    Milled,  // inside the machine; comes back to the quarry shortly
    Held,
    Seating, // flying to a slot in the bed
    InBed,
    Falling,
    Setting, // flying to a slot in the wall
    Built,   // part of the wall, and out of the physics
}

#[derive(Clone, Copy, PartialEq)]
enum Tip {
    Idle,
    Squat, // anticipation: the truck settles before the bed moves
    Raising,
    Holding,
    Lowering,
}

#[derive(Clone, Copy, PartialEq)]
enum Grab {
    None,
    RockIdx(usize),
    Truck,
}

pub struct DumpTruck {
    // rocks, in world coordinates
    rx: [f32; ROCKS],
    ry: [f32; ROCKS],
    vx: [f32; ROCKS],
    vy: [f32; ROCKS],
    state: [Rock; ROCKS],
    slot: [usize; ROCKS],
    facet: [u8; ROCKS],
    size: [f32; ROCKS], // radius multiplier, so they are not all identical
    spin: [f32; ROCKS],
    kind: [Kind; ROCKS],
    shape: [u32; ROCKS], // seed for the outline
    wall_slot: [usize; ROCKS],

    // truck
    /// +1 faces right, -1 faces left. The load always leaves out of the back,
    /// so this decides which side a tip lands on.
    facing: f32,
    /// 0..1 through a turn; 1 means settled. The truck squashes to nothing at
    /// the halfway point and comes back the other way round, which reads as
    /// turning without needing any rotation maths.
    turn: f32,
    tx: f32,
    tv: f32,
    tv_smooth: f32,   // what the camera and the engine note read
    drag_target: f32, // where the finger wants the truck; applied in step()
    wheel_a: f32,
    susp: f32, // suspension offset, positive is compressed
    susp_v: f32,
    lamp: f32, // headlight flash, decays

    // interaction
    grab: Grab,
    grab_off: f32,
    grab_offy: f32,
    moved_far: bool,
    grab_x0: f32,
    grab_y0: f32,

    // bed
    tip: f32,
    phase: Tip,
    hold_t: f32,
    spill_t: f32,

    // world
    cam: f32,
    shake: f32,
    count: u32,
    pulse: f32,
    hauled: u32,
    /// 0 until the wall is finished, then it climbs to 1 as the flag goes up.
    flag: f32,
    cheered: bool,

    // the crusher
    feed_t: [f32; ROCKS],   // descent down the throat, or time spent milled
    mill_of: [u8; ROCKS],   // which machine swallowed it
    gravel: [u32; MILLS],   // every rock ever put through that one
    gems: [u32; MILLS],
    mill: [f32; MILLS],     // how hard it is working, 0..1, decays
    jaw: [f32; MILLS],      // the jaw's shudder

    // ambience
    puff_t: f32,
    puffs: [(f32, f32, f32); 6], // x, y, age
    puff_i: usize,
    bird_x: f32,
    bird_y: f32,
    bird_a: f32,

    accum: f32,
    moved: bool,
    ready: bool,

    // the frame the current positions were computed for, so a resize can
    // rescale the world instead of throwing it away
    last_world_w: f32,
    last_u: f32,
    last_rock_line: f32,
}

impl DumpTruck {
    pub const fn new() -> DumpTruck {
        DumpTruck {
            rx: [0.0; ROCKS],
            ry: [0.0; ROCKS],
            vx: [0.0; ROCKS],
            vy: [0.0; ROCKS],
            state: [Rock::Ground; ROCKS],
            slot: [0; ROCKS],
            facet: [0; ROCKS],
            size: [1.0; ROCKS],
            spin: [0.0; ROCKS],
            kind: [Kind::Granite; ROCKS],
            shape: [0; ROCKS],
            wall_slot: [0; ROCKS],
            facing: 1.0,
            turn: 1.0,
            tx: 0.0,
            tv: 0.0,
            tv_smooth: 0.0,
            drag_target: 0.0,
            wheel_a: 0.0,
            susp: 0.0,
            susp_v: 0.0,
            lamp: 0.0,
            grab: Grab::None,
            grab_off: 0.0,
            grab_offy: 0.0,
            moved_far: false,
            grab_x0: 0.0,
            grab_y0: 0.0,
            tip: 0.0,
            phase: Tip::Idle,
            hold_t: 0.0,
            spill_t: 0.0,
            cam: 0.0,
            shake: 0.0,
            count: 0,
            pulse: 0.0,
            hauled: 0,
            flag: 0.0,
            cheered: false,
            feed_t: [0.0; ROCKS],
            mill_of: [0; ROCKS],
            gravel: [0; MILLS],
            gems: [0; MILLS],
            mill: [0.0; MILLS],
            jaw: [0.0; MILLS],
            puff_t: 0.0,
            puffs: [(0.0, 0.0, 9.0); 6],
            puff_i: 0,
            bird_x: 0.0,
            bird_y: 0.0,
            bird_a: 0.0,
            accum: 0.0,
            moved: false,
            ready: false,
            last_world_w: 0.0,
            last_u: 0.0,
            last_rock_line: 0.0,
        }
    }

    // ------------------------------------------------------------- geometry

    fn faces_left(&self) -> bool {
        self.facing < 0.0
    }
    /// Facing left, the cab takes the near end and the bed the far one. Every
    /// other measurement is derived from these two, so swapping them here is
    /// what actually turns the truck around.
    fn bed_x(&self, l: &L) -> f32 {
        if self.faces_left() {
            self.tx + l.cab_w + l.truck_w * 0.030
        } else {
            self.tx
        }
    }
    fn cab_x(&self, l: &L) -> f32 {
        if self.faces_left() {
            self.tx
        } else {
            self.tx + l.bed_w + l.truck_w * 0.030
        }
    }
    fn chassis_y(&self, l: &L) -> f32 {
        l.ground + l.wheel_r * 0.24 - l.wheel_r * 0.82 + self.susp
    }
    fn bed_y(&self, l: &L) -> f32 {
        self.chassis_y(l) - l.bed_h - l.truck_w * 0.006
    }
    fn cab_y(&self, l: &L) -> f32 {
        self.chassis_y(l) - l.cab_h
    }
    /// The truck occupies [tx, tx + truck_w] whichever way round it is. This
    /// used to be derived from the cab, which put it at the LEFT end once the
    /// truck turned — collapsing every hit box to an empty range, so a turned
    /// truck could not be tapped, driven or tipped at all.
    fn truck_right(&self, l: &L) -> f32 {
        self.tx + l.truck_w
    }

    fn slot_pos(&self, l: &L, s: usize) -> (f32, f32) {
        let across = l.bed_w - l.wall * 2.0 - l.rock_r * 2.0;
        let x = self.bed_x(l)
            + l.wall
            + l.rock_r
            + across * (s as f32 / (BED_SLOTS - 1) as f32);
        let y = self.bed_y(l) + l.bed_h * 0.42;
        // the bed lifts at the cab end, so the load slides toward the back
        let arm = if self.faces_left() {
            self.bed_x(l) + l.bed_w - x
        } else {
            x - self.bed_x(l)
        };
        (x, y - self.tip * arm)
    }

    /// The crusher stands at the far end of the world, and its hopper is on the
    /// LEFT flank on purpose: arriving from the quarry he is facing right, and
    /// a tip throws the load out behind him — so driving just past the hopper
    /// and tipping drops it straight in, with no turning required. Turning is
    /// for the wall.
    /// Ground left clear beyond each machine, so neither is jammed against the
    /// end of the world.
    fn pad(&self, l: &L) -> f32 {
        l.truck_w * 0.8
    }

    /// Mill 0 sits at the left end, mill 1 at the right, each placed exactly
    /// where a load lands when the truck is driven as far that way as it will
    /// go. Driving to the end IS arriving at a crusher: no parking puzzle.
    ///
    /// The left one can only be fed facing left, and the right one facing
    /// right, because the load always leaves out of the back — so between them
    /// they give the turn button a reason to exist.
    fn hopper_x(&self, l: &L, m: usize) -> f32 {
        if m == 0 {
            self.pad(l) + l.truck_w * 1.15
        } else {
            l.world_w - self.pad(l) - l.truck_w * 1.15
        }
    }

    /// How far the truck may drive. It stops at the machines rather than
    /// driving through them, which also means arriving is automatic: hold the
    /// drag until it will not go any further and the bed is over the mouth.
    fn drive_range(&self, l: &L) -> (f32, f32) {
        (self.pad(l), l.world_w - l.truck_w - self.pad(l))
    }

    /// Mouth of the funnel: top, bottom, and half-width at the top. The top has
    /// to sit BELOW the loaded bed or a tipped rock would have to fly upward to
    /// get in — which is exactly what it used to do, landing on the ground
    /// instead and then sinking through it.
    fn mouth(&self, l: &L) -> (f32, f32, f32) {
        (
            l.ground - l.truck_w * 0.19,
            l.ground - l.truck_w * 0.01,
            l.truck_w * 0.23,
        )
    }

    /// Which machine, if any, a rock at (x, y) is falling into. Caught in the
    /// air at the mouth rather than after it has landed: that is what makes it
    /// visibly drop IN instead of vanishing on the ground beside the thing.
    fn into_mouth(&self, l: &L, x: f32, y: f32, r: f32) -> Option<usize> {
        let (top, bot, half) = self.mouth(l);
        if y + r < top || y > bot {
            return None;
        }
        for m in 0..MILLS {
            if fabs(x - self.hopper_x(l, m)) < half {
                return Some(m);
            }
        }
        None
    }

    /// A far wider catch, used once a rock has already come to rest: near
    /// enough counts, and it rolls in. Same rule as the wall — he should never
    /// have to solve an approach puzzle to use the thing.
    fn beside_hopper(&self, l: &L, x: f32) -> Option<usize> {
        (0..MILLS).find(|&m| fabs(x - self.hopper_x(l, m)) < l.truck_w * 0.42)
    }

    /// Where the wall is being built: open ground just past the far end of the
    /// quarry, so it comes into view soon after leaving the rocks — and so
    /// getting a load into it means turning the truck round.
    fn wall_x(&self, l: &L) -> f32 {
        self.hopper_x(l, 0) + l.truck_w * 0.9 + (ROCKS as f32) * l.rock_r * 3.3 + l.rock_r * 6.0
    }

    fn slot_pos_wall(&self, l: &L, s: usize) -> (f32, f32) {
        let d = l.rock_r * 2.15;
        let (col, row) = SLOT_GRID[s];
        (
            self.wall_x(l) + d * (col + 0.5),
            l.rock_line - l.rock_r - row * d * 0.86,
        )
    }

    fn slot_taken(&self, s: usize) -> bool {
        (0..ROCKS).any(|i| {
            matches!(self.state[i], Rock::Built | Rock::Setting) && self.wall_slot[i] == s
        })
    }

    /// The nearest empty slot, if the rock came to rest anywhere on the
    /// building site. There is no distance test beyond the site's own width on
    /// purpose: tipping a whole load at the wall should build the wall, not
    /// leave a heap beside it with two stones that happened to land true.
    fn free_wall_slot(&self, l: &L, x: f32) -> Option<usize> {
        // The catch area is far wider than the wall itself — wider than the
        // truck, so a load tipped anywhere near the site lands in it. The wall
        // is only about four rocks across; a window that size is narrower than
        // the truck that has to straddle it, and every load misses.
        let d = l.rock_r * 2.15;
        let centre = self.wall_x(l) + d * 2.0;
        if fabs(x - centre) > l.truck_w * 0.62 {
            return None;
        }
        let mut best = None;
        let mut best_d = l.world_w;
        for s in 0..SLOTS {
            if self.slot_taken(s) {
                continue;
            }
            let (sx, _) = self.slot_pos_wall(l, s);
            let gap = fabs(sx - x);
            if gap < best_d {
                best_d = gap;
                best = Some(s);
            }
        }
        best
    }

    fn wall_built(&self) -> u32 {
        (0..ROCKS)
            .filter(|&i| self.state[i] == Rock::Built)
            .count() as u32
    }

    /// Put a rock into the wall if it came to rest near an empty slot. Used by
    /// a hand-placed rock and by one tipped out of the bed alike, so hauling a
    /// load to the wall builds it without him having to place each stone.
    fn try_wall(&mut self, l: &L, i: usize) -> bool {
        if let Some(s) = self.free_wall_slot(l, self.rx[i]) {
            self.wall_slot[i] = s;
            self.state[i] = Rock::Setting;
            self.vx[i] = 0.0;
            self.vy[i] = 0.0;
            return true;
        }
        false
    }

    fn free_slot(&self) -> Option<usize> {
        for s in 0..BED_SLOTS {
            let mut taken = false;
            for i in 0..ROCKS {
                if matches!(self.state[i], Rock::InBed | Rock::Seating) && self.slot[i] == s {
                    taken = true;
                    break;
                }
            }
            if !taken {
                return Some(s);
            }
        }
        None
    }

    fn in_bed_count(&self) -> u32 {
        let mut n = 0;
        for i in 0..ROCKS {
            if matches!(self.state[i], Rock::InBed | Rock::Seating) {
                n += 1;
            }
        }
        n
    }

    /// Where a rock of radius `r` comes to rest at world x, sitting on the
    /// ground or on top of whatever is already piled there. This is what makes
    /// the world remember: dumped rocks stay, and they stack.
    /// `from_y` is the height the rock is currently at. Only rocks strictly
    /// BELOW it can hold it up — without that test two rocks at the same
    /// height each stack on the other, every frame, and the pair climbs off
    /// the top of the world at thirty pixels a frame.
    fn rest_y(&self, l: &L, x: f32, from_y: f32, r: f32, ignore: usize) -> f32 {
        let mut best = l.rock_line - r;
        for j in 0..ROCKS {
            if j == ignore || self.state[j] != Rock::Ground {
                continue;
            }
            let rj = l.rock_r * self.size[j];
            let reach = (r + rj) * 0.86;
            if fabs(x - self.rx[j]) < reach && self.ry[j] > from_y + rj * 0.25 {
                let cand = self.ry[j] - (r + rj) * 0.78;
                if cand < best {
                    best = cand;
                }
            }
        }
        best
    }

    fn over_bed(&self, l: &L, x: f32, y: f32) -> bool {
        let m = l.rock_r * 1.6;
        x > self.bed_x(l) - m
            && x < self.bed_x(l) + l.bed_w + m
            && y > self.bed_y(l) - l.bed_h * 1.4
            && y < self.bed_y(l) + l.bed_h + m
    }

    fn on_truck(&self, l: &L, x: f32, y: f32) -> bool {
        let m = l.rock_r * 0.4;
        x > self.bed_x(l) - m
            && x < self.truck_right(l) + m
            && y > self.cab_y(l) - m
            && y < l.ground + l.wheel_r
    }

    fn on_cab(&self, l: &L, x: f32, y: f32) -> bool {
        // the cab's own span, not "everything right of the cab" — that would
        // swallow the bed once the truck is turned round, and a tap meant to
        // tip the load would sound the horn instead
        x > self.cab_x(l)
            && x < self.cab_x(l) + l.cab_w
            && y > self.cab_y(l)
            && y < self.chassis_y(l)
    }

    // ---------------------------------------------------------------- setup

    fn deal(&mut self, l: &L, rng: &mut Rng) {
        // The quarry is the left end of the world; the rest is open ground to
        // build on.
        // Three loose clusters rather than a ruler-straight line, but with a
        // hard minimum gap: rocks closer than the stacking reach are read as
        // piled on one another and build a staircase into the sky.
        let min_gap = l.rock_r * (BOULDER + BOULDER) * 1.15;
        let mut x = l.rock_r * 2.0;
        for i in 0..ROCKS {
            self.size[i] = match rng.next() % 8 {
                0 | 1 => PEBBLE,
                2 => BOULDER,
                _ => STONE,
            };
            self.kind[i] = match rng.next() % 16 {
                0 => Kind::Gem,
                1 | 2 | 3 => Kind::Sandstone,
                4 | 5 => Kind::Basalt,
                _ => Kind::Granite,
            };
            self.shape[i] = rng.next();
            self.facet[i] = (rng.next() % 4) as u8;
            self.spin[i] = 0.0;
            self.state[i] = Rock::Ground;
            self.vx[i] = 0.0;
            self.vy[i] = 0.0;
            self.slot[i] = 0;

            self.rx[i] = x;
            self.ry[i] = l.rock_line - l.rock_r * self.size[i];

            // a gap inside a cluster, a bigger one between clusters
            x += if i % 4 == 3 {
                min_gap * (2.0 + rng.unit())
            } else {
                min_gap * (1.0 + rng.unit() * 0.5)
            };
        }
        // Always at least one gem, wherever the rolls landed.
        self.kind[(rng.next() as usize) % ROCKS] = Kind::Gem;

        // Park the truck in the middle of the quarry, so the first thing on
        // screen is a truck surrounded by rocks.
        let field_mid = (self.rx[0] + self.rx[ROCKS - 1]) * 0.5;
        self.tx = clamp(field_mid - l.truck_w * 0.5, 0.0, l.world_w - l.truck_w);
        self.tv = 0.0;
        self.susp = 0.0;
        self.susp_v = 0.0;
        self.wheel_a = 0.0;
        self.cam = clamp(self.tx + l.truck_w * 0.5 - l.w * 0.5, 0.0, l.world_w - l.w);
        self.count = 0;
        self.ready = true;
        self.remember(l);
    }

    fn remember(&mut self, l: &L) {
        self.last_world_w = l.world_w;
        self.last_u = l.u;
        self.last_rock_line = l.rock_line;
    }

    /// Carry the world across a change of frame shape. Rocks keep their
    /// position as a fraction of the world, and their height as an offset from
    /// the rock line scaled by the short side — so a pile he built stays a pile
    /// he built when the tablet is turned over.
    fn rescale(&mut self, l: &L) {
        if self.last_world_w <= 0.0 || self.last_u <= 0.0 {
            return;
        }
        let fx = l.world_w / self.last_world_w;
        let fu = l.u / self.last_u;

        for i in 0..ROCKS {
            self.rx[i] *= fx;
            let above = self.last_rock_line - self.ry[i];
            self.ry[i] = l.rock_line - above * fu;
            self.vx[i] *= fx;
            self.vy[i] *= fu;
        }
        // Stones already in the wall belong to their slot, not to a scaled
        // coordinate: the wall itself has moved and resized too.
        for i in 0..ROCKS {
            if self.state[i] == Rock::Built {
                let (wx, wy) = self.slot_pos_wall(l, self.wall_slot[i]);
                self.rx[i] = wx;
                self.ry[i] = wy;
            }
        }
        self.tx = clamp(self.tx * fx, 0.0, l.world_w - l.truck_w);
        self.drag_target = self.tx;
        self.tv = 0.0;
        self.tv_smooth = 0.0;
        self.cam = clamp(
            self.tx + l.truck_w * 0.5 - l.w * 0.5,
            0.0,
            l.world_w - l.w,
        );
        self.remember(l);
    }


    fn start_turn(&mut self) {
        // Not while the bed is up: the load would swap ends mid-pour.
        if self.turn >= 1.0 && self.phase == Tip::Idle {
            self.turn = 0.0;
            self.susp_v += 90.0;
            sfx(audio::TURN, 1.0);
        }
    }

    fn start_tip(&mut self) {
        if self.phase == Tip::Idle && self.in_bed_count() > 0 {
            self.phase = Tip::Squat;
            self.hold_t = 0.0;
            self.spill_t = 0.0;
            // anticipation: the truck settles on its springs before it lifts
            self.susp_v += 220.0;
        }
    }

    fn puff(&mut self, x: f32, y: f32) {
        self.puffs[self.puff_i] = (x, y, 0.0);
        self.puff_i = (self.puff_i + 1) % self.puffs.len();
    }

    // ----------------------------------------------------------------- step

    fn step(&mut self, l: &L, dt: f32, rng: &mut Rng) {
        let s = dt / 1000.0;

        // ------------------------------------------------------- suspension
        // A spring with damping. Every impact pushes it; it settles on its own.
        self.susp_v += -self.susp * 260.0 * s;
        self.susp_v *= 1.0 - 6.0 * s;
        self.susp += self.susp_v * s;
        self.susp = clamp(self.susp, -l.wheel_r * 0.5, l.wheel_r * 0.5);

        if self.turn < 1.0 {
            let was = self.turn;
            self.turn += s * 2.6;
            // halfway through it is edge-on and invisible; that is where it
            // actually swaps round
            if was < 0.5 && self.turn >= 0.5 {
                self.facing = -self.facing;
            }
            if self.turn > 1.0 {
                self.turn = 1.0;
            }
            self.moved = true;
        }

        // the crushers: a held note while they work, and jaws that shudder
        let mut work = 0.0;
        for m in 0..MILLS {
            self.mill[m] = if self.mill[m] > 0.0 { self.mill[m] - s * 0.8 } else { 0.0 };
            self.jaw[m] = if self.jaw[m] > 0.0 { self.jaw[m] - s * 3.0 } else { 0.0 };
            if self.mill[m] > work {
                work = self.mill[m];
            }
        }
        sfx(audio::MILL, work);

        // the wall
        if self.wall_built() as usize >= SLOTS {
            if !self.cheered {
                self.cheered = true;
                sfx(audio::DONE, 1.0);
            }
            if self.flag < 1.0 {
                self.flag += s * 1.1;
                if self.flag > 1.0 {
                    self.flag = 1.0;
                }
                self.moved = true;
            }
        } else {
            self.cheered = false;
            if self.flag > 0.0 {
                self.flag -= s * 2.0;
                if self.flag < 0.0 {
                    self.flag = 0.0;
                }
                self.moved = true;
            }
        }

        self.lamp = if self.lamp > 0.0 { self.lamp - s * 1.6 } else { 0.0 };
        self.pulse = if self.pulse > 0.0 { self.pulse - s * 2.2 } else { 0.0 };
        self.shake = if self.shake > 0.0 { self.shake - s * 5.0 } else { 0.0 };

        // ---------------------------------------------------------- driving
        // The pointer handler only records where the finger wants the truck.
        // Moving it here, on the fixed timestep, is what makes velocity a real
        // per-second quantity instead of a per-event delta that assumed 60 Hz —
        // which is what used to make the camera jump on a fast drag.
        let (lo_x, hi_x) = self.drive_range(l);
        if self.grab == Grab::Truck {
            let want = clamp(self.drag_target, lo_x, hi_x);
            // Follow the finger with a little weight rather than snapping to
            // it. Snapping made the POSITION exact but the velocity spiky:
            // pointer events arrive at a different rate than this fixed step,
            // so one step saw a jump and the next saw nothing at all. The
            // camera lead is driven by that velocity, so the whole world
            // twitched — worst when he moved the truck slowly, which is when
            // each step's jump is smallest and the on/off pattern loudest.
            let follow = clamp(14.0 * s, 0.0, 1.0);
            let dx = (want - self.tx) * follow;
            self.tx += dx;
            self.tv = dx / s;
        } else {
            // coast to a stop when he lets go
            self.tv *= 1.0 - 3.4 * s;
            self.tx += self.tv * s;
            self.tx = clamp(self.tx, lo_x, hi_x);
        }
        self.tv_smooth += (self.tv - self.tv_smooth) * 8.0 * s;
        self.wheel_a += self.tv_smooth * s / (l.wheel_r * 0.9);

        // engine note follows speed; the host holds one oscillator for this
        let speed = {
            let v = fabs(self.tv_smooth) / (l.w * 0.9);
            if v > 1.0 {
                1.0
            } else {
                v
            }
        };
        sfx(audio::ENGINE, speed);
        sfx(
            audio::REVERSE,
            if self.tv_smooth < -l.w * 0.06 { 1.0 } else { 0.0 },
        );

        // exhaust, always, so an untouched screen is never a still image
        self.puff_t += s;
        let rate = 0.42 - speed * 0.28;
        if self.puff_t > rate {
            self.puff_t = 0.0;
            let px = self.cab_x(l) + l.cab_w * 0.18;
            let py = self.cab_y(l) - l.u * 0.012;
            self.puff(px, py);
        }
        for p in self.puffs.iter_mut() {
            if p.2 < 2.0 {
                p.2 += s;
                p.1 -= l.u * 0.055 * s;
                p.0 += l.u * 0.012 * s;
            }
        }

        // a bird crosses now and then
        if self.bird_a <= 0.0 {
            if rng.unit() < 0.004 {
                self.bird_a = 1.0;
                self.bird_x = self.cam - l.u * 0.1;
                self.bird_y = l.u * (0.12 + rng.unit() * 0.2);
            }
        } else {
            self.bird_x += l.u * 0.16 * s;
            self.bird_y -= l.u * 0.01 * s;
            if self.bird_x > self.cam + l.w + l.u * 0.1 {
                self.bird_a = 0.0;
            }
        }

        // ---------------------------------------------------------- tipping
        match self.phase {
            Tip::Squat => {
                self.hold_t += s;
                if self.hold_t > 0.22 {
                    self.phase = Tip::Raising;
                }
                self.moved = true;
            }
            Tip::Raising => {
                self.tip += s * 1.35 * TIP_MAX;
                if self.tip >= TIP_MAX {
                    self.tip = TIP_MAX;
                    self.phase = Tip::Holding;
                    self.hold_t = 0.0;
                }
                self.moved = true;
            }
            Tip::Holding => {
                self.hold_t += s;
                if self.hold_t > 0.5 && self.in_bed_count() == 0 {
                    self.phase = Tip::Lowering;
                }
                self.moved = true;
            }
            Tip::Lowering => {
                self.tip -= s * 1.15 * TIP_MAX;
                if self.tip <= 0.0 {
                    self.tip = 0.0;
                    self.phase = Tip::Idle;
                    self.susp_v += 90.0; // the bed lands back on the frame
                }
                self.moved = true;
            }
            Tip::Idle => {}
        }
        // Hydraulics are a held note that tracks the bed, like the engine —
        // not a one-shot that finishes long before the bed does.
        sfx(
            audio::TIP,
            match self.phase {
                Tip::Squat | Tip::Raising | Tip::Lowering => self.tip / TIP_MAX + 0.15,
                _ => 0.0,
            },
        );

        // Rocks leave one at a time so the numeral ticks down rather than
        // snapping to zero. That tick is the whole counting lesson.
        if matches!(self.phase, Tip::Raising | Tip::Holding) && self.tip > TIP_MAX * 0.45 {
            self.spill_t += s;
            if self.spill_t > 0.16 {
                self.spill_t = 0.0;
                // spill the rock nearest the open end first, whichever end
                // that is now
                let mut pick = -1i32;
                let mut best = l.world_w * 2.0;
                for i in 0..ROCKS {
                    if self.state[i] == Rock::InBed {
                        let (sx, _) = self.slot_pos(l, self.slot[i]);
                        let d = if self.faces_left() { -sx } else { sx };
                        if d < best {
                            best = d;
                            pick = i as i32;
                        }
                    }
                }
                if pick >= 0 {
                    let i = pick as usize;
                    self.state[i] = Rock::Falling;
                    // out of the back: left when facing right, right when left
                    self.vx[i] = -self.facing * (l.u * 0.18 + rng.unit() * l.u * 0.14);
                    self.vy[i] = -(l.u * 0.06) - rng.unit() * l.u * 0.06;
                    self.set_count(self.count.saturating_sub(1));
                    self.moved = true;
                }
            }
        }

        // ---------------------------------------------------------- physics
        for i in 0..ROCKS {
            let r = l.rock_r * self.size[i];
            match self.state[i] {
                Rock::Falling => {
                    self.vy[i] += l.u * 3.4 * s;
                    self.rx[i] += self.vx[i] * s;
                    self.ry[i] += self.vy[i] * s;
                    if let Some(m) = self.into_mouth(l, self.rx[i], self.ry[i], r) {
                        self.state[i] = Rock::Feeding;
                        self.mill_of[i] = m as u8;
                        self.feed_t[i] = 0.0;
                        self.vx[i] = 0.0;
                        self.vy[i] = 0.0;
                        self.moved = true;
                        continue;
                    }
                    self.vx[i] *= 1.0 - 1.5 * s;
                    self.spin[i] += self.vx[i] * s / (r * 0.9); // big rocks turn slower

                    let floor = self.rest_y(l, self.rx[i], self.ry[i], r, i);
                    if self.ry[i] >= floor {
                        self.ry[i] = floor;
                        let impact = fabs(self.vy[i]) / (l.u * 0.9);
                        if let Some(m) = self.beside_hopper(l, self.rx[i]) {
                            self.state[i] = Rock::Feeding;
                            self.mill_of[i] = m as u8;
                            self.feed_t[i] = 0.0;
                            self.vx[i] = 0.0;
                            self.vy[i] = 0.0;
                        } else if self.free_wall_slot(l, self.rx[i]).is_some() && self.try_wall(l, i) {
                            // landed on the building site: straight into the wall
                        } else if fabs(self.vy[i]) > l.u * 0.35 {
                            // heavier rocks keep less of the bounce
                            let restitution = 0.30 * (1.0 - 0.30 * (self.size[i] - PEBBLE));
                            self.vy[i] = -self.vy[i] * restitution;
                            self.vx[i] *= 0.55;
                            let hit = if floor < l.rock_line - r - 1.0 {
                                audio::ROCK_HIT // came to rest on another rock
                            } else {
                                audio::LAND
                            };
                            sfx(hit, if impact > 1.0 { 1.0 } else { impact });
                            self.shake = if impact > 0.6 { 0.6 } else { impact };
                            self.puff(self.rx[i], self.ry[i] + r * 0.6);
                        } else if let Some(m) = self.beside_hopper(l, self.rx[i]) {
                            self.state[i] = Rock::Feeding;
                            self.mill_of[i] = m as u8;
                            self.feed_t[i] = 0.0;
                            self.vx[i] = 0.0;
                            self.vy[i] = 0.0;
                        } else if self.try_wall(l, i) {
                            // it came down on the building site
                        } else {
                            // A gentle landing still makes a noise. Silence
                            // here breaks the rule the whole game is built on:
                            // everything he does answers back.
                            let soft = if floor < l.rock_line - r - 1.0 {
                                audio::ROCK_HIT
                            } else {
                                audio::LAND
                            };
                            sfx(soft, 0.12);
                            self.vy[i] = 0.0;
                            self.vx[i] = 0.0;
                            self.state[i] = Rock::Ground;
                            self.hauled += 1;
                        }
                    }
                    self.rx[i] = clamp(self.rx[i], r + 2.0, l.world_w - r - 2.0);
                    self.moved = true;
                }
                Rock::Feeding => {
                    // Down the throat: converging on the centre line as the
                    // funnel narrows, and shrinking as it goes out of sight.
                    let m = self.mill_of[i] as usize;
                    let (top, bot, _) = self.mouth(l);
                    let hx = self.hopper_x(l, m);
                    self.feed_t[i] += s * 1.5;
                    let k = if self.feed_t[i] > 1.0 { 1.0 } else { self.feed_t[i] };
                    self.rx[i] += (hx - self.rx[i]) * 7.0 * s;
                    self.ry[i] = top + (bot - top) * k;
                    if self.feed_t[i] >= 1.0 {
                        self.state[i] = Rock::Milled;
                        self.feed_t[i] = 0.0;
                        self.gravel[m] += 1;
                        self.mill[m] = 1.0;
                        self.jaw[m] = 1.0;
                        self.shake = 0.7;
                        // dust coughs out of the top as it bites
                        self.puff(hx, top - l.truck_w * 0.10);
                        sfx(audio::CRUNCH, self.size[i]);
                        if self.gravel[m] % 5 == 0 {
                            self.gems[m] += 1;
                            sfx(audio::GEM, self.gems[m] as f32);
                        }
                    }
                    self.moved = true;
                }
                Rock::Milled => {
                    // Back to the quarry shortly. Nothing is ever used up: a
                    // world that empties is a losing condition wearing a
                    // friendly hat.
                    self.feed_t[i] += s;
                    if self.feed_t[i] > 3.5 {
                        self.state[i] = Rock::Ground;
                        self.feed_t[i] = 0.0;
                        self.rx[i] =
                            self.hopper_x(l, 0) + l.truck_w * 0.9 + rng.unit() * l.rock_r * 6.0;
                        self.ry[i] = l.rock_line - l.rock_r * self.size[i];
                        self.vx[i] = 0.0;
                        self.vy[i] = 0.0;
                        self.puff(self.rx[i], self.ry[i]);
                    }
                    self.moved = true;
                }
                Rock::Setting => {
                    let (tx, ty) = self.slot_pos_wall(l, self.wall_slot[i]);
                    self.rx[i] += (tx - self.rx[i]) * 13.0 * s;
                    self.ry[i] += (ty - self.ry[i]) * 13.0 * s;
                    self.spin[i] *= 1.0 - 6.0 * s;
                    if fabs(tx - self.rx[i]) < 2.0 && fabs(ty - self.ry[i]) < 2.0 {
                        self.rx[i] = tx;
                        self.ry[i] = ty;
                        self.spin[i] = 0.0;
                        self.state[i] = Rock::Built;
                        sfx(audio::SET, self.wall_built() as f32);
                        self.shake = 0.35;
                        self.puff(tx, ty + l.rock_r);
                    }
                    self.moved = true;
                }
                Rock::Built => {}
                Rock::Seating => {
                    let (tx, ty) = self.slot_pos(l, self.slot[i]);
                    self.rx[i] += (tx - self.rx[i]) * 15.0 * s;
                    self.ry[i] += (ty - self.ry[i]) * 15.0 * s;
                    if fabs(tx - self.rx[i]) < 2.0 && fabs(ty - self.ry[i]) < 2.0 {
                        self.rx[i] = tx;
                        self.ry[i] = ty;
                        self.state[i] = Rock::InBed;
                        self.susp_v += 130.0 * self.size[i]; // the truck feels it
                        sfx(audio::SEAT, self.size[i]);
                    }
                    self.moved = true;
                }
                Rock::InBed => {
                    let (tx, ty) = self.slot_pos(l, self.slot[i]);
                    self.rx[i] = tx;
                    self.ry[i] = ty;
                }
                _ => {}
            }
        }

        // No sideways separation pass here, on purpose. Nudging settled rocks
        // apart fights the stacking rule — a rock gets pushed aside, drops
        // because it no longer overlaps anything, then gets pushed again — and
        // a pile ratchets itself across the world a pixel at a time. That is
        // what made dumped rocks wander off. Overlapping rocks are meant to
        // stack, which rest_y already does.
        for i in 0..ROCKS {
            if self.state[i] == Rock::Ground {
                let r = l.rock_r * self.size[i];
                let floor = self.rest_y(l, self.rx[i], self.ry[i], r, i);
                // let settled rocks fall if the pile under them moved
                if self.ry[i] < floor - 0.5 {
                    self.ry[i] += (floor - self.ry[i]) * 9.0 * s;
                    self.moved = true;
                } else {
                    self.ry[i] = floor;
                }
            }
        }

        // ----------------------------------------------------------- camera
        // Leads slightly in the direction of travel, the way a chase camera
        // does, and never shows past the ends of the world.
        // No lead while his finger is on the truck. A chase camera leading the
        // direction of travel is right for momentum, but during a direct drag
        // it fights the hand: the lead is driven by velocity, velocity is
        // sampled at the fixed step rather than at the pointer's rate, and the
        // resulting wobble shows up as the truck stuttering backwards.
        let lead = if self.grab == Grab::Truck {
            0.0
        } else {
            clamp(self.tv_smooth * 0.22, -l.w * 0.12, l.w * 0.12)
        };
        let want = clamp(
            self.tx + l.truck_w * 0.5 - l.w * 0.5 + lead,
            0.0,
            l.world_w - l.w,
        );
        self.cam += (want - self.cam) * 6.0 * s;
    }

    fn set_count(&mut self, n: u32) {
        if n != self.count {
            self.count = n;
            self.pulse = 1.0;
            sfx(audio::COUNT, n as f32);
        }
    }
}

impl Game for DumpTruck {
    fn reset(&mut self, rng: &mut Rng) {
        let l = layout();
        self.grab = Grab::None;
        self.tip = 0.0;
        self.phase = Tip::Idle;
        self.hold_t = 0.0;
        self.spill_t = 0.0;
        self.pulse = 0.0;
        self.shake = 0.0;
        self.lamp = 0.0;
        self.hauled = 0;
        self.accum = 0.0;
        self.bird_a = 0.0;
        self.puffs = [(0.0, 0.0, 9.0); 6];
        self.deal(&l, rng);
    }

    fn relayout(&mut self, rng: &mut Rng) {
        let l = layout();
        if !self.ready {
            self.reset(rng);
            return;
        }
        // A rock in flight or in hand has nowhere sensible to go at a new
        // scale; put it down before rescaling.
        self.grab = Grab::None;
        for i in 0..ROCKS {
            if matches!(self.state[i], Rock::Held | Rock::Falling) {
                self.state[i] = Rock::Falling;
                self.vx[i] = 0.0;
            }
        }
        self.rescale(&l);
    }

    fn update(&mut self, dt: f32, rng: &mut Rng) -> bool {
        let l = layout();
        if !self.ready {
            self.deal(&l, rng);
        }
        self.moved = false;
        self.accum += if dt > 100.0 { 100.0 } else { dt };
        while self.accum >= 16.0 {
            self.accum -= 16.0;
            self.step(&l, 16.0, rng);
        }
        self.moved
    }

    fn pointer(&mut self, sx: f32, sy: f32, phase: u32, _rng: &mut Rng) {
        let l = layout();
        let x = sx + self.cam; // screen space into world space
        let y = sy;

        match phase {
            DOWN => {
                // A second contact cannot steal the first. The page filters
                // extra pointers too, but a small hand plants three fingers at
                // once and this is the layer that can actually be tested.
                if self.grab != Grab::None {
                    return;
                }
                self.moved_far = false;
                self.grab_x0 = x;
                self.grab_y0 = y;

                // Nearest grabbable rock wins, by squared distance — no sqrt.
                let mut pick = -1i32;
                // A wide, forgiving grab — except right on the cab, where a
                // nearby rock would otherwise steal the taps meant for the
                // horn. Gating on the whole truck instead would disable the
                // wide radius exactly where the rocks are once he has driven
                // over to them.
                let reach = if self.on_cab(&l, x, y) {
                    l.rock_r * 2.4
                } else {
                    l.rock_r * 3.8
                };
                let mut best = reach * reach;
                for i in 0..ROCKS {
                    // Only rocks on the ground. A rock in the bed must not be
                    // grabbable: it sits exactly where the hand lands to drive
                    // the truck, so it would steal every drive and every tap.
                    // A loaded bed is emptied by tipping it, which is also how
                    // the real thing works.
                    // Built rocks can be taken back out — nothing he makes is
                    // ever locked in.
                    let grabbable =
                        matches!(self.state[i], Rock::Ground | Rock::Built);
                    if !grabbable {
                        continue;
                    }
                    let dx = x - self.rx[i];
                    let dy = y - self.ry[i];
                    let d2 = dx * dx + dy * dy;
                    if d2 < best {
                        best = d2;
                        pick = i as i32;
                    }
                }

                if pick >= 0 {
                    let i = pick as usize;
                    self.state[i] = Rock::Held;
                    self.grab = Grab::RockIdx(i);
                    self.grab_off = self.rx[i] - x;
                    self.grab_offy = self.ry[i] - y;
                    sfx(audio::PICKUP, self.size[i]);
                    self.moved = true;
                } else if self.on_truck(&l, x, y) {
                    // Drag to drive, tap to tip — decided on release.
                    self.grab = Grab::Truck;
                    self.grab_off = self.tx - x;
                    self.drag_target = self.tx;
                    self.tv = 0.0;
                } else {
                    // No dead pixels: a touch on nothing still does something.
                    self.puff(x, y);
                    self.moved = true;
                }
            }

            MOVE => match self.grab {
                Grab::RockIdx(i) => {
                    let r = l.rock_r * self.size[i];
                    self.rx[i] = clamp(x + self.grab_off, r, l.world_w - r);
                    self.ry[i] = clamp(y + self.grab_offy, r, l.rock_line - r);
                    if fabs(x - self.grab_x0) + fabs(y - self.grab_y0) > l.u * 0.022 {
                        self.moved_far = true;
                    }
                    self.moved = true;
                }
                Grab::Truck => {
                    // Record the intent only. step() moves the truck and
                    // derives the velocity from real elapsed time.
                    let (dlo, dhi) = self.drive_range(&l);
                    self.drag_target = clamp(x + self.grab_off, dlo, dhi);
                    // Drag versus tap is decided by how far the FINGER moved,
                    // not the truck. Judging it by the truck means that once
                    // it is pinned against the end of the world every push
                    // reads as a tap and tips the load out by surprise.
                    if fabs(x - self.grab_x0) > l.u * 0.022 {
                        self.moved_far = true;
                    }
                    self.moved = true;
                }
                Grab::None => {}
            },

            UP => {
                match self.grab {
                    Grab::RockIdx(i) => {
                        if self.over_bed(&l, x, y)
                            && self.phase == Tip::Idle
                            && self.free_slot().is_some()
                        {
                            let s = self.free_slot().unwrap();
                            self.slot[i] = s;
                            self.state[i] = Rock::Seating;
                            self.set_count(self.count + 1);
                        } else if self.try_wall(&l, i) {
                            // laid straight into the wall
                        } else {
                            // Anywhere else is simply "on the ground". Never
                            // wrong, never refused.
                            self.state[i] = Rock::Falling;
                            if !self.moved_far {
                                self.vy[i] = 0.0;
                            }
                        }
                    }
                    Grab::Truck => {
                        if !self.moved_far {
                            // it was a tap
                            if self.on_cab(&l, x, y) {
                                sfx(audio::HORN, 1.0);
                                self.lamp = 1.0;
                                self.susp_v += 60.0;
                            } else {
                                self.start_tip();
                            }
                        }
                    }
                    Grab::None => {}
                }
                self.grab = Grab::None;
                self.moved = true;
            }
            _ => {}
        }
    }

    fn flip(&mut self) {
        self.start_turn();
    }

    fn can_flip(&self) -> bool {
        true
    }

    fn facing(&self) -> i32 {
        if self.faces_left() {
            -1
        } else {
            1
        }
    }

    fn score(&self) -> u32 {
        self.count
    }

    fn best(&self) -> u32 {
        self.hauled
    }

    fn draw(&mut self, fb: &mut Frame) {
        let l = layout();
        let (w, h) = (l.w as i32, l.h as i32);

        // Camera shake on impact, a couple of pixels at most.
        let sh = if self.shake > 0.0 {
            (self.shake * l.u * 0.012) as i32
        } else {
            0
        };
        let cam = self.cam;
        // world x -> screen x
        let sx = |x: f32| (x - cam) as i32;

        // ------------------------------------------------------------- sky
        fb.fill(SKY);

        let sky_top = self.cab_y(&l);
        if sky_top > l.u * 0.22 {
            // parallax: distant things move less than the ground
            cloud(fb, (l.w * 0.26 - cam * 0.15) as f32, sky_top * 0.28, l.u * 0.075);
            cloud(fb, (l.w * 0.95 - cam * 0.15) as f32, sky_top * 0.52, l.u * 0.055);
            cloud(fb, (l.w * 1.8 - cam * 0.15) as f32, sky_top * 0.36, l.u * 0.065);
        }

        if self.bird_a > 0.0 {
            let bx = sx(self.bird_x);
            let by = self.bird_y as i32;
            let f = ((self.bird_x / (l.u * 0.06)) as i32) % 2;
            let sp = (l.u * 0.016) as i32 + 1;
            fb.rect(bx - sp, by - if f == 0 { sp } else { 0 }, sp, 2, BIRD);
            fb.rect(bx + 1, by - if f == 0 { sp } else { 0 }, sp, 2, BIRD);
        }

        fb.rect(0, (l.horizon - l.u * 0.14) as i32, w, (l.u * 0.10) as i32, SKY_BAND);

        // far hills, slower than the ground
        let far = cam * 0.35;
        fb.disc(
            (l.w * 0.32 - far) as i32,
            (l.horizon + l.u * 0.10) as i32,
            (l.u * 0.30) as i32,
            HILL_FAR,
        );
        fb.disc(
            (l.w * 1.30 - far) as i32,
            (l.horizon + l.u * 0.08) as i32,
            (l.u * 0.26) as i32,
            HILL_FAR,
        );
        let near = cam * 0.62;
        fb.disc(
            (l.w * 0.85 - near) as i32,
            (l.horizon + l.u * 0.12) as i32,
            (l.u * 0.24) as i32,
            HILL,
        );
        fb.disc(
            (l.w * 1.75 - near) as i32,
            (l.horizon + l.u * 0.11) as i32,
            (l.u * 0.22) as i32,
            HILL,
        );

        // ---------------------------------------------------------- ground
        let gy = l.ground as i32 + sh;
        fb.rect(0, gy, w, h, DIRT);
        fb.rect(0, gy, w, (l.u * 0.016) as i32, DIRT_TOP);

        // the quarry face at the left end of the world
        // The quarry floor is the left end of the world: darker, dug out.
        let qx = sx(l.w * 0.52);
        if qx > 0 {
            let lip = (l.u * 0.05) as i32;
            fb.rect(0, gy, qx, lip, QUARRY_FACE);
            fb.rect(0, gy + lip, qx, h, QUARRY);
        }

        fb.rect(0, (l.rock_line - l.u * 0.028) as i32 + sh, w, h, DIRT);
        fb.rect(
            0,
            (l.rock_line - l.u * 0.028) as i32 + sh,
            w,
            (l.u * 0.010) as i32,
            DIRT_TOP,
        );

        // Bushes at fixed world spots: landmarks, so driving somewhere means
        // arriving somewhere rather than watching a brown line scroll.
        let bush_r = l.u * 0.035;
        let mut b = l.w * 0.95;
        while b < l.world_w {
            let bx = sx(b);
            if bx > -80 && bx < w + 80 {
                let by = (l.ground - bush_r * 0.55) as i32 + sh;
                fb.disc(bx - (bush_r * 0.8) as i32, by, (bush_r * 0.75) as i32, BUSH_DARK);
                fb.disc(bx + (bush_r * 0.8) as i32, by, (bush_r * 0.7) as i32, BUSH_DARK);
                fb.disc(bx, by - (bush_r * 0.4) as i32, bush_r as i32, BUSH);
            }
            b += l.w * 0.62;
        }

        let mut g = 0;
        while g < l.world_w as i32 {
            let px = sx(g as f32);
            if px > -20 && px < w + 20 {
                fb.rect(px, gy + 10 + (g % 17), 7, 4, DIRT_SPECK);
                // tyre tracks along the near dirt
                fb.rect(px, (l.rock_line + l.u * 0.045) as i32 + sh, 16, 4, TRACK);
                fb.rect(px + 6, (l.rock_line + l.u * 0.085) as i32 + sh, 12, 3, TRACK);
            }
            g += 34;
        }

        // --------------------------------------------------- the crushers
        for m in 0..MILLS {
            self.draw_crusher(fb, &l, sh, &sx, m);
        }

        // ------------------------------------------------------- the wall
        // Empty slots are drawn as pale outlines: he can see the shape of the
        // thing before it exists, and how much of it is left.
        for st in 0..SLOTS {
            if self.slot_taken(st) {
                continue;
            }
            let (wx, wy) = self.slot_pos_wall(&l, st);
            let ri = l.rock_r as i32;
            // A hollow chalk outline, not a filled shape: filled, it reads as
            // a pale rock already sitting there. Hollow, it reads as a space
            // waiting for one — which is the whole instruction.
            fb.rock(sx(wx), wy as i32 + sh, ri, 0x2A6B_13C5, CHALK);
            fb.rock(sx(wx), wy as i32 + sh, ri - 4, 0x2A6B_13C5, DIRT);
        }

        // ----------------------------------------------------------- truck
        self.draw_truck(fb, &l, sh);

        // Rocks on the ground and in the air are drawn after the truck: they
        // are in front of it, which is what makes them findable and grabbable
        // instead of hidden behind a wheel.
        for i in 0..ROCKS {
            if self.state[i] == Rock::Feeding {
                let r = l.rock_r * self.size[i] * (1.0 - self.feed_t[i] * 0.85);
                if r > 1.5 {
                    self.draw_rock_at(fb, i, sx(self.rx[i]), self.ry[i] as i32 + sh, r, false);
                }
                continue;
            }
            if matches!(
                self.state[i],
                Rock::Ground | Rock::Falling | Rock::Setting | Rock::Built
            ) {
                if matches!(self.state[i], Rock::Built | Rock::Setting) {
                    // A stone in the wall is dressed to fit its slot. Left at
                    // its own size the courses read as a heap someone tipped
                    // there rather than something built.
                    self.draw_rock_at(
                        fb,
                        i,
                        sx(self.rx[i]),
                        self.ry[i] as i32 + sh,
                        l.rock_r,
                        false,
                    );
                    if self.state[i] == Rock::Built {
                        let ri = l.rock_r as i32;
                        fb.rect(
                            sx(self.rx[i]) - ri,
                            self.ry[i] as i32 + sh + ri - 2,
                            ri * 2,
                            3,
                            MORTAR,
                        );
                    }
                } else {
                    self.draw_rock(fb, &l, i, sx(self.rx[i]), self.ry[i] as i32 + sh, false);
                }
            }
        }

        // the flag, once it is finished
        if self.flag > 0.0 {
            let (bx, by) = self.slot_pos_wall(&l, SLOTS - 2);
            let px = sx(bx);
            let base = by as i32 + sh - l.rock_r as i32;
            let tall = (l.u * 0.20 * self.flag) as i32;
            fb.rect(px, base - tall, 3, tall, POLE);
            let fw = (l.u * 0.075 * self.flag) as i32;
            let fh = (l.u * 0.05 * self.flag) as i32;
            for row in 0..fh {
                // a pennant: a triangle, drawn as narrowing rows
                let inset = (fw * (row - fh / 2).abs() * 2) / (fh.max(1));
                fb.rect(px + 3, base - tall + row, (fw - inset).max(1), 1, FLAG_C);
            }
        }
        if let Grab::RockIdx(i) = self.grab {
            self.draw_rock(fb, &l, i, sx(self.rx[i]), self.ry[i] as i32, true);
        }

        // ---------------------------------------------------------- puffs
        // No alpha in the framebuffer, so a puff fades by being blended
        // toward whatever it sits in front of — sky above the horizon, dirt
        // below it. Otherwise it just turns into an opaque blob and stops.
        for p in self.puffs.iter() {
            if p.2 < 1.8 {
                let t = ((p.2 / 1.8) * 255.0) as u32;
                let r = (l.u * 0.010 + p.2 * l.u * 0.013) as i32;
                if r <= 0 {
                    continue;
                }
                let (from, to) = if p.1 < l.ground {
                    (SMOKE, SKY)
                } else {
                    (DUST, DIRT)
                };
                let c = (
                    lerp(from.0, to.0, t),
                    lerp(from.1, to.1, t),
                    lerp(from.2, to.2, t),
                );
                fb.disc(sx(p.0), p.1 as i32, r, c);
            }
        }

        // -------------------------------------------------------------- UI
        let base = (l.u * 0.040) as i32;
        let bump = (self.pulse * l.u * 0.010) as i32;
        let scale = base + bump; // the number pulses when it changes
        let pad = (l.u * 0.05) as i32;
        fb.number(pad, pad, self.count, scale, NUMERAL);

        // capacity pips: how many more fit, without needing numbers
        let pip = (l.u * 0.026) as i32;
        for s in 0..BED_SLOTS {
            let c = if s < self.count as usize {
                PIP_FULL
            } else {
                PIP_EMPTY
            };
            fb.disc(pad + base * 5 + s as i32 * pip * 3, pad + base * 2, pip, c);
        }
    }
}

impl DumpTruck {
    fn draw_crusher<F: Fn(f32) -> i32>(
        &self,
        fb: &mut Frame,
        l: &L,
        sh: i32,
        sx: &F,
        m: usize,
    ) {
        let t = l.truck_w;
        let hopper = self.hopper_x(l, m);
        let hc = sx(hopper);
        if hc < -(t as i32) * 3 || hc > l.w as i32 + t as i32 * 3 {
            return; // off screen
        }
        let g = l.ground as i32 + sh;
        let (top_f, bot_f, half_f) = self.mouth(l);
        let hop_top = top_f as i32 + sh;
        let hop_bot = bot_f as i32 + sh;
        let half = half_f as i32;

        // Mill 0 stands at the left end and faces the other way, so the whole
        // machine is mirrored about its own mouth.
        let flip = m == 0;
        let dir = if flip { -1 } else { 1 };
        let jolt = if self.jaw[m] > 0.0 {
            ((self.jaw[m] * 11.0) as i32 % 3) - 1
        } else {
            0
        };
        // a span at `off` from the mouth, `w` wide, on the machine's side
        let sp = |off: i32, w: i32| -> (i32, i32) {
            let a = hc + dir * off;
            let b = hc + dir * (off + w);
            (if a < b { a } else { b }, (b - a).abs().max(1))
        };

        let body_w = (t * 0.52) as i32;
        let body_top = g - (t * 0.62) as i32;
        let (bx, bw) = sp(half + (t * 0.04) as i32, body_w);

        // ---- conveyor out to a heap on the far side of the mouth, where the
        // truck never parks. Behind the funnel, so the funnel stands in front.
        let heap_cx = hc - dir * (t * 0.42) as i32;
        let belt_hi = body_top + (t * 0.06) as i32;
        let belt_lo = g - (t * 0.22) as i32;
        let belt_from = if flip { bx + bw } else { bx };
        let steps = (belt_from - heap_cx).abs().max(1);
        let mut step = 0;
        while step < steps {
            let k = step as f32 / steps as f32;
            let px = heap_cx + (belt_from - heap_cx) * step / steps;
            let py = belt_lo + ((belt_hi - belt_lo) as f32 * k) as i32;
            fb.rect(px, py, 4, (t * 0.035) as i32, STEEL_DARK);
            fb.rect(px, py, 4, 3, STEEL_LIGHT);
            step += 4;
        }
        fb.rect(heap_cx + dir * (t * 0.10) as i32, belt_lo, 3, g - belt_lo, STEEL_DARK);

        // ---- the body
        fb.rect(bx + jolt, body_top, bw, g - body_top, STEEL);
        fb.rect(bx + jolt, body_top, bw, (t * 0.02) as i32, STEEL_LIGHT);
        fb.rect(bx + jolt, g - (t * 0.06) as i32, bw, (t * 0.02) as i32, STEEL_DARK);
        // The jaw sits high on the body on purpose. Lower down it is behind
        // the parked truck, and the one moment that shows the machine actually
        // doing something happens where nobody can see it.
        let open = ((t * 0.11) as i32 - (self.jaw[m] * t * 0.085) as i32).max(2);
        fb.rect(
            bx + jolt + bw / 7,
            body_top + (t * 0.07) as i32,
            bw * 5 / 7,
            open,
            MOUTH,
        );
        // a rocker above the jaw that drops as it bites
        let rock_y = body_top + (t * 0.02) as i32 + (self.jaw[m] * t * 0.03) as i32;
        fb.rect(bx + jolt + bw / 5, rock_y, bw * 3 / 5, (t * 0.03) as i32, STEEL_LIGHT);

        let mut hz = 0;
        while hz < bw - (t * 0.05) as i32 {
            fb.rect(
                bx + jolt + hz + 4,
                body_top + (t * 0.22) as i32,
                (t * 0.035) as i32,
                (t * 0.04) as i32,
                HAZARD,
            );
            hz += (t * 0.075) as i32;
        }
        // chimney on the outboard side
        let (cx2, cw2) = sp(half + (t * 0.04) as i32 + body_w - (t * 0.11) as i32, (t * 0.05) as i32);
        fb.rect(cx2, body_top - (t * 0.10) as i32, cw2, (t * 0.10) as i32, STEEL_DARK);

        // ---- the funnel, mouth low enough that a tipped load falls into it
        let depth = (hop_bot - hop_top).max(1);
        let wall = (t * 0.024) as i32 + 2;
        // legs
        fb.rect(hc - (t * 0.03) as i32, hop_bot, (t * 0.06) as i32, g - hop_bot, STEEL_DARK);
        for row in 0..depth {
            let k = row as f32 / depth as f32;
            let hw = (half as f32 * (1.0 - k * 0.78)) as i32;
            let y = hop_top + row;
            fb.rect(hc - hw, y, hw * 2, 1, MOUTH);
            fb.rect(hc - hw - wall, y, wall, 1, STEEL_LIGHT);
            fb.rect(hc + hw, y, wall, 1, STEEL_DARK);
        }
        fb.rect(hc - half - wall + jolt, hop_top - 4, (half + wall) * 2, 5, STEEL_LIGHT);
        fb.rect(hc - half - wall + jolt, hop_top + 1, (half + wall) * 2, 2, HAZARD);

        // ---- the gravel it has made
        let heap = self.gravel[m].min(48) as i32;
        if heap > 0 {
            let hh = ((t * 0.02) as i32 + (heap as f32 * t * 0.011) as i32).min((t * 0.30) as i32);
            let hw = hh * 3;
            for row in 0..hh {
                let k = row as f32 / hh as f32;
                let w2 = (hw as f32 * 0.5 * (1.0 - k * k)) as i32;
                fb.rect(
                    heap_cx - w2,
                    g + (l.rock_r * 0.5) as i32 - row,
                    w2 * 2,
                    1,
                    if row > hh - 3 { GRAVEL_DARK } else { GRAVEL },
                );
            }
            for k in 0..self.gems[m].min(6) {
                let gx = heap_cx - (l.rock_r * 0.9) as i32 + k as i32 * (l.rock_r * 0.62) as i32;
                let gy = g + (l.rock_r * 0.5) as i32 - hh - (l.rock_r * 0.30) as i32;
                fb.disc(gx, gy, (l.rock_r * 0.30) as i32, GEM);
                fb.rect(gx - 1, gy - 2, 3, 2, GEM_LIGHT);
            }
        }
    }

    fn draw_truck(&self, fb: &mut Frame, l: &L, sh: i32) {
        // Mid-turn the truck is squashed toward its own centre line; at the
        // halfway point it is edge-on and the facing has already swapped, so
        // it comes back out the other way round. No rotation maths, and it
        // reads exactly like a truck turning.
        let squash = {
            let t = 1.0 - 2.0 * self.turn;
            let a = if t < 0.0 { -t } else { t };
            if self.turn >= 1.0 {
                1.0
            } else if a < 0.04 {
                0.04
            } else {
                a
            }
        };
        let mid = self.tx + l.truck_w * 0.5;
        // world x -> screen x, squashed about the truck's centre line
        let sx = |x: f32| -> f32 { (mid + (x - mid) * squash) - self.cam };
        // a span, as an integer left edge and width of at least one pixel
        let seg = |a: f32, b: f32| -> (i32, i32) {
            let (p, q) = (sx(a), sx(b));
            let (lo, hi) = if p < q { (p, q) } else { (q, p) };
            let wpx = (hi - lo) as i32;
            (lo as i32, if wpx < 1 { 1 } else { wpx })
        };

        let flip = self.faces_left();
        let bed0 = self.bed_x(l);
        let cab0 = self.cab_x(l);
        let by = self.bed_y(l) as i32 + sh;
        let cy = self.cab_y(l) as i32 + sh;
        let ch_y = self.chassis_y(l) as i32 + sh;
        let bh = l.bed_h as i32;
        let ch = l.cab_h as i32;
        let near = (l.bed_h * 0.326) as i32;

        // chassis rail
        let (rx, rw) = seg(self.tx + 6.0, self.tx + l.truck_w - 6.0);
        fb.rect(rx, ch_y, rw, (l.wheel_r * 0.58) as i32, BODY_DARK);

        // wheels, with lugs that turn as it drives
        let wy = (l.ground + l.wheel_r * 0.24) as i32 + sh;
        // Offsets along a part, measured from its front. Mirrored when the
        // truck faces the other way, so the rear axles stay at the rear.
        let along = |base: f32, span: f32, frac: f32| -> f32 {
            base + span * if flip { 1.0 - frac } else { frac }
        };
        let wheels = [
            along(bed0, l.bed_w, 0.20),
            along(bed0, l.bed_w, 0.62),
            along(cab0, l.cab_w, 0.50),
        ];
        // seen edge-on a wheel is a slot, not a circle
        let wr = (l.wheel_r * (0.30 + 0.70 * squash)) as i32;
        for &wx in &wheels {
            let px = sx(wx) as i32;
            fb.disc(px, wy, wr, TIRE);
            fb.disc(px, wy, (wr as f32 * 0.46) as i32, HUB);
            let base = (self.wheel_a * 1.27) as i32;
            for k in 0..3 {
                let (dx, dy) = SPIN[(((base + k * 3) % 8) + 8) as usize % 8];
                fb.rect(
                    px + (dx * wr as f32 * 0.55) as i32 - 2,
                    wy + (dy * l.wheel_r * 0.26) as i32 - 2,
                    3,
                    3,
                    LUG,
                );
            }
        }

        // cab
        let (cx, cw) = seg(cab0, cab0 + l.cab_w);
        fb.rect(cx, cy, cw, ch, CAB_C);
        fb.rect(cx, cy + ch - ch / 8, cw, ch / 8, CAB_DARK);
        fb.rect(cx + cw / 7, cy + ch / 6, cw - cw / 3, ch / 3, GLASS);
        fb.rect(cx + cw / 7, cy + ch / 6 + ch / 4, cw - cw / 3, ch / 10, GLASS_DARK);
        // headlight on the leading face, whichever that is
        let lamp_w = (cw / 9).max(1);
        let lamp_x = if flip { cx - lamp_w / 2 } else { cx + cw - lamp_w / 2 };
        fb.rect(
            lamp_x,
            cy + ch * 5 / 8,
            lamp_w,
            ch / 7,
            if self.lamp > 0.0 { LAMP } else { LAMP_OFF },
        );
        // exhaust stack, on the cab's inboard side
        let stack = if flip {
            cab0 + l.cab_w * 0.80
        } else {
            cab0 + l.cab_w * 0.20
        };
        let (ex, ew) = seg(stack, stack + l.cab_w * 0.09);
        fb.rect(ex, cy - (l.u * 0.020) as i32, ew, (l.u * 0.022) as i32, BODY_DARK);

        // bed: back panel, then the load, then the near wall over the top of
        // it so the rocks read as being down inside
        let k = self.tip;
        let (bx, bw) = seg(bed0, bed0 + l.bed_w);
        let wall = (seg(bed0, bed0 + l.wall).1).max(1);

        fb.shear_rect_dir(bx, by, bw, bh, k, flip, BODY_DARK);
        for i in 0..ROCKS {
            if matches!(self.state[i], Rock::InBed | Rock::Seating) {
                let r = l.rock_r * self.size[i] * squash;
                self.draw_rock_at(
                    fb,
                    i,
                    sx(self.rx[i]) as i32,
                    self.ry[i] as i32 + sh,
                    if r < 2.0 { 2.0 } else { r },
                    false,
                );
            }
        }
        fb.shear_rect_dir(bx, by + bh - near, bw, near, k, flip, BODY);
        fb.shear_rect_dir(bx, by + bh - near, bw, near / 6, k, flip, BODY_LIGHT);
        fb.shear_rect_dir(bx, by, wall, bh, k, flip, BODY);
        fb.shear_rect_dir(bx + bw - wall, by, wall, bh, k, flip, BODY);
        fb.shear_rect_dir(bx, by, wall, bh / 13, k, flip, BODY_LIGHT);
        fb.shear_rect_dir(bx + bw - wall, by, wall, bh / 13, k, flip, BODY_LIGHT);
    }

    fn draw_rock(&self, fb: &mut Frame, l: &L, i: usize, cx: i32, cy: i32, held: bool) {
        self.draw_rock_at(fb, i, cx, cy, l.rock_r * self.size[i], held);
    }

    fn draw_rock_at(&self, fb: &mut Frame, i: usize, cx: i32, cy: i32, r: f32, held: bool) {
        let ri = r as i32;
        if ri < 2 || cx < -ri * 3 || cx > width() as i32 + ri * 3 {
            return;
        }

        let (body, light) = self.kind[i].colors();

        if held {
            fb.disc(cx, cy, ri + (r * 0.22) as i32, ROCK_HELD);
        }
        // The same shape again, darker and nudged down, so what shows beneath
        // is a shadowed edge in the rock's own colour. A disc here left a
        // rounded bottom poking out from under an angular rock.
        let dark = (
            (body.0 as u16 * 70 / 100) as u8,
            (body.1 as u16 * 70 / 100) as u8,
            (body.2 as u16 * 70 / 100) as u8,
        );
        fb.rock(cx, cy + (r * 0.16) as i32, ri, self.shape[i], dark);
        fb.rock(cx, cy, ri, self.shape[i], body);
        // the highlight facet turns as the rock tumbles
        let idx = ((self.facet[i] as i32 + (self.spin[i] as i32)) % 8 + 8) as usize % 8;
        let (dx, dy) = SPIN[idx];
        let u = (r * 0.34) as i32;
        fb.rect(
            cx + (dx * r * 0.34) as i32 - u / 2,
            cy + (dy * r * 0.34) as i32 - u / 2,
            u,
            u * 3 / 4,
            light,
        );
    }
}

/// Three overlapping discs. Enough of a cloud at this scale.
fn cloud(fb: &mut Frame, x: f32, y: f32, r: f32) {
    fb.disc((x - r * 0.9) as i32, y as i32, (r * 0.72) as i32, CLOUD);
    fb.disc(x as i32, (y - r * 0.3) as i32, r as i32, CLOUD);
    fb.disc((x + r * 1.0) as i32, y as i32, (r * 0.66) as i32, CLOUD);
    fb.rect(
        (x - r * 1.5) as i32,
        y as i32,
        (r * 3.0) as i32,
        (r * 0.7) as i32,
        CLOUD,
    );
}

