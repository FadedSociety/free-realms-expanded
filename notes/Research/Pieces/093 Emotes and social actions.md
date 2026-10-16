# 093 Emotes and social actions

Status: bounded recovery returned for lead review, 2026-10-09 05:37 UTC. Queue acceptance was
"Original action/animation IDs and restrictions; actual client acceptance with no guessed motion".
Delivered: the recovered emote command table, the 73 quick-chat rows that carry each animation id,
the animation-slot names behind those ids, the restriction flags the shipped client data holds, a
server-side validator built on them, and a read-only checker over the preserved inputs. It is
**not** a playability claim: no native client run was made here and animation id 3200 (the `Dance`
phrase) stays an explicit gap. Commands below ran from
`%USERPROFILE%\Documents\Projects\CartoonMMO\project`.

## Owned artifacts

| Path | What |
| --- | --- |
| `prototype/osfr-tools/src/restoration/piece_093.rs` | Recovered command table, quick-chat action rows, animation slot names, restriction flags and `validate_emote`. Not registered in `src/lib.rs` (lead-owned). |
| `prototype/osfr-tools/tests/piece_093.rs` | 8 behavioral tests; `#[path]`-includes the module so it compiles before lead registration. |
| `scripts/pieces/piece_093.py` | Read-only extractor/checker over the preserved lab inputs; writes the private lab bundle. |
| `tests/pieces/test_piece_093.py` | 12 stdlib tests: synthetic rejection paths plus hash/shape pins on the real shipped files. |
| `../lab/reports/piece-093-20261009T053734Z/` | Private evidence: `emotes.json`, `findings.json`, `provenance.json`, `generated-tables.txt`. |

## Recovered originals

**Command table.** `lab/decompiled/client-ui/ScriptsBase.lua` (SHA-256 `c7c71c7c…`) defines
`EmoteHandler.emotes` at line 41960: 72 `{cmd = "...", id = N}` rows (lines 41961-42043). `Init`
(line 42044) walks that array with `BindCommandToLua(self.emotes[i].cmd, ...)`, so a repeated
command is overwritten by the **last** row. Only `curious` repeats: id 2469 first, id 148 second,
so the command resolves to 148 and 2469 is a dead binding. `hi`/`hey` share 2474 and `boo`/`scare`
share 219 as separate commands. Each binding calls `Ui.ProcessQuickChatCommand(id)`.

**Quick-chat payload.** `lab/decompiled/server/Gateway/Gateway.MapChat/ChatSystem.cs` line 9,
`SendQuickChatPacket`, tunnels a `BaseQuickChatPacket` (opcode 67, subtype 1) payload that the
native client exposes as the `BaseClient.QuickChatData` table
(`pseudocode.c:1572450`, `FUN_00b96c00("BaseClient.QuickChatData", ...)`). The payload is
2470 rows x 10 little-endian u32 columns; the client Lua `ChatLog:AddQuickChatPhrases`
(`ScriptsBase.lua:10144`) reads column 0 as the row id, column 1 as the menu parent, column 2 as the
display string, column 3 as the optional say string and column 4 as the animation id. Editing that
row data is how the original client is told which motion a phrase plays.

**Emote menu tree.** Row 13 (`display locale 8083` = "Emotes") is the menu root; six category rows
hang off it and 72 animated children hang off those categories:

| Category row | Label locale | Children |
| --- | --- | --- |
| 129 | 36636 "Basics" | 15 |
| 130 | 36637 "Dramatics" | 12 |
| 131 | 36638 "Instructions" | 13 |
| 132 | 36619 "Style" | 7 |
| 133 | 36614 "Sweet Moves" | 9 |
| 134 | 36612 "Theatrics" | 16 |

Plus one animated row outside the menu, quick-chat id 143 (`/wave`), parent `0xFFFFFFFF`. Total 73
recovered actions. 71 of the 72 handler rows resolve to an action row; the one that does not is the
dead `curious` 2469.

