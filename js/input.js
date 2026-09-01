// Turning touches, keys and the D-pad into calls on the module.
//
// Exactly one finger drives the game. A small hand plants a palm, a thumb and
// two more fingers on the glass at once; without that rule every extra contact
// starts a fresh grab and the first finger's rock is left stranded mid-air.

import { DIRS, DOWN, MOVE, UP, gameAt } from "./config.js"
import { el, spendHint } from "./ui.js"

export function wireInput (canvas, { wasm, toFrame }, { onMenu }) {
  const turn = (dir) => wasm.turn(dir)
  const again = () => wasm.restart()

  /* ------------------------------------------------------------- keyboard */

  addEventListener("keydown", (e) => {
    if (e.key === "Escape") { onMenu(false); return }
    if (e.key in DIRS) {
      e.preventDefault()
      turn(DIRS[e.key])
    } else if (e.key === " " || e.key === "Enter") {
      if (wasm.status() === 2) { e.preventDefault(); again() }
    } else if (e.key === "Tab") {
      e.preventDefault()
      onMenu(true)            // the keyboard's way into the hidden menu
    }
  })

  /* --------------------------------------------------------- on-screen keys */

  el("again").addEventListener("click", again)

  for (const b of document.querySelectorAll(".pad button")) {
    // pointerdown, not click: a toddler lifts a finger somewhere else entirely
    b.addEventListener("pointerdown", (e) => {
      e.preventDefault()
      turn(Number(b.dataset.dir))
    })
  }

  el("turn").addEventListener("pointerdown", (e) => {
    e.preventDefault()
    wasm.flip()
    el("turn").classList.toggle("spun")
  })

  /* --------------------------------------------------------------- pointer */

  let sx = 0, sy = 0, active = null

  canvas.addEventListener("pointerdown", (e) => {
    if (active !== null) return             // a second finger is ignored
    e.preventDefault()
    canvas.setPointerCapture(e.pointerId)
    active = e.pointerId
    sx = e.clientX; sy = e.clientY
    spendHint()                             // he has acted; the instruction goes
    wasm.pointer(...toFrame(e), DOWN)
  })

  canvas.addEventListener("pointermove", (e) => {
    if (e.pointerId !== active) return
    e.preventDefault()
    wasm.pointer(...toFrame(e), MOVE)
  })

  function endPointer (e) {
    if (e.pointerId !== active) return
    active = null
    wasm.pointer(...toFrame(e), UP)

    // Swipe-to-turn, for the games that steer.
    if (!gameAt(wasm.current()).keys) return
    const dx = e.clientX - sx, dy = e.clientY - sy
    if (Math.hypot(dx, dy) < 24) {
      if (wasm.status() === 2) again()
      return
    }
    if (Math.abs(dx) > Math.abs(dy)) turn(dx > 0 ? 1 : 3)
    else turn(dy > 0 ? 2 : 0)
  }

  canvas.addEventListener("pointerup", endPointer)
  canvas.addEventListener("pointercancel", endPointer)
  canvas.addEventListener("lostpointercapture", (e) => {
    if (e.pointerId === active) active = null
  })
  // Last resort: a stuck id would ignore every future touch, leaving no input
  // and no sound with no way back short of a reload.
  for (const ev of ["pointerup", "pointercancel"]) {
    addEventListener(ev, (e) => { if (e.pointerId === active) active = null })
  }
}
