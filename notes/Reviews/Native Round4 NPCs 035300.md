# Native Round4 NPCs 035300

These runs test integration round 4 (W59), plus the lead's fixes, in the native client. Runs `lab/runs/client-baseline-20261014T033541Z`, `…034234Z`, `…034759Z` and `…035300Z` each start from a fresh copy of the kart-weapons test character.

## Lead fixes, in order of discovery

1. **Character-select appearance.** W59 had replaced the character-list row payload with a base64 name. It is restored to the appearance record; character select still renders the model (033541).
2. **No content NPCs spawned (033541).** NPC visibility was keyed by the regional content `zone_id`s (package labels such as Lakeshore = 8), but the player is in world zone 0. All regional and Legacy spawns share one continent: Legacy's world spawn (-1904.9, …) matches our spawn (-1901, …). `world.rs` now puts every content spawn in `OPEN_WORLD_ZONE = 0` (R). `npc_interaction::check_range` accepts the open-world zone and lets the 12 m distance decide reach.
3. **Only data-backed placements spawn.** Regional rows with reconstructed POI or zone-centre positions (621) are skipped, so 578 Legacy-placed regional NPCs and the extra spawns appear.
4. **NPCs tipped over (034234).** Ricky Danger rendered lying sideways. AddNpc `rotation` is a facing direction `(sin h, 0, cos h, 0)`, not a quaternion: every captured row has y = w = 0, and Legacy's heading convention matches. Fixed via `world::facing`. In 034759 Ricky stands upright. The owner spotted the NPC on the left.
5. **NPCs not interactable (034759).** Clicking walked to Ricky but opened no menu. Content spawns used the derived all-zero `NpcSpawn::default()` (`is_interactable` 0, `interact_range` 0). `world::content_spawn_template` now copies the constants that all 23 captured AddNpc rows share: `is_interactable` 1, `interact_range` 100, `disposition` 1, `movement_type` 2, chat-bubble colours, animation and profile fields.

## Result (035300)

| Check | Result |
| --- | --- |
| Character select renders the appearance | Passed |
| Content NPCs near spawn (Ricky Danger, kart crew, others) | Passed: upright, with nameplates |
| Click NPC → radial menu with "Talk" (26/8 → 26/9) | Passed |
| Talk → dialogue window titled "Ricky Danger" (26/10 → 26/3) | Passed: the window opens, the body is empty |
| Leave the dialogue | **Failed.** The dialogue has zero choices, and neither Esc nor clicking closes it, so the player is stuck in dialogue camera |

## Open (next integration round)

- 26/3 must carry at least one close/"Goodbye" choice, or the native close path must be found (reader `00a9ef10`, 20-byte choice rows). Dialogue text content is unrecovered.
- Content quest offers, vendor shop window and buy/sell are still untested natively. The dialogue lock must be fixed first.
- `knocker_m_boss.adr` (an extra spawn near the opening) has no base model in the client (`ActorFailures.log`). Filter extra spawns whose model files lack a `.dme`.
