//! The framebuffer and everything that draws into it.
//!
//! `Frame` is a zero-sized handle onto one static RGBA buffer. Games take it by
//! `&mut` so the borrow checker still serialises drawing, but it costs nothing
//! to pass and the buffer itself stays in `.bss` — the same trick that keeps
//! the module small.

pub type Rgb = (u8, u8, u8);

/// The buffer is allocated once at the largest size we will ever render, and
/// the live frame is a packed `w * h` region at the front of it. Only the
/// occupied part is ever touched, so a small frame costs a small fill.
pub const MAX_W: usize = 1200;
pub const MAX_H: usize = 1200;
pub const MAX_PIXELS: usize = MAX_W * MAX_H * 4;

static mut FRAME: [u8; MAX_PIXELS] = [0; MAX_PIXELS];
static mut W: usize = 480;
static mut H: usize = 480;

#[inline]
pub fn width() -> usize {
    unsafe { core::ptr::read(core::ptr::addr_of!(W)) }
}

#[inline]
pub fn height() -> usize {
    unsafe { core::ptr::read(core::ptr::addr_of!(H)) }
}

/// Resize the live region. Clamped to the buffer we actually own, so a wrong
/// number from the host can never walk off the end.
pub fn set_size(w: usize, h: usize) {
    let w = if w < 16 { 16 } else if w > MAX_W { MAX_W } else { w };
    let h = if h < 16 { 16 } else if h > MAX_H { MAX_H } else { h };
    unsafe {
        core::ptr::write(core::ptr::addr_of_mut!(W), w);
        core::ptr::write(core::ptr::addr_of_mut!(H), h);
    }
}

pub struct Frame;

impl Frame {
    /// Single-threaded by construction: wasm calls in one at a time.
    pub fn new() -> Frame {
        Frame
    }

