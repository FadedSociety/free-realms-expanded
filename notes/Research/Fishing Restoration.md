# Fishing Restoration

Updated October 9, 2026. Builder scope: fishing only. Status: bounded wire/session
module landed with focused tests; fishing is **not** playable and no native client
run was made.

## What the game actually does

Free Realms fishing is a **native ForgeLight world activity, not a Flash minigame**.
The client binary contains a `FishingProcessor`, a `ControllerFishing`, and a packet
family `BaseFishingPacket` on SOE opcode **138 (0x8a)** with 16 sub-opcodes. The
Flash SWFs under the UI windows (`fishingMeter_REV2.swf`, `fishingmeters_bbe.swf`,
`fishingStatus.swf`, `caughtStatus.swf`, `icon_reel.swf`, `icon_fish_combo_bbe.swf`,
`fishFinder.swf`, `fish_moving_target.swf`) are presentation only; the tension
meter, distance meter, status text, reel and combo widgets are driven from native
code through `ScriptsBase.lua` (`Fishing`, `FishFinder`, `FishingTensionMeter`,
`FishingDistanceMeter`, `FishingReel`, `FishingStatusText`,
`FishingCaughtStatusText`, `FishingTutorial`).

Content inventory (`lab/reports/minigame-content-inventory.json`): minigames 595
"Learn to Fish!" and 596–601 ("Darklit Lagoon", "Brambleback's Bayou",
"Rainbow Lake", "Sacred Grove Shallows", "Wintery Basin", "Frostbitten Banks")
are all TYPE 21. Fishing holes and the tutorial are content rows, not separate
client implementations.

## Evidence

Native client pseudocode (`lab/decompiled/client/FreeRealms/pseudocode.c`):

- `FUN_00b66830` (line 1533596) reads the base header: two 16-bit fields, opcode
  then sub-opcode. `FUN_00b69550` (line 1536143) writes the same two fields.
- `FUN_008fb810` (1051194) writes an eight-byte actor handle; `FUN_0091a640`
  (1076021) and `FUN_008e2410` move a four-float vector.
- Client request writers: `FUN_00b6a400` (register, 1536857), `FUN_00b6a380`
  (cast animation, 1536834), `FUN_00b6a460` (cast, 1536879), `FUN_00b6a4d0`
  (reel-in, 1536902), `FUN_00b6b510`/`FUN_00b69550` (special, 1537747).
- Server packet readers: `FUN_00b66890` (1533624, update data), `FUN_00b668f0`
  (1533649, cast animation echo), `FUN_00b66990` (1533686, reel-in echo),
  `FUN_00b66a00` (1533718, spawn bobber), `FUN_00b66a80` (1533753, update
  bobber), `FUN_00b66b80` (1533803, update school), `FUN_00b66be0` (1533823,
  fishing result), `FUN_00b68810` (1535343, despawn school), `FUN_00b69130`
  (1535877, spawn school).
- Dispatch: `FUN_00b6cd80` (1538879) handles sub-opcodes 1, 3, 4, 5, 7, 8, 10,
  11, 12, 13 and 14. Cast/reel requests are also received, so the server
  broadcasts them to proxies.
- Controller state machine `FUN_00b6cd80` callers set native state
  `ControllerFishing + 0x1c` to 6 (cast), 4 and 5 (hook/reel), and a duplicate
  request in the same state is ignored by comparing the actor handle pair.
- Sound/event names: `Fishing:StartCasting`, `Fishing:EndCasting`,
  `Fishing:StartUnderwater`, `Fishing:EndUnderwater`, `Fishing:ClickedAbility`,
  `MG_Fishing_Cast_Perfect`, `MG_Fishing_Cast_Invalid`, `MG_Fishing_HookFish`,
  `MG_Fishing_ReelIn`, `MG_Fishing_Fish_Interested`, `MG_Fishing_HookItem`.

