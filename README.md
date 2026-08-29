# Games — Rust on WebAssembly

A small arcade cabinet in `no_std` Rust, compiled to one ~18 KB wasm module with
**exactly one import**. No wasm-bindgen, no generated glue, no allocator, no
dependencies. Several games share one framebuffer and one set of exports;
switching between them is a function call, not another download.

The single import is `host_sfx(id, param)`. For a child who cannot read, sound
is not decoration — it is half the feedback channel — so the module gave up its
import-free property to get it. Nothing is loaded for it either: the host
synthesises every sound with a handful of WebAudio nodes, so there are no audio
files, no loading and no licences.

Built for a two-year-old, so the board is edge to edge and the viewport is
locked: no scroll, no bounce, no pinch zoom, no pull-to-refresh. There is no
visible chrome at all — nothing on screen invites a stray tap.

## Run it

```sh
./serve.sh          # http://localhost:8080
```

`www/games.wasm` is prebuilt, so this works without a Rust toolchain.

## Build it

```sh
rustup target add wasm32-unknown-unknown
./build.sh
```

## The games

| # | Game | Input | Fail state |
| --- | --- | --- | --- |
| 0 | **Snake** | arrows, WASD, swipe, D-pad | yes |
| 1 | **Load the Dump Truck** | drag and tap | none |

**Load the Dump Truck** — a sandbox, not a task.

- **Drag a rock** into the bed and a large numeral counts up. Capacity is four.
- **Drag the truck** to drive it. The world is 3.2 screens wide; the camera
  follows, leading slightly in the direction of travel, with the hills and
  clouds on their own parallax layers.
- **Tap the bed** and it tips. The rocks tumble out one at a time, so the
  numeral ticks 4, 3, 2, 1, 0 rather than snapping to zero — that tick is the
  whole counting lesson.
- **Tap the cab** for the horn and the headlight.
- **Dumped rocks stay dumped.** They land where the truck was, stack into a
  pile, and are still there on the next trip. Hauling twelve rocks from the
  quarry to a spot of his choosing builds something that persists. This is the
  difference between a loop that resets and a world that remembers.

There is no timer, no score, no fail state and no error feedback. A rock
released anywhere is simply on the ground there. The grab radius is more than
twice the rock's own radius: at this age intent is what matters, not precision.

Craft details that cost little and change how it feels: the truck squats on its
springs before the bed lifts (anticipation), the suspension takes a real bump
when a rock lands in the bed, the wheels turn, the camera shakes on impact,
exhaust puffs so an untouched screen is never a still image, a bird crosses now
and then, rocks vary in size and tumble as they fall, and one of the twelve is
worth finding. Every touch does something, including a touch on empty sky.

## How it works

Rust owns one static RGBA framebuffer, sized once at the largest frame it will
ever render and used as a packed `w * h` region at the front. Both games derive
every measurement from those two numbers, so the scene fills whatever shape the
screen is — portrait, landscape, or square — without a letterbox or a stretched
pixel. Rust draws into it every frame. JavaScript wraps that region of linear memory in a `Uint8ClampedArray`,
hands it to `ImageData`, and calls `putImageData`. Nothing is serialised and
nothing is copied on the JS side; the canvas reads wasm memory directly.

```js
const view  = new Uint8ClampedArray(wasm.memory.buffer, wasm.frame_ptr(), side * side * 4);
const frame = new ImageData(view, side, side);
// each rAF:
wasm.tick(dt);
ctx.putImageData(frame, 0, 0);
```

## Switching games

There is no button. Hold the **top-left corner for 1.6 seconds** and a switcher
appears; a ring fills while you hold, and sliding off cancels it. A quick tap, a
resting palm or a stray grab never gets there, which is the point — a floating
button on screen is exactly the thing a two-year-old will find and press. The
switcher closes itself after ten seconds. On a desktop, <kbd>Tab</kbd> opens it
and <kbd>Esc</kbd> closes it.

### The interface

| Export | Meaning |
| --- | --- |
| `init(seed)` | seed the RNG, deal the first game |
| `game_count()` | how many games the module carries |
| `select(id)` | switch games and deal it fresh |
| `current()` | which game is on screen |
| `restart()` | deal the current game again |
| `turn(dir)` | 0 up, 1 right, 2 down, 3 left |
| `tick(dt) -> u32` | advance `dt` ms, redraw, return 1 if anything moved |
| `pointer(x, y, phase)` | finger in framebuffer space; 0 down, 1 move, 2 up |
| `resize(w, h)` | reshape the framebuffer and re-deal |
| `frame_ptr()` | byte offset of the framebuffer |
| `frame_w()` / `frame_h()` | current framebuffer dimensions |
| `score()` / `best()` | counters, meaning set by the current game |
| `host_sfx(id, param)` | **imported**: the host plays a sound |
| `status()` | 0 waiting, 1 running, 2 over |

