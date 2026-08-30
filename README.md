```
   ╔═══════════════════════════════════════════════════════╗
   ║                                                       ║
   ║    ██████ ██   ██ ██ ██   ██████  ██████ ███████      ║
   ║   ██      ██   ██ ██ ██   ██   ██ ██     ██   ██      ║
   ║   ██      ███████ ██ ██   ██   ██ █████  ███████      ║
   ║   ██      ██   ██ ██ ██   ██   ██ ██     ██   ██      ║
   ║    ██████ ██   ██ ██ ██████████  ██████ ██   ██       ║
   ║                                                       ║
   ║                   G  A  M  E  S                       ║
   ║                                                       ║
   ║           ·  I N S E R T   F I N G E R  ·             ║
   ║                                                       ║
   ╚═══════════════════════════════════════════════════════╝
```

# ▶ [P L A Y](https://mhdsid.github.io/children-games/)

**Two games. No adverts. No timers. No way to lose.**
Made for a two-and-a-half-year-old who likes anything with wheels.

Works on a phone, a tablet, or a laptop. Nothing to install.

---

```
    ╭───────────────────────────────────────────────╮
    │  G A M E   1                                  │
    ╰───────────────────────────────────────────────╯
```

## 🚚 LOAD THE DUMP TRUCK

```
                        ___________
          .-''''-.     |  o     o  |
         (  ROCK  )    |    ___    |___
          '-....-'     |___|   |______  \
     ____________________|  DUMP TRUCK |  |
    |  o   o   o   o   o |             |  |
    '--(O)-----(O)-------'---(O)-------(O)'
```

There is a quarry full of rocks and a big yellow truck.

| What he does | What happens |
| :-- | :-- |
| **Drags a rock** | it goes in the back of the truck, and the big number counts **UP** |
| **Drags the truck** | it drives! The wheels turn, the engine revs, the whole world scrolls past |
| **Taps the back** | the truck tips — rocks tumble out one at a time and the number counts **DOWN** … 4 · 3 · 2 · 1 · 0 |
| **Taps the cab** | 📣 **HONK!** and the headlight flashes |
| **Taps anything else** | a little puff of dust, because nothing should ever do nothing |

Rocks come in three sizes and four kinds now — granite, sandstone, basalt, and
one that is **not a rock at all**. Big ones land harder, bounce less, and roll
more slowly than little ones.

### ⭐ The good bit

**The rocks stay where he puts them.**

Drive across the world, tip out a load, and that pile is *still there* when he
comes back. Make one big mountain. Make five little ones. Line them up along
the road. Nobody tidies it away.

That's the whole game: carry things somewhere, and the world remembers.

### 🔎 Things to find

- One of the twelve rocks is **not a rock**. It's green and it sparkles. Keep looking.
- Birds fly past. Clouds drift. The truck puffs smoke while it waits.
- Drive right to the end of the world and see what's out there.

---

```
    ╭───────────────────────────────────────────────╮
    │  G A M E   2                                  │
    ╰───────────────────────────────────────────────╯
```

## 🐍 SNAKE

```
      ·  ·  ·  ·  ·  ·  ·  ·  ·  ·
      ·  ·  ▓▓▓▓▓▓▓▓▓▒  ·  ·  ·  ·
      ·  ·  ▓  ·  ·  ·  ·  ·  ·  ·
      ·  ·  ▓  ·  ·  ●  ·  ·  ·  ·
      ·  ·  ·  ·  ·  ·  ·  ·  ·  ·
```

The old one. Eat the apple, grow longer, don't bump into the walls.

**Swipe** the way you want to go, or use the **arrow buttons** at the bottom.
On a laptop, the **arrow keys** or **WASD**.

This one you *can* lose — and that's fine, it's the only one that can.

---

```
    ╭───────────────────────────────────────────────╮
    │  F O R   G R O W N - U P S                    │
    ╰───────────────────────────────────────────────╯
```

## 🔀 Swapping games

There is **no button on screen** — on purpose. A floating button is exactly the
thing a small hand finds and presses by accident.

> **Press and hold the top-left corner for about 2 seconds.**
> A ring fills up while you hold. Then pick a game.

Sliding your finger off cancels it. The menu closes itself after ten seconds.
On a laptop: <kbd>Tab</kbd> opens it, <kbd>Esc</kbd> closes it.

