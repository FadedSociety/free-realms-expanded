# Native opening quest journal

The fresh opening campaign now has an isolated Rust native quest-journal writer in `prototype/osfr-tools/src/opening_journal.rs`. It replaces the inherited research character's 25 unrelated quests with accepted, unturned-in opening entries, and produces live native accept/counter/objective-complete/turn-in packets. Gameplay integration is owned and reviewed by the main agent. This note records static native and byte-oracle proof; actual-client UI acceptance remains a separate gate.

## Proven field corrections

The supplied C# serializer preserves bytes but leaves most names as `Unknown`. The preserved native client reader supplies more precise types and behavior:

| Supplied field | Native destination | Proven meaning |
| --- | --- | --- |
| Quest `Unknown` | record +0x00 | Quest ID |
| Quest `Unknown2`, `Unknown3` | +0x04, +0x08 | Title/description locale IDs, also corroborated by recovered locale text |
| Quest `Unknown5` | +0x10 | MembersOnly; native datasource column9 |
| Quest `Unknown11` | +0x154 | SystemQuest; native datasource column12 |
| Quest `Unknown9` | +0x28 | **f32**, overall progress; supplied C# declares an int containing its bit pattern |
| Objective tuple `Item1` | temporary objective +0xf0 | Raw tuple key; runtime quest constructor normalizes it to definition ID |
| Objective `Unknown` | +0x00 | Objective definition ID, separate from tuple key |
| Objective `Unknown5` | +0xd0 | State; activation sets 1, completion sets 2 |
| Objective `Unknown6` | +0xd4 | **Current counter**; native update writes this field |
| Objective `Unknown7` | +0xd8 | Required amount; native activation writes this field |
| Objective `Unknown8` | +0xdc | Additional unknown field, **not** the current counter |
| Root `Unknown` | quest manager +0x10fc | Completed quest count; subtype 12 writes the same destination |
| Root `Unknown2` | +0x10f4 | Selected quest ID; native selection/removal logic uses it |
| Root `Unknown3` | +0x10f8 | AutoSwitchSelectedQuest bool; independently proven through native Lua binding/getter/setter |
| Root `Unknown4`, `Unknown5` | +0x1100, +0x1104 | Unknown i32 values; preserved byte-for-byte (captured 10 and 30) |

The old `prepare-gameplay-quests.py` binds and patches objective `Unknown8`, so its research quest counter patch targets the wrong native field. The new opening writer writes `Unknown6` correctly. Captured examples support the native finding: quest462 has current=3, required=6 in Unknown6/7; quest1648 has 4/6; Unknown8 is 0 in both. The old research path is distinct and was not edited by this subagent.

The captured quest1617 has objective definition14096 but tuple key1. The temporary raw reader preserves key1, but runtime constructor00baf380 copies ObjectiveData with its definition ID as the new key;00bab950 stores that ID at runtime+0xf0. Consequently incremental updates use14096, while full records retain their original tuple keys. The earlier conclusion that live updates must use ordinal keys was incorrect; see [[Native Journal Live Objective Keys]] for original-executable disassembly and the regression fix.

The original Lua and native datasource audit in [[Native Quest Journal UI]] proves how this data reaches the current Hero's Journal/Other Quests tab and the helper. Native run072258 delivered an initially empty journal and accepted the first opening quest with a compiled-serializer-exact subtype3 record; see [[Native Creator Delivery Proof]]. It emitted no separate subtype9 update, so that run does not establish live counter-update acceptance.

## Wire records

All fields are little endian. Bool is one byte. A full quest record is:

```text
i32 quest ID, title locale ID, description locale ID, unknown4
bool unknown5
i64 unknown6
i32 unknown7
bool unknown8
f32 overall progress
RewardBundle
i32 objective count
repeat:
  i32 key, definition ID, name locale ID, description locale ID
  bool unknown4
  RewardBundle
  i32 state, current, required, unknown8
  bool unknown9
  i32 unknown10
i32 unknown10
bool unknown11, unknown12
```

The root is `i32 quest count`, full records, then `i32 completed count`, `i32 selected quest ID`, `bool selection flag`, and two unknown i32 values. Fresh state derives count/selection from campaign entries, clears the inherited selection flag, and preserves the last eight root bytes. A rewarded quest is omitted from the active journal on reconnect.

Opcode49 uses **u16 opcode + i32 subtype**, not a one-byte subtype. `OpeningJournal::updates()` returns inner packets; Gateway supplies the normal seven-byte tunnel envelope.

| Subtype | Payload after six-byte base header | Inner bytes | Native evidence |
| --- | --- | --- | --- |
| 3 QuestAdd | Full quest record, bool select flag | Variable | `FUN_00c7cc80`, strict wrapper `00c7ccd0` |
| 4 QuestComplete | i32 quest ID | 10 | `00c7b230`; dispatch calls `00c7c130`, which erases the quest |
| 8 ObjectiveActivated | i32 quest ID, key, required; bool | 19 | `00c7aeb0`, mutation `00c7a910` |
| 9 ObjectiveUpdate | i32 quest ID, key, current; f32 progress; bool; i32 extra | 27 | `00c7af60`, mutation `00c7a990` |
| 10 ObjectiveComplete | i32 quest ID, key; f32 progress; i32 extra; bool | 23 | `00c7b070`, mutation `00c7aa10` |
| 12 CompletedQuestCountUpdate | i32 completed count | 10 | `00c7b300`; dispatch assigns manager +0x10fc |

