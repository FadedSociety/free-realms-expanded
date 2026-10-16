# Startup World Content

Research snapshot: 2026-10-13. Scope: startup messages 08, 09, 10-32, 38, 41 and 45 from the private Rust-gateway capture. This is a packet compatibility builder and captured content export, not recovered production-server policy.

## Result

`prototype/osfr-tools/src/startup_world.rs` contains typed builders and strict parsers for all six message schemas. `tests/startup_world.rs` has ordinary synthetic field-change tests and an ignored private byte oracle. All 23 AddNpc packets reproduce their private capture bytes exactly.

The byte oracle also reproduced the captured quick-chat table, POI stream, journal definitions, welcome payload and MOTD. It exported parsed rows to `lab/reports/worldcontent-20261013T203151Z/world-content.json` (762,559 bytes). The export contains only these six static-content messages. It contains no player fields or player chat. Quick-chat content is represented by numeric ids; NPC names belong to static actor rows.

| Capture index | Inner message | Parsed rows |
|---:|---|---:|
| 08 | 67/1 quick-chat definitions | 2,470 |
| 09 | 57 POI definitions | 149 |
| 10-32 | 35/2 AddNpc | 23 |
| 38 | 209/1 journal definitions | 4 regions, 10 hubs, 58 hub-quests, 16 stickers |
| 41 | 93 welcome screen | 3 content pairs, 6 claim-code rows |
| 45 | 87 MOTD | 2 strings, 16 and 188 wire bytes |

The 23 captured NPC rows have no attachment, effect-tag or character-variable entries. Synthetic coverage exercises an attachment, an effect-tag row, and character-variable keys; those synthetic rows are serializer/parser checks, not independent native byte-oracle evidence for populated sublists.

## Recovered client layouts

All packets use the tunnel frame recorded in `Startup Message Builders.md`: outer bytes `05 00 01`, little-endian `u32` inner length, then the inner message. The builders produce this wrapper; parsers consume the inner message. Native-map queries used the active database through `python -I scripts/native_map/nmq.py`.

### Quick chat, 67/1

Dispatcher `00b966e0` routes family 67; counted reader `00b90750` reads a row count, a separate integer hash key, then calls `00b8f090`. Disassembly of `00b8f090` confirms nine consecutive `u32` row fields. The captured packet has 2,470 rows of 40 bytes each: key plus nine fields. Key and row id agree in all 2,470 captured entries.

The Social guide records the client Lua consumer columns: row id, menu parent, display string id, optional say string id and animation id. The remaining four fields are preserved as integers. The client reader alone proves the count/key/nine-dword wire shape, not every field-name-to-offset mapping. The row field names in the Rust struct are a content crosswalk and remain `(R)` where the native reader does not name a field. No quick-chat text is copied into the JSON.

### POIs, opcode 57

`ClientPointOfInterestManager_OnDefinitionReply` (`00b98b70`, likely label checked in bytes) constructs reply 57 (`00b97ad0`), reads a length-delimited payload and replaces the existing rows. `PoiDefinitionReply_ReadRowStream` (`00b98990`) consumes a one-byte row marker until zero. `PoiDefinitionRow_ReadFields` (`00b98290`) reads 13 dwords (three ids, three position floats, heading, and six further dwords), a signed length-prefixed byte string, and a trailing dword. Marker and field widths are recovered from the client; names for fields not identified by the client remain `(R)` in the row type.

### NPC spawns, 35/2

`PlayerUpdatePacketAddNpc_ReadBody` (`00921a10`, confirmed) reads the AddNpc body after the 35/2 header. It contains a 64-bit actor identifier, the name/model identifiers, scalar fields and flags, two float4 values, lists, counted strings, and trailing scalar/string fields. The parser consumes each capture exactly, including the variable string lengths in captures 12, 30 and 31.

Native helper evidence checked in bytes:

- `009131c0` and `008e86f0`: counted attachment rows with three length-prefixed strings and three dwords.
- `0090aa10` and `008e93e0`: counted effect-tag rows; the reader consumes the fixed typed fields retained by `NpcEffectTag`.
- `008e8510`: one dword, two strings, one dword and another string, used by the customization fields.
- `0090afd0`: counted `u32` character-variable keys.

The client reader establishes field widths and order. Semantic names beyond actor id, name/model ids, transform and directly identified flags are a crosswalk from the pinned AGPL-3.0 `FreeRealms-Legacy` packet schema; they are not presented as recovered server rules. Unknown values remain named in the type and marked `(R)`. No source code was copied from that reference.

### Journal definitions, 209/1

Handler `00a45d80` reads a `u16` family and `u32` subtype; subtype 1 replaces the definition tables. Body reader `00a45d10` requires complete consumption and delegates to `00a45cc0`. The four table readers consume a count, key and fixed dword arrays: `00a45740`/`00a442c0` read 7 fields, `00a457e0`/`00a443d0` read 10, `00a45880`/`00a44540` read 3, and `00a45920`/`00a445c0` read 9. Table widths and order are client facts. Per-column labels are unresolved, so each row keeps a typed `fields` array marked `(R)`.

### Welcome screen, opcode 93

`ParseWelcomeScreenPayload` (`00939200`) reads the packet opcode and delegates to `ReadU16BoolStringsU32Pair` (`009264b0`). The reader consumes a boolean, counted pairs of dwords, counted rows containing a length-prefixed string, three dwords and another length-prefixed string, then two trailing dwords. `ApplyWelcomeScreenPayload` (`00ad0790`) copies the pair rows and mixed rows and stores the trailing values. The exact widths and array order are client facts. Field meanings for data the native reader does not name are marked `(R)`; the UI claim-code columns are mapped by `WelcomeInfo` data-source functions in Gateway and Transport.

