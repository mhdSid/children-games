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

/// Eight unit directions, an eighth-turn apart. Used for anything that needs
/// to point somewhere without `sin` and `cos`, neither of which exists in core.
pub const DIRS8: [(f32, f32); 8] = [
    (1.0, 0.0),
    (0.707, 0.707),
    (0.0, 1.0),
    (-0.707, 0.707),
    (-1.0, 0.0),
    (-0.707, -0.707),
    (0.0, -1.0),
    (0.707, -0.707),
];

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
    /// `from_right` moves the pivot to the far end, so a tipping bed can lift
    /// whichever way the truck happens to be facing.
    pub fn shear_rect_dir(
        &mut self,
        x: i32,
        y: i32,
        w: i32,
        h: i32,
        k: f32,
        from_right: bool,
        c: Rgb,
    ) {
        for i in 0..w {
            let idx = if from_right { w - 1 - i } else { i };
            let dy = (k * idx as f32) as i32;
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

    /// An eight-sided lump, each corner pushed out by a different amount taken
    /// from `seed`. Rocks are not balls: flat faces and corners read as stone
    /// where a disc reads as a bubble. Deterministic, so a rock keeps its shape
    /// for life.
    pub fn rock(&mut self, cx: i32, cy: i32, r: i32, seed: u32, c: Rgb) {
        if r < 4 {
            self.disc(cx, cy, r, c);
            return;
        }
        let mut px = [0i32; 8];
        let mut py = [0i32; 8];
        for k in 0..8 {
            let bits = ((seed >> (k * 3)) & 7) as f32;
            let rad = r as f32 * (0.72 + bits * 0.055);
            px[k] = cx + (DIRS8[k].0 * rad) as i32;
            py[k] = cy + (DIRS8[k].1 * rad) as i32;
        }

        let (mut ymin, mut ymax) = (py[0], py[0]);
        for k in 1..8 {
            if py[k] < ymin {
                ymin = py[k];
            }
            if py[k] > ymax {
                ymax = py[k];
            }
        }

        // Scanline fill: for each row, the span between the outermost edge
        // crossings. Half-open on y so a shared corner is counted once.
        for y in ymin..=ymax {
            let mut lo = i32::MAX;
            let mut hi = i32::MIN;
            for k in 0..8 {
                let j = (k + 1) % 8;
                let (y0, y1) = (py[k], py[j]);
                if (y0 <= y && y1 > y) || (y1 <= y && y0 > y) {
                    let t = (y - y0) as f32 / (y1 - y0) as f32;
                    let x = px[k] as f32 + (px[j] - px[k]) as f32 * t;
                    let xi = x as i32;
                    if xi < lo {
                        lo = xi;
                    }
                    if xi > hi {
                        hi = xi;
                    }
                }
            }
            if hi >= lo {
                self.rect(lo, y, hi - lo + 1, 1, c);
            }
        }
    }

    /// Filled ellipse. A pond seen from above is one, and so is every band of
    /// water inside it.
    pub fn fill_ellipse(&mut self, cx: i32, cy: i32, rx: i32, ry: i32, c: Rgb) {
        if rx < 1 || ry < 1 {
            return;
        }
        for dy in -ry..=ry {
            let y = cy + dy;
            if y < 0 || y >= height() as i32 {
                continue;
            }
            // In f32, not i32: rx*rx*ry*ry overflows a 32-bit integer at any
            // pond-sized radius, and the comparison then returns nonsense —
            // every row comes out full width and the ellipse draws as a slab.
            let fy = dy as f32 / ry as f32;
            let rem = 1.0 - fy * fy;
            if rem <= 0.0 {
                continue;
            }
            let mut dx = rx;
            while dx > 0 && {
                let fx = dx as f32 / rx as f32;
                fx * fx > rem
            } {
                dx -= 1;
            }
            self.rect(cx - dx, y, dx * 2 + 1, 1, c);
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

/// Fractional part, for anything that repeats.
#[inline]
pub fn frac(x: f32) -> f32 {
    let t = x - (x as i32) as f32;
    if t < 0.0 {
        t + 1.0
    } else {
        t
    }
}

/// One cycle of a wave over `t` in 0..1, peaking at +-1.
///
/// Two parabolic humps rather than a real sine: `core` has no trigonometry, a
/// lookup table would be 256 hand-written literals, and at the amplitude water
/// actually moves nobody can tell the difference.
pub fn wave(t: f32) -> f32 {
    let p = frac(t);
    let (sign, u) = if p < 0.5 {
        (1.0, p * 2.0)
    } else {
        (-1.0, (p - 0.5) * 2.0)
    };
    // The bare parabola is 6% out at the quarter turn, which is enough to make
    // a "circle" drawn from it look like a diamond. One refinement pass brings
    // it inside a tenth of a percent, which is far better than a pond needs.
    let s = 4.0 * u * (1.0 - u);
    sign * s * (0.775 + 0.225 * s)
}

/// The quarter-turn companion to `wave`, so a point can be put on a circle.
#[inline]
pub fn wave_q(t: f32) -> f32 {
    wave(t + 0.25)
}

/// Cheap deterministic hash, for scattering things that must look scattered
/// and then stay put.
#[inline]
pub fn hash(n: u32) -> u32 {
    let mut x = n.wrapping_mul(2654435761);
    x ^= x >> 15;
    x = x.wrapping_mul(2246822519);
    x ^= x >> 13;
    x
}

/// A ring drawn as broken dashes around an ellipse.
///
/// Ripples on real water are not circles anyone drew: they are arcs that fade
/// in and out, cross each other, and never quite close. Skipping runs of the
/// ring by a hash is what turns a geometric circle into one of those.
pub fn arc_dashes(
    fb: &mut Frame,
    cx: f32,
    cy: f32,
    rx: f32,
    ry: f32,
    seed: u32,
    thick: i32,
    c: Rgb,
) {
    if rx < 2.0 || ry < 2.0 {
        return;
    }
    let steps = ((rx + ry) * 1.6) as u32;
    if steps == 0 {
        return;
    }
    for i in 0..steps {
        // dashes about eight samples long, present roughly two times in three
        if hash(seed ^ (i / 8)) % 3 == 0 {
            continue;
        }
        let t = i as f32 / steps as f32;
        let x = cx + rx * wave_q(t);
        let y = cy + ry * wave(t);
        fb.rect(x as i32, y as i32, thick, thick, c);
    }
}

/// Ellipse outline, thickness `th`. Ripples on a pond seen at an angle are
/// ellipses, and drawing one directly is far cheaper than scaling a circle.
pub fn ellipse(fb: &mut Frame, cx: i32, cy: i32, rx: i32, ry: i32, th: i32, c: Rgb) {
    if rx < 1 || ry < 1 {
        return;
    }
    let inner_rx = rx - th;
    let inner_ry = ry - th;
    for dy in -ry..=ry {
        let y = cy + dy;
        if y < 0 || y >= height() as i32 {
            continue;
        }
        // widest x on this row, walked in from the edge — in f32, because the
        // integer form overflows at any pond-sized radius
        let fy = dy as f32 / ry as f32;
        let rem = 1.0 - fy * fy;
        if rem <= 0.0 {
            continue;
        }
        let mut dx = rx;
        while dx > 0 && {
            let fx = dx as f32 / rx as f32;
            fx * fx > rem
        } {
            dx -= 1;
        }
        let mut ix = 0;
        if inner_rx > 0 && inner_ry > 0 && dy.abs() < inner_ry {
            let iy = dy as f32 / inner_ry as f32;
            let irem = 1.0 - iy * iy;
            if irem > 0.0 {
                ix = inner_rx;
                while ix > 0 && {
                    let fx = ix as f32 / inner_rx as f32;
                    fx * fx > irem
                } {
                    ix -= 1;
                }
            }
        }
        if ix > 0 {
            fb.rect(cx - dx, y, dx - ix, 1, c);
            fb.rect(cx + ix, y, dx - ix, 1, c);
        } else {
            fb.rect(cx - dx, y, dx * 2 + 1, 1, c);
        }
    }
}

/// Linear interpolation on colour channels, 0..=255.
pub fn lerp(a: u8, b: u8, t: u32) -> u8 {
    (a as u32 + ((b as i32 - a as i32) * t as i32 / 255) as u32) as u8
}
