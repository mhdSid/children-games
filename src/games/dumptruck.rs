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
use crate::engine::{clamp, fabs, height, lerp, sfx, width, Frame, Rgb, Rng, DOWN, MOVE, UP};
use crate::games::Game;

const ROCKS: usize = 12;
const BED_SLOTS: usize = 4;
const WORLD_SCALE: f32 = 3.2; // world is this many screens wide
const TIP_MAX: f32 = 0.62;

/// Unit vectors on an eighth turn, so wheels and tumbling rocks can rotate
/// without `sin` or `cos` — neither of which exists in `core`.
const SPIN: [(f32, f32); 8] = [
    (1.0, 0.0),
    (0.707, 0.707),
    (0.0, 1.0),
    (-0.707, 0.707),
    (-1.0, 0.0),
    (-0.707, -0.707),
    (0.0, -1.0),
    (0.707, -0.707),
];

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
const ROCK_DARK: Rgb = (94, 98, 102);
const ROCK_HELD: Rgb = (233, 200, 120);
const GEM: Rgb = (86, 160, 150);
const GEM_LIGHT: Rgb = (140, 205, 194);

const SMOKE: Rgb = (176, 186, 186);
const DUST: Rgb = (186, 164, 130);
const BIRD: Rgb = (66, 84, 90);
const BUSH: Rgb = (86, 122, 92);
const BUSH_DARK: Rgb = (66, 98, 72);
const TRACK: Rgb = (120, 94, 60);

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
    // empty brown, and sky with clouds in it reads far better than dirt.
    let ground = h - u * 0.44;
    let horizon = ground - u * 0.09;

    // The world is 2.4 screens wide, so the truck does not have to be small to
    // leave room to drive — it only has to fit above the horizon.
    let truck_w = {
        let by_w = w * 0.55;
        let by_h = ground * 0.95;
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
        rock_line: ground + u * 0.075,
        horizon,
        truck_w,
        bed_w: truck_w * 0.619,
        bed_h: truck_w * 0.274,
        cab_w: truck_w * 0.351,
        cab_h: truck_w * 0.351,
        wall: truck_w * 0.0357,
        wheel_r: truck_w * 0.101,
        rock_r: truck_w * 0.090,
    }
}

// -------------------------------------------------------------------- state

#[derive(Clone, Copy, PartialEq)]
enum Rock {
    Ground,
    Held,
    Seating,
    InBed,
    Falling,
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
    special: [bool; ROCKS], // one of them is worth finding

