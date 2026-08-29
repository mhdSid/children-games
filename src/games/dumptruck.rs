//! Load the Dump Truck.
//!
//! Drag rocks into the bed and a big numeral counts up. Tap the truck and the
//! bed tips, the rocks tumble out one at a time, and the numeral counts back
//! down — which is the actual point of the game.
//!
//! There is no way to lose, no timer, and no error state. Dropping a rock
//! somewhere silly just drops it on the ground, where it can be picked up
//! again.
//!
//! Every measurement is a fraction of the live framebuffer, so the scene fills
//! whatever shape it is handed.

use crate::engine::{clamp, fabs, height, width, Frame, Rgb, Rng, DOWN, MOVE, UP};
use crate::games::Game;

const ROCKS: usize = 5;
const TIP_MAX: f32 = 0.62; // shear factor at full tip

// ------------------------------------------------------------------- colours

const SKY: Rgb = (150, 190, 194);
const SKY_BAND: Rgb = (163, 201, 203);
const HILL: Rgb = (108, 146, 136);
const CLOUD: Rgb = (191, 216, 216);
const DIRT: Rgb = (134, 106, 68);
const DIRT_TOP: Rgb = (158, 128, 84);
const DIRT_SPECK: Rgb = (118, 92, 58);
const DIRT_FORE: Rgb = (116, 90, 56);

const BODY: Rgb = (201, 154, 46); // brass, straight from snake
const BODY_DARK: Rgb = (158, 116, 26);
const BODY_LIGHT: Rgb = (226, 186, 88);
const CAB_C: Rgb = (200, 71, 31); // oxide
const CAB_DARK: Rgb = (156, 50, 20);
const GLASS: Rgb = (172, 208, 210);
const GLASS_DARK: Rgb = (128, 168, 172);

const TIRE: Rgb = (38, 44, 48);
const HUB: Rgb = (186, 190, 194);
const HUB_DARK: Rgb = (140, 144, 148);

const ROCK_C: Rgb = (129, 133, 137);
const ROCK_LIGHT: Rgb = (166, 170, 174);
const ROCK_DARK: Rgb = (94, 98, 102);
const ROCK_HELD: Rgb = (233, 200, 120);

const NUMERAL: Rgb = (36, 46, 52);
const PIP_FULL: Rgb = (201, 154, 46);
const PIP_EMPTY: Rgb = (120, 152, 156);

// -------------------------------------------------------------------- layout

/// The whole scene, derived from the frame every time it is needed. Cheap
/// enough to recompute and it means nothing is baked to one screen shape.
struct L {
    w: f32,
    h: f32,
    rock_r: f32,
    grab_r: f32,
    ground_y: f32,
    fore_y: f32,
    rest_y: f32,
    bed_x: f32,
    bed_y: f32,
    bed_w: f32,
    bed_h: f32,
    wall: f32,
    cab_x: f32,
    cab_y: f32,
    cab_w: f32,
    cab_h: f32,
    chassis_y: f32,
    wheel_y: f32,
    wheel_r: f32,
}

