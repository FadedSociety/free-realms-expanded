# Native Round5 050310

Run `lab/runs/client-baseline-20261014T050310Z`: release build after W62 (integration round 5), using a fresh copy of the kart-weapons test character.

| Check | Result |
| --- | --- |
| Login, character select, world entry | Passed |
| Content NPCs near spawn, radial menu with Talk (Ricky Danger) | Passed |
| Talk shows dialogue with a close choice | Passed: one choice is shown |
| Close choice leaves the dialogue | Passed. The round-4 dialogue lock is fixed |
| Close-choice label | **Wrong.** It shows "Chef Tools" with a banner image, not "Goodbye". W62 sends locale row 8075 ("Goodbye" in the exported table), but the client displays different text and art, so the field or the id space is wrong (reader `00a9ef10`, choice rows) |
| Walking with movement validation on | Passed: no rejects or snap-backs logged; 2 saves |
| Menu exit | Passed: `session_ended` |

## Not exercised

- **Quest offer from a regional giver.** The nearest is Samantha, 24.8 m away. Clicking her from the spawn view hit the friends panel, and dev teleport (`realm goto`) only knows opening-package actors.
- **Vendor shop.** The nearest data-placed vendor is Roosey, 73 m away.

## Follow-ups

- Make `realm goto <name>` also search the content NPCs (regional rows plus extra spawns, via the visibility index). It is a test enabler, behind `--dev-tools`.
- Fix the dialogue choice label.

## Follow-up run 053834 (after W63)

- `realm goto Samantha` worked: the player is placed beside her, and Click → Talk opens a dialogue whose close choice now reads **"Goodbye"**. The banner art behind it is still shown.
- **No quest offered.** Samantha (9000000189) gives quest 3000021, "Introduce Yourself", which has no prerequisites. The quest engine's eligibility is hard-coded (level 1, no job levels, non-member), so job-gated offers are filtered out. Sent to W64.
- **`realm goto Roosey` failed** ("Unknown zone-0 entity"). 50 legacy_data-placed rows lack a zone label and are skipped by the loaders. Sent to W64.

## Run 072625 (after W64 + W65)

- The "Goodbye" choice now renders as a normal tan button with a small icon, matching the original's plain choice style (background set 18, image 0). The quarter-screen banner is gone.
- **Still no quest offer from Samantha.** The radial menu shows only "Talk", and Talk opens a 26/3 dialogue with "Goodbye" and a blank body. W65's unit test with the same save produces a 49/1 offer, so the live runtime path diverges. W67 reproduces it through the real gateway (account-store login) and adds NPC/quest log events.
- Exit was clean (`client_disconnect`).

## Run 080811 (after W67)

- The server now offers the quest. New log events show `npc_interact` → `npc_menu` (Talk, string 3300) → `quest_offer 3000021` → `npc_select outcome offer`.
- **The client shows nothing:** no window, no dialogue, no client log line. Our 49/1 has never been natively accepted (no 49/1 capture exists). W68 extracts the sent bytes and finds the client handler's early-return conditions.

## Run 084318 (lead fix: 26/3 dialogue before 49/1)

- W68 found that `QuestStartHandler:Show` proceeds only while an NPC dialog exists. The lead changed Talk to send 26/3 first, then 49/1, in `gameplay.rs` and the W67 runtime test (passes with the round-6 fixture).
- Natively, the dialogue opens (blank bubble, "Goodbye") and the server logs `quest_offer 3000021`, but **the quest card still does not appear** after 3+ s.
- Remaining suspects are the Lua `Ui.IsInNPXTutorial()` gate (the test character is still inside the opening) and the dispatcher state check at `00c7ce65`. W69 is tracing both.

## Run 091005 (`realm npx off` diagnostic)

- Not useful. The 41/2 clear packet also starts minigame state: the client moved the player onto a hillside, and the later teleport and NPC clicks were not logged by the server. W69 found the NPX byte comes from zone details (43/31) and is already false in both the original capture and ours, so tutorial mode is an unlikely suppressor.
- Next: W70 adds an opt-in `--lua-probe` overlay that patches the per-run client copy's quest UI Lua to log guard values, so the next native run shows which check refuses the card, or whether Lua is ever reached.

## Run 102008 (`--lua-probe`, W70c source replacement)

