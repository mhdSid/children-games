//! Snake, sized to whatever frame it is given.
//!
//! The board is derived from the live framebuffer rather than baked in, so on a
//! tall phone it is a tall board. Cell size is chosen to keep the column count
//! in a sensible range at any aspect.

use crate::engine::{height, lerp, width, Frame, Rgb, Rng};
use crate::games::Game;

// Upper bounds on the board, which fix the size of the arrays below.
const MAX_COLS: usize = 34;
const MAX_ROWS: usize = 34;
const MAX_CELLS: usize = MAX_COLS * MAX_ROWS;

const MIN_CELL: usize = 12;

const BACKDROP: Rgb = (18, 24, 30);
const GRID_DOT: Rgb = (32, 44, 53);
const TAIL: Rgb = (63, 125, 110); // verdigris
const HEAD: Rgb = (201, 154, 46); // brass
const FOOD: Rgb = (200, 71, 31); // oxide
const FOOD_CORE: Rgb = (233, 138, 90);
const DIM: u16 = 90;

const UP: u8 = 0;
const RIGHT: u8 = 1;
const DOWN: u8 = 2;

const START_STEP_MS: f32 = 130.0;
const MIN_STEP_MS: f32 = 65.0;
const SPEEDUP_MS: f32 = 2.5;

pub struct Snake {
    body: [u16; MAX_CELLS],
    taken: [bool; MAX_CELLS],
    head: usize,
    len: usize,
    dir: u8,
    queued: u8,
    food: u16,

    cols: usize,
    rows: usize,
    cell: usize,
    ox: i32, // leftover pixels, split so the board sits centred
    oy: i32,

    score: u32,
    best: u32,
    dead: bool,
    started: bool,
    accum: f32,
    step_ms: f32,
    frames: u32,
}

impl Snake {
    pub const fn new() -> Snake {
        Snake {
            body: [0; MAX_CELLS],
            taken: [false; MAX_CELLS],
            head: 0,
            len: 0,
            dir: RIGHT,
            queued: RIGHT,
            food: 0,
            cols: 1,
            rows: 1,
            cell: MIN_CELL,
            ox: 0,
            oy: 0,
            score: 0,
            best: 0,
            dead: false,
            started: false,
            accum: 0.0,
            step_ms: START_STEP_MS,
            frames: 0,
        }
    }

    /// Pick a cell size that keeps the board inside the array bounds and the
    /// columns in a playable range, then centre whatever pixels are left over.
    fn measure(&mut self) {
        let (w, h) = (width(), height());

        // Size the cell off the frame's long and short sides rather than off
        // width alone, so a portrait phone and the same phone on its side get
        // the same chunky board instead of one tall thin one.
        let (long, short) = if w > h { (w, h) } else { (h, w) };
        let by_long = (long + 31) / 32; // at most 32 cells down the long side
        let by_short = (short + 17) / 18; // at most 18 across the short side
        let mut cell = if by_long > by_short { by_long } else { by_short };
        if cell < MIN_CELL {
            cell = MIN_CELL;
        }

        let mut cols = w / cell;
        let mut rows = h / cell;
        if cols > MAX_COLS {
            cols = MAX_COLS;
        }
        if rows > MAX_ROWS {
            rows = MAX_ROWS;
        }
        if cols < 6 {
            cols = 6;
        }
        if rows < 6 {
            rows = 6;
        }

        self.cell = cell;
        self.cols = cols;
        self.rows = rows;
        self.ox = (w as i32 - (cols * cell) as i32) / 2;
        self.oy = (h as i32 - (rows * cell) as i32) / 2;
    }

    fn cells(&self) -> usize {
        self.cols * self.rows
    }

    fn place_food(&mut self, rng: &mut Rng) {
        let cells = self.cells();
        if self.len >= cells {
            return;
        }
        loop {
            let candidate = (rng.next() as usize) % cells;
            if !self.taken[candidate] {
                self.food = candidate as u16;
                return;
            }
        }
    }

    fn tail_index(&self) -> usize {
        (self.head + MAX_CELLS - (self.len - 1)) % MAX_CELLS
    }

