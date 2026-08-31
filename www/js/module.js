// The wasm module, and the one pointer that is the entire boundary between it
// and this page.

import Sound from "./sound.js"
import { renderSize } from "./viewport.js"

/**
 * Loads the module and returns a small object around it. The exports are used
 * directly everywhere else; this only owns the things the page has to keep in
 * step with them — the canvas size, and the ImageData view into wasm memory.
 */
export async function load (canvas) {
  const ctx = canvas.getContext("2d", { alpha: false })
  const imports = { env: { host_sfx: (id, p) => Sound.play(id, p) } }

  let instance
  try {
    ({ instance } = await WebAssembly.instantiateStreaming(fetch("games.wasm"), imports))
  } catch {
    // instantiateStreaming needs application/wasm; fall back if the server
    // sends something else.
    const bytes = await fetch("games.wasm").then((r) => r.arrayBuffer());
    ({ instance } = await WebAssembly.instantiate(bytes, imports))
  }

  const wasm = instance.exports
  wasm.init((Date.now() ^ (performance.now() * 1000)) >>> 0)

  let frame = null, fw = 0, fh = 0

  /**
   * Match the framebuffer to the space the page has for it. The module has no
   * allocator, so its memory never grows and the view never detaches.
   */
  function fit () {
    const [w, h] = renderSize()
    if (w === fw && h === fh) return

    fw = w; fh = h
    wasm.resize(w, h)          // re-deals or rescales, the game decides which
    canvas.width = w
    canvas.height = h

    const view = new Uint8ClampedArray(wasm.memory.buffer, wasm.frame_ptr(), w * h * 4)
    frame = new ImageData(view, w, h)
  }

  /** Advance and blit. Nothing is serialised; the canvas reads wasm memory. */
  function present (dt) {
    wasm.tick(dt)
    ctx.putImageData(frame, 0, 0)
  }

  /** Client coordinates into framebuffer space. */
  function toFrame (e) {
    const r = canvas.getBoundingClientRect()
    return [(e.clientX - r.left) * fw / r.width,
      (e.clientY - r.top)  * fh / r.height]
  }

  return { wasm, fit, present, toFrame }
}