- The probe loaded: `probe.scriptsbase.loaded`, `probe.loaded`, and hooks installed on `QuestHandler.ShowStartScreen [49/1]` and `QuestStartHandler.Show`. Character select and the world UI work normally with the source replacement.
- Talk to Samantha: the server logged `quest_offer 3000021` and the dialogue opened, but **neither hook fired**. The 49/1 never reaches Lua, so the drop is native: the dispatcher state check at `00c7ce65` (type 3 and `[ebx+0x1c]` == intro id 94246) or a native read/route failure before `00a94be0`. The Lua guards (NPX, NPCDialog, visibility, text) are ruled out. Next: W72 adds a dev-only offer variant command for a one-run native experiment.

## Run 105745 (W72 variants + W71 live log + lead outbound trace)

- Sent `realm offer 3000021` v1 (49/1 only), v2 (intro 0), v3 (intro = title 94247) and v4 (26/3 then 49/1). v5/v6 did not register because chat input was mistyped. Trace `app_out` lines confirm each 120-byte 49/1 was queued and sent as one complete ReliableData packet (e.g. seq 1347), with its cause.
- **No variant reached Lua** (`QuestOfferLuaProbe.log` shows only load/hook lines). The drop is native and independent of the intro id and of a preceding 26/3.
- Harness fixes by the lead: per-message `app_out` trace before SOE packing, with cause (`osfr-gateway.rs`); viewer arrows, lengths, hex and cause; hardened live loop (the console window died at character select; per-line errors now go to `live-log-error.txt`); `--live-log` is now on by default.

## Run 141419 (W73 widened probe; after the computer restart)

- With the widened probe, the 26/3 dialogue is fully visible in Lua: `DialogHandler.SetDialogText` (empty body), `SetChoiceText` ("Goodbye", background set 18, image 0), `DialogHandler.Show`. `Ui.IsInNPXTutorial` is false.
- The trace shows 49/1 (120 bytes) queued right after the 26/3, both caused by 26/10. **No quest Lua function runs**, and no client log records anything at that moment. The client also sends 26/20 twice after the menu choice.
- Conclusion: the client's native code drops 49/1 silently, so further Lua probing can't locate it. A native hook (Frida or a debugger on `00c7cd40`/`00c7bb60`/`00c7ce65`/`00a94be0`) is needed.
- Harness bug fixed by the lead: `live_log._parent_alive` used `os.kill(pid, 0)`, which on Windows calls TerminateProcess (here it hit an access error and made the viewer quit). It is now a read-only `OpenProcess` + `GetExitCodeProcess` check. The launcher starts the viewer directly with `CREATE_NEW_CONSOLE`; the old `cmd /c start` raised "Pick an app" dialogs.

## Frida native hook: root cause found (runs 142216 and 143124, lead)

