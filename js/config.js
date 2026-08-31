// What the page needs to know about each game in the wasm module.
//
// The module owns the rules and the pixels; this is only the chrome around
// them — what to call it, which controls it wants, and what to say underneath.

/** Pointer phases, matching `engine::{DOWN, MOVE, UP}` on the Rust side. */
export const DOWN = 0, MOVE = 1, UP = 2

/** Arrow and WASD keys to the module's direction numbering. */
export const DIRS = {
  ArrowUp: 0, ArrowRight: 1, ArrowDown: 2, ArrowLeft: 3,
  w: 0, d: 1, s: 2, a: 3, W: 0, D: 1, S: 2, A: 3
}

/** Must match `engine::frame::{MAX_W, MAX_H}`. */
export const MAX_W = 1200, MAX_H = 1200

/**
 * Short side of the framebuffer; the long side follows the viewport's aspect.
 * Everything in the games is a fraction of the frame, so this sets detail, not
 * how big anything looks.
 */
export const BASE = 480

/**
 * One entry per game, in the module's own order.
 *
 * `keys` decides whether the swipe reader and the arrow keys are live. `pad`
 * is separate on purpose: the on-screen D-pad is off because it is one more
 * thing on the glass for a small hand to find, and snake is perfectly
 * playable by swiping. Pointer events are always forwarded either way — a game
 * that ignores them inherits an empty `pointer` from the Game trait.
 */
export const GAMES = [
  {
    title: "Snake",
    keys: true,
    pad: false,
    readout: true,
    score: "Score", best: "Best",
    hint: (state, score) =>
      state === 0 ? "Swipe or tap a direction to start"
        : state === 1 ? ""
          :               "Game over — " + score + (score === 1 ? " apple" : " apples")
  },
  {
    title: "Dump Truck",
    keys: false,
    pad: false,
    readout: false,
    score: "Loaded", best: "Most",
    hint: (_state, score) =>
      score === 0 ? "Drag the rocks into the truck"
        : "Tap the truck to tip them out"
  }
]

export const gameAt = (id) => GAMES[id] || GAMES[0]