**Animation slots.** `client/Resources/AnimationTypes.xml` (SHA-256 `20c55ba0…`) `Emotes` branch
(line 994, id block 3001-3900) names 71 of the 72 animation ids the quick-chat rows reference:
`emo_no` = 3317, `emo_cheer` = 3306, `emo_scare` = 3369, `emo_dance_basic` = 3201,
`emo_dance_01..04` = 3211-3214, `emo_outfit_all` = 3001 and the seven `emo_outfit_*` pieces,
`emo_talk_basic` = 3101, `emo_spraycan` = 3351 (line 1242). Locale strings for the phrase labels
come from `CodeStringMappings.txt` rows 4750-4777 (`emote_Afraid^7928`, `emote_Curious^23470`,
`emote_Salute^428888`, …) resolved against the community locale export (SHA-256 `f1f22a30…`).

## Recovered restrictions (original)

* `hideSlotType="1"` on every named emote slot except `emo_outfit_all` (3001): the wielded item is
  hidden while the clip plays.
* `freezeMovement="1"` on `emo_outfit_all` (3001) and `emo_spraycan` (3351): the actor is held
  still for those clips.
* `disableBoneMasking="1"` on `emo_sleep_standing` (3329).
* `priority="1"` on `emo_outfit_all`, `emo_talk_basic`, `emo_dance_basic`, `emo_dance_01..04`.
* `AnimationTransitionRules.xml:196` `Any -> Emote` runs `numLoops="1"` (one shot) with
  easeIn/easeOut 0.4; line 213 returns `Emote -> Fidgets`.
* Mount gating: `ChatLog:enableEmotesButton(0)` while a `CurrentMount` data row exists
  (`ScriptsBase.lua:30655-30663`, also on mount select at line 34998) and `(1)` once it is empty.
* Menu gating in `ChatLog:CreateQuickChatPhrase` (line 10131): emote mode creates rows whose parent
  is not 0 and re-parents rows of the Emotes root to 0; canned-chat mode drops the root row and its
  category rows. Category children are created in both modes.

## Reconstruction boundary (not historical)

`piece_093.rs::validate_emote` is a **restoration policy**, not recovered server code: it resolves a
command through the recovered bindings (last-wins), refuses unknown commands, refuses requests while
mounted (mirroring the recovered button state), refuses movement for the `freezeMovement` slots and
refuses the animation id that has no recovered slot. The client-side originals it mirrors are cited
above; the server-side rule itself is ours to own. The Open Free Realms server decompile only
decodes and echoes the quick-chat channel packet (`HandleQuickChatSendChatToChannelPacket`) and does
not carry an emote restriction model.

Inferred, not asserted: the payload column roles for columns 1, 5-9 come from the tree the values
form, while columns 0/2/3/4 are confirmed by the client Lua reader. The checker validates the tree
shape (six category rows under row 13, all animated rows parented there or top-level) rather than
trusting the column guess blindly.

## Gaps (explicit)

* **Animation id 3200.** The `Dance` phrase (quick-chat id 184, display locale 60 = "Dance")
  carries animation id 3200, which has no `AnimationSlot` row in `AnimationTypes.xml` and no
  `CodeAnimationMappings.txt` row. No motion is invented for it; `validate_emote` rejects it as
  unrecovered.
* **Wave plays `emo_hey`.** Phrase id 143 (display locale 23475 = "Wave") carries animation 3354
  `emo_hey`; `emo_wave` (3333) is carried by the top-level Hello/Goodbye rows (payload rows 3/4,
  locales 8074/8075). Recorded as recovered data, not corrected.
* **`emo_spraycan` (3351)** carries `freezeMovement` but no quick-chat row in the recovered payload
  reaches it; its trigger is not recovered.
* **Dead binding** `curious` = 2469 has no data row and no client-visible effect after the later
  binding wins.
* **Wiki corpus is silent.** `lab/references/freerealms-wiki/wiki.sqlite` has 0 chunks matching
  "emote" (checked with the FTS table during discovery), so restrictions below the client data are
  uncorroborated by community text.
* **No native run.** This worker did not drive the shared client window; the native gate belongs to
  the lead.