fn layout() -> L {
    let w = width() as f32;
    let h = height() as f32;

    // The scene is built up from the bottom, and every vertical gap is a
    // fraction of the WIDTH. Keying them to the height instead would stretch
    // the truck away from the rocks on a tall phone and leave a dead field of
    // dirt between them; this way the composition stays tight and the extra
    // height all goes to sky.
    // Gaps scale with the short side. Using the width would collapse the
    // scene on a wide, short frame: the horizon ends up above the top of the
    // screen and there is no room left to draw a truck.
    let u = if w < h { w } else { h };
    let rock_r = u * 0.052;
    let rest_y = h - rock_r - u * 0.10;
    let fore_y = rest_y - rock_r - u * 0.07;
    let ground_y = fore_y - u * 0.14;

    // The truck is a fixed shape scaled to the width, capped so it cannot
    // overrun the sky on a short landscape frame.
    let truck_w = {
        let by_w = w * 0.80;
        let by_h = ground_y * 1.25;
        if by_w < by_h {
            by_w
        } else {
            by_h
        }
    };
    let truck_x = (w - truck_w) * 0.5;

    let bed_w = truck_w * 0.619;
    let cab_w = truck_w * 0.351;
    let bed_h = truck_w * 0.274;
    let cab_h = truck_w * 0.351;
    let wheel_r = truck_w * 0.101;

    let wheel_y = ground_y + wheel_r * 0.24;
    let chassis_y = wheel_y - wheel_r * 0.82;

    L {
        w,
        h,
        rock_r,
        // Grab radius is deliberately far larger than the rock. At this age
        // the intent is what matters, not the precision.
        grab_r: rock_r * 2.3,
        ground_y,
        fore_y,
        rest_y,
        bed_x: truck_x,
        bed_y: chassis_y - bed_h - truck_w * 0.006,
        bed_w,
        bed_h,
        wall: truck_w * 0.0357,
        cab_x: truck_x + bed_w + truck_w * 0.030,
        cab_y: chassis_y - cab_h,
        cab_w,
        cab_h,
        chassis_y,
        wheel_y,
        wheel_r,
    }
}

// -------------------------------------------------------------------- state

#[derive(Clone, Copy, PartialEq)]
enum Rock {
    Ground,
    Held,
    Seating, // flying to its slot in the bed
    InBed,
    Falling,
}

#[derive(Clone, Copy, PartialEq)]
enum Tip {
    Idle,
    Raising,
    Holding,
    Lowering,
}

pub struct DumpTruck {
    rx: [f32; ROCKS],
    ry: [f32; ROCKS],
    vx: [f32; ROCKS],
    vy: [f32; ROCKS],
    state: [Rock; ROCKS],
    slot: [usize; ROCKS],
    facet: [u8; ROCKS], // which way the rock's highlight sits

    held: i32, // index of the rock under the finger, -1 for none
    grab_dx: f32,
    grab_dy: f32,

    tip: f32,
    phase: Tip,
    hold_t: f32,
    spill_t: f32,

