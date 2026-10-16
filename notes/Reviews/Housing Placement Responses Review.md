# Housing Placement Responses Review

Independent read-only review of [[Research/Housing Placement Responses]] and
`lab/reports/housing-placement-responses-20261009T1705Z/`. Writes are this note
and `lab\reports\housing-placement-independent-review-20261009T165402Z/` only; no worker artifact, source, content, save,
UI or process file changed.

## Verdict

**Worker claims independently reproduced.** The pinned `p2.pcap`
(`65ac0801...`) yields 81 inner-127 messages: sub 2 is **client-to-server**
(8 x 42-byte extracted bodies = 2-byte subtype + 40-byte payload), sub 40 is
**server-to-client** (9 x 229-byte extracted bodies = 2-byte subtype +
133-byte community-order fixture prefix + 94-byte undecoded tail; 133+94 = 227
payload bytes, +2 subtype = 229), sub 41 is **server-to-client** (8 x 10-byte
extracted bodies = 2-byte subtype + 8-byte guid), and sub 53 has **0** captured
frames. The extracted body excludes the 2-byte family opcode 127, so the full
inner packet is two bytes longer in each case. Identity caveats stand (first
dword unidentified, rotation semantics unknown, tail undecoded).

## 1. Directions, counts and raw lengths

My own tunnel splitter and counting code over the pinned piece-022 transport
decoder (`pcap-recompute.json`):

- c2s sub 2 = **8** records, every extracted body 42 bytes (2 subtype + 40)
- s2c sub 40 = **9** records, every extracted body 229 bytes (2 subtype + 227
  payload = 133 prefix + 94 tail)
- s2c sub 41 = **8** records, every extracted body 10 bytes (2 subtype + 8)
- sub 53 = **0** (both directions)
- Same scan: c2s sub 1 = 12, s2c sub 42 = 9, s2c sub 44 = 19, s2c sub 45 = 2,
  c2s sub 3 = 7, s2c sub 51 = 2, matching the note's flow.
- The worker's 25 vector files are byte-identical to the pcap bodies (sha256
  25/25).

## 2. Field layouts and the community oracle

- Sub 2 (`0x0200` + 40-byte body): first dwords
  `4260, 4260, 4378, 4179, 4214, 4594, 3996, 4595`; rotation `(0,0,0,0)` in all
  eight; scale `1.0` in all eight; every request position equals the paired
  sub-40 record position (**8/8 echo**).
- Sub 40 prefix: I implemented the unchanged community
  `FixtureInstance.Serialize` order myself in Python (guid, house guid, fixture
  definition id, unknown4, position, rotation, tilt, npc guid, tint id,
  unknown10, customization(type, texture alias, tint alias, tint id, texture
  override), unknown11, unknown12, unknown13, xml, scale, bool, unknown17) and
  re-serialized the parsed fields: **9/9 prefixes byte-identical**, each exactly
  133 bytes (`oracle-recompute.json`) — the dotnet oracle result re-derived with
  different tooling.
- Sub 40 tail: 94 bytes in all nine records; each starts with the same fixture
  guid; each contains two `1.0` floats; all nine tails are distinct. The ids
  `7466` and `34` appear under my bounded u32 scan only in frames 31507 and
  31919, consistent with the note's "e.g." example; the tail otherwise stays
  undecoded.
- Sub 41: eight unique guids, all in the sub-40 set (`c001..c009`), `c003`
  absent — the note's cross-link reproduced.
- Synthetic vectors: `synthetic-sub2.bin` is byte-identical to captured frame
  31494's body; `synthetic-sub41.bin` = `0x2900` + guid `c009`;
  `synthetic-sub53.bin` = `0x3500` + length 25 + "Fixture placement denied.".

## 3. Bounded limits

No native acceptance; the 94-byte tail and the sub-2 first dword stay
unidentified; the community serializer is a reconstructed oracle, not original
server source; raw capture bytes and decompiled code remain in the lab.