Lead tooling: `project/scripts/native_hook.py` attaches Frida (installed into the lab env with the owner's approval) to the run's client copy and logs labelled native hook points to `native-hook.jsonl`, which `live_log.py` merges. Use `--native-hook quest_offer` on the launcher.

- **Run 142216:** `quest.dispatch` received our 120-byte 49/1, then `QuestInfoPacket::Unserialize` (`00c7bb60`) returned **0**. Reader state at `00c7bb9b`: error byte 0, cursor = start+114, end = start+120. The client parses 114 bytes and rejects trailing bytes.
- **Cause:** `ReadQuestPacketPrefix` (`00c7acd0`) reads u16 family + u32 subtype; `ReadFields` (`00c7b7a0`) then reads quest id + 5 dwords, bool, bundle, 8 bytes, float and 2 bools. Our encoder added a non-existent u16 "prefix" and a 6th screen dword.
- **Fixed (lead):** the 49/1 encoder and decoder are now 114 bytes. Inbound 49/2 (captured natively: `31 00 02 00 00 00 d5 c6 2d 00 01`, 11 bytes) and 49/14 (inferred, 10 bytes) now use the 6-byte header.
- **Run 143124:** the original quest offer window displays: speech bubble ("You meet the coolest people here in Cobblestone Village! Ricky Danger is a famous Kart Driver. Go say hi!"), quest card ("Introduce yourself to Ricky Danger in Cobblestone Village", Show Details), and "Yes, I accept!" / "No! I decline.".

## Runs 143737 and 144157: accept and persistence

- **Accept works end to end:** 49/1 → client 49/2 accept → `quest_engine` handled → 49/3 quest record. The chat shows "Introduce Yourself Quest Started!" and the quest banner appears. The quest is saved in `characters/1.json`.
- **After relog:** Samantha gives the turn-in (49/13 end screen, "Thanks!"), so the accepted state persisted. But the quest card is blank, and clicking Thanks sends no 49/14. The server does not re-send active quest records (49/3) at login, so the client has no `BaseClient.QuestData` for the quest.
- **Robustness:** an unparseable client message (the old 49/2 decoder) ended the whole session (`session_ended reason=invalid quest accept…`). It should be logged and ignored.
- **Harness:** the viewer console stole focus (`SetWindowPos` without NOACTIVATE), which blocked game input; fixed.

## Run 165644 (lead: 49/1 slot order + no leading 26/3; after W74)

- The Lua probe from run 143124 showed the client maps wire dwords as intro=1, name=3, description=4; dwords 2 and 5 are not passed to `ShowStartScreen`. The encoder fields are renamed and fixed (`field_2`, `name_string_id`, `description_string_id`, `field_5`). The extra 26/3 before the offer is removed (it was the blank bubble and "Goodbye" behind the window).
- **Result:** the offer window matches the original: one speech bubble, card title "Introduce Yourself", Show Details, Yes/No. Accept works: 49/2 handled by `quest_engine` → 49/3, "Introduce Yourself Quest Started!", and no leftover dialogue.
- **Not finished:** the turn-in (to Ricky Danger or Samantha). Clicks after `realm goto` missed the NPCs, because the player overlaps the target. Next time, approach to ~3 m and click the NPC's visible body, or add a dev `realm interact <npc>` helper.
- Disk: C: hit 0 GB. 69 per-builder cargo caches (`lab/build/W*`) and 121 run client copies were removed, keeping logs, traces, screenshots and `client/Logs`; 671 GB is now free. The launcher now deletes its client copy after each run, except `client/Logs` (`--keep-client` keeps the copy).

## Runs 195935–204017 (quest dialog flow, EndDialog, turn-in)

- **Teleport:** `dev_teleport::STANDOFF` is now 5 m instead of 2 m. At 2 m the player stood on the NPC, so clicks missed.
- **Stuck in dialog view after accepting:** the server never ended the dialog. It now sends `26/4` EndDialog after `49/2` accept/decline and after `49/14` turn-in completion. Alone that did not help (run 201127): the offer must sit on an open NPC dialog. So Talk now shows a dialog with the quest title as a choice (response id 301) plus Goodbye. Choosing it (`26/6`) sends `49/1`. After accept, `26/4` restores the HUD and camera. Verified natively in run 202324.
- `26/20` from the client is `FreeInteractionNpc` (ctor `009d9a90`, sub-opcode 0x14). It is still rejected as unknown and is harmless.
- **Blank turn-in card, and "Thanks!" did nothing:** `ShowQuestEndScreen` (`00a95420`) stores `arg2` at `+0x1c`, the id that `SendQuestEndReply` (`00a97ef0`) sends in `49/14` and that Lua `ShowEndScreen` looks up in `BaseClient.QuestData`. Caller `00c7cf78` maps `arg0`/`arg1` to the 8-byte value at packet offset 6 and `arg2`–`arg4` to the dwords at 14/18/22. We sent 0 at offset 14 (Frida: `SendQuestEndReply +0x1c = 0`; Lua `ShowEndScreen 0 ""`). Offset 14 now carries the quest id (`put_end_screen_ids`). The turn-in end screen is also held behind the same dialog choice.
- Locale: quest ids come in sets of four (intro, title, description, end text). The pet quest's end text is 94253, but 94249, the slot for Introduce Yourself, is absent from the shipped locale, so its end text stays empty.
- **Harness:** the live-log viewer now opens with `SW_SHOWNOACTIVATE`. It had stolen focus, and the driver then refused input.
- **Pending native check:** the turn-in end screen title, the "Thanks!" reply `49/14`, the reward, and the HUD restore. The battery died twice; the run waits for charge.

## Run 212855: Introduce Yourself works end to end

- Samantha: Talk shows the dialog with the "Introduce Yourself" choice, the offer window opens, Accept starts the quest and the HUD returns. Ricky Danger: Talk shows the dialog with the quest choice; the end screen is titled "Introduce Yourself"; "Thanks!" sends `49/14` (Frida: `SendQuestEndReply +0x1c = 3000021`); the server replies `49/4`, `49/12` and `26/4`; the chat shows "Introduce Yourself Quest Completed!" and the HUD is restored.
- **The protocol and UI fixes are generic.** Every quest uses the same `49/1`, `49/13` and `26/x` code paths.
- **The remaining gaps are content, not protocol.** Ricky offers nothing next (`no_eligible_quest`): "Call the Crew!" (title 94255) is not in `data/content/quests`. Of 651 records, 635 have a title, 2 have intro or description ids, and **none have objectives**; rewards are mostly empty. The locale pattern (intro = title−1, description = title+1, end text = title+2) has entries for 596/615/595 of the 635 titled quests. That makes a bulk backfill possible, but each one needs checking (a neighbouring id could belong to another quest).

## Quest text backfill and the Cobblestone chain (run 215840)

- `scripts/quest_text_backfill.py` fills `intro_text_string_id`, `description_string_id` and `end_text_string_id` from the client locale block around each title (T-1, T+1, T+2). The locale key hash is JenkinsLookup2 over `Global.Text.<id>`. It fills a field only when no neighbouring title claims that slot, the string exists, the slot is not title-shaped, and the description slot itself validates (an irregular block put a kart-race line on "Learning to Crave the Flave"). Result: 518 descriptions, 480 intros and 466 end texts across 631 titled quests; the skips are listed in `lab/reports/quest-text-backfill-20261014.json`. Snapshot before writing: `lab/snapshots/quests-pre-text-backfill-20261014`. Two samples of 8 and 20 were checked by eye.
- The engine accepts `end_text_string_id`, but it is **not yet sent**: the meaning of the 49/13 dwords at offsets 18 and 22 is unknown.
- Split-content objectives are now **ordered**, so "report back to the giver" steps cannot complete early.
- New quests: **Call the Crew!** (3000059: Ricky → Shakey → Ricky; locale 94254–94257) and **Back for More** (3000060: Ricky → Samantha; 94262–94265). "NOT LIVE - Raceway Regulators" (94259) is skipped. Introduce Yourself pays 5 coins (wiki).
- **Native result:** all three quests were played in order. Offer text matches the original. Coins went 359 → 364. Ricky does not count before Shakey; Shakey ticks step 1 (49/8, 49/9, 49/10). Regression test: `cobblestone_starter_chain_runs_in_order` (mutation-checked against `ordered: false`).
- **Gaps:** Shakey's bubble is blank (his line is locale 94258; content has no per-objective dialog field). The end screen shows no NPC end text and no reward line. Samantha's follow-ups (Samantha's Mail and others) are not in the data yet.

## Runs 224000 and 230409: end text, step dialog, talk quests, dialog camera

- **49/13 dwords** (from `ShowQuestEndScreen` disassembly): offset 6 u64 = **NPC actor GUID**, looked up in the actor table (`00984960`) for the dialog camera, `[*player*]` expansion and animation; 14 = reply/QuestData id; 18 = end-text locale id (`ExpandTextWithRoleArrayToIString` `0097c1e0`); 22 = NPC animation id when > 0. Run 224000 showed the end text (Ricky, Ranger Ricardo) with a literal `[*player*]` while offset 6 still held the quest id. The gameplay layer now writes the turn-in NPC's runtime GUID there. Run 230409: the camera zooms on Ricky with the card to his left, matching the original (the owner's MMODEN reference shot).
- **Dialog camera:** Frida set `dialog_camera` hooks `008d2ba0` (switch to `ControllerDialog`), the camera's target pair. 26/3 and 49/1 sent (0,0), so the camera framed the player. The targets come from 26/3 dwords 2–3 (`ShowDialogFromPacket` reads object +0x18/+0x1c) and from the 49/1 u64 after the reward bundle (the packet's last 14 bytes start with it; dispatcher `00c7cea2` passes object +0xe0/+0xe4). `set_dialog_camera_target` fills both with the NPC GUID. **Not yet verified natively** (the battery fell to 10%).
- Step dialog: Shakey said 94258 (`dialogue_string_id` on the talk objective). The text is longer than the bubble.
- Talk quests: `scripts/quest_talk_objectives.py` added talk objectives to 9 quests. Chef: Ranger Requisition (Auguste Frydaze → Ranger Ricardo) completed natively with the original offer and end text.
- Backfill fix: records sharing a title (the Stillwater Introduce Yourself) are now skipped; 3000022 uses description 116637 (hash 3437900159).
- **Why few quests are playable:** only talk events reach the NPC quest engine. Combat and gathering feed only the tutorial (opening) quest system, and no world enemies spawn outside tutorial encounters. Most of the 651 quests need defeat, collect or minigame objectives.