## Verification

```
> ..\..\scripts\cargo.ps1 test --test piece_093
   Compiling osfr-tools v0.1.0 (%USERPROFILE%\Documents\Projects\CartoonMMO\project\prototype\osfr-tools)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.90s
     Running tests\piece_093.rs (target\debug\deps\piece_093-e131c4789c460ca9.exe)
running 8 tests
test emote_menu_categories_match_the_recovered_locale_ids ... ok
test quick_chat_rows_carry_the_recovered_animation_ids ... ok
test animation_names_cover_every_referenced_id_except_the_recorded_gap ... ok
test menu_gating_matches_the_client_lua_filters ... ok
test recovered_command_table_matches_the_client_lua ... ok
test validation_accepts_recovered_emotes_and_rejects_restricted_requests ... ok
test repeated_command_binding_resolves_last_and_the_old_id_is_a_dead_binding ... ok
test restriction_flags_come_from_the_animation_block ... ok
test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

RED before the module existed: `error: couldn't find file tests\..\src\restoration\piece_093.rs`.

```
> & '..\lab\tools\wiki-env\Scripts\python.exe' -m unittest discover -s tests/pieces -p "test_piece_093.py" -v
Ran 12 tests in 0.277s
OK
```

RED before the checker existed: `FileNotFoundError: ...project\scripts\pieces\piece_093.py`.

```
> & '..\lab\tools\wiki-env\Scripts\python.exe' 'scripts\pieces\piece_093.py'
{"actions": 73, "findings": ["dead-command-binding", "dance-animation-unrecovered",
 "wave-plays-emo-hey", "spraycan-slot-unreferenced"], "handler_rows": 72,
 "reports_dir": "...\\lab\\reports\\piece-093-20261009T053734Z", "resolved_handler_bindings": 71,
 "stamp": "20261009T053734Z", "unrecovered_animation_ids": [3200]}
```

Bundle counts: `handler_rows` 72, `unique_handler_commands` 71, `actions` 73,
`resolved_handler_bindings` 71, `named_animations` 71, `say_string_rows` 10. Input hashes pinned in
`provenance.json`; `emotes.json` carries the command/binding/action/animation/restriction tables and
the locale id map.

## Remaining integration and native gates

1. Lead registers the module in `src/lib.rs` and drops the `#[path]` include (shared file, lead-owned).
2. Gateway wiring: broadcast a validated action row to nearby players and play animation id `N` for
   each; `validate_emote` is the intended entry point, not a replacement for the packet writer.
3. Native acceptance: press `/wave`, `/cheer`, `/dance` in the client and confirm the animation
   names above play, the wielded item hides, movement is refused for `emo_outfit_all`, and the menu
   is disabled while mounted. `Dance` (3200) cannot be accepted until the slot is recovered.
4. Recovery still open: animation id 3200, the trigger for `emo_spraycan` (3351), and any historical
   membership or chat-channel restriction on individual emote rows.

## Skills applied

* **Ponytail** (full): reused the preserved client resources and server payload instead of writing a
  new format reader; the module is tables plus one lookup/validation path, and the checker is the
  single runnable guard.
* **Caveman**: concise working updates; this note and the code comments stay in plain prose.
* **game-recon**: identified the engine-side data sources (`AnimationTypes.xml`,
  `AnimationTransitionRules.xml`, `CodeStringMappings.txt`, `emotes.gfx`/`emotes.lst.z` in the
  client manifest) before attributing any behavior.
* **game-research-websearch / community-first**: searched the preserved inputs and the read-only
  wiki corpus first; the wiki had no emote coverage, so the first-party payload and Lua stayed the
  authority.
* **reverse-engineering**: decoded the tunneled quick-chat payload and matched it against the Lua
  reader (`AddQuickChatPhrases`) and the animation slot block; no native disassembly beyond the
  existing pseudocode export was needed.
* Not used: `asset-pipeline` (no engine-ready asset was produced) and `game-automation` (native
  testing is lead-owned for this item).