    count: u32,
    loaded_best: u32,
    accum: f32,
    moved: bool,
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
            held: -1,
            grab_dx: 0.0,
            grab_dy: 0.0,
            tip: 0.0,
            phase: Tip::Idle,
            hold_t: 0.0,
            spill_t: 0.0,
            count: 0,
            loaded_best: 0,
            accum: 0.0,
            moved: false,
        }
    }

    /// Where slot `s` sits, accounting for how far the bed is tipped.
    fn slot_pos(&self, l: &L, s: usize) -> (f32, f32) {
        let across = l.bed_w - l.wall * 2.0 - l.rock_r * 2.0;
        let x = l.bed_x + l.wall + l.rock_r + across * (s as f32 / (ROCKS - 1) as f32);
        let y = l.bed_y + l.bed_h * 0.5;
        // the bed shears upward toward the cab, so slots ride with it
        (x, y - self.tip * (x - l.bed_x))
    }

    fn free_slot(&self) -> Option<usize> {
        for s in 0..ROCKS {
            let mut taken = false;
            for i in 0..ROCKS {
                if (self.state[i] == Rock::InBed || self.state[i] == Rock::Seating)
                    && self.slot[i] == s
                {
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

    /// Generous drop zone: anywhere over the bed, plus a wide margin above it.
    fn over_bed(&self, l: &L, x: f32, y: f32) -> bool {
        let m = l.rock_r * 1.5;
        x > l.bed_x - m
            && x < l.bed_x + l.bed_w + m
            && y > l.bed_y - l.bed_h
            && y < l.bed_y + l.bed_h + m * 1.3
    }

    fn on_truck(&self, l: &L, x: f32, y: f32) -> bool {
        let m = l.rock_r * 0.5;
        x > l.bed_x - m
            && x < l.cab_x + l.cab_w + m
            && y > l.cab_y - m
            && y < l.ground_y + l.wheel_r
    }

    fn start_tip(&mut self) {
        if self.phase == Tip::Idle && self.count > 0 {
            self.phase = Tip::Raising;
            self.spill_t = 0.0;
        }
    }

    fn scatter(&mut self, l: &L, rng: &mut Rng) {
        for i in 0..ROCKS {
            self.state[i] = Rock::Ground;
            self.rx[i] = l.w * (i as f32 + 0.5) / ROCKS as f32;
            self.ry[i] = l.rest_y;
            self.vx[i] = 0.0;
            self.vy[i] = 0.0;
            self.slot[i] = 0;
            self.facet[i] = (rng.next() % 4) as u8;
        }
    }

    /// Push overlapping ground rocks apart along x only — no square roots, and
    /// chunky rocks resting slightly wrong looks fine.
    fn separate(&mut self, l: &L) {
        for _ in 0..2 {
            for a in 0..ROCKS {
                if self.state[a] != Rock::Ground {
                    continue;
                }
                for b in (a + 1)..ROCKS {
                    if self.state[b] != Rock::Ground {
                        continue;
                    }
                    let d = self.rx[b] - self.rx[a];
                    let min = l.rock_r * 2.0 - 4.0;
                    if fabs(d) < min {
                        let push = (min - fabs(d)) * 0.5;
                        let dir = if d < 0.0 { -1.0 } else { 1.0 };
                        self.rx[a] -= push * dir;
                        self.rx[b] += push * dir;
                    }
                }
            }
        }
        for i in 0..ROCKS {
            if self.state[i] == Rock::Ground {
                self.rx[i] = clamp(self.rx[i], l.rock_r + 4.0, l.w - l.rock_r - 4.0);
            }
        }
    }

    fn step(&mut self, l: &L, dt: f32, rng: &mut Rng) {
        let s = dt / 1000.0;

        // ---------------------------------------------------------- tipping
        match self.phase {
            Tip::Raising => {
                self.tip += s * 1.5 * TIP_MAX;
                if self.tip >= TIP_MAX {
                    self.tip = TIP_MAX;
                    self.phase = Tip::Holding;
                    self.hold_t = 0.0;
                }
                self.moved = true;
            }
            Tip::Holding => {
                self.hold_t += s;
                if self.hold_t > 0.55 && self.count == 0 {
                    self.phase = Tip::Lowering;
                }
                self.moved = true;
            }
            Tip::Lowering => {
                self.tip -= s * 1.3 * TIP_MAX;
                if self.tip <= 0.0 {
                    self.tip = 0.0;
                    self.phase = Tip::Idle;
                }
                self.moved = true;
            }
            Tip::Idle => {}
        }

        // Once the bed is well past halfway, rocks leave one at a time so the
        // numeral ticks 5, 4, 3, 2, 1, 0 instead of dropping straight to zero.
        if (self.phase == Tip::Raising || self.phase == Tip::Holding) && self.tip > TIP_MAX * 0.5 {
            self.spill_t += s;
            if self.spill_t > 0.13 {
                self.spill_t = 0.0;
                // spill the rock nearest the open (left) end first
                let mut pick = -1i32;
                let mut best = l.w * 2.0;
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
                    self.vx[i] = -(l.w * 0.15) - rng.unit() * l.w * 0.12;
                    self.vy[i] = -(l.h * 0.06) - rng.unit() * l.h * 0.08;
                    if self.count > 0 {
                        self.count -= 1;
                    }
                    self.moved = true;
                }
            }
        }

        // ---------------------------------------------------------- physics
        for i in 0..ROCKS {
            match self.state[i] {
                Rock::Falling => {
                    self.vy[i] += l.h * 2.9 * s;
                    self.rx[i] += self.vx[i] * s;
                    self.ry[i] += self.vy[i] * s;
                    self.vx[i] *= 1.0 - 1.6 * s;

                    if self.ry[i] >= l.rest_y {
                        self.ry[i] = l.rest_y;
                        if self.vy[i] > l.h * 0.4 {
                            self.vy[i] = -self.vy[i] * 0.32; // one small bounce
                            self.vx[i] *= 0.6;
                        } else {
                            self.vy[i] = 0.0;
                            self.vx[i] = 0.0;
                            self.state[i] = Rock::Ground;
                        }
                    }
                    self.rx[i] = clamp(self.rx[i], l.rock_r + 4.0, l.w - l.rock_r - 4.0);
                    self.moved = true;
                }
                Rock::Seating => {
                    let (tx, ty) = self.slot_pos(l, self.slot[i]);
                    self.rx[i] += (tx - self.rx[i]) * 14.0 * s;
                    self.ry[i] += (ty - self.ry[i]) * 14.0 * s;
                    if fabs(tx - self.rx[i]) < 1.5 && fabs(ty - self.ry[i]) < 1.5 {
                        self.rx[i] = tx;
                        self.ry[i] = ty;
                        self.state[i] = Rock::InBed;
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

        self.separate(l);
    }
}

impl Game for DumpTruck {
    fn reset(&mut self, rng: &mut Rng) {
        let l = layout();
        self.held = -1;
        self.tip = 0.0;
        self.phase = Tip::Idle;
        self.hold_t = 0.0;
        self.spill_t = 0.0;
        self.count = 0;
        self.accum = 0.0;
        self.scatter(&l, rng);
    }

    fn update(&mut self, dt: f32, rng: &mut Rng) -> bool {
        let l = layout();
        self.moved = false;
        // Fixed timestep, same accumulator idea as snake: identical behaviour
        // on a 60 Hz and a 120 Hz display.
        self.accum += if dt > 100.0 { 100.0 } else { dt };
        while self.accum >= 16.0 {
            self.accum -= 16.0;
            self.step(&l, 16.0, rng);
        }
        self.moved
    }

    fn pointer(&mut self, x: f32, y: f32, phase: u32, _rng: &mut Rng) {
        let l = layout();
        match phase {
            DOWN => {
                // Nearest grabbable rock wins, by squared distance — no sqrt.
                let mut pick = -1i32;
                let mut best = l.grab_r * l.grab_r;
                for i in 0..ROCKS {
                    // Ground rocks can always be picked up, even mid-dump —
                    // waiting out the tipping animation feels broken to a
                    // toddler. Rocks in the bed are only grabbable at rest.
                    let grabbable = match self.state[i] {
                        Rock::Ground => true,
                        Rock::InBed => self.phase == Tip::Idle,
                        _ => false,
                    };
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
                    if self.state[i] == Rock::InBed && self.count > 0 {
                        self.count -= 1;
                    }
                    self.state[i] = Rock::Held;
                    self.held = pick;
                    self.grab_dx = self.rx[i] - x;
                    self.grab_dy = self.ry[i] - y;
                    self.moved = true;
                } else if self.on_truck(&l, x, y) {
                    self.start_tip();
                }
            }
            MOVE => {
                if self.held >= 0 {
                    let i = self.held as usize;
                    self.rx[i] = clamp(x + self.grab_dx, l.rock_r, l.w - l.rock_r);
                    self.ry[i] = clamp(y + self.grab_dy, l.rock_r, l.rest_y);
                    self.moved = true;
                }
            }
            UP => {
                if self.held >= 0 {
                    let i = self.held as usize;
                    if self.over_bed(&l, x, y) && self.phase == Tip::Idle {
                        if let Some(s) = self.free_slot() {
                            self.slot[i] = s;
                            self.state[i] = Rock::Seating;
                            self.count += 1;
                            if self.count > self.loaded_best {
                                self.loaded_best = self.count;
                            }
                        } else {
                            self.state[i] = Rock::Falling;
                        }
                    } else {
                        // Anywhere else is simply "on the ground". Never wrong.
                        self.state[i] = Rock::Falling;
                        self.vx[i] = 0.0;
                        self.vy[i] = 0.0;
                    }
                    self.held = -1;
                    self.moved = true;
                }
            }
            _ => {}
        }
    }

    fn score(&self) -> u32 {
        self.count
    }

    fn best(&self) -> u32 {
        self.loaded_best
    }

    fn draw(&mut self, fb: &mut Frame) {
        let l = layout();
        let (w, h) = (l.w as i32, l.h as i32);

        // ------------------------------------------------------------ world
        fb.fill(SKY);

        // Clouds, placed in whatever sky the frame actually has above the cab.
        let sky = l.cab_y;
        let u0 = if l.w < l.h { l.w } else { l.h };
        if sky > u0 * 0.22 {
            cloud(fb, l.w * 0.26, sky * 0.30, u0 * 0.075);
            cloud(fb, l.w * 0.72, sky * 0.56, u0 * 0.055);
        }

        // A haze band sitting just above the horizon.
        let u = if l.w < l.h { l.w } else { l.h };
        fb.rect(0, (l.ground_y - u * 0.30) as i32, w, (u * 0.13) as i32, SKY_BAND);

        fb.disc(
            (l.w * 0.20) as i32,
            (l.ground_y + u * 0.04) as i32,
            (u * 0.31) as i32,
            HILL,
        );
        fb.disc(
            (l.w * 0.78) as i32,
            (l.ground_y + u * 0.06) as i32,
            (u * 0.275) as i32,
            HILL,
        );

        let dirt_top = (l.ground_y + l.wheel_r * 0.24) as i32;
        fb.rect(0, dirt_top, w, h, DIRT);
        fb.rect(0, dirt_top, w, (l.h * 0.018) as i32, DIRT_TOP);

        // A darker near bank, so the rock row reads as being in front of the
        // truck rather than beside it.
        fb.rect(0, l.fore_y as i32, w, h, DIRT_FORE);
        fb.rect(0, l.fore_y as i32, w, (l.h * 0.013) as i32, DIRT_TOP);

        // scattered grit in both bands, so neither is a flat slab
        let mut sx = 14;
        while sx < w {
            fb.rect(sx, dirt_top + 18 + (sx % 23), 7, 4, DIRT_SPECK);
            fb.rect(sx + 11, l.fore_y as i32 + 16 + (sx % 19), 6, 4, DIRT_SPECK);
            sx += 37;
        }

        // ------------------------------------------------------------ truck
        fb.rect(
            l.bed_x as i32 + 6,
            l.chassis_y as i32,
            (l.cab_x + l.cab_w - l.bed_x) as i32 - 12,
            (l.wheel_r * 0.58) as i32,
            BODY_DARK,
        );

        let wheels = [
            l.bed_x + l.bed_w * 0.375,
            l.bed_x + l.bed_w * 0.75,
            l.cab_x + l.cab_w * 0.457,
        ];
        for &wx in &wheels {
            fb.disc(wx as i32, l.wheel_y as i32, l.wheel_r as i32, TIRE);
            fb.disc(wx as i32, l.wheel_y as i32, (l.wheel_r * 0.46) as i32, HUB);
            fb.disc(wx as i32, l.wheel_y as i32, (l.wheel_r * 0.20) as i32, HUB_DARK);
        }

        // cab
        let (cx, cy, cw, ch) = (
            l.cab_x as i32,
            l.cab_y as i32,
            l.cab_w as i32,
            l.cab_h as i32,
        );
        fb.rect(cx, cy, cw, ch, CAB_C);
        fb.rect(cx, cy + ch - ch / 8, cw, ch / 8, CAB_DARK);
        fb.rect(cx + cw / 7, cy + ch / 6, cw - cw / 3, ch / 3, GLASS);
        fb.rect(cx + cw / 7, cy + ch / 6 + ch / 4, cw - cw / 3, ch / 10, GLASS_DARK);
        fb.rect(cx + cw - cw / 16, cy + ch * 5 / 8, cw / 10, ch / 7, BODY_LIGHT);

        // bed — sheared upward toward the cab as it tips
        let k = self.tip;
        let (bx, by, bw, bh) = (
            l.bed_x as i32,
            l.bed_y as i32,
            l.bed_w as i32,
            l.bed_h as i32,
        );
        let wall = l.wall as i32;
        let near = (l.bed_h * 0.326) as i32;

        // Back panel first — the inside of the box, so it sits darker.
        fb.shear_rect(bx, by, bw, bh, k, BODY_DARK);

        // Rocks in the bed go next, so the near wall can close over them.
        for i in 0..ROCKS {
            if self.state[i] == Rock::InBed || self.state[i] == Rock::Seating {
                draw_rock(fb, l.rock_r, self.rx[i], self.ry[i], self.facet[i], false);
            }
        }

        // Near wall and posts, drawn over the load: the rocks now read as
        // being down inside the bed rather than balanced on top of it.
        fb.shear_rect(bx, by + bh - near, bw, near, k, BODY);
        fb.shear_rect(bx, by + bh - near, bw, near / 6, k, BODY_LIGHT);
        fb.shear_rect(bx, by, wall, bh, k, BODY);
        fb.shear_rect(bx + bw - wall, by, wall, bh, k, BODY);
        fb.shear_rect(bx, by, wall, bh / 13, k, BODY_LIGHT);
        fb.shear_rect(bx + bw - wall, by, wall, bh / 13, k, BODY_LIGHT);

        // ------------------------------------------------------------ rocks
        // Everything not in the bed: on the ground, tumbling out, or in hand.
        for i in 0..ROCKS {
            match self.state[i] {
                Rock::Ground | Rock::Falling => {
                    draw_rock(fb, l.rock_r, self.rx[i], self.ry[i], self.facet[i], false)
                }
                _ => {}
            }
        }
        if self.held >= 0 {
            let i = self.held as usize;
            draw_rock(fb, l.rock_r, self.rx[i], self.ry[i], self.facet[i], true);
        }

        // --------------------------------------------------------------- UI
        // the count, big enough to be the whole interface
        let scale = (u * 0.040) as i32;
        let pad = (u * 0.05) as i32;
        fb.number(pad, pad, self.count, scale, NUMERAL);

        // capacity pips, so "how many more fit" is visible without numbers
        let pip = (u * 0.026) as i32;
        for s in 0..ROCKS {
            let filled = s < self.count as usize;
            let c = if filled { PIP_FULL } else { PIP_EMPTY };
            fb.disc(
                pad + scale * 5 + s as i32 * pip * 3,
                pad + scale * 2,
                pip,
                c,
            );
        }
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

fn draw_rock(fb: &mut Frame, r: f32, x: f32, y: f32, facet: u8, held: bool) {
    let cx = x as i32;
    let cy = y as i32;
    let ri = r as i32;

    if held {
        fb.disc(cx, cy, ri + (r * 0.22) as i32, ROCK_HELD);
    }
    fb.disc(cx, cy, ri, ROCK_C);
    // a flat dark base so it reads as sitting rather than floating
    fb.rect(cx - ri + ri / 5, cy + ri - ri / 3, ri * 2 - ri * 2 / 5, ri / 3, ROCK_DARK);
    // one highlight facet, rotated per rock so the five are not identical
    let u = ri / 2;
    let (hx, hy) = match facet {
        0 => (-u - u / 3, -u - u / 2),
        1 => (u / 3, -u - u / 2),
        2 => (-u - u / 2, u / 6),
        _ => (u / 6, -u / 2),
    };
    fb.rect(cx + hx, cy + hy, u, u * 3 / 4, ROCK_LIGHT);
}