That menu is also where **Start a fresh world**, **Sound on / off** and
**Full screen** live.

## 📱 Which way up?

Both work. **Landscape suits it better** — it is a side-on driving game, so
turning the tablet sideways gives the truck room and makes it much bigger. Added
to the Home Screen it will ask for landscape by itself.

Turning the device over mid-game keeps everything: the pile he built is still
his pile, in the same place.

## 🔒 Handing over the tablet

The page locks itself down as much as a web page can: it will not scroll,
bounce, pinch-zoom, pull-to-refresh, or pop a text-selection magnifier.

For a proper lock, use the tablet's own:

- **iPad** — Settings ▸ Accessibility ▸ **Guided Access**, then triple-click the side button.
- **Android** — Settings ▸ Security ▸ **App pinning**.

## 📏 House rules

These are the rules the games are built to. They are not negotiable.

```
   ✓  every touch does something          ✗  no timers
   ✓  targets far bigger than they need   ✗  no countdowns
   ✓  near enough is good enough          ✗  no score, no streaks
   ✓  a sound for everything              ✗  no buzz when he's wrong
   ✓  he can always undo                  ✗  no reading required
   ✓  a good place to stop                ✗  no way to lose*
```

<sub>* except Snake, which is a game about losing, and he'll work that out.</sub>

## 🔊 About the sound

Every noise — the engine, the hydraulics, the horn, rocks landing, the counting
notes — is **made up on the spot** by the browser. There are no sound files.
Turn it off in the hold-the-corner menu if you're on a bus.

---
---

<details>
<summary><h2>🔧 Under the hood (for the other kind of reader)</h2></summary>

A small arcade cabinet in `no_std` Rust, compiled to one ~18 KB wasm module with
**exactly one import**. No wasm-bindgen, no generated glue, no allocator, no
dependencies. Both games share one framebuffer and one set of exports; switching
between them is a function call, not another download.

The single import is `host_sfx(id, param)`. For a child who cannot read, sound is
half the feedback channel, so the module gave up its import-free property to get
it. Nothing is loaded for it either: the host synthesises every sound from a
handful of WebAudio nodes.

### Run it

```sh
./serve.sh          # http://localhost:8080
```

`www/games.wasm` is prebuilt, so this works without a Rust toolchain.

### Build it

```sh
rustup target add wasm32-unknown-unknown
./build.sh
```

### How it works

Rust owns one static RGBA framebuffer, sized once at the largest frame it will
ever render and used as a packed `w * h` region at the front. Both games derive
every measurement from those two numbers, so the scene fills whatever shape the
screen is — portrait, landscape or square — with no letterbox and no stretched
pixel. Rust draws into it every frame; JavaScript wraps that region of linear
memory in a `Uint8ClampedArray`, hands it to `ImageData`, and calls
`putImageData`. Nothing is serialised and nothing is copied on the JS side.

```js
const view  = new Uint8ClampedArray(wasm.memory.buffer, wasm.frame_ptr(), w * h * 4);
const frame = new ImageData(view, w, h);
// each rAF:
wasm.tick(dt);
ctx.putImageData(frame, 0, 0);
```

### The interface

| Export | Meaning |
| --- | --- |
| `init(seed)` | seed the RNG, deal the first game |
| `game_count()` | how many games the module carries |
| `select(id)` | switch games and deal it fresh |
| `current()` | which game is on screen |
| `restart()` | deal the current game again |
| `turn(dir)` | 0 up, 1 right, 2 down, 3 left |
| `pointer(x, y, phase)` | finger in framebuffer space; 0 down, 1 move, 2 up |
| `resize(w, h)` | reshape the framebuffer and re-deal |
| `tick(dt) -> u32` | advance `dt` ms, redraw, return 1 if anything moved |
| `frame_ptr()` | byte offset of the framebuffer |
| `frame_w()` / `frame_h()` | current framebuffer dimensions |
| `score()` / `best()` | counters, meaning set by the current game |
| `status()` | 0 waiting, 1 running, 2 over |
| `host_sfx(id, param)` | **imported**: the host plays a sound |

### Layout

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
www/
  index.html          the shell: viewport lock, WebAudio synth, corner-hold menu
  games.wasm          prebuilt
