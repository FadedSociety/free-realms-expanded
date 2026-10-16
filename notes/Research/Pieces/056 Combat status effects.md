# 056 Combat status effects

2026-10-08 UTC. Work item 056 acceptance: source-defined stacking/duration/immunity and
restart/disconnect behaviour. Shipped artefacts:

- `prototype/osfr-tools/src/restoration/piece_056.rs` - recovered effect-tag wire plus a bounded
  per-actor status ledger.
- `prototype/osfr-tools/tests/piece_056.rs` - 16 focused tests (`#[path]` include; the module is not
  registered in `src/lib.rs`, which the lead owns).
- `scripts/pieces/piece_056.py` + `tests/pieces/test_piece_056.py` - read-only evidence checker and
  18 stdlib tests over the preserved corpus.
- Private evidence: `lab/reports/piece-056-20261009T042519Z/` (`evidence.json`,
  `provenance.json`, `run.log`, `negative-control.json` and the tampered control corpus).

No native client was launched, no shared file was touched, and nothing here is a playable claim.

## Original facts recovered

Wire family, from the decompiled original server and the original client binary:

| Packet | Ids | Size | Body |
| --- | --- | --- | --- |
| `ClientUpdatePacketAddEffectTag` | 38/16 | 93 B | i32 tag id + `ClientEffectTag` (85 B) |
| `ClientUpdatePacketRemoveEffectTag` | 38/17 | 8 B | i32 tag id |
| `PlayerUpdatePacketUpdateCharacterState` | 35/20 | 16 B | u64 guid + i32 status flags |
| `PlayerUpdatePacketAddEffectTagCompositeEffect` | 35/41 | 36 B | u64 guid, i32 tag, i32 composite effect, u64 source guid, i64 unused |
| `PlayerUpdatePacketRemoveEffectTagCompositeEffect` | 35/42 | 16 B | u64 guid + i32 tag |
| `PlayerUpdatePacketEffectTagCompositeEffectsEnable` | 35/43 | - | id recovered, **body not recovered** |

- `decompiled/server/LibSOE/SOE/{BaseClientUpdatePackets,BasePlayerUpdatePackets,BasePackets}.cs`
  supply the numbers; `decompiled/server/Gateway/Gateway.Login/ClientPcData.cs` supplies the
  `EffectTag` (19 members) and `ClientEffectTag` (4 more) order and `BinaryWriter` widths, hence
  72 + 13 = 85 bytes. The community writer in `references/Sanctuary/...` happens to agree.
- The numbers are **sub** opcodes, not main opcodes: the original client calls
  `FUN_0089d180(0x10)`/`(0x11)` before the `ClientUpdatePacketAddEffectTag`/`RemoveEffectTag`
  vftable stores, and `FUN_0089fbb0(0x29)`/`0x2a`/`0x2b`/`0x14` before the
  `PlayerUpdatePacketAddEffectTagCompositeEffect`/`Remove`/`Enable`/`UpdateCharacterState` stores.
  The same 35-family helper receives `0x23` for `PlayerUpdatePacketHitPointModification`, whose enum
  value is already 35. A community serializer that writes 41/41 would not match this client.
- Client buff-row contract (`decompiled/client-ui/ScriptsBase.lua`): `GroupHandler.BuffsDsName =
  "BaseClient.PlayerEffectTags"`; each row is read as `(instanceID, iconID, duration, timeLeft,
  name)` from columns 0-4; `AddAbilityEffect` returns immediately when the icon is 0;
  `BuffsUpdated` removes any locally tracked buff row the datasource no longer carries.
- Persistence: `ClientPcData` carries `List<(int TagId, ClientEffectTag)> EffectTags` and writes
  `count` then `(TagId, tag)` into the self-to-client payload, which is the replayable
  login/zone-in document, not a runtime-only field.

Captured retail row (`lab/working/OSFR.Server/OSFR Server/Customize/PacketSendSelfToClient.json`):
exactly one tag - TagId 2, duration (`Unknown7`) 3600, icon (`Unknown20`) 7334, name (`Unknown22`)
4270, `Unknown2` 495, `Unknown13` composite effect 0, `Unknown10`/`Unknown11` 142556/3978,
`Unknown6` the float bits of 100.0. Icon 7334 is the image id of original stat 37 "Coin Boost"
(`Resources/CharacterStatDefinitions.txt`, NameId 140800) and 4270 reads "Power Hour Coin Boost" in
the recovered locale, i.e. a one-hour coin boost - independent corroboration that `Unknown7` is a
duration in seconds.