    // truck
    tx: f32,
    tv: f32,
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
            special: [false; ROCKS],
            tx: 0.0,
            tv: 0.0,
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
            puff_t: 0.0,
            puffs: [(0.0, 0.0, 9.0); 6],
            puff_i: 0,
            bird_x: 0.0,
            bird_y: 0.0,
            bird_a: 0.0,
            accum: 0.0,
            moved: false,
            ready: false,
        }
    }

    // ------------------------------------------------------------- geometry

    fn bed_x(&self, _l: &L) -> f32 {
        self.tx
    }
    fn cab_x(&self, l: &L) -> f32 {
        self.tx + l.bed_w + l.truck_w * 0.030
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
    fn truck_right(&self, l: &L) -> f32 {
        self.cab_x(l) + l.cab_w
    }

    fn slot_pos(&self, l: &L, s: usize) -> (f32, f32) {
        let across = l.bed_w - l.wall * 2.0 - l.rock_r * 2.0;
        let x = self.bed_x(l)
            + l.wall
            + l.rock_r
            + across * (s as f32 / (BED_SLOTS - 1) as f32);
        let y = self.bed_y(l) + l.bed_h * 0.42;
        (x, y - self.tip * (x - self.bed_x(l)))
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
        x > self.cab_x(l) && x < self.truck_right(l) && y > self.cab_y(l) && y < self.chassis_y(l)
    }

    // ---------------------------------------------------------------- setup

    fn deal(&mut self, l: &L, rng: &mut Rng) {
        // The quarry is the left end of the world; the rest is open ground to
        // build on.
        for i in 0..ROCKS {
            self.size[i] = 0.78 + rng.unit() * 0.5;
            self.facet[i] = (rng.next() % 4) as u8;
            self.spin[i] = 0.0;
            self.special[i] = false;
            self.state[i] = Rock::Ground;
            self.vx[i] = 0.0;
            self.vy[i] = 0.0;
            self.slot[i] = 0;

            // Spacing must clear the widest pair, or the stacking rule reads
            // them as piled on each other and builds a staircase instead of
            // laying them out. Biggest rock is 1.28r; 3.3r also leaves gaps to
            // see the truck through. The field fills the left of the world and
            // leaves the right open — somewhere to haul TO.
            self.rx[i] = l.rock_r * 2.0 + i as f32 * l.rock_r * 3.3;
            self.ry[i] = l.rock_line - l.rock_r * self.size[i];
        }
        // One rock is worth finding on the fiftieth play.
        self.special[(rng.next() as usize) % ROCKS] = true;

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
    }


    fn start_tip(&mut self) {
        if self.phase == Tip::Idle && self.in_bed_count() > 0 {
            self.phase = Tip::Squat;
            self.hold_t = 0.0;
            self.spill_t = 0.0;
            // anticipation: the truck settles on its springs before it lifts
            self.susp_v += 220.0;
            sfx(audio::TIP, 1.0);
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

        self.lamp = if self.lamp > 0.0 { self.lamp - s * 1.6 } else { 0.0 };
        self.pulse = if self.pulse > 0.0 { self.pulse - s * 2.2 } else { 0.0 };
        self.shake = if self.shake > 0.0 { self.shake - s * 5.0 } else { 0.0 };

        // ---------------------------------------------------------- driving
        if self.grab != Grab::Truck {
            // coast to a stop when he lets go
            self.tv *= 1.0 - 3.4 * s;
            self.tx += self.tv * s;
            self.tx = clamp(self.tx, 0.0, l.world_w - l.truck_w);
        }
        self.wheel_a += self.tv * s / (l.wheel_r * 0.9);

        // engine note follows speed; the host holds one oscillator for this
        let speed = {
            let v = fabs(self.tv) / (l.w * 0.9);
            if v > 1.0 {
                1.0
            } else {
                v
            }
        };
        sfx(audio::ENGINE, speed);
        sfx(
            audio::REVERSE,
            if self.tv < -l.w * 0.06 { 1.0 } else { 0.0 },
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

        // Rocks leave one at a time so the numeral ticks down rather than
        // snapping to zero. That tick is the whole counting lesson.
        if matches!(self.phase, Tip::Raising | Tip::Holding) && self.tip > TIP_MAX * 0.45 {
            self.spill_t += s;
            if self.spill_t > 0.16 {
                self.spill_t = 0.0;
                let mut pick = -1i32;
                let mut best = l.world_w * 2.0;
                for i in 0..ROCKS {
                    if self.state[i] == Rock::InBed {
                        let (sx, _) = self.slot_pos(l, self.slot[i]);
                        if sx < best {
                            best = sx;
                            pick = i as i32;
                        }
                    }
                }
                if pick >= 0 {
                    let i = pick as usize;
                    self.state[i] = Rock::Falling;
                    self.vx[i] = -(l.u * 0.18) - rng.unit() * l.u * 0.14;
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
                    self.vx[i] *= 1.0 - 1.5 * s;
                    self.spin[i] += self.vx[i] * s * 0.06;

                    let floor = self.rest_y(l, self.rx[i], self.ry[i], r, i);
                    if self.ry[i] >= floor {
                        self.ry[i] = floor;
                        let impact = fabs(self.vy[i]) / (l.u * 0.9);
                        if fabs(self.vy[i]) > l.u * 0.35 {
                            self.vy[i] = -self.vy[i] * 0.30;
                            self.vx[i] *= 0.55;
                            sfx(audio::LAND, if impact > 1.0 { 1.0 } else { impact });
                            self.shake = if impact > 0.6 { 0.6 } else { impact };
                            self.puff(self.rx[i], self.ry[i] + r * 0.6);
                        } else {
                            self.vy[i] = 0.0;
                            self.vx[i] = 0.0;
                            self.state[i] = Rock::Ground;
                            self.hauled += 1;
                        }
                    }
                    self.rx[i] = clamp(self.rx[i], r + 2.0, l.world_w - r - 2.0);
                    self.moved = true;
                }
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
        let lead = clamp(self.tv * 0.35, -l.w * 0.16, l.w * 0.16);
        let want = clamp(
            self.tx + l.truck_w * 0.5 - l.w * 0.5 + lead,
            0.0,
            l.world_w - l.w,
        );
        self.cam += (want - self.cam) * 4.0 * s;
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
                self.moved_far = false;
                self.grab_x0 = x;
                self.grab_y0 = y;

                // Nearest grabbable rock wins, by squared distance — no sqrt.
                let mut pick = -1i32;
                let mut best = (l.rock_r * 2.4) * (l.rock_r * 2.4);
                for i in 0..ROCKS {
                    // Only rocks on the ground. A rock in the bed must not be
                    // grabbable: it sits exactly where the hand lands to drive
                    // the truck, so it would steal every drive and every tap.
                    // A loaded bed is emptied by tipping it, which is also how
                    // the real thing works.
                    let grabbable = self.state[i] == Rock::Ground;
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
                    let want = clamp(x + self.grab_off, 0.0, l.world_w - l.truck_w);
                    let dx = want - self.tx;
                    // Drag versus tap is decided by how far the FINGER moved,
                    // not the truck. Judging it by the truck means that once
                    // it is pinned against the end of the world every push
                    // reads as a tap and tips the load out by surprise.
                    if fabs(x - self.grab_x0) > l.u * 0.022 {
                        self.moved_far = true;
                    }
                    // velocity comes from the drag, so the wheels and the
                    // engine note follow the hand
                    self.tv = dx * 60.0;
                    self.tx = want;
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

        // ----------------------------------------------------------- truck
        self.draw_truck(fb, &l, sh);

        // Rocks on the ground and in the air are drawn after the truck: they
        // are in front of it, which is what makes them findable and grabbable
        // instead of hidden behind a wheel.
        for i in 0..ROCKS {
            if matches!(self.state[i], Rock::Ground | Rock::Falling) {
                self.draw_rock(fb, &l, i, sx(self.rx[i]), self.ry[i] as i32 + sh, false);
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
    fn draw_truck(&self, fb: &mut Frame, l: &L, sh: i32) {
        let bx = (self.bed_x(l) - self.cam) as i32;
        let cx = (self.cab_x(l) - self.cam) as i32;
        let by = self.bed_y(l) as i32 + sh;
        let cy = self.cab_y(l) as i32 + sh;
        let ch_y = self.chassis_y(l) as i32 + sh;
        let (bw, bh) = (l.bed_w as i32, l.bed_h as i32);
        let (cw, ch) = (l.cab_w as i32, l.cab_h as i32);
        let wall = l.wall as i32;
        let near = (l.bed_h * 0.326) as i32;
        let wr = l.wheel_r as i32;

        // chassis rail
        fb.rect(
            bx + 6,
            ch_y,
            (self.truck_right(l) - self.bed_x(l)) as i32 - 12,
            (l.wheel_r * 0.58) as i32,
            BODY_DARK,
        );

        // wheels, with lugs that turn as it drives
        let wy = (l.ground + l.wheel_r * 0.24) as i32 + sh;
        let wheels = [
            self.bed_x(l) + l.bed_w * 0.30,
            self.bed_x(l) + l.bed_w * 0.72,
            self.cab_x(l) + l.cab_w * 0.46,
        ];
        for &wx in &wheels {
            let px = (wx - self.cam) as i32;
            fb.disc(px, wy, wr, TIRE);
            fb.disc(px, wy, (l.wheel_r * 0.46) as i32, HUB);
            // three lugs on an eighth-turn table: rotation without trig
            let base = (self.wheel_a * 1.27) as i32;
            for k in 0..3 {
                let (dx, dy) = SPIN[(((base + k * 3) % 8) + 8) as usize % 8];
                fb.rect(
                    px + (dx * l.wheel_r * 0.26) as i32 - 2,
                    wy + (dy * l.wheel_r * 0.26) as i32 - 2,
                    4,
                    4,
                    LUG,
                );
            }
        }

        // cab
        fb.rect(cx, cy, cw, ch, CAB_C);
        fb.rect(cx, cy + ch - ch / 8, cw, ch / 8, CAB_DARK);
        fb.rect(cx + cw / 7, cy + ch / 6, cw - cw / 3, ch / 3, GLASS);
        fb.rect(cx + cw / 7, cy + ch / 6 + ch / 4, cw - cw / 3, ch / 10, GLASS_DARK);
        fb.rect(
            cx + cw - cw / 14,
            cy + ch * 5 / 8,
            cw / 9,
            ch / 7,
            if self.lamp > 0.0 { LAMP } else { LAMP_OFF },
        );
        // exhaust stack
        fb.rect(cx + cw / 6, cy - (l.u * 0.020) as i32, cw / 12, (l.u * 0.022) as i32, BODY_DARK);

        // bed: back panel, load, then the near wall closes over it so the
        // rocks read as being down inside rather than balanced on top
        let k = self.tip;
        fb.shear_rect(bx, by, bw, bh, k, BODY_DARK);
        for i in 0..ROCKS {
            if matches!(self.state[i], Rock::InBed | Rock::Seating) {
                self.draw_rock(
                    fb,
                    l,
                    i,
                    (self.rx[i] - self.cam) as i32,
                    self.ry[i] as i32 + sh,
                    false,
                );
            }
        }
        fb.shear_rect(bx, by + bh - near, bw, near, k, BODY);
        fb.shear_rect(bx, by + bh - near, bw, near / 6, k, BODY_LIGHT);
        fb.shear_rect(bx, by, wall, bh, k, BODY);
        fb.shear_rect(bx + bw - wall, by, wall, bh, k, BODY);
        fb.shear_rect(bx, by, wall, bh / 13, k, BODY_LIGHT);
        fb.shear_rect(bx + bw - wall, by, wall, bh / 13, k, BODY_LIGHT);
    }

    fn draw_rock(&self, fb: &mut Frame, l: &L, i: usize, cx: i32, cy: i32, held: bool) {
        let r = l.rock_r * self.size[i];
        let ri = r as i32;
        if ri < 2 || cx < -ri * 3 || cx > l.w as i32 + ri * 3 {
            return;
        }

        let (body, light) = if self.special[i] {
            (GEM, GEM_LIGHT)
        } else {
            (ROCK_C, ROCK_LIGHT)
        };

        if held {
            fb.disc(cx, cy, ri + (r * 0.22) as i32, ROCK_HELD);
        }
        fb.disc(cx, cy, ri, body);
        fb.rect(
            cx - ri + ri / 5,
            cy + ri - ri / 3,
            ri * 2 - ri * 2 / 5,
            ri / 3,
            ROCK_DARK,
        );
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

