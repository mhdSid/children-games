// Everything that stops the page itself from moving, and everything about
// fitting the framebuffer to the screen.
//
// A small hand drags, presses, and plants three fingers at once. None of that
// should ever scroll, bounce, zoom, or select anything.

import { MAX_W, MAX_H, BASE } from "./config.js"

/** Kill every gesture the browser would otherwise act on itself. */
export function lockGestures () {
  const stop = (e) => e.preventDefault()
  document.addEventListener("touchmove",     stop, { passive: false })
  document.addEventListener("gesturestart",  stop, { passive: false })
  document.addEventListener("gesturechange", stop, { passive: false })
  document.addEventListener("contextmenu",   stop)
  document.addEventListener("dblclick",      stop, { passive: false })

  // iOS still nudges the page on rotate and on dismissing the keyboard.
  addEventListener("orientationchange", () => setTimeout(() => scrollTo(0, 0), 120))
}

/**
 * The framebuffer matches the viewport's aspect, so the canvas can fill the
 * screen without a letterbox and without stretching a pixel. Clamped to the
 * buffer the module actually owns, keeping the aspect honest.
 */
export function renderSize () {
  const vw = Math.max(1, innerWidth)
  const vh = Math.max(1, innerHeight)
  const ar = vw / vh

  let w, h
  if (ar <= 1) { w = BASE; h = Math.round(BASE / ar) }
  else         { h = BASE; w = Math.round(BASE * ar) }

  if (h > MAX_H) { h = MAX_H; w = Math.round(MAX_H * ar) }
  if (w > MAX_W) { w = MAX_W; h = Math.round(MAX_W / ar) }

  return [
    Math.max(16, Math.min(MAX_W, w)),
    Math.max(16, Math.min(MAX_H, h))
  ]
}

const fsEnabled = () =>
  document.fullscreenEnabled || document.webkitFullscreenEnabled

const inFullscreen = () =>
  document.fullscreenElement || document.webkitFullscreenElement

/** Returns false where the browser has no fullscreen for a plain element. */
export function canFullscreen () {
  return Boolean(fsEnabled())
}

export function toggleFullscreen () {
  const d = document.documentElement
  if (inFullscreen()) {
    (document.exitFullscreen || document.webkitExitFullscreen).call(document)
    return
  }
  Promise.resolve(
    (d.requestFullscreen || d.webkitRequestFullscreen).call(d, { navigationUI: "hide" })
  ).then(() => {
    // Both orientations work; landscape simply suits a side-on driving game
    // better, so ask for it and ignore a refusal.
    if (screen.orientation && screen.orientation.lock) {
      screen.orientation.lock("landscape").catch(() => {})
    }
  }).catch(() => {})
}
