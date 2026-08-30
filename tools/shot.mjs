// Renders frames straight out of the framebuffer to PNG, so a change can be
// looked at rather than inferred.  node tools/shot.mjs [w] [h] [tag]
import fs from "fs"; import zlib from "zlib";
const OUT = process.env.SHOT_DIR || ".";
const crcT = [...Array(256)].map((_, n) => { let c = n; for (let k = 0; k < 8; k++) c = c & 1 ? 0xEDB88320 ^ (c >>> 1) : c >>> 1; return c >>> 0; });
const crc = b => { let c = 0xFFFFFFFF; for (const x of b) c = crcT[(c ^ x) & 255] ^ (c >>> 8); return (c ^ 0xFFFFFFFF) >>> 0; };
const chunk = (t, d) => { const l = Buffer.alloc(4); l.writeUInt32BE(d.length); const td = Buffer.concat([Buffer.from(t), d]); const c = Buffer.alloc(4); c.writeUInt32BE(crc(td)); return Buffer.concat([l, td, c]); };
function png(p, w, h, out) {
  const raw = Buffer.alloc((w * 4 + 1) * h);
  for (let y = 0; y < h; y++) Buffer.from(p.buffer, p.byteOffset + y * w * 4, w * 4).copy(raw, y * (w * 4 + 1) + 1);
  const ih = Buffer.alloc(13); ih.writeUInt32BE(w, 0); ih.writeUInt32BE(h, 4); ih[8] = 8; ih[9] = 6;
  fs.writeFileSync(out, Buffer.concat([Buffer.from([137,80,78,71,13,10,26,10]), chunk("IHDR", ih), chunk("IDAT", zlib.deflateSync(raw)), chunk("IEND", Buffer.alloc(0))]));
}
const W = Number(process.argv[2] || 844), H = Number(process.argv[3] || 390);
const TAG = process.argv[4] || `${W}x${H}`;
const { instance } = await WebAssembly.instantiate(fs.readFileSync("www/games.wasm"), { env: { host_sfx: () => {} } });
const w = instance.exports;
w.init(4242); w.select(1); w.resize(W, H);
for (let i = 0; i < 4; i++) w.tick(16);
png(new Uint8Array(w.memory.buffer, w.frame_ptr(), W * H * 4), W, H, `${OUT}/${TAG}.png`);
console.log(`${OUT}/${TAG}.png  ${w.frame_w()}x${w.frame_h()}`);
