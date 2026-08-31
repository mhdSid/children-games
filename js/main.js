// Wires the pieces together and runs the frame loop.

import Sound from "./sound.js"
import { load } from "./module.js"
import { lockGestures } from "./viewport.js"
import { applyGame, paintFrame, wireUi, openSheet, closeSheet, el } from "./ui.js"
import { wireInput } from "./input.js"

const canvas = el("board")

lockGestures()

// Any first touch anywhere unlocks audio — not just one landing on the canvas,
// which misses taps on the menu and any tap before the module has loaded.
for (const ev of ["pointerdown", "touchend", "click"]) {
  document.addEventListener(ev, () => Sound.unlock(), { capture: true })
}

// A page can come back from being hidden, from the bfcache, or from losing
// focus, and iOS suspends audio for all three.
const revive = () => { Sound.resume(); Sound.watch() }
document.addEventListener("visibilitychange", () => {
  if (document.hidden) Sound.suspend(); else revive()
})
addEventListener("pageshow", revive)
addEventListener("focus", revive)

load(canvas).then((game) => {
  const { wasm, fit, present } = game

  wireUi(wasm, {
    onPick: (id) => {
      if (id === wasm.current()) return
      wasm.select(id)
      applyGame(wasm, id)
    },
    onRestart: () => wasm.restart(),
    onLight: () => wasm.next_theme()
  })
  wireInput(canvas, game, {
    onMenu: (open) => (open ? openSheet() : closeSheet())
  })

  fit()
  applyGame(wasm, wasm.current())
  addEventListener("resize", () => { scrollTo(0, 0); fit() })

  let prev = performance.now()
  let frames = 0

  requestAnimationFrame(function loop (now) {
    const dt = now - prev
    prev = now

    // Four times a second is plenty to catch a dead audio context, and cheap
    // enough not to matter.
    if ((frames++ & 15) === 0) Sound.watch()

    present(dt)
    paintFrame(wasm)
    requestAnimationFrame(loop)
  })
}).catch((err) => {
  el("hint").hidden = false
  el("hint").textContent = "Module failed to load — serve this folder over HTTP."
  console.error(err)
})
