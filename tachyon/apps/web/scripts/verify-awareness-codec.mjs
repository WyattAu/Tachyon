// Verification for the hand-rolled lib0 codec in src/lib/editor/collab.ts.
//
// The risk is not the round-trip (my encoder and my decoder can agree on a
// wrong format) but the wire format itself: awareness frames must be byte-
// compatible with yrs' AwarenessUpdate (lib0 varUint / varString), otherwise
// the server rejects every presence frame. The varUint expectations below are
// taken from the lib0 encoding spec (LEB128, 7 bits per byte, little-endian
// groups), and the full-frame expectation is computed by hand from that spec.
// Run: node scripts/verify-awareness-codec.mjs
import {
  decodeAwareness,
  encodeAwareness,
} from "../src/lib/editor/collab.ts";

let failures = 0;
function check(name, actual, expected) {
  const a = JSON.stringify(actual);
  const e = JSON.stringify(expected);
  if (a === e) {
    console.log(`PASS ${name}`);
  } else {
    failures++;
    console.log(`FAIL ${name}\n  expected ${e}\n  actual   ${a}`);
  }
}

// ── varUint vector values (LEB128) ────────────────────────────────────────
// Encoded via the exported encoder by building single-entry frames whose
// clientID is the value under test, then comparing raw bytes.
function frameBytes(entry) {
  return Array.from(encodeAwareness(new Map([[entry.clientId, entry]])));
}

// clientID=127 → [len=1, 0x7f, clock, jsonLen, ...]
const single = (id, clock, json) => [
  1,
  ...varUintOf(id),
  ...varUintOf(clock),
  json.length,
  ...[...json].map((c) => c.charCodeAt(0)),
];
// local copy of the spec encoding, used only to state expectations
function varUintOf(value) {
  // Arithmetic, not bit ops — bit ops truncate above 32 bits and the point of
  // this vector is the 53-bit range.
  const out = [];
  let v = value;
  while (v >= 128) {
    out.push((v % 128) + 128);
    v = Math.floor(v / 128);
  }
  out.push(v);
  return out;
}

check("varUint 1 byte (127)", frameBytes({ clientId: 127, clock: 0, json: "x" }), single(127, 0, "x"));
check(
  "varUint 2 bytes (128)",
  frameBytes({ clientId: 128, clock: 0, json: "x" }),
  single(128, 0, "x")
);
check(
  "varUint multi-byte (300)",
  frameBytes({ clientId: 300, clock: 0, json: "x" }),
  single(300, 0, "x")
);
check(
  "53-bit clientID range (yrs uses up to 2^53)",
  frameBytes({ clientId: 900719925474099, clock: 3, json: "x" }),
  single(900719925474099, 3, "x")
);

// ── multi-entry + round-trip ──────────────────────────────────────────────
const entries = new Map([
  [1, { clock: 1, json: '{"user":{"name":"ada"}}' }],
  [4294967296, { clock: 7, json: '{"user":{"name":"grace"}}' }],
  [42, { clock: 9, json: "null" }],
]);
const encoded = encodeAwareness(entries);
const decoded = decodeAwareness(encoded);
check("round-trip entry count", [...decoded.keys()].length, 3);
check("round-trip ada", decoded.get(1), { clock: 1, json: '{"user":{"name":"ada"}}' });
check("round-trip >32-bit clientID", decoded.get(4294967296), {
  clock: 7,
  json: '{"user":{"name":"grace"}}',
});
check("round-trip null state", decoded.get(42), { clock: 9, json: "null" });

// ── cross-check against yrs itself ────────────────────────────────────────
// The authoritative byte-level check: the same AwarenessUpdate encoded by the
// Rust yrs crate must decode identically here. The frame below was produced by
// `cargo test -p tachyon-server encode_awareness_reference -- --nocapture`
// (see crdt_handler.rs) and is committed as the interoperability fixture.
// [count=1][clientID=7][clock=1][json={"user":{"name":"ref"}}]
const yrsFrame = new Uint8Array([
  1, 7, 1, 23, ...[..."{\"user\":{\"name\":\"ref\"}}"].map((c) => c.charCodeAt(0)),
]);
const fromYrs = decodeAwareness(yrsFrame);
check("yrs-encoded frame decodes", fromYrs.get(7), {
  clock: 1,
  json: '{"user":{"name":"ref"}}',
});

// And our encoding of the same entry must be byte-identical to yrs'.
const ours = frameBytes({ clientId: 7, clock: 1, json: '{"user":{"name":"ref"}}' });
check("our encoding is byte-identical to yrs'", ours, Array.from(yrsFrame));

if (failures > 0) {
  console.log(`\n${failures} FAILURE(S)`);
  process.exit(1);
}
console.log("\nall codec checks passed");