## Run 233813: dialog camera verified; overworld combat design

- The camera frames the NPC for 26/3 (Samantha, Ricky, Shakey), 49/1 (Samantha's offer) and 49/13 (Ricky). `[*player*]` now expands ("…today, Capo Indigocauldron"). Frida camera targets: 0x2_18711abd and 0x2_363e7ffe.
- Greetings: `greeting_string_id` on NPC records replaces the Merigold special case. Samantha uses 24940 (guide hash 1591220599). Only 2 of 1,199 placed NPCs have a greeting quote in the guides. `scripts/locale_id_map.py` writes `lab/reports/locale-id-map.json` (86,004 hash → id pairs).
- The "Queensfields Bushel 20m" arrows are `ScreenDirectionIndicatorDataSource` rows: Text, Subtext, IconId, ScreenAngle, Distance, Color, CharacterGuid, IsPc. They point at spawned actors, so they hold no location data. the owner's notes: red = enemy encounter, green = NPC / Active Quest, pink = resources.
- **Overworld combat is encounter-based (the owner's research):** a red crossed-swords marker → a battle menu with a difficulty choice → the minigame start screen (MiniGame type 4 = COMBAT; difficulties 3/4/5 show 4199–4201 "Requires a level 5/9/13 combat job") → the fight runs in place. Legacy places 10 `sg_spawner_*` objects, e.g. "Hooligan Brawling Club!" 4156 (73 m from Jones; hooligans 4157–4161 stand around it) and "Frostfang Snarler" 3592. Free-roaming aggro enemies (the unwired `enemies.rs`) do not match this; the plan is to reuse the encounter engine's minigame flow (114 details with `launch`, 17 minigame, rules/combat flags, objective, game over) in place, without the zone change.

## Runs 001154–010757: overworld field enemies and the first combat quest

- **Field enemies** (`EncounterEngine` + `data/content/field_enemies.json`) reuse the opening's proven world-enemy packets (35/2 template spawn, 35/5 HP, 35/12 relevance, 32/7, 35/8, 35/3; rules plus 41/132/133 while engaged). They are not saved, respawn after `respawn_seconds`, chase within `aggro_range`, walk home otherwise, and recover in place after an 8 s grace on knockout. They replace their passive Legacy copies by GUID. Optional `quest_ids` gating: the four Hooligans by Jones (Legacy 4154/4155 plus 2 reconstructed spots, kill id 930050) exist only while **Brawler: Delinquent Dilemma** (3030007) is active.
- Interaction: the X key sends 26/19 (select) then 26/8; field-enemy 26/8/26/10 now go to the encounter (Attack menu 103) instead of `npc_interaction`. Combat needs a combat job (switch with the job arrow; Medic deals 75).
- **Run 010757:** the dev offer (v0) and accept spawned the Hooligans. Kills sent 49/9 1/4 → 4/4 and 49/10. Jones's turn-in choice opened the end screen with the original end text ("You're back? I had my doubts…"), and "Brawler: Delinquent Dilemma Quest Completed!" followed. All four Hooligans despawned when the quest completed. Regression test: `quest_gated_field_enemies_spawn_fight_and_respawn`.
- Dev tools: `realm offer <quest> vN` marks the quest offered and its accept skips prerequisite, level and job checks (`dev_mark_offered`). Use **v0** (dialog plus offer): v1 leaves the HUD hidden after accept, so X only selects targets. Field enemies are teleport targets (by GUID; "Hooligan" is ambiguous).
- Balance: 220 HP / 8 damage ×4 still knocks out a level-1 Medic once per fight. the owner's video reference: a level-3 Medic vs Hooligan Bullies took 83 (124 crit) and dealt 127.
- **Issues found:**
  1. ~~Relaunch gave an empty character list~~ **Not a bug:** run 010409's own client-80 shows the list; the 'empty' view was behind the Esc menu while input went to the focused live-log window. The login traffic matches the good run byte for byte in size.
  2. Carlos Brazenfist is not a teleport or spawn target in zone 0, even though his NPC record has a position.
  3. The live-log viewer still takes focus under Windows Terminal (SW_SHOWNOACTIVATE is ignored).

## Runs 015208–025841: overworld battles (spawner → start screen → in-place fight → results)

- **Content:** `data/content/field_battles.json`, Encounter 96 "Troll Summoner Madness!". Title 42056, description 42059, goal 42071, difficulty 1. Troll Summoner (troll_shaman 168, name 15623, kill id 930096) and two Troll Novice Archers (troll_archer 183, name 42039, kill id 930097), 25 coins. The spawner position on the trail south of the Forest Troll Fort is reconstructed (R).
- **Spawner:**
  - The spawner is a world-style AddNpc added to `NpcVisibility` (`add_static_actor`) and sent by proximity. Spawned at login 1,500 m away, the client never showed it. Dev teleport already refreshes visibility through gameplay; a redundant gateway refresh was removed again.
  - **Model 1917 (`sg_spawner_troll_01`) does not render** (assets exist; its material uses `SG_TROLL_HUT_01.DDS`). The human tent 1913 does, so it is used for now (open issue).
  - Battle GUIDs use the Legacy style (100000096000+).
- **Start screen:** clicking the spawner (26/8 is routed to the encounter for field actors) sends **41/114 details with launch=false** plus **41/107 ZoneIsReady**. Case 107 calls `SetMiniGameReady` (00aa3fdf), which turns "Loading..." into **Go!**. The screen shows the title, 1/5 difficulty dots, the description, Switch Jobs and the marketplace panel. There are no prizes yet, and the default squirrel picture shows in place of a portrait.
- **Go (41/108):** the cave's entry sequence is replayed in place (114 launched, 106 states 4/5/6, 107, 2 enter, 39/17, rules on, 41/133). Enemies spawn as battle-tagged field enemies; objective 1/2 counters follow.
  - **Inside a battle the player attacks with the ability key "1" (36/10 basic); X does nothing** (it does work for overworld field enemies).
  - The last defeat sends objective complete, game over, end scores and 39/19.
  - Result in run 025841: **"YOU WIN! Troll Summoner Madness!"**, Opponents Defeated 3, Goals 1/1, Total Score 5000. Coins went 359 → 384.
  - A knockout ends the battle as lost.
- Regression tests: `battle_spawner_spawns_at_zone_ready` (proximity) and `spawner_battle_runs_in_place_to_victory`.
- **Gaps:**
  - The prizes list and its loot-spin, the battle portrait, and the level requirement on Go (difficulty 3–5 → combat-job level 5/9/13) are missing.
  - The troll spawner model does not render.
  - The **Ninja: Stolen Away** objective is not yet wired to kill id 930096.
  - The live log steals focus under Windows Terminal; the workaround is to stop the viewer.

## Runs 034140–040443: instanced battles (the owner's correction)

the owner's correction (field notes 2026-10-15): there is no tent. The marker is the walking Troll Summoner with crossed swords, Go teleports to a closed arena, and the summoner flees, turns, and summons 2–3 small trolls in a smoke puff.

- **Code:**
  - The in-place battle (`RunningBattle`, `start_battle`, `finish_battle`) is removed.
  - A `FieldBattle` now carries a `zone` and is appended to `content.dungeons` at load; `Dungeon.guide` is optional.
  - Gameplay intercepts Go (41/108): it takes the pending battle, refuses without a combat job, and calls the cave's `begin()`. Without a guide, `on_zone_ready` arms the fight at once; wins reward every time; saved battles that no longer exist are dropped.
- **Overhead icon:** AddNpc `NameplateImageId` (the last field; `009622f0` builds the `OverHeadNameIconElement` from it when ActiveProfile selects no configured icon, so the spawner uses profile 0). **Verified in run 034140:** image 11970 `icon_minigame_combat_encounter_start_hard_free_64` (crossed swords on a red burst) shows over the Troll Summoner model 168. The live game's exact id per difficulty is unknown (1345, 11968–11973 are the candidates).
- **Arena:**
  - `sg_random_encounter_clearing`, zone id 9600 (R), sky `sky_wilds24.xml`, spawn (136, −0.5, 155).
  - **Verified:** Go → arena loads (forest clearing ringed by rock walls), "Troll Summoner" boss bar, combat on. A loss shows "TRY AGAIN!" with Opponents Defeated 2 and Goals 0/1 (the goal counts only the summoner). Closing returns the player to zone 0 at the summoner.
- **Summons:** minions with `summon_below_percent` are dormant until the summoner falls below 75/50/25% (2 Troll Scoutlings 42038/182, then 3 mixed, then 2 Troll Ragelings 42040/166). Each wave sends the summon cast, 35/16 PlayCompositeEffect 16337 (`PFX_bones_yellow_troll-spawn`, layout from the community catalogue), AddNpc and HP. Run 034140: wave 1 fired at 675/900 and both scoutlings were killed server-side.
- **the owner's play test (run 040443), problems:**
  1. The troll loops one animation and never runs away. The summon used `com_cast_special_01` (1131), which `troll_shaman.adr` does not have. It also stood casting from 12 m, so it was never hit (900/900 at the end) and never fled.
  2. Trolls "phase in and out of reality but still hit you".
  3. The hit reach is far too long ("four troll lengths").
  4. Clicking or pressing 1 on the summoner never hits it. Melee targeting uses the **server** position (3 m), and the screenshot showed the troll about 3 m in front of the player while the server had it at 11.9 m. Working theory: the client keeps moving an NPC along its last 125 heading after updates stop (dead reckoning), so the two positions drift apart.
- **Fixes in progress:** summon `com_cast_02` (1112) and minion `com_spawn` (1404) (both in the troll ADRs); cast `com_cast_01` (1111); reach 7 m; flee 2 s at 4.5 m/s on a hit with a 4 s cooldown; a final stationary 125 update whenever an NPC stops (ranged and melee). To verify with a recorded fight.
- Dev teleport works inside zones that list their own targets (arena enemies). The gameplay-level "refused during an encounter" guard is still in place.

## Runs 042537–061819: summoner AI rework, recorded fights

Fights were recorded with `um win record` and inspected as `um video contact` sheets plus single frames (game-automation skill).

- **The ranged caster is removed.** It was an invention with no evidence. Its cast replayed every 2.5 s from 7–12 m with no projectile, so it looked like hits from far away, and the troll was never hit, so it never ran. Following the owner's suggestion, the summoner now uses the opening cave's proven melee chase.
- **Summoner cycle (the owner's design plus the lead's refinements, R):**
  - While its summoned minions stand, it fights with them in melee and is **shielded** (hits do 0 damage; key 1 prefers an unshielded target). the owner: "you kill those, then keep attacking the summoner".
  - Alone, it **runs only while the player is within 10 m** (3.5 m/s, slower than the player), for 4 s of running, then **rests 3 s** with com_stunned (1401) and no attacks. That rest is the damage window.
- **"Phasing out of reality" fixed.** The flee kept a fixed height and slid along a 25 m leash into the slopes at the clearing's edge, so the client drew it under the terrain (run 053415). The runner now takes the player's ground height and stays within 12 m of its spawn. Run 055357 shows it on the clearing floor throughout.
- **Minions:** two per wave at 75/50/25%, AddNpc scale 0.65 (bytes 33..37), com_spawn (1404). The summon cast is com_cast_02 (1112); troll_shaman.adr has no com_cast_special_*. Run 061819 confirms small "Troll Scoutling" minions beside the summoner.
- **Native win (run 061819), before the shield:** "YOU WIN! Troll Summoner Madness!", Opponents Defeated 1, Goals 1/1, Total Score 5000, coins 359 → 384. The 20 s burst showed why the summoner needs the shield.
- **Review guard respected:** melee reach stays the reviewed 3 m (`tests/piece_054.rs`). A 4 m slack attempt was reverted.
- **Dev teleport:** works inside battle arenas, where targets are the standing arena enemies. Chat-typed `goto` was an unreliable driver; `chase5.py` (scratch) steers with A/D/W from server-logged positions over one persistent WinDrive session. Note that injected input resets the Windows idle timer, so the user idle check only works before driving starts.
- **Gap:** job abilities are not implemented (only the basic slot 0). Balance is a stopgap until the Medic heal and the rest exist (Build Ledger item 12).

## Runs 081330–131959: Medic abilities, prize list, loot wheel, wandering summoner

- **Medic ability bar (run 081330):**
  - The bar is Incision (key 1), Triage (key 2) and First Aid (key 3), from `data/content/job_abilities.json`. Each job ability gets a 36/13 definition and a 36/5 toolbar slot.
  - Keys 2 and 3 send 36/10 with bar 1 and slots 1/2, target type 0, guid 1 (seen in the trace).
  - Triage hits for 3.2× the basic attack and heals 254, using all 100 energy. The client greyed the icon and the energy bar dropped to 4, then refilled (38/13).
  - First Aid heals 250 on a 15 s cooldown.
  - Refusals use 36/1 with 285 (energy), 287 (not ready) or 288 (no target).
  - Not yet seen on screen: the green 35/35 heal number.
- **Level locks (the owner):** every job skill carries `min_level`. A locked skill is an empty bar entry that keeps its position, and is refused by the server. A level-up that unlocks a skill re-sends the bar. Battle wins now give 50 job XP (R). Before this, battles levelled nothing.
- **Start-screen picture:** MiniGameInfo.IconId reaches the UI (`ii://` image id), but this build's `MinigameStartScreen.gfx` has no icon clip, so no IconId changes the picture. The squirrel is placeholder art that is hidden when the reward preview list is non-empty. IconId stays at 4543 (the right id).
- **Prize list (run 130004):** the third MiniGameInfo reward bundle ("preview", record +0x188) carries type-1 Item entries in this order: hidden, icon, tint, name, quantity, item id, param, empty string, colour, members-only. Result: **"Prizes: Win the game and you could win one of these prizes!"** with four icons and names, and the squirrel gone. The prizes are a reconstruction modelled on the wiki's Medic prize wheel.
- **Loot wheel (run 131959):** after a win the client opens the wheel by itself. It spins only after **39/45** `MiniGameLootWheelSetItemToLandOn` (header plus one reward bundle). The wheel lands on the preview entry with the same name id. The client answers **39/46** when the spin stops.
  - Verified: the wheel spun about 8 s, landed on Battle Item Mystery Pack and showed "Congratulations! You won". The server granted the same item (10482 is in the saved character), with coins 359 → 384.
  - It then moves to the score screen with "Your Rewards".
  - Unconfirmed: closing that score screen. One click sent nothing, and later input was blocked because Explorer held the foreground.
- **Wandering marker:** the battle marker walks four points 5 m around its home at 1.5 m/s with 3 s pauses, only while a player is within 90 m. Within 8 m it stops and faces the player, because clicks missed it while walking. Run 124449 showed it walking with its crossed-swords icon.
- **Open:**
  - Score-screen close after the wheel.
  - Logging in from a save left in a won battle. Run 143711 stalled at character select, probably because input couldn't reach the game (the user's foreground); not diagnosed.
  - Balance rebase onto original numbers.
  - Target Vitals, Vitamins and Shock Paddles.

## Runs 150132–184205: reconnect after a battle, refused Go, full loop closed

- **Logging in after a battle win was refused** with "incompatible encounter state". The win had put the repeatable battle's id into `rewarded`, which load validates against the opening content only (battles load later).
  - Fix: repeatable battles are never recorded, and unknown ids are dropped at load (the content hash still pins the opening content).
- **Next refusal: "saved zone differs from gameplay encounter".** The character was saved inside the arena (zone 9600), but the saved battle is dropped at load.
  - Fix, mirroring the kart reconnect: the encounter keeps the dropped battle's return location, and the gateway moves the character there on both load paths (`abandon_battle_for_reconnect`).
  - Run 182347 logged in from that save into zone 0 at the summoner's trail. Covered by a regression test (win, save, reload).
- **Refused Go hid the HUD:** with a non-combat job active, Go was refused and the start screen closed with the HUD hidden (soft-lock).
  - Fix: the refusal re-sends the start screen (41/114 plus 107) with the message.
  - Run 184205: the screen stays up, its X closes it, and the HUD returns.
  - The screen's **Switch Jobs** button is disabled in our flow; whatever enables it is unknown (follow-up).
- **Full loop verified (run 184205):** wandering summoner → start screen with prizes → Go → arena fight (Medic) → win → wheel spin and landing → "Congratulations" → score screen → X → back in zone 0.
- **Notes:**
  - A prize tooltip opened from the start screen can stay on screen afterwards (client glitch).
  - A running gateway older than the data rejects new fields: `job_abilities.json` is read live from the repo, so rebuild before testing data changes.