Server (preserved original): `lab/decompiled/server/LibSOE/SOE/BaseFishingPackets.cs`
defines the same 16 sub-opcodes; `BasePackets.cs:85` maps `BaseFishingPacket = 138`.
The client constructors install matching values (`FUN_00b695f0` → 3, `FUN_00b69770`
→ 4, `FUN_00b60c70` → 12, `FUN_00b60d20` → 13, `FUN_00b62cb0` → 15), so client and
server agree on the numbering. The server has no fishing serializer; nothing in the
community repositories (`OpenSourceFreeRealms`, `Sanctuary`, `FreeRealms-Legacy`)
defines fishing payloads beyond names.

Client UI windows and Lua: `lab/working/OSFR.Client/OSFR Client/UI/UiModules/Main/
wndFishing*.xml`, `wndFishFinder.xml`; Lua sections in
`lab/decompiled/client-ui/ScriptsBase.lua` (Fishing 44147, FishFinder 44223,
FishingStatusText 44417, FishingCaughtStatusText 44447, FishingTensionMeter 44477,
FishingDistanceMeter 44513, FishingReel 44555, FishingTutorial 28056). Extracts are
preserved in `lab/decompiled/minigames/fishing/`.

Wiki corpus (read-only query of `lab/references/freerealms-wiki/wiki.sqlite`):
Fisherman page lists rods (Simple Bamboo, Golden Reel = short cast; Metal, Red
Scoped = greater distance; Golden Scoped = deep water), the six fishing holes, and
16 lures that each raise the chance of three specific fish by 10%, plus a Treasure
Magnet lure. Page text is exported to
`lab/decompiled/minigames/fishing/wiki-fisherman-page.txt`. The corpus contains no
"Learn to Fish" quest page and no cast/tension rules.

## Recovered wire schema

Header: `u16 138`, `u16 sub-opcode`, little-endian, then the body.

Client requests:

| Sub | Name | Body |
| --- | --- | --- |
| 2 | RegisterPlayerRequest | handle(8) + u8 |
| 5 | CastAnimRequest | handle(8) + f32 + vec4(16) |
| 6 | CastRequest | handle(8) + vec4(16) + u8 |
| 7 | ReelInRequest | handle(8) + u8 |
| 15 | SpecialRequest | handle(8) |

Server bodies decoded in the module: sub 1 handle + vec4; sub 5 and 7 echoes as
above; sub 8 handle + u32 + vec4 + vec4; sub 10 handle + u32 + two flags; sub 12
u32 school id; sub 13 u32 + vec4; sub 14 the result body (handle, u32, flag, u32,
u32, blob, f32, u32, f32, four u32, blob, blob, u32, f32, flag, u32). Sub 3, 4,
11, 16 and 18 bodies were not recovered and stay opaque. The result reader flags
NaN floats; the module rejects them the same way.

## Reconstructed, not recovered

- Phase labels and the `Session` ordering rules (register before cast, cast
  animation only while casting, reel-in only once hooked). The client's duplicate
  guard is recovered; the rest is a documented reconstruction.
- Session actor binding: the first `RegisterPlayerRequest` binds the session to one
  eight-byte actor handle, same-actor re-registration is a no-op, requests or
  results from another handle are refused, and `reset` unbinds. This whole policy
  is reconstructed, not asserted historical.
- Value validation in front of any state mutation: non-finite vectors, non-finite
  distance and negative distance are refused. The only recovered value rejection
  is the NaN check inside the native result reader (`FUN_00b66be0`); the rest is a
  deliberately stricter local guard. The wire codec remains lossless, so refused
  values still decode and re-encode byte-for-byte.
- `MAX_BODY` is a decoder-side allocation bound for opaque bodies, not a recovered
  native maximum; no evidence ties the client trace buffer to wire packet size.
- No timing, no tension thresholds, no catch tables, and no server authority for a
  catch are implemented. Nothing here should be described as original balance.