    #[inline]
    fn buf(&mut self) -> &'static mut [u8; MAX_PIXELS] {
        unsafe { &mut *core::ptr::addr_of_mut!(FRAME) }
    }

    pub fn ptr() -> *const u8 {
        core::ptr::addr_of!(FRAME) as *const u8
    }

    pub fn fill(&mut self, c: Rgb) {
        let live = width() * height() * 4;
        let buf = self.buf();
        let mut i = 0;
        while i < live {
            buf[i] = c.0;
            buf[i + 1] = c.1;
            buf[i + 2] = c.2;
            buf[i + 3] = 255;
            i += 4;
        }
    }

    pub fn rect(&mut self, x: i32, y: i32, w: i32, h: i32, c: Rgb) {
        if w <= 0 || h <= 0 {
            return;
        }
        let (fw, fh) = (width(), height());
        let x0 = if x < 0 { 0 } else { x as usize };
        let y0 = if y < 0 { 0 } else { y as usize };
        let x1 = core::cmp::min(fw, (x + w).max(0) as usize);
        let y1 = core::cmp::min(fh, (y + h).max(0) as usize);
        let buf = self.buf();
        for py in y0..y1 {
            let row = py * fw * 4;
            for px in x0..x1 {
                let i = row + px * 4;
                buf[i] = c.0;
                buf[i + 1] = c.1;
                buf[i + 2] = c.2;
                buf[i + 3] = 255;
            }
        }
    }

    /// A rect sheared vertically: column `i` sits `k * i` pixels higher than
    /// column 0. Cheaper than a rotation and, for a tipping truck bed, reads
    /// the same — no trigonometry anywhere in the module.
    pub fn shear_rect(&mut self, x: i32, y: i32, w: i32, h: i32, k: f32, c: Rgb) {
        for i in 0..w {
            let dy = (k * i as f32) as i32;
            self.rect(x + i, y - dy, 1, h, c);
        }
    }

    pub fn disc(&mut self, cx: i32, cy: i32, r: i32, c: Rgb) {
        if r <= 0 {
            return;
        }
        let rr = r * r;
        for dy in -r..=r {
            let py = cy + dy;
            if py < 0 || py >= height() as i32 {
                continue;
            }
            // Widest span at this row without a square root: walk in from the
            // edge until we are inside the circle.
            let mut dx = r;
            while dx > 0 && dx * dx + dy * dy > rr {
                dx -= 1;
            }
            self.rect(cx - dx, py, dx * 2 + 1, 1, c);
        }
    }

    /// A lumpy blob: four discs offset on an eighth-turn table, sized from the
    /// low bits of `seed`. Deterministic, so a rock keeps its shape, and enough
    /// to stop twelve rocks reading as twelve identical circles.
    pub fn blob(&mut self, cx: i32, cy: i32, r: i32, seed: u32, c: Rgb) {
        if r < 3 {
            self.disc(cx, cy, r, c);
            return;
        }
        const OFF: [(i32, i32); 4] = [(-1, 0), (1, 0), (0, -1), (0, 1)];
        self.disc(cx, cy, (r * 4) / 5, c);
        for (k, (dx, dy)) in OFF.iter().enumerate() {
            let bits = (seed >> (k * 3)) & 7;
            let lobe = (r * (5 + bits as i32)) / 12;
            let px = cx + dx * (r - lobe) / 2;
            let py = cy + dy * (r - lobe) / 2;
            self.disc(px, py, lobe, c);
        }
    }

    pub fn dim(&mut self, amount: u16) {
        let live = width() * height() * 4;
        let buf = self.buf();
        let mut i = 0;
        while i < live {
            buf[i] = ((buf[i] as u16 * amount) / 255) as u8;
            buf[i + 1] = ((buf[i + 1] as u16 * amount) / 255) as u8;
            buf[i + 2] = ((buf[i + 2] as u16 * amount) / 255) as u8;
            i += 4;
        }
    }

    // ------------------------------------------------------------- numerals

    /// 3x5 digits, drawn at integer scale. Big enough to be the whole UI.
    pub fn digit(&mut self, x: i32, y: i32, d: u32, scale: i32, c: Rgb) {
        let glyph = &DIGITS[(d % 10) as usize];
        for (row, bits) in glyph.iter().enumerate() {
            for col in 0..3 {
                if bits & (1 << (2 - col)) != 0 {
                    self.rect(
                        x + col * scale,
                        y + row as i32 * scale,
                        scale,
                        scale,
                        c,
                    );
                }
            }
        }
    }

    pub fn number(&mut self, x: i32, y: i32, n: u32, scale: i32, c: Rgb) {
        if n < 10 {
            self.digit(x, y, n, scale, c);
            return;
        }
        let mut digits = [0u32; 10];
        let mut count = 0;
        let mut v = n;
        while v > 0 && count < 10 {
            digits[count] = v % 10;
            v /= 10;
            count += 1;
        }
        for i in 0..count {
            let d = digits[count - 1 - i];
            self.digit(x + i as i32 * scale * 4, y, d, scale, c);
        }
    }
}

const DIGITS: [[u8; 5]; 10] = [
    [0b111, 0b101, 0b101, 0b101, 0b111],
    [0b010, 0b110, 0b010, 0b010, 0b111],
    [0b111, 0b001, 0b111, 0b100, 0b111],
    [0b111, 0b001, 0b111, 0b001, 0b111],
    [0b101, 0b101, 0b111, 0b001, 0b001],
    [0b111, 0b100, 0b111, 0b001, 0b111],
    [0b111, 0b100, 0b111, 0b101, 0b111],
    [0b111, 0b001, 0b001, 0b001, 0b001],
    [0b111, 0b101, 0b111, 0b101, 0b111],
    [0b111, 0b101, 0b111, 0b001, 0b111],
];

// ------------------------------------------------------- no_std float helpers
// core has no f32::abs / min / max — those live in std. These are the only
// three we need, and avoiding sqrt entirely keeps the module dependency-free.

#[inline]
pub fn fabs(x: f32) -> f32 {
    if x < 0.0 {
        -x
    } else {
        x
    }
}

#[inline]
pub fn fmin(a: f32, b: f32) -> f32 {
    if a < b {
        a
    } else {
        b
    }
}

#[inline]
pub fn fmax(a: f32, b: f32) -> f32 {
    if a > b {
        a
    } else {
        b
    }
}

#[inline]
pub fn clamp(v: f32, lo: f32, hi: f32) -> f32 {
    fmin(fmax(v, lo), hi)
}

/// Linear interpolation on colour channels, 0..=255.
pub fn lerp(a: u8, b: u8, t: u32) -> u8 {
    (a as u32 + ((b as i32 - a as i32) * t as i32 / 255) as u32) as u8
}
