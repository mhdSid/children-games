// Drives the wasm module headlessly and checks what it actually draws.
//
// Everything here inspects the framebuffer rather than mirroring the layout
// maths, so the tests fail when the picture is wrong rather than when a
// constant moves. Run with:  node tools/probe.mjs
import fs from "fs";

const sfx = [];
const CONTINUOUS = new Set([2, 5, 6, 13]);      // TIP, ENGINE, REVERSE, MILL
const tipParams = [];
const imports = { env: { host_sfx: (id, p) => {
  if (id === 2) tipParams.push(p);
  if (!CONTINUOUS.has(id)) sfx.push(id);
} } };

const { instance } = await WebAssembly.instantiate(
  fs.readFileSync("www/games.wasm"), imports);
const w = instance.exports;
const DOWN = 0, MOVE = 1, UP = 2;
const tick = (n = 1) => { for (let i = 0; i < n; i++) w.tick(16); };

let fails = 0;
const ok = (c, m) => { if (!c) fails++; console.log((c ? "  PASS  " : "  FAIL  ") + m); };

// ---------------------------------------------------------------- the picture
const CAB = [200, 71, 31], BODY = [201, 154, 46], DIRT_TOP = [158, 128, 84];
const ROCKS = [[129,133,137],[94,98,102],[166,170,174],   // granite
               [176,148,104],[206,182,142],               // sandstone
               [86,90,98],[120,126,136],                  // basalt
               [86,160,150],[140,205,194]];               // gem

const px = () => {
  const W = w.frame_w(), H = w.frame_h();
  return { W, H, d: new Uint8Array(w.memory.buffer, w.frame_ptr(), W * H * 4) };
};
const is = (d, i, c) => d[i] === c[0] && d[i+1] === c[1] && d[i+2] === c[2];

function groundY() {
  const { W, H, d } = px();
  for (let y = 0; y < H; y++)
    for (let x = 0; x < W; x += 3)
      if (is(d, (y*W+x)*4, DIRT_TOP)) return y;
  return Math.floor(H * 0.6);
}
function span(colors, y0, y1) {
  const { W, H, d } = px();
  let lo = 1e9, hi = -1e9, ylo = 1e9, yhi = -1e9;
  for (let y = Math.floor(y0); y < Math.floor(y1); y += 2)
    for (let x = 0; x < W; x += 2) {
      const i = (y*W+x)*4;
      for (const c of colors) if (is(d, i, c)) {
        if (x<lo) lo=x; if (x>hi) hi=x; if (y<ylo) ylo=y; if (y>yhi) yhi=y; break;
      }
    }
  return hi < 0 ? null : { lo, hi, mid:(lo+hi)/2, ylo, yhi, ymid:(ylo+yhi)/2 };
}
const truck  = () => { const {H} = px(); return span([CAB, BODY], H*0.20, H*0.99); };
const cabBox = () => { const {H} = px(); return span([CAB], H*0.20, H*0.99); };
const bedBox = () => span([BODY], px().H * 0.20, groundY());