    fn step(&mut self, rng: &mut Rng) {
        // A turn only commits on a step, so mashing keys between steps can
        // never fold the snake back into itself.
        if (self.queued + 2) % 4 != self.dir {
            self.dir = self.queued;
        }

        let cell = self.body[self.head] as usize;
        let (mut x, mut y) = ((cell % self.cols) as i32, (cell / self.cols) as i32);
        match self.dir {
            UP => y -= 1,
            RIGHT => x += 1,
            DOWN => y += 1,
            _ => x -= 1,
        }

        if x < 0 || y < 0 || x >= self.cols as i32 || y >= self.rows as i32 {
            self.die();
            return;
        }

        let next = y as usize * self.cols + x as usize;
        let eating = next as u16 == self.food;
        let tail = self.body[self.tail_index()] as usize;

        // Running into the tail is fine when the tail is about to vacate.
        if self.taken[next] && !(next == tail && !eating) {
            self.die();
            return;
        }

        self.head = (self.head + 1) % MAX_CELLS;
        self.body[self.head] = next as u16;
        self.taken[next] = true;
        self.len += 1;

        if eating {
            self.score += 1;
            if self.score > self.best {
                self.best = self.score;
            }
            self.step_ms = if self.step_ms - SPEEDUP_MS > MIN_STEP_MS {
                self.step_ms - SPEEDUP_MS
            } else {
                MIN_STEP_MS
            };
            self.place_food(rng);
        } else {
            let t = self.tail_index();
            self.taken[self.body[t] as usize] = false;
            self.len -= 1;
        }
    }

    fn die(&mut self) {
        self.dead = true;
        if self.score > self.best {
            self.best = self.score;
        }
    }
}

impl Game for Snake {
    fn reset(&mut self, rng: &mut Rng) {
        self.measure();

        self.taken = [false; MAX_CELLS];
        self.len = 0;
        self.head = 0;
        self.dir = RIGHT;
        self.queued = RIGHT;
        self.score = 0;
        self.dead = false;
        self.started = false;
        self.accum = 0.0;
        self.step_ms = START_STEP_MS;

        let row = self.rows / 2;
        let start = self.cols / 2 - 2;
        for i in 0..3 {
            let cell = (row * self.cols + start + i) as u16;
            self.head = (self.head + 1) % MAX_CELLS;
            self.body[self.head] = cell;
            self.taken[cell as usize] = true;
            self.len += 1;
        }
        self.place_food(rng);
    }

    fn key(&mut self, dir: u32) {
        if dir < 4 && !self.dead {
            self.queued = dir as u8;
            self.started = true;
        }
    }

    fn update(&mut self, dt: f32, rng: &mut Rng) -> bool {
        let mut moved = false;
        if self.started && !self.dead {
            self.accum += if dt > 100.0 { 100.0 } else { dt };
            while self.accum >= self.step_ms && !self.dead {
                self.accum -= self.step_ms;
                self.step(rng);
                moved = true;
            }
        }
        moved
    }

    fn score(&self) -> u32 {
        self.score
    }

    fn best(&self) -> u32 {
        self.best
    }

    fn status(&self) -> u32 {
        if self.dead {
            2
        } else if self.started {
            1
        } else {
            0
        }
    }

    fn draw(&mut self, fb: &mut Frame) {
        self.frames = self.frames.wrapping_add(1);

        fb.fill(BACKDROP);

        let cell = self.cell as i32;
        let dot = if cell >= 18 { 2 } else { 1 };

        // faint dot at every cell centre: reads as a board, not a void
        for cy in 0..self.rows {
            for cx in 0..self.cols {
                let x = self.ox + cx as i32 * cell + cell / 2;
                let y = self.oy + cy as i32 * cell + cell / 2;
                fb.rect(x, y, dot, dot, GRID_DOT);
            }
        }

        // apple, breathing on a 60-frame triangle wave
        let phase = self.frames % 60;
        let swell = (if phase < 30 { phase } else { 60 - phase } / 15) as i32;
        let fx = self.ox + (self.food as usize % self.cols) as i32 * cell;
        let fy = self.oy + (self.food as usize / self.cols) as i32 * cell;
        let pad = cell / 5 - swell * (cell / 10);
        fb.rect(fx + pad, fy + pad, cell - pad * 2, cell - pad * 2, FOOD);
        fb.rect(
            fx + cell * 2 / 5,
            fy + cell * 2 / 5,
            cell / 5,
            cell / 5,
            FOOD_CORE,
        );

        // body, tail-to-head, colour ramping verdigris to brass
        let inset = if cell >= 16 { 2 } else { 1 };
        for i in 0..self.len {
            let idx = (self.head + MAX_CELLS - (self.len - 1 - i)) % MAX_CELLS;
            let c = self.body[idx] as usize;
            let x = self.ox + (c % self.cols) as i32 * cell;
            let y = self.oy + (c / self.cols) as i32 * cell;

            let t = if self.len > 1 {
                (i * 255 / (self.len - 1)) as u32
            } else {
                255
            };
            let col = (
                lerp(TAIL.0, HEAD.0, t),
                lerp(TAIL.1, HEAD.1, t),
                lerp(TAIL.2, HEAD.2, t),
            );
            fb.rect(x + inset, y + inset, cell - inset * 2, cell - inset * 2, col);
        }

        if self.dead {
            fb.dim(DIM);
        }
    }
}
