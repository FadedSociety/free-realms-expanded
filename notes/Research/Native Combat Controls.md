# Native combat controls

2026-10-08 UTC. Read-only evidence from the preserved original executable, original Lua and community serializers. This note supports testing; it does not claim successful player attacks through the action bar.

## Original defaults and separate target routes

The original PE contains a 24-byte-stride keybinding table at `0x1b41988`. Forty-eight records were read independently, including these defaults:

| Action | Action ID | Original keysym | Key |
| --- | --- | --- | --- |
| ActionBarSlot1 | 19 | 0x31 | 1 |
| ActionBarSlot2–9 | 20–27 | 0x32–0x39 | 2–9 |
| TabTargetSelect | 39 | 0xff09 | Tab |
| TriggerClosestInteraction | 45 | 0x58 | X |
| DockOpenQuests | 31 | 0x4c | L |
| DockOpenAtlas | 28 | 0x4d | M |

These are executable defaults, not proof that a user has never changed a runtime binding. The original Windows-to-keysym mapper `004426b5` corroborates the key representation.

Native input dispatcher `00b02880` sends action39 to `00a2ac20`. This scans eligible nearby NPCs, requires hostile Disposition0 at actor+0x178, rejects the dead flag at +0x6f8 bit7, checks a positive dot product against the player's facing direction, and performs a line-of-sight test. It then cycles the eligible targets and assigns the selected GUID through `008c6c10`. Facing the enemy matters; this is not an unrestricted target-anything key.

Action45/X instead reads the cached closest-interaction GUID and invokes `0093bff0`. Original Lua shows that the prompt reads `BaseClient.ClosestInteractionTargetData`, and clicking it calls `Ui.DoClosestInteraction()` (43501/43539). This is separate from combat target selection. Repeated X can therefore keep reopening Darkthorne while a selected combat target is handled elsewhere. This audit does not infer the precise closest-target ranking rule or claim that X must select an enemy.

The supplied `Cursors.txt` identifies cursor1 as `cursor_interaction_combat` and cursor5 as `cursor_talk`, matching the enemy/guide relevance records.

## Action-key dispatch and current investigation

ActionBarSlot1 dispatches index0 through `008a7eb0`; slots2–4 dispatch indexes1–3. Lua `GameDock.OnUserEvent("activateAbility", index)` calls `Ui.ClickedAbility(index)` (30481–30488), with separate fishing/minigame ability cases.

The native dispatcher first checks `008a5850`. Types3,6–9,14–16,22–23 and1009 route through `008a7e40` mode4 to `009b6590`, which emits Lua `MiniGameFlash:OnKeyPressed`. Other active minigames with current-game+0x61 false route through mode3 to `009aaeb0`→`00c44160`, the minigame toolbar. Normal job toolbar activation uses mode1→`009cf430`→`00a36a70` after state/mana gates.

**Field correction:** an early provisional interpretation called the cave details' first value8 its minigame type. The exact pinned community serializer shows that field is **ZoneContext**, not MiniGameType. The actual MiniGameType is the last1 in `[title,-1,description,1,profile,1]`; original Lua identifies combat type4. Do not change ZoneContext based on the provisional interpretation. `gateway_oracle` owns the current MiniGameType1 / +0x61 / toolbar-reader investigation and any recommendation. Key defaults and the conditional native dispatch path are proven; which toolbar the current cave activates still needs that verification.

## Reproduction

`lab/reports/extract-native-combat-controls.py` verifies the preserved PE key table and extracts 14 relevant native functions with original pseudocode line numbers into `native-combat-controls.c/.json`. It records the executable hash and the field correction above.

No live input, launch, shared source edit or saved-progress change was performed by this research.