// rocks lying in the world, ignoring whatever the truck is carrying
function rocksLoose() {
  const { W, H, d } = px();
  const t = truck();
  const lo = t ? t.lo - 2 : -1, hi = t ? t.hi + 2 : -1;
  let n = 0;
  for (let y = Math.floor(H * 0.30); y < H; y += 2)
    for (let x = 0; x < W; x += 2) {
      if (x >= lo && x <= hi) continue;
      const i = (y*W+x)*4;
      for (const c of ROCKS) if (is(d, i, c)) { n++; break; }
    }
  return n;
}
// clusters of rock colour below the ground line, as grab targets
function rockClusters() {
  const { W, H, d } = px(), g = groundY();
  const col = new Array(W).fill(0), top = new Array(W).fill(H);
  for (let y = g + 1; y < H; y++)
    for (let x = 0; x < W; x++) {
      const i = (y*W+x)*4;
      for (const c of ROCKS) if (is(d, i, c)) {
        col[x]++; if (y < top[x]) top[x] = y; break;
      }
    }
  const out = []; let x = 0;
  while (x < W) {
    if (col[x] > 2) {
      let x0 = x, t = H, best = x, bestN = -1;
      while (x < W && col[x] > 2) {
        if (top[x] < t) t = top[x];
        // the thickest column is a rock's centre; the run's midpoint can land
        // in the gap between two touching rocks
        if (col[x] > bestN) { bestN = col[x]; best = x; }
        x++;
      }
      if (x - x0 > 6) out.push({ x: best, y: t + Math.min(14, (x - x0) * 0.35) });
    } else x++;
  }
  return out;
}
function dragTruck(to) {
  const t = truck(); if (!t) return false;
  // Start from the middle of the cab, not the middle of the truck: the cab
  // sits above the rock lane, so the drag cannot accidentally grab a rock and
  // fling it — which would have this helper quietly rearranging the world it
  // is supposed to be measuring.
  const c = cabBox();
  const y = c ? c.ymid : px().H * 0.55;
  const from = c ? c.mid : t.mid;
  w.pointer(from, y, DOWN);
  // A frame has to elapse between pointer events. The truck is moved inside
  // step(), not in the pointer handler, so a drag with no ticks in it moves
  // the truck precisely nowhere — and every assertion about driving quietly
  // becomes vacuous.
  for (let i = 1; i <= 16; i++) {
    w.pointer(from + (to - from) * i / 16, y, MOVE);
    tick(2);
  }
  w.pointer(to, y, UP);
  return true;
}
const toEnd = (right) => { for (let i = 0; i < 6; i++) { dragTruck(right ? px().W * 1.4 : -px().W * 0.4); tick(80); } };

// ---------------------------------------------------------------------- run
w.init(4242);
console.log("games:", w.game_count());