### MOTD, opcode 87

`PacketMOTDCtor` (`008ea4e0`, confirmed) sets opcode 87 and initializes two embedded strings. The client body reader is not mapped. The implementation uses the two signed-length UTF-8 strings present in the capture and reproduces the complete frame exactly. This encoding is capture-backed reconstruction `(R)`, not a recovered client-reader rule. The values are server MOTD content, not player chat.

## Reconstructed policy and limits

No gameplay or account rule was added. `(R)` comments identify unknown row semantics and capture-valued fields retained for compatibility. Captured rows are a convenient seed for a content loader; they do not establish original retail content. Journal column meanings and some quick-chat/NPC field semantics remain unresolved. MOTD has no mapped client body reader. Populated nested NPC lists still need an independent native capture before claiming their exact compatibility. The module is not registered in the shared library/gateway, and no native client was launched.

## Verification

Private Rust target: `%USERPROFILE%\Documents\Projects\CartoonMMO\lab\build\W05-worldcontent`.

Commands and outcomes:

Final check: A handoff recheck used the private toolchain's `rustfmt.exe --edition 2024 --check` on only `src/startup_world.rs` and `tests/startup_world.rs` — passed. A repeat of repository-wide `cargo fmt --check` returned nonzero because it reported formatting differences in unrelated/shared files including `src/game_store.rs` and `src/inventory.rs`; the check changed no files. The attempted Cargo file-filter form (`cargo fmt --check -- src/startup_world.rs tests/startup_world.rs`) is unsupported by Cargo and returned `unexpected argument`; the direct scoped `rustfmt` check is the passing owned-file result.

1. From `prototype/osfr-tools`, `..\..\scripts\cargo.ps1 fmt`; then `..\..\scripts\cargo.ps1 fmt --check` — passed. The initial check identified formatting differences only in the two owned files; formatting changed no shared file.
2. With the private target above, `..\..\scripts\cargo.ps1 test --test startup_world` — 6 passed, 0 failed, 1 ignored (the private oracle). Cargo reported one existing dead-code warning in `src/housing.rs::Lot::instance_index`.
3. With `OSFR_GATEWAY_REFERENCE=%USERPROFILE%\Documents\Projects\CartoonMMO\lab\references\rust-gateway-reference` and `OSFR_WORLD_CONTENT_EXPORT=%USERPROFILE%\Documents\Projects\CartoonMMO\lab\reports\worldcontent-20261013T203151Z\world-content.json`, the wrapper was called with `@('test','--test','startup_world','--','--ignored','--nocapture')` — 1 passed, 0 failed; generated the JSON above. The first direct PowerShell attempt stripped the `--` separator and Cargo rejected `--ignored`; the explicit array invocation passed it correctly.
4. A `python -I` JSON audit parsed the export, confirmed row counts above, confirmed all NPC capture nested-list maxima are zero, verified MOTD wire string lengths (16, 188), and confirmed the export has only the six intended top-level message groups.

Native byte inspection used `python -I scripts/native_map/nmq.py dis` on the likely/speculative readers before relying on their labels. Commands run:

```text
python -I scripts/native_map/nmq.py show 00921a10 00b98b70 00b90750 00a45d80 00a45d10 00ad0790 008ea4e0
python -I scripts/native_map/nmq.py dis 00b98b70 220
python -I scripts/native_map/nmq.py dis 00b98990 300
python -I scripts/native_map/nmq.py dis 00b98290 280
python -I scripts/native_map/nmq.py dis 00b983a5 240
python -I scripts/native_map/nmq.py dis 00b9847e 220
python -I scripts/native_map/nmq.py dis 00b90750 180
python -I scripts/native_map/nmq.py dis 00b8f090 180
python -I scripts/native_map/nmq.py dis 0090aa10 160
python -I scripts/native_map/nmq.py dis 009131c0 180
python -I scripts/native_map/nmq.py dis 00922410 180
python -I scripts/native_map/nmq.py dis 009264b0 180
python -I scripts/native_map/nmq.py dis 00a45cc0 200
python -I scripts/native_map/nmq.py dis 00a45d10 120
python -I scripts/native_map/nmq.py dis 00a45d80 150
python -I scripts/native_map/nmq.py dis 00a45740 160
python -I scripts/native_map/nmq.py dis 00a457e0 160
python -I scripts/native_map/nmq.py dis 00a45880 160
python -I scripts/native_map/nmq.py dis 00a45920 160
python -I scripts/native_map/nmq.py dis 00a442c0 120
python -I scripts/native_map/nmq.py dis 00a443d0 100
python -I scripts/native_map/nmq.py dis 00a44540 100
python -I scripts/native_map/nmq.py dis 00a445c0 100
python -I scripts/native_map/nmq.py dis 008e93e0 100
python -I scripts/native_map/nmq.py dis 008e86f0 140
python -I scripts/native_map/nmq.py dis 0090afd0 100
python -I scripts/native_map/nmq.py dis 008e91e0 140
python -I scripts/native_map/nmq.py dis 008e7210 100
python -I scripts/native_map/nmq.py dis 00ad0790 232
python -I scripts/native_map/nmq.py dis 00939200 220
python -I scripts/native_map/nmq.py dis 008ea4e0 128
```

Confirmed AddNpc reader `00921a10` was inspected with `nmq show`; its accepted label is confirmed. The private byte oracle and JSON audit commands and results are recorded above.