## Layout

```
src/
  lib.rs              the host ABI and dispatch — the only extern "C" in the tree
  engine/
    frame.rs          framebuffer, rect, disc, sheared rect, numerals
    rng.rs            xorshift32
    audio.rs          the one import, and the sound ids
  games/
    mod.rs            the Game trait and the registry
    snake.rs
    dumptruck.rs
```

## Adding a game

1. Write `src/games/yours.rs` with a `pub struct` and `impl Game`. The trait
   needs `reset`, `update` and `draw`; `pointer`, `key`, `score`, `best` and
   `status` all have defaults, so a pointer-only game ignores the keyboard for
   free and a game with no fail state never reports `status() == 2`.
2. Add `pub mod yours;`, a `static mut`, and a match arm in
   `src/games/mod.rs`, then bump `COUNT`.
3. Add an entry to `GAMES` in `www/index.html` — its title, whether it wants
   the D-pad and swipe reader (`keys`), whether it shows the score readout, and
   its hint text.

Read `engine::width()` and `engine::height()` rather than assuming a size, and
derive the layout from them. `reset` is called on every resize for exactly this
reason.

Every game gets its own `static`. Dispatch hands back a `&mut dyn Game`, which
costs a vtable and no allocator — that is what lets several games live in one
module and switch with no fetch.

## Notes on the Rust side

- **`no_std`, no allocator.** Snake is a fixed ring buffer of 576 cell indices
  plus a `[bool; 576]` occupancy map, so self-collision is O(1). The dump truck
  is five fixed-size arrays. There is no heap anywhere.
- **The framebuffer lives outside every game.** A zeroed static lands in `.bss`
  and costs nothing in the module; as a struct field on a partly-initialised
  value it would have been emitted as megabytes of literal zeroes. That one move
  is the difference between a huge `.wasm` and a 16 KB one, even though the
  buffer itself is sized for a 1200 x 1200 frame.
- **No trigonometry, no square roots.** `core` has neither — they live in `std`.
  The tipping bed is a *sheared* rectangle rather than a rotated one, grab tests
  compare squared distances, and rock separation resolves along one axis. So the
  module needs no `libm` and no dependencies at all.
- **Turns commit on a step, not on a keypress.** Queuing the direction and
  applying it inside `step()` means mashing two keys between steps can't fold
  the snake into itself.
- **Fixed timestep.** `tick` accumulates real elapsed time and drains it in
  fixed chunks, so both games run the same on a 60 Hz and a 120 Hz display.
  `dt` is clamped at 100 ms so a backgrounded tab doesn't teleport anything.
- **Vertical gaps scale with the short side.** Keying them to the width instead
  pushes the horizon off the top of a wide, short frame and leaves no room to
  draw a truck. Every scene measurement is a fraction of `min(w, h)`.
- **A rock only rests on rocks strictly below it.** Without that test two rocks
  at the same height each stack on the other, every frame, and the pair climbs
  off the top of the world at thirty pixels a frame. There is deliberately no
  sideways separation pass either: nudging settled rocks apart fights the
  stacking rule and walks a pile across the world a pixel at a time.
- **Drag versus tap is decided by how far the finger moved, not the truck.**
  Judging it by the truck means that once it is pinned against the end of the
  world, every push reads as a tap and tips the load out by surprise.
- **Rocks in the bed are not grabbable.** They sit exactly where a hand lands to
  drive the truck, so they would steal every drive and every tap. A loaded bed
  is emptied by tipping, which is also how the real thing works.
- **Rocks are drawn in front of the truck, on a nearer ground line.** On the
  same line they vanish behind the wheels and read as buried.
- **Determinism.** xorshift32 seeded from JS — same seed, same apples.

## Where to take it

The `no_std` + framebuffer pattern scales a long way as long as Rust does all
the drawing. Sound has been taken; persistence — so a pile survives a reload —
would need a second import. That is the point where reaching for `wasm-bindgen`
starts paying for itself, and this project stops well short of it.

Still open on the truck: terrain that is not a flat line (a hill to climb
changes driving from translation into a decision), a second vehicle, and
somewhere for the rocks to *go* — a crusher, a hole to fill — so the pile has a
purpose beyond existing.
