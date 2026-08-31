// The whole audio side: a WebAudio synth for every sound the module asks for,
// and the watchdog that keeps it alive on a phone.
//
// Nothing here is loaded from a file. Every noise is made on the spot, which
// is why the game has no assets at all.

const Sound = (() => {
  const OFF_KEY = "games.muted"
  let ac = null, master = null, noiseBuf = null
  let eng = null, engGain = null, engFilt = null
  let hyd = null, hydGain = null
  let mill = null, millGain = null, millFilt = null
  let lap = null, lapGain = null, lapFilt = null
  let silent = null

  // iPhones and iPads mute Web Audio outright when the ringer switch is set to
  // silent. A looping (silent) media element moves the page onto the media
  // playback path, which that switch does not gag. It is a workaround, not a
  // guarantee — behaviour varies by iOS version.
  const SILENT_WAV = "data:audio/wav;base64,UklGRrQBAABXQVZFZm10IBAAAAABAAEAQB8AAEAfAAABAAgAZGF0YZABAACAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICA"
  function holdSession () {
    if (silent) { silent.play().catch(() => {}); return }
    try {
      silent = document.createElement("audio")
      silent.setAttribute("playsinline", "")
      silent.loop = true
      silent.preload = "auto"
      silent.volume = 0.02
      silent.src = SILENT_WAV
      silent.play().catch(() => {})
    } catch { silent = null }
  }
  let revNext = 0
  // watchdog bookkeeping
  let lastClock = -1, stalls = 0, rebuilds = 0, played = 0, dropped = 0
  let muted = false
  try { muted = localStorage.getItem(OFF_KEY) === "1" } catch { muted = false }

  function teardown () {
    try { if (eng) eng.stop() } catch {}
    try { if (hyd) hyd.stop() } catch {}
    try { if (mill) mill.stop() } catch {}
    try { if (lap) lap.stop() } catch {}
    try { if (ac) ac.close() } catch {}
    ac = master = noiseBuf = null
    eng = engGain = engFilt = hyd = hydGain = null
    mill = millGain = millFilt = null
    lap = lapGain = lapFilt = null
    lastClock = -1; stalls = 0
  }

  // iOS can hand back a context that says "running" while its graph is dead —
  // typically after an interruption (a call, another app's audio, the screen
  // locking). Resuming does not revive it; only a fresh graph does.
  function rebuild () {
    rebuilds++
    teardown()
    const c = ensure()
    if (c && c.state !== "running") c.resume().catch(() => {})
    holdSession()
  }

  function ensure () {
    if (ac) return ac
    const C = window.AudioContext || window.webkitAudioContext
    if (!C) return null
    ac = new C()
    ac.onstatechange = () => {
      if (ac && ac.state === "suspended") ac.resume().catch(() => {})
    }

    master = ac.createGain()
    master.gain.value = muted ? 0 : 0.45
    master.connect(ac.destination)

    noiseBuf = ac.createBuffer(1, Math.floor(ac.sampleRate * 0.5), ac.sampleRate)
    const d = noiseBuf.getChannelData(0)
    for (let i = 0; i < d.length; i++) d[i] = Math.random() * 2 - 1

    // One oscillator held for the life of the page: the engine note. Its
    // pitch and volume follow the truck's speed, which arrives every frame.
    eng = ac.createOscillator()
    eng.type = "sawtooth"
    eng.frequency.value = 42
    engFilt = ac.createBiquadFilter()
    engFilt.type = "lowpass"
    engFilt.frequency.value = 260
    engFilt.Q.value = 6
    engGain = ac.createGain()
    engGain.gain.value = 0
    eng.connect(engFilt); engFilt.connect(engGain); engGain.connect(master)
    eng.start()

    // The hydraulics are held too, so the note lasts exactly as long as the
    // bed is moving instead of finishing early and leaving it rising silently.
    hyd = ac.createOscillator()
    hyd.type = "sawtooth"
    hyd.frequency.value = 120
    hydGain = ac.createGain()
    hydGain.gain.value = 0
    const hydFilt = ac.createBiquadFilter()
    hydFilt.type = "bandpass"
    hydFilt.frequency.value = 520
    hydFilt.Q.value = 3
    hyd.connect(hydFilt); hydFilt.connect(hydGain); hydGain.connect(master)
    hyd.start()

    // The crusher's rumble is held too, so it lasts exactly as long as the
    // machine is working. Filtered noise rather than a tone — a mill grinding
    // is not a note.
    mill = ac.createBufferSource()
    mill.buffer = noiseBuf
    mill.loop = true
    millFilt = ac.createBiquadFilter()
    millFilt.type = "lowpass"
    millFilt.frequency.value = 220
    millFilt.Q.value = 4
    millGain = ac.createGain()
    millGain.gain.value = 0
    mill.connect(millFilt); millFilt.connect(millGain); millGain.connect(master)
    mill.start()

    // The pond's own voice: always there under everything, rising when the
    // water has been disturbed.
    lap = ac.createBufferSource()
    lap.buffer = noiseBuf
    lap.loop = true
    lapFilt = ac.createBiquadFilter()
    lapFilt.type = "bandpass"
    lapFilt.frequency.value = 420
    lapFilt.Q.value = 0.8
    lapGain = ac.createGain()
    lapGain.gain.value = 0
    lap.connect(lapFilt); lapFilt.connect(lapGain); lapGain.connect(master)
    lap.start()
    return ac
  }

  function tone (type, f0, f1, dur, gain, dest) {
    const o = ac.createOscillator(), g = ac.createGain()
    o.type = type
    o.frequency.setValueAtTime(f0, ac.currentTime)
    if (f1 !== f0) o.frequency.exponentialRampToValueAtTime(Math.max(1, f1), ac.currentTime + dur)
    g.gain.setValueAtTime(0.0001, ac.currentTime)
    g.gain.exponentialRampToValueAtTime(gain, ac.currentTime + 0.008)
    g.gain.exponentialRampToValueAtTime(0.0001, ac.currentTime + dur)
    o.connect(g); g.connect(dest || master)
    o.start(); o.stop(ac.currentTime + dur + 0.02)
  }

  function thud (dur, cut, gain) {
    const src = ac.createBufferSource(); src.buffer = noiseBuf
    const f = ac.createBiquadFilter(); f.type = "lowpass"; f.frequency.value = cut
    const g = ac.createGain()
    g.gain.setValueAtTime(gain, ac.currentTime)
    g.gain.exponentialRampToValueAtTime(0.0001, ac.currentTime + dur)
    src.connect(f); f.connect(g); g.connect(master)
    src.start(); src.stop(ac.currentTime + dur)
  }

  // Pentatonic, so the counting notes are always consonant however they land.
  const NOTES = [392, 440, 523, 587, 659, 784, 880]

  function play (id, p) {
    // Not resumed yet: the watchdog and the document-level unlock retry, so
    // there is no point hammering resume() sixty times a second from here.
    if (!ac || ac.state !== "running") { dropped++; return }
    played++
    switch (id) {
      case 0: tone("triangle", 300, 620, 0.10, 0.16); break            // pickup
      case 1: thud(0.16, 700, 0.30); tone("sine", 180, 110, 0.12, 0.16); break // seat
      case 2: {                                                          // hydraulics
        if (!hydGain) break
        const t = ac.currentTime
        hydGain.gain.setTargetAtTime(p > 0 ? 0.020 + p * 0.030 : 0, t, 0.06)
        hyd.frequency.setTargetAtTime(120 + p * 260, t, 0.07)
        break
      }
      case 3: thud(0.20 + p * 0.16, 380 + p * 900, 0.20 + p * 0.45)    // rock lands
        tone("sine", 150 - p * 40, 70, 0.16, 0.10 + p * 0.12); break
      case 4: tone("square", 392, 392, 0.26, 0.13)                     // horn
        tone("square", 294, 294, 0.26, 0.10); break
      case 5: {                                                          // engine
        if (!engGain) break
        const t = ac.currentTime
        engGain.gain.setTargetAtTime(0.030 + p * 0.075, t, 0.08)
        eng.frequency.setTargetAtTime(40 + p * 74, t, 0.09)
        engFilt.frequency.setTargetAtTime(220 + p * 900, t, 0.09)
        break
      }
      case 6: {                                                          // reversing
        if (p > 0.5 && ac.currentTime > revNext) {
          revNext = ac.currentTime + 0.62
          tone("square", 1050, 1050, 0.16, 0.07)
        }
        break
      }
      case 15: {                                                         // a stone hits the water
        const big = 0.3 + p * 0.7
        // the pitch drop is what makes it read as depth
        tone("sine", 620 - p * 260, 90, 0.18 + p * 0.10, 0.16 * big)
        thud(0.22 + p * 0.12, 520 + p * 900, 0.16 * big)
        setTimeout(() => thud(0.30, 2600, 0.05 * big), 60)   // the spray after
        break
      }
      case 16:                                                           // onto the bank
        thud(0.16, 420 + p * 300, 0.30)
        tone("sine", 150, 70, 0.12, 0.10)
        break
      case 17: {                                                         // the water, moving
        if (!lapGain) break
        const t = ac.currentTime
        lapGain.gain.setTargetAtTime(p > 0.02 ? 0.010 + p * 0.030 : 0.004, t, 0.30)
        lapFilt.frequency.setTargetAtTime(320 + p * 900, t, 0.30)
        break
      }
      case 18: {                                                         // a frog
        const base = 150 + p * 90
        tone("square", base, base * 0.72, 0.11, 0.13)
        setTimeout(() => tone("square", base * 1.25, base * 0.8, 0.15, 0.12), 120)
        break
      }
      case 19: {                                                         // a dragonfly
        const o = ac.createOscillator(), g = ac.createGain(), lfo = ac.createOscillator()
        const lg = ac.createGain()
        o.type = "sawtooth"
        o.frequency.value = 210
        lfo.type = "sine"
        lfo.frequency.value = 48                        // the wing flutter
        lg.gain.value = 60
        lfo.connect(lg); lg.connect(o.frequency)
        g.gain.setValueAtTime(0.0001, ac.currentTime)
        g.gain.exponentialRampToValueAtTime(0.06, ac.currentTime + 0.02)
        g.gain.exponentialRampToValueAtTime(0.0001, ac.currentTime + 0.5)
        o.connect(g); g.connect(master)
        o.start(); lfo.start()
        o.stop(ac.currentTime + 0.52); lfo.stop(ac.currentTime + 0.52)
        break
      }
      case 20:                                                           // something small
        tone("sine", 900 + p * 500, 500, 0.07, 0.07 + p * 0.05)
        break
      case 22:                                                           // a beetle
        thud(0.05, 3200, 0.10 + p * 0.12)
        tone("square", 2400 + p * 900, 1900, 0.03, 0.03)
        break
      case 23: {                                                         // wings
        const src = ac.createBufferSource(); src.buffer = noiseBuf
        const f = ac.createBiquadFilter(); f.type = "bandpass"
        f.frequency.value = 1400; f.Q.value = 1.6
        const g = ac.createGain()
        g.gain.setValueAtTime(0.0001, ac.currentTime)
        g.gain.exponentialRampToValueAtTime(0.05 * (0.4 + p), ac.currentTime + 0.03)
        g.gain.exponentialRampToValueAtTime(0.0001, ac.currentTime + 0.34)
        src.connect(f); f.connect(g); g.connect(master)
        src.start(); src.stop(ac.currentTime + 0.36)
        break
      }
      case 21: {                                                         // off in the trees
        const n = [1568, 1760, 2093][Math.floor(p * 3) % 3]
        tone("sine", n, n * 1.06, 0.09, 0.05)
        setTimeout(() => tone("sine", n * 1.18, n * 1.1, 0.07, 0.04), 110)
        break
      }
      case 12:                                                           // a rock goes into the jaws
        thud(0.26, 900 + p * 700, 0.42 + p * 0.30)
        tone("sawtooth", 150 - p * 30, 60, 0.22, 0.14)
        break
      case 13: {                                                         // the mill, while it works
        if (!millGain) break
        const t = ac.currentTime
        millGain.gain.setTargetAtTime(p > 0.02 ? 0.020 + p * 0.055 : 0, t, 0.10)
        millFilt.frequency.setTargetAtTime(180 + p * 520, t, 0.12)
        break
      }
      case 14:                                                           // a gem
        [0, 90, 190].forEach((d, i) =>
          setTimeout(() => tone("sine", [784, 1047, 1319][i], [784, 1047, 1319][i], 0.26, 0.13), d))
        break
      case 10:                                                           // stone set into the wall
        thud(0.12, 1500, 0.28)
        tone("triangle", 300 + p * 55, 260 + p * 55, 0.16, 0.13)
        break
      case 11: {                                                         // the wall is finished
        [0, 140, 280, 460].forEach((t, i) =>
          setTimeout(() => tone("triangle", [523, 659, 784, 1047][i], [523, 659, 784, 1047][i], 0.30, 0.16), t))
        break
      }
      case 9:                                                            // turning round
        tone("square", 300, 470, 0.16, 0.10)
        setTimeout(() => tone("square", 470, 300, 0.16, 0.08), 150)
        break
      case 8:                                                            // rock onto rock
        thud(0.13, 1900 + p * 2200, 0.16 + p * 0.30)
        tone("triangle", 520 - p * 90, 300, 0.09, 0.07 + p * 0.06)
        break
      case 7: {                                                          // the count changed
        const n = Math.max(0, Math.min(NOTES.length - 1, Math.round(p)))
        tone("triangle", NOTES[n], NOTES[n], 0.20, 0.13)
        break
      }
    }
  }

  return {
    // Browsers will not start audio until the user has touched the page, and
    // iOS in particular wants a source to have actually run inside the gesture
    // before it considers the graph live. Resuming alone is not enough there.
    unlock () {
      holdSession()
      const c = ensure()
      if (!c) return
      if (c.state === "suspended") c.resume().catch(() => {})
      try {
        const b = c.createBuffer(1, 1, c.sampleRate)
        const src = c.createBufferSource()
        src.buffer = b
        src.connect(c.destination)
        src.start(0)
      } catch {}
    },
    state: () => (ac ? ac.state : "none"),

    /// Called a few times a second from the frame loop. Everything about
    /// mobile audio that breaks after it has started working is caught here:
    /// a context that got suspended, and one that claims to be running while
    /// its clock stands still.
    watch () {
      if (!ac) return
      if (ac.state !== "running") {
        ac.resume().catch(() => {})
        holdSession()
        return
      }
      if (ac.currentTime === lastClock) {
        if (++stalls >= 3) { stalls = 0; rebuild() }
      } else {
        stalls = 0
        lastClock = ac.currentTime
      }
      if (silent && silent.paused) silent.play().catch(() => {})
    },

    info: () => ({
      state: ac ? ac.state : "none",
      clock: ac ? ac.currentTime.toFixed(1) : "-",
      rebuilds,
      played,
      dropped,
      media: silent ? (silent.paused ? "paused" : "playing") : "none"
    }),
    /// An unmistakable noise, for working out whether the problem is the game
    /// or the device.
    test () {
      this.unlock()
      if (!ac || ac.state !== "running") return false
      master.gain.value = 0.6
      tone("square", 523, 523, 0.18, 0.20)
      setTimeout(() => tone("square", 784, 784, 0.22, 0.20), 190)
      // back to the canonical level, not to whatever it happened to be —
      // pressing test twice used to latch the loud setting permanently
      setTimeout(() => { if (master) master.gain.value = muted ? 0 : 0.45 }, 500)
      return true
    },
    play,
    isMuted: () => muted,
    toggle () {
      muted = !muted
      try { localStorage.setItem(OFF_KEY, muted ? "1" : "0") } catch {}
      if (master) master.gain.value = muted ? 0 : 0.45
      return muted
    },
    suspend () { if (ac && ac.state === "running") ac.suspend().catch(() => {}) },
    resume ()  {
      if (ac && ac.state !== "running") ac.resume().catch(() => {})
      holdSession()
    }
  }
})()

export default Sound