```

### Adding a game

1. Write `src/games/yours.rs` with a `pub struct` and `impl Game`. The trait needs
   `reset`, `update` and `draw`; `pointer`, `key`, `score`, `best` and `status`
   have defaults, so a pointer-only game ignores the keyboard for free and a game
   with no fail state never reports `status() == 2`.
2. Add `pub mod yours;`, a `static mut`, and a match arm in `src/games/mod.rs`,
   then bump `COUNT`.
3. Add an entry to `GAMES` in `www/index.html` — its title, whether it wants the
   D-pad and swipe reader (`keys`), whether it shows the score readout, and its
   hint text.

Read `engine::width()` and `engine::height()` rather than assuming a size, and
derive the layout from them. `reset` is called on every resize for this reason.

Every game gets its own `static`. Dispatch hands back a `&mut dyn Game`, which
costs a vtable and no allocator — that is what lets several games live in one
module and switch with no fetch.

### Notes on the Rust side

- **`no_std`, no allocator.** Snake is a fixed ring buffer of cell indices plus a
  `[bool; N]` occupancy map, so self-collision is O(1). The dump truck is a
  handful of fixed-size arrays. There is no heap anywhere.
- **The framebuffer lives outside every game.** A zeroed static lands in `.bss`
  and costs nothing in the module; as a struct field on a partly-initialised
  value it would have been emitted as megabytes of literal zeroes.
- **No trigonometry, no square roots.** `core` has neither. The tipping bed is a
  *sheared* rectangle rather than a rotated one, wheels and tumbling rocks index
  an eight-entry unit-vector table, and grab tests compare squared distances. So
  the module needs no `libm` and no dependencies at all.
- **Vertical gaps scale with the short side.** Keying them to the width pushes
  the horizon off the top of a wide, short frame and leaves no room for a truck.
- **A rock only rests on rocks strictly below it.** Without that test two rocks at
  the same height each stack on the other, every frame, and the pair climbs off
  the top of the world at thirty pixels a frame. There is deliberately no sideways
  separation pass either: nudging settled rocks apart fights the stacking rule and
  walks a pile across the world a pixel at a time.
- **Drag versus tap is decided by how far the finger moved, not the truck.**
  Judging it by the truck means that once it is pinned against the end of the
  world, every push reads as a tap and tips the load out by surprise.
- **Rocks in the bed are not grabbable.** They sit exactly where a hand lands to
  drive, so they would steal every drive and every tap. A loaded bed is emptied by
  tipping, which is also how the real thing works.
- **Turns commit on a step, not on a keypress.** Queuing the direction and
  applying it inside `step()` means mashing two keys between steps can't fold the
  snake into itself.
- **Fixed timestep.** `tick` accumulates real elapsed time and drains it in fixed
  chunks, so both games run the same on a 60 Hz and a 120 Hz display. `dt` is
  clamped at 100 ms so a backgrounded tab doesn't teleport anything.
- **Determinism.** xorshift32 seeded from JS — same seed, same apples.

### Where to take it

Sound has been taken; persistence — so a pile survives a reload — would need a
second import. That is the point where reaching for `wasm-bindgen` starts paying
for itself, and this project stops well short of it.

Still open on the truck: terrain that is not a flat line (a hill to climb turns
driving from translation into a decision), a second vehicle, and somewhere for
the rocks to *go* — a crusher, a hole to fill — so the pile has a purpose beyond
existing.

### Checking it

```sh
node tools/probe.mjs        # 64 assertions, driven through the real module
node tools/shot.mjs 844 390 # render a frame to a PNG and look at it
```

`probe.mjs` inspects the framebuffer rather than mirroring the layout maths, so
it fails when the picture is wrong rather than when a constant moves. It found
every real bug in this game: rocks stacking on each other and climbing off the
top of the world, piles walking sideways, a drag being read as a tap at the edge
of the world, and rocks that were invisible behind the wheels.

### Deploying

`main` holds the source. The playable build is the contents of `www/` published
to the `gh-pages` branch:

```sh
./deploy.sh
```

It publishes everything in `www/`, and refuses to publish at all if the page's
script does not parse — a syntax error there is invisible until someone opens
the page, and by then it is live.

</details>