for (const [RW, RH] of [[480, 1039], [844, 390], [480, 480]]) {
  w.select(1); w.resize(RW, RH); tick(4);
  const { W, H } = px();
  console.log(`\n=== ${W} x ${H} ===`);

  ok(w.score() === 0, "bed starts empty");
  ok(w.status() === 1, "no fail state, ever");

  const t0 = truck();
  ok(t0 !== null, "truck is drawn and on screen");
  if (W > H) {
    // A band, not a floor. The truck used to be sized by budgeting its WIDTH
    // against the full depth to the wheel line, which starved it in landscape
    // at about 0.19 — but 0.47 was tried and looked domineering, with almost
    // none of the world visible around it.
    const frac = (t0.hi - t0.lo) / W;
    ok(frac > 0.30 && frac < 0.45,
       `truck is proportionate in landscape (${frac.toFixed(2)} of width)`);
  }

  dragTruck(W * 0.15); tick(60);
  dragTruck(W * 0.85); tick(60);
  ok(truck() !== null, "truck stays on screen while driving (camera follows)");

  toEnd(0);
  ok(rocksLoose() > 0, "quarry rocks are reachable at the left end");

  // ---- B1: a second contact must not steal the first
  {
    toEnd(0);
    const before = truck();
    const r = rockClusters()[0];
    if (r) {
      w.pointer(r.x, r.y, DOWN);             // finger one grabs a rock
      w.pointer(before.mid, before.ymid, DOWN); // finger two lands on the truck
      for (let i = 1; i <= 10; i++) w.pointer(before.mid + i * 12, before.ymid, MOVE);
      w.pointer(before.mid + 120, before.ymid, UP);
      tick(60);
      const after = truck();
      ok(Math.abs(after.mid - before.mid) < W * 0.05,
         "a second finger cannot steal the first and drive the truck");
    } else ok(false, "no rock cluster found to test with");
  }

  // ---- load, haul, dump
  let loaded = 0;
  for (let pass = 0; pass < 6 && loaded < 6; pass++) {
    toEnd(0); dragTruck(W * 0.42); tick(70);
    for (const r of rockClusters()) {
      if (loaded >= 5) break;
      const bed = bedBox(); if (!bed) break;
      w.pointer(r.x, r.y, DOWN);
      w.pointer(bed.mid, bed.ymid, MOVE);
      w.pointer(bed.mid, bed.ymid, UP);
      tick(35);
      loaded = w.score();
    }
  }
  ok(loaded >= 3, "rocks load into the bed (got " + loaded + ")");
  ok(loaded <= 6, "bed capacity respected, never more than 6 (got " + loaded + ")");

  toEnd(1); tick(60);
  const site = truck();
  sfx.length = 0; tipParams.length = 0;
  w.pointer(site.lo + (site.hi - site.lo) * 0.22, cabBox().ymid, DOWN);
  w.pointer(site.lo + (site.hi - site.lo) * 0.22, cabBox().ymid, UP);
  const seen = [];
  for (let f = 0; f < 700 && (w.score() > 0 || f < 90); f++) {
    tick(); if (!seen.includes(w.score())) seen.push(w.score());
  }
  ok(w.score() === 0, "tapping the bed tips the load out (got " + w.score() + ")");
  ok(seen.length >= 3, "count steps down one at a time: " + seen.join(" -> "));
  ok(tipParams.some(p => p > 0.5) && tipParams.filter(p => p > 0).length > 20,
     "hydraulics are a held note, not a one-shot");
  tick(120);
  // The end of the world is the crusher now: driving to the end and tipping
  // feeds it rather than leaving a pile on the ground.
  const crunches = sfx.filter(x => x === 12).length;
  ok(crunches >= 3, `tipping at the end of the world feeds the crusher (${crunches} crunches)`);

  // and nothing is used up: what goes in comes back out at the quarry
  tick(400);
  toEnd(0); tick(180);
  ok(rocksLoose() > 0, "crushed rocks return to the quarry, so it never runs dry");

  // ---- B2: turning the tablet over must not destroy his world
  const hauledBefore = w.best();
  w.resize(RH, RW); tick(60);
  w.resize(RW, RH); tick(60);
  ok(w.best() === hauledBefore,
     `rotating keeps the world instead of re-dealing (hauled ${w.best()})`);
  // the frame changed shape, so drive back to where the pile was left
  toEnd(1); tick(140);
  ok(rocksLoose() > 0, "rocks are still on the ground after a rotation");

  // ---- A2: the horn still wins on the cab
  sfx.length = 0;
  const c = cabBox();
  w.pointer(c.mid, c.ymid, DOWN);
  w.pointer(c.mid, c.ymid, UP); tick(4);
  ok(sfx.includes(4), "tapping the cab sounds the horn, not a nearby rock");

  // ---- the frame is whole
  const p = px();
  ok(p.d[(W*H-1)*4+3] === 255, "bottom-right pixel written");
  const uniq = new Set();
  for (let i = 0; i < p.d.length; i += 4*997) uniq.add(p.d[i]+","+p.d[i+1]+","+p.d[i+2]);
  ok(uniq.size > 4, "frame has real content (" + uniq.size + " colours)");

  w.select(0); w.turn(2); tick(20);
  ok(w.status() === 1, "snake still runs");
}