Full accepted opening records initialize objectives as active(state1) or complete(state2), so separate subtype8 packets are unnecessary for the current bounded campaign. Counter changes send9, followed by10 when reaching the requirement; their objective key is the runtime definition ID, not the full tuple ordinal. Completing objectives keeps the quest visible for turn-in. Turn-in sends4, then12 with the absolute campaign count. Repeated unchanged state emits no packets. Regressive state or changed definitions are rejected.

Native source evidence is outside Git in `lab/decompiled/client/FreeRealms/pseudocode.c`. Targeted extracts and original line numbers are `lab/reports/quest-native-functions.c` and `.json`. Key functions: `00940c60` (original line1104250), `0093c840` (1101042), `008fd770` (1052733), `00922810` (1083321), `0092b050` (1089439), and dispatch `00c7cd40` (1746234). Ghidra was read-only; packet vtables contain destruction rather than serialization methods, so static readers were traced instead.

## Neutral fields and reconstruction limits

Only proven identities, locale references, counters, objective states and overall progress are populated. Remaining unknown quest/objective fields and UI flags are zero. QuestAdd's final bool is true because the native dispatch explicitly includes it in its select-new-quest condition. ObjectiveUpdate's extra i32 and ObjectiveComplete's optional lookup i32 are zero. The latter skips an optional native lookup/presentation branch in dispatch. Original dialogues and reward-wheel presentation are not recovered by these neutral packets.

Each neutral RewardBundle is the actual supplied serializer's empty record: bool false, nine zero i32, two zero i64, two zero i32, zero entry count, final zero i32 (69bytes). The authoritative opening engine grants actual inventory/coin rewards separately. Acceptance timestamp is zero because persistent acceptance times are not supplied. Quest progress is a reconstructed average of normalized objective counts in [0,1]. All objectives are journal-visible as active; ordered gameplay triggers remain enforced by OpeningEngine. These choices are deliberate reconstruction policies, not claims of the original server's implementation.

Reference loading checks schema/version, safe direct filename, bounded file size, offset, SHA-256 and full journal parse with no trailing bytes. Only recovered reward type1(item) is accepted in original reference verification; other future reference types fail explicitly. Full-self patching checks both length headers and the exact unique original journal span, then recalculates both lengths and preserves surrounding payload. It cannot be applied twice. Apply before inventory changes move the original span.

## Verification and reproduction

Five focused tests pass, including the explicit private-reference test:

- Fresh and empty campaigns remove the inherited journal and reset count/selection.
- An independent cursor reader verifies full records, proper current-counter location, keys, floats, native packet lengths and exact consumption.
- Accept, counter update, objective completion, turn-in and repeated state produce the intended packet sequence; malformed references, corrupt spans, invalid lengths and regressions fail.
- The private original 7,970-byte span loads and parses successfully. All surrounding actual captured self bytes remain identical after replacement.
- A separate .NET Framework helper invokes **the supplied compiled** `Gateway.dll` method `ClientPcData.ReadFromJSON`. Its original output equals all 203,837 bytes of the captured self payload. For a synthetic opening record its output equals all 196,215 bytes of the Rust-patched payload. This checks more than a duplicate Rust writer.

Private helper/input/output paths:

```text
lab/reports/QuestSerializerOracle.cs
lab/reports/QuestSerializerOracle.exe
lab/reports/prepare-quest-serializer-oracle.py
lab/reports/quest-original-serializer-oracle.bin
lab/reports/quest-synthetic-serializer-input.json
lab/reports/quest-synthetic-serializer-oracle.bin
lab/reports/native-opening-journal-verification.json
```

From the project root, first generate the independent oracle:

```powershell
& 'C:\Windows\Microsoft.NET\Framework64\v4.0.30319\csc.exe' /nologo '/out:%USERPROFILE%\Documents\Projects\CartoonMMO\lab\reports\QuestSerializerOracle.exe' '..\lab\reports\QuestSerializerOracle.cs'
& '..\lab\reports\QuestSerializerOracle.exe' '..\lab\working\OSFR.Server\OSFR Server' '..\lab\working\OSFR.Server\OSFR Server\Customize\PacketSendSelfToClient.json' '..\lab\reports\quest-original-serializer-oracle.bin'
& '..\lab\tools\wiki-env\Scripts\python.exe' '..\lab\reports\prepare-quest-serializer-oracle.py'
& '..\lab\reports\QuestSerializerOracle.exe' '..\lab\working\OSFR.Server\OSFR Server' '..\lab\reports\quest-synthetic-serializer-input.json' '..\lab\reports\quest-synthetic-serializer-oracle.bin'
```

Then from `prototype/osfr-tools`:

```powershell
$env:OSFR_JOURNAL_CONTENT='%USERPROFILE%/Documents/Projects/CartoonMMO/lab/references/rust-gameplay'
$env:OSFR_JOURNAL_SELF='%USERPROFILE%/Documents/Projects/CartoonMMO/lab/references/rust-gateway-reference/0007-login.bin'
$env:OSFR_JOURNAL_ORACLE='%USERPROFILE%/Documents/Projects/CartoonMMO/lab/reports/quest-synthetic-serializer-oracle.bin'
& '../../scripts/cargo.ps1' -CargoArgs @('test','--test','opening_journal','--','--include-ignored')
```

Result: 5 passed, 0 failed. Normal `test --test opening_journal` runs four synthetic tests and leaves the private-data test ignored. The main agent also reported its integrated packet-only opening/cave-return/restart sequence passing after wiring this module; that is not a screenshot or a native-client journal rendering test.

Remaining acceptance: launch the reviewed build with a fresh campaign, verify native quest titles/objectives and live counters visually, complete/turn in an opening quest, and verify completed count/journal on restart. Do not claim this static/oracle proof as completion of the full native client playthrough.