Immunity vocabulary, original locale strings only: 70204 "immune to stun", 70209 "immune to slow",
70214 "immune to poison", 436620 "Forced Effect Immunity", 436622 "Illusion Immunity", 436675
"Fear Immunity", 438016 "Knockback Immunity", 438557 "Death Prevent + Knockback/Root Immunity",
438559 "Status Immunity". The community wiki's `Combat` page (revision 10466) additionally lists the
damaging/detrimental effects and their player-visible behaviour; that text is community policy, not
server source.

## Inferred meaning (labelled)

Duration seconds (`Unknown7`), icon (`Unknown20`) and name (`Unknown22`) are treated as pinned.
`Unknown2` carries the definition id in the module because 495 is the only captured value, but no
recovered table indexes it (`ActorCompositeEffectDefinitions.xml` has no id 495). `Unknown10`,
`Unknown11`, `Unknown9`, `Unknown13` (composite effect), `Unknown15`/`Unknown16` and the float
`Unknown6` stay unrecovered. A second preserved literal - the 97-byte hex blob in
`Gateway.Player/UpdateStats.cs` - does not line up with the 93-byte layout and is deliberately
unused.

## Reconstructed policy (not historical behaviour)

- Tag ids allocate monotonically from 1 and are never reused while a row is active.
- Stacking: the same definition id from the same source refreshes the existing row and keeps its
  tag id; a different source stacks an independent row. The client store is keyed by tag id, so both
  outcomes are representable, but no recovered server code settles which rule was original.
- Immunity is caller-owned state: an immune kind is refused before any row or packet is produced;
  the module never computes damage, drain or healing from an effect.
- Disconnect: `profile_rows` writes live rows (duration replaced by remaining time) into the
  recovered `ClientPcData` shape; `restore` subtracts downtime, drops fully elapsed rows, keeps ids
  and re-sends the buff row. `disconnect_clear` covers a server that does not persist.
- `MAX_EFFECT_SECONDS` (86 400) is a local sanity bound, not a recovered maximum.

## Commands and results

```
scripts/cargo.ps1 test --test piece_056
  16 passed; 0 failed          (RED first: the same target failed with
                               "couldn't find file tests\..\src\restoration\piece_056.rs")

wiki-env python -m unittest discover -s tests/pieces -p "test_piece_056.py"
  18 tests OK

wiki-env python scripts/pieces/piece_056.py --lab ../lab `
  --out ../lab/reports/piece-056-20261009T042519Z --stamp 20261009T042519Z
  {"captured_rows": 1, "client_effect_tag_bytes": 85, "problems": [],
   "evidence_sha256": "811ac1b26707ec09c59e2adcddfc8bc28971363e19b7a0c3534a6fb50d6e6021"}

wiki-env python ../lab/reports/piece-056-20261009T042519Z/negative-control.py
  {"passed": true, "missing": [], "problems": 14}

rustfmt --edition 2024 --check <both Rust files>
  clean
```

The negative control tampers one field per check in a copied corpus (member type, packet id, Lua
zero-icon guard, captured duration, stat image id, immunity string, client constructor sub-opcode)
and confirms the checker reports every one of them.

## Remaining gates and gaps

- Lead integration: the module is only reachable through its own test target. Registering
  `piece_056` under `src/restoration/`/`lib.rs` and routing encounter combat through the ledger is
  the lead's call; the module deliberately exposes no gameplay numbers.
- Native acceptance is **not** claimed. It needs a visible buff row (icon + name + countdown) in the
  original client, its removal at expiry, and a disconnect/restart replay that keeps a live row and
  drops an elapsed one.
- Historical fidelity gaps: no server-side apply/stack/immunity/cleanse logic, no table for the
  definition id, no meaning for `StartTime`/`StopTime`, and an unimplemented 35/43 body.

Skills applied: game-recon (engine/format identification of the decompiled server and client
artefacts), game-research-websearch (wiki corpus query for the detrimental-effect vocabulary),
reverse-engineering (original client constructor/dispatch reading), plus RTK, Ponytail and Caveman
as directed. No native (game-automation) testing was performed; this item owns no shared window.