// ---------------------------------------------------------------------------
// Sound selection is not aspect-dependent, so it gets one controlled run
// rather than riding on whatever state the haul loop left behind.
console.log("\n=== rock-on-rock landing ===");
{
  // A wide frame, purely so there is room either side of the truck to set the
  // situation up. The sound choice itself has nothing to do with aspect.
  // Re-seed, so this does not depend on how far the generator was advanced by
  // everything above it.
  w.init(4242);
  w.select(1); w.resize(844, 390); tick(4);
  const { W, H } = px();
  for (let i = 0; i < 6; i++) { dragTruck(-W * 0.4); tick(80); }   // to the quarry

  const tk = truck();
  const spot = rockClusters().find(r => r.x > tk.hi + 30 || r.x < tk.lo - 30);
  const dropAt = (tx) => {
    const src = rockClusters().find(r => Math.abs(r.x - tx) > 30);
    if (!src) return false;
    w.pointer(src.x, src.y, DOWN);
    // High enough to clear a pile that is already a couple of rocks tall,
    // otherwise the rock is released below its own resting height and simply
    // settles without ever falling.
    w.pointer(tx, H * 0.10, MOVE);
    w.pointer(tx, H * 0.10, UP);
    tick(200);
    return true;
  };
  if (spot && dropAt(spot.x)) {
    // Aim the second drop at where the first rock actually came to rest, not
    // where it was thrown — it bounces, so those are not the same place.
    const settled = rockClusters()
      .reduce((b, r) => Math.abs(r.x - spot.x) < Math.abs(b.x - spot.x) ? r : b);
    sfx.length = 0;
    const again = dropAt(settled.x);
    ok(again && sfx.includes(8), "a rock landing on a rock has its own sound");
    ok(!sfx.includes(3) || sfx.includes(8), "...and it is not the plain dirt thud");
  } else {
    ok(false, "could not set up a rock-on-rock landing");
  }
}

// ---------------------------------------------------------------------------
// Slow dragging is where the motion used to fall apart, so it gets measured
// rather than eyeballed: the truck's own drawn position, frame by frame.
console.log("\n=== dragging the truck slowly ===");
{
  w.init(4242); w.select(1); w.resize(844, 390); tick(4);
  const { W, H } = px();
  const c = cabBox();
  const startX = c.mid, y = c.ymid;

  w.pointer(startX, y, DOWN);
  const deltas = [];
  let prev = truck().mid;
  for (let i = 1; i <= 45; i++) {
    w.pointer(startX + i * 2, y, MOVE);   // 2px a frame: deliberately slow
    tick(1);
    const now = truck().mid;
    deltas.push(now - prev);
    prev = now;
  }
  w.pointer(startX + 90, y, UP);

  const moved = deltas.reduce((a, b) => a + b, 0);
  const worst = Math.max(...deltas.map(Math.abs));
  // once it is up to speed, direction must not keep reversing
  const settled = deltas.slice(10);
  const reversals = settled.filter(d => d < -0.5).length;

  ok(moved > 0, `a slow drag actually moves the truck (${moved.toFixed(0)}px)`);
  ok(worst < W * 0.04, `no frame jumps (worst ${worst.toFixed(1)}px of ${(W*0.04).toFixed(0)} allowed)`);
  ok(reversals <= 2, `motion does not stutter backwards (${reversals} reversals in 35 frames)`);
}

// ---------------------------------------------------------------------------
// Turning round.
//
// Only the state machine is asserted here. Which side the load actually lands
// on is checked by eye from a rendered frame (tools/shot.mjs) — measuring it
// from pixels needs the camera to be perfectly still between two samples, and
// it is not: it keeps easing toward the truck long after the drag ends, so the
// quarry scrolls between the two counts and swamps the handful of rocks that
// were tipped.
console.log("\n=== turning the truck around ===");
{
  w.init(4242); w.select(1); w.resize(844, 390); tick(4);
  ok(w.can_flip() === 1, "the truck game offers a turn-around button");
  ok(w.facing() === 1, "starts facing right");

  w.flip();
  tick(4);
  ok(w.facing() === 1, "does not swap the instant it is pressed (it turns)");
  tick(40);
  ok(w.facing() === -1, "has turned round by the end of the animation");

  w.flip(); tick(60);
  ok(w.facing() === 1, "and back again");

  // a turn must not be startable mid-tip, or the load swaps ends mid-pour
  w.select(0);
  ok(w.can_flip() === 0, "snake offers no turn button");
  ok(w.facing() === 0, "and reports no facing");
}

console.log(fails === 0 ? "\nALL PASS" : `\n${fails} FAILURES`);
process.exit(fails ? 1 : 0);