## Interface and integration

`prototype/osfr-tools/src/fishing.rs` (self-contained, path-included by
`prototype/osfr-tools/tests/fishing.rs`; not registered in `lib.rs`). Public API:
`OPCODE`, `Sub`, `Handle`, `Vec4`, `ClientRequest::{encode,decode}`, `ResultBody`,
`ServerPacket::decode_known`, `Outcome`, `Phase`, `Session`.

Root integration: add `pub mod fishing;` to `lib.rs`, route opcode 138 payloads to
`ClientRequest::decode` for inbound client requests after the SOE codec strips
framing, and use `ServerPacket::decode_known` only for outbound evidence. Drive
`Session::request` from the decoded request before granting any reward. No save,
launcher, gateway, or content file was touched by this task.

## Verification

RED then GREEN, focused target only:

- `scripts/cargo.ps1 test --test fishing` first failed to compile
  (`couldn't find file tests\..\src\fishing.rs`), log in
  `lab/reports/fishing-restoration-20261009T015330Z/red-cargo-test-fishing.log`.
- After implementation the same command passes 7 tests, 0 failures, 0 ignored,
  no warnings; log in `.../green-cargo-test-fishing.log`. The intermediate run
  exposed two wrong test constants (UpdateData body and one extra u32 in the
  result fixture), which were fixed in the test, not by weakening the module.
- Lead-review follow-up added five regressions (foreign actor, re-registration and
  reset policy, invalid values without mutation, lossless codec for refused values,
  foreign result). RED first: the same command failed with six compile errors for
  the missing `Error::ForeignActor`, `Error::InvalidValue` and `Session::actor`
  (`.../red-cargo-test-fishing-followup.log`); after implementation it passes
  **12 tests, 0 failures**, log in `.../green-cargo-test-fishing-followup.log`.
- `rustfmt --edition 2021` was run on both owned files only, after the crate-wide
  formatting pass another worker applied; hashes in `.../sha256-manifest.txt`.
- Original-source offsets were re-checked against the preserved pseudocode before
  the wording fixes: blob count in `FUN_00b66be0`, the trace-buffer comment, and
  the reader entry points referenced for the remaining gaps.

## Open gaps (ranked)

1. Server authority: nothing decides which fish bites. `FishingPacketUpdateData`,
   `FishInfoUpdate`, `SpawnFishRun` and `SpawnProxiedFishingSchool` bodies must be
   recovered (readers `FUN_00b67240`, `FUN_00b691d0`, `FUN_00b69130`, `FUN_00b6a170`)
   before a real catch can be validated.
2. Rod reach tiers, deep-water rule and lure bias exist only as wiki text; no item
   or content row binding was recovered.
3. Tension/distance meter math lives in the Flash widgets
   (`fishingMeter_REV2.swf`, `fishingmeters_bbe.swf`); FFDec is available but the
   widget script was not decompiled in this task.
4. No native client run; fishing cannot yet be exercised from the server side.

## Skills applied

- `reverse-engineering`: tool-by-evidence choice; confirmed native code before
  considering FFDec, then used it only for UI window/asset evidence.
- `game-recon`: fingerprinting the fishing subsystem (native subsystem vs Flash
  minigame) before choosing the route.
- `game-research-websearch` + pinned community-first order: community repositories
  and preserved originals were checked before writing any custom reader; the wiki
  corpus supplied rods, holes and lures.
- `mod-any-game`: lab-first workflow, decompiled evidence kept outside the Git repo,
  no game files committed.
- `RTK`: command output was read condensed; recovery path used only when needed.
- `ponytail`: one module, one focused test target, no new dependencies, no
  speculative abstractions; unknown packs stay opaque instead of guessed.
- `caveman`: terse report style in chat; this note stays plain prose because it is a
  persisted artifact.
- `asset-pipeline`: not applicable; no assets were produced or converted.
