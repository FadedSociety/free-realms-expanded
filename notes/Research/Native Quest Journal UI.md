# Native quest journal UI

2026-10-08 UTC. The preserved client UI and native callbacks support the new opcode49 opening journal. The native miscellaneous datasource has no additional quest flag filter for reserved opening IDs. This alone does not prove the rendered Other Quests page works. Original Hero's Journal chapter/sticker metadata is a separate opcode209 system and remains unrestored. Actual run084544 now accepts the rendered title, description and persisted objective states after a second Other Quests bookmark click. Live updates remain a separate acceptance gate.

## Original UI recovery

Applied the existing [unluac community tool](https://github.com/viruscamp/unluac), pinned `9c59af46483a932a47224bf2ac84ad8a63094da5` (MIT), to supplied `UI/ScriptsBase.bin`. The file has a standard Lua5.1 bytecode header and is 2,897,335 bytes, SHA256 `d301f2c4206db9c2a5714d79b703914763383edd5918dd2087fcb6e1c661956d`. unluac version1.2.3.561 compiled with the existing private JDK21 and produced 2,121,717 bytes of inferred Lua. Its full disassembly reassembled to **byte-identical original bytecode**. This proves a complete lossless bytecode parse, not original source names/formatting or recompilation of the inferred Lua source.

Private output: `lab/decompiled/client-ui/ScriptsBase.lua`, `.asm` and `ScriptsBase-roundtrip.bin`. Tool classes: `lab/tools/unluac/classes`. Native extracts: `lab/reports/quest-ui-native-functions.c/.json`; bounded vtable/callback evidence: `quest-ui-vtables.json`; independent Capstone5.0.7 x86 disassembly: `native-quest-ui-disassembly.txt`. Raw/decompiled game material stays outside Git.

```powershell
$taskUnluacSources = Get-ChildItem -LiteralPath '../lab/references/unluac/src' -Filter '*.java' -Recurse | ForEach-Object FullName
& '../lab/tools/jdk/jdk-21.0.12.1+1/bin/javac.exe' -d '../lab/tools/unluac/classes' $taskUnluacSources
& '../lab/tools/jdk/jdk-21.0.12.1+1/bin/java.exe' -Xmx2g -cp '../lab/tools/unluac/classes' unluac.Main --output '../lab/decompiled/client-ui/ScriptsBase.lua' '../lab/working/OSFR.Client/OSFR Client/UI/ScriptsBase.bin'
```

## Journal route and visibility

`HotKeyEvents:ToggleQuests` (Lua38984) toggles **StoryBook**, not legacy `wndQuestScreen`. It refuses to open during Atlas, Stuff, minigames, overworld combat or disabled hotkeys. The cached [Quests article revision45209](https://freerealms.fandom.com/wiki/Quests?oldid=45209) documents physical **L**. The native Lua proves the handler destination. The later combat-controls audit recovered the original default key table at0x1b41988 and confirms action31 DockOpenQuests uses physical L (0x4c).

`StoryBook:Show` (14646) enables the hub, sticker and miscellaneous quest datasources. `PopulateOtherQuests` (14788) reads `BaseClient.AdventurersJournal.MiscQuestData`; `PopulatePageTabs` adds the Misc tab when this datasource has rows. Selecting a miscellaneous quest calls `QuestScreen.SelectMiscQuestFromBook` and reads `BaseClient.MiscQuestObjectiveDataBook` (14878).

The native miscellaneous datasource constructor is `00c98ce0`; rebuild `00c97de0` iterates **every active native quest record** and includes a record when `00a432d0(questID)` is false. That predicate checks the 32-bucket hub-quest **definition** map, not a quest visibility flag. Reserved reconstructed IDs910001+ are absent from original hub definitions and therefore belong in Other Quests. Pinned Sanctuary `AdventurersJournalInfoPacket` corroborates opcode209/subtype1 dictionaries in order: Regions, Hubs, HubQuests, Stickers. Opcode49 records alone do not restore that original chapter book.

## Helper and live refresh

The helper is initially enabled but minimized in supplied `UserOptions.ini`. `QuestHelper:ShowHelper` (15039) hides it while StoryBook is visible, `HUD.isInStartingZone` is true, an activity portal is open, the tutorial forbids it, or a non-fishing minigame is active outside tutorial. The NPX tutorial can disable its journal button. Expand the helper to see its goals; an 80px minimized helper is not evidence of missing quest data.

`QuestHelper:PopulateHelper` (15133) calls `QuestScreen.SelectQuestInHelper`, reads `BaseClient.QuestData` and `BaseClient.QuestObjectiveData`, adds goals and enables objective datasource script events. Native49/3 dispatch `00c7cd40`→`00a92680` with select=true calls `00a8f2b0`, which invokes `QuestHandler:PopulateHelper`. Native49/9 mutation `00a931b0` calls `00a8f130`→`QuestHandler:UpdateObjective` independently of the optional presentation bool. Subtype10 `00a936e0` likewise updates objective UI. Lua quest/objective datasource update handlers refresh journal/helper and minimap targets.

Runtime-key correction: those callbacks occur only after a successful lookup. Original00baf380/00bab950 rekey decoded objectives by definition ID when creating the runtime quest. The raw wire tuple ordinal is therefore not a valid live-update key when it differs from the definition ID. [[Native Journal Live Objective Keys]] records the executable proof and narrow49/9+10 repair; its new native acceptance remains pending.

## Proven field names and states

| Field | Proof | Result |
| --- | --- | --- |
| RootUnknown3, manager+0x10f8 | Lua binding SetAutoSwitchSelectedQuest→00c0a200→00a10880→00c7c5d0; getter00c0a260→00a10870 reads the same byte | AutoSwitchSelectedQuest; current fresh false is valid, and Add's force-select=true still selects |
| QuestUnknown5,+0x10 | Native QuestData/MiscQuestData column9 and Lua column name | MembersOnly; current false is appropriate |
| QuestUnknown11,+0x154 | Same native column12 and Lua column name | SystemQuest; current false does not hide it |
| QuestUnknown7,+0x20 | Datasource job badge/level/icon lookups | Profile reference; zero gives neutral presentation |
| QuestUnknown10,+0x150 | Native/Lua column mapping | IconId; zero is neutral |
| Objective state1/state2 | Objective datasource00cf7fd0 | Active GFXstatus2 / complete GFXstatus3 |
| Current/required,+0xd4/+0xd8 | Formatter00a8b9a0 receives these fields | Active goals with required>1 use localized count/total formatting |

Setter00c7c5d0 inverts AutoSwitchSelectedQuest when constructing the client SetQuestLock packet; this explains the old lock terminology. The native client immediately applies a helper selection and refreshes the datasource before emitting its selection request. Server persistence of manual helper selection was not established; reconnect currently selects the first active campaign entry by reconstruction policy.

QuestUnknown4 is an additional locale helper text, not a hide flag. Unknown8 and wire-last Unknown12 remain incompletely named; no evidence requires them to be true for visibility. Empty rewards/neutral unknowns preserve valid serialization but do not recreate original dialogue or reward-wheel presentation. The reconstructed Brawler-selection goal currently uses locale IDs0 and may show blank native goal text; its counts still serialize correctly. See [[Native Opening Quest Journal]] and [[Native Creator Delivery Proof]].

## Native rendering accepted in run084544

Actual original-client run `client-baseline-20261008T084544Z` now proves the typed reserved-ID journal renders. Pressing L opens the chapter book. The first Other Quests bookmark click left both parchment pages blank; a **second click of the same bookmark** displayed Other Quests1/30, The Last Best Hope, its description, a green completed Defend Darkthorne from the Gloam objective, and pending Talk to Pappy Farnum. Saved quest910002 counters are [1,0]. No membership, SystemQuest or profile flags were changed.

The original StoryBook.gfx was read with existing JPEXS26.3.0 into private AS2/XML output. Lua14788 passes every MiscQuestData row into `addQuestListItem`, and Lua14846 only creates the Other Quests tab when native rowcount is positive. Native00c97d70/00c97de0 independently count/build all active quests absent the hub-definition map; these do not hide quests by membership, system flag or profile. The movie pushes every supplied row. An undefined fifth `isQuestActive` argument only makes the completion boolean false; it does not filter the entry.

The original movie supplies the presentation explanation: `pageTabClicked(-1)` shows the miscellaneous page, then `ReturnToPage(-1)` hides both pages for the page-flip animation. Terminal animation frames17/35 call `setPageType` to show the chosen page. Clicking the already-selected -1 tab again shows the miscellaneous page and skips this hide/flip branch. The blank screenshot also lacked the page's static Other Quests/Objectives labels and count. The successful repeat click therefore establishes a page-visibility problem, not missing journal rows. The exact reason the first animation failed to restore its page remains unresolved; no movie, Lua or visibility-flag patch is justified by current evidence.

Private evidence: `lab/reports/journal-other-quests-rendering-audit.json`, `lab/decompiled/client-ui/storybook/scripts`, and run084544's `quest-list-tab_small.png` / `quest-tab-second-click_small.png`. Every original movie/dependency already has verified catalog CRC. This accepts title/objective rendering after reload; live counter updates and the remaining opening playthrough still require their own native checks.


## 084544 rendered-page investigation

The first bookmark click left the content page blank, including static headings. The later second-click screenshot accepts actual title, description, count1/30 and objective states [1,0], as recorded in the preceding peer section. Native row/field validity is now corroborated by visible rendering; the first page-flip failure remains unresolved.

Independent focused extract `lab/reports/native-misc-quest-filter-proof.json` rechecks both00c97d70 (GetRowCount) and00c97de0 (rebuild). Both iterate the same live quest list and exclude only IDs present in the hub-quest definition map through00a432d0. Neither checks SystemQuest, membership, profile, objective state, acceptance time or title. Column getter00c985f0 also has no hidden-record condition. Profile0 only produces neutral job badge/icon/level. Native constructor00baf490 resolves TitleID, DescriptionID and HelperTextID through the locale manager to cached strings; no external quest definition is needed for that lookup.

The original StoryBook movie47511 bytes, its1077-byte dependency list and all33 unique listed assets match the preserved catalog CRCs (`run083946/analysis/storybook-asset-proof.json`). No StoryBook asset failure or concrete Lua error is present in the reviewed logs. Nil datasource hazards in original Lua remain possible code paths, not observed errors. Peer review explains the repeat-click route in the preceding accepted rendering section. Changing arbitrary wire flags is unsupported by the native evidence and successful rendered result.


## Independent campaign restart proof

Run084544 has2,258 recorded frames,2,239 gateway frames and2,237 independently verified post-session CRCs with complete reliable reassembly. Its whole153,719-byte self payload matches the unmodified supplied C# serializer. The saved campaign restarts with wallet70, eleven inventory entries, rewarded encounter411 and only active quest910002 with counters[1,0]. Original native objective states are complete2/current1/required1 and active1/current0/required1. The selected quest is910002. This is the real campaign save, not the synthetic controls fixture.

Every existing gameplay section and both identity files are exactly preserved across the run. The only additional gameplay field is the expected optional `smelting` default: version1, next instance77000001, completed0, active null. The initial snapshot is byte-identical to the preserved post-cave parent60198b; ending position/facing match the last native movement packet. Original World Ready reports the restored starting location. No new rewards, combat results, quest mutations or zone transitions occur in this restart/UI run.

The independent reviewer also inspected `quest-tab-second-click_small.png` and confirmed the peer's title/description/green-check/pending-marker result. Full restart proof and reproduction: `run084544/analysis/native-campaign-restart-proof.json`, `lab/reports/verify-native-campaign-restart.py`; see [[../Reviews/Native Campaign Restart 084544]]. No input or save/shared-code edits were performed by this audit.
