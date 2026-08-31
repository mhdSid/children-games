// The chrome: the readout, the hint, the D-pad, the turn button, and the menu
// hidden behind a corner hold.
//
// The guiding rule is that a two-year-old shares this screen. Anything he can
// press must either be part of the game or harmless; anything that is neither
// is not on screen at all.

import Sound from "./sound.js"
import { gameAt } from "./config.js"
import { canFullscreen, toggleFullscreen } from "./viewport.js"

export const el = (id) => document.getElementById(id)

/* ------------------------------------------------------------ the readout */

export function applyGame (wasm, id) {
  const g = gameAt(id)
  el("score-label").textContent = g.score
  el("best-label").textContent  = g.best
  el("pad").hidden  = !g.pad
  el("turn").hidden = !wasm.can_flip()
  el("hint").classList.toggle("above-pad", g.keys)
  el("readout").classList.toggle("on", g.readout);
  [...el("picks").querySelectorAll(".pick")].forEach((b, i) =>
    b.setAttribute("aria-pressed", String(i === id)))
}

/** Called every frame: the score, the hint, and whether to offer a restart. */
export function paintFrame (wasm) {
  const g     = gameAt(wasm.current())
  const state = wasm.status()
  const score = wasm.score()

  el("score").textContent = score
  el("best").textContent  = wasm.best()
  el("again").hidden      = state !== 2

  const text = g.hint(state, score)
  el("hint").textContent = text
  el("hint").hidden = text === ""
}

/* -------------------------------------------------------------- the menu */

let sheetTimer = 0

function paintDiag () {
  const i = Sound.info()
  el("diag").textContent =
    `audio ${i.state} · clock ${i.clock}s · played ${i.played} · dropped ${i.dropped}\n` +
    `rebuilds ${i.rebuilds} · media ${i.media}`
}

function paintMute () {
  paintDiag()
  const st = Sound.state()
  el("mute").textContent =
    Sound.isMuted()  ? "Sound off" :
      st === "running" ? "Sound on"  :
        "Sound on (tap to start)"
}

export function openSheet () {
  paintMute()
  el("sheet").classList.add("open")
  clearTimeout(sheetTimer)
  // Closes itself, so a sheet left open finds its way back to the game.
  sheetTimer = setTimeout(closeSheet, 10000)
}

export function closeSheet () {
  clearTimeout(sheetTimer)
  el("sheet").classList.remove("open")
}

/* ------------------------------------------------------- the corner hold
   The only way to change games. There is deliberately no button: a floating
   one is exactly the thing a small hand finds and presses. */

const HOLD_MS = 1600

function cornerHold (onDone) {
  let id = null, start = 0, raf = 0, hx = 0, hy = 0

  const end = () => {
    cancelAnimationFrame(raf)
    id = null
    el("ring").classList.remove("on")
    el("ring").style.setProperty("--p", 0)
  }

  const step = (now) => {
    const p = Math.min(1, (now - start) / HOLD_MS)
    el("ring").style.setProperty("--p", p.toFixed(3))
    if (p >= 1) { end(); onDone(); return }
    raf = requestAnimationFrame(step)
  }

  el("hold").addEventListener("pointerdown", (e) => {
    e.preventDefault()
    el("hold").setPointerCapture(e.pointerId)
    id = e.pointerId
    hx = e.clientX; hy = e.clientY
    start = performance.now()
    el("ring").classList.add("on")
    raf = requestAnimationFrame(step)
  })

  // Sliding off cancels: this has to be a deliberate, steady hold.
  el("hold").addEventListener("pointermove", (e) => {
    if (id === e.pointerId && Math.hypot(e.clientX - hx, e.clientY - hy) > 28) end()
  })
  for (const ev of ["pointerup", "pointercancel", "pointerleave"]) {
    el("hold").addEventListener(ev, (e) => { if (id === e.pointerId) end() })
  }
}

/* ------------------------------------------------------------------ wiring */

export function wireUi (wasm, { onPick, onRestart }) {
  const host = el("picks")
  host.replaceChildren()
  const wrap = document.createElement("div")
  wrap.style.display = "grid"
  wrap.style.gap = "10px"
  for (let i = 0; i < wasm.game_count(); i++) {
    const b = document.createElement("button")
    b.type = "button"
    b.className = "pick"
    b.textContent = gameAt(i).title
    b.setAttribute("aria-pressed", "false")
    b.addEventListener("click", () => { onPick(i); closeSheet() })
    wrap.append(b)
  }
  host.append(wrap)

  cornerHold(openSheet)

  el("close").addEventListener("click", closeSheet)
  el("sheet").addEventListener("click", (e) => {
    if (e.target === el("sheet")) closeSheet()
  })

  // Somewhere to start over. The truck game keeps everything he does, so
  // without this a world he is unhappy with is one he is stuck in.
  el("fresh").addEventListener("click", () => { onRestart(); closeSheet() })

  el("mute").addEventListener("click", () => { Sound.unlock(); Sound.toggle(); paintMute() })

  el("test").addEventListener("click", () => {
    const played = Sound.test()
    el("test").textContent =
      !played         ? "Blocked — tap again" :
        Sound.isMuted() ? "Muted — turn sound on" :
          "Heard nothing? Check the silent switch"
    setTimeout(() => { el("test").textContent = "Test sound"; paintMute() }, 3500)
  })

  if (canFullscreen()) {
    el("full").hidden = false
    el("full").addEventListener("click", toggleFullscreen)
  }
}
