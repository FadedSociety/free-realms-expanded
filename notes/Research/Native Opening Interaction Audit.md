# Native opening interaction audit

Later source correction: [[Native Interaction Acquisition Range]] proves that byte133 is only the community field label `InteractRange`; the original closest/X prompt radius instead comes from the separate float237 (`Unknown51`), currently0→actual global7 m fallback. The historical framing audit below remains useful for packet layout, but its byte133 label must not be read as proof of native prompt acquisition distance.

2026-10-08 UTC. A read-only comparison found no packet-layout mismatch in the current opening actor/menu integration. This checks generic native interaction framing against pinned Sanctuary source and the supplied-server NPC template. It does not establish a successful native opening playthrough or recover original quest-dialog scripts.

| Packet | Direction | Verified layout / current behavior |
| --- | --- | --- |
| 26/8 | Client → server | 13 bytes: u16 opcode, u16 subtype, u64 actor GUID, bool. Same-zone proximity is checked; this requests a menu without advancing the opening. |
| 26/9 | Server → client | Actor GUID, auto-select bool, option count, seven i32 fields per option, UTF8 name, two final bools. The single opening option uses local action ID1. |
| 26/10 | Client → server | 16 bytes: u16 opcode, u16 subtype, u64 actor GUID, i32 option ID. Selecting option1 rechecks proximity and performs one typed talk/gather/craft interaction. |
| 35/12 | Server → client | Count and entries containing GUID, has-cursor bool, conditional cursor byte, final bool. Cursor5 is `cursor_talk` in supplied `Resources/Cursors.txt`. |
| 35/2 | Server → client | The verified 334-byte NPC template has `IsInteractable=true` at offset224 and original range100 at offset133. Per-actor fields are patched before inserting the variable name at106. |

The menu's first bool is false, so the community helper's single-option auto-selection is disabled. Its final `[1,0]` matches the community helper. `InteractionData.DescString` exists in the class but is not serialized; the Rust menu correctly emits seven integers rather than eight. NPC labels use recovered locale IDs3300 (*Talk*) and432156 (*Interact*).

The content currently has27 actors,24 of them non-enemy actors that can receive native menus. Acceptance requires observing the talk cursor and visible menu, then the actual client's26/10 option1 for the intended actor and corresponding persisted journal progress. Acceptance, objective actions and turn-in can require separate player selections. Out-of-range or wrong-zone interactions must leave progress unchanged. After cave loading, the guide and return interaction require the same checks.

The original quest IDs, dynamic spawn bindings and full quest-offer/dialog behavior remain separate evidence gaps. The current generic menu action binds the locally reconstructed opening progression. Native49 journal layouts are documented in [[Native Opening Quest Journal]]. Earlier actual sandbox menu gameplay supports protocol compatibility but does not prove these opening actors and objectives work together.

Private source hashes, exact template fields and acceptance gates are saved in [native-opening-interaction-audit.json](<%USERPROFILE%/Documents/Projects/CartoonMMO/lab/reports/native-opening-interaction-audit.json>). Reproduce with `lab/tools/analyze-opening-interactions.py`. No shared Rust files, content, identity or live game were changed by this audit.

## Original terrain and nearest-target controls

During native run `20261008T072258Z`, original `FabledRealms` terrain was checked with the pinned, unmodified ForgeLightToolkit GCNK reader. Its exported global triangle coordinates match the native initial position's ground height within0.043 units. Darkthorne's local floor at `[-1897,412.6]` in X/Z is **-40.148827**, making the configured Y=-39.7 about0.449 above render terrain. The journal floor at `[-1901,418]` is **-40.383523**, about0.684 below its configured Y. The player's later `[-1873,-46.76,415]` sample is downhill, about25–29 units from these actors and outside both12-unit interaction radii. It does not justify lowering their Y to-46.76.

The supplied original models were read with community ADR/DME readers. `sg_book_01_lod0.dme` is a thin book: local bounds span0.684×0.156×0.914 units. `fairy_darkthorne_f.dme` spans1.772×2.424×0.642; its ADR separately records model-scale2.333 while the supplied NPC packet template records scale1. The exact native use of that ADR field, animation and ray-hit bounds has not been established, so these are model measurements rather than a claim about final screen size or click volume.

The recovered original Lua confirms nearest-target interaction is an intended control. `ScriptsBase.lua:32784` names `TriggerClosestInteraction`; `32813` reads the mapped key from `BaseClient.ControllerOptions`. At `43501`, the prompt follows `BaseClient.ClosestInteractionTargetData`, and at `43539`, clicking that prompt invokes `Ui.DoClosestInteraction()`. The prompt can be hidden by its stored option, disabled interaction, a scoreboard, housing editing or action progress. The native run's tested key is X; the Lua does not hardcode that key.

Approach Darkthorne near **[-1895,-40.3,411]** (render floor-40.350421) and verify the prompt names Darkthorne before using the interaction key or prompt. For the journal, approach **[-1902,-40.37,419]** (floor-40.419285). These points put the intended actor closest, avoiding an X press selecting the journal while the player intended Darkthorne. Selecting the visible menu remains a separate action. No runtime placement, key binding or live input was changed by this investigation.

This establishes nearby render ground and the intended UI route. It does not prove the collision/CDTA surfaces, line-of-sight policy or a numeric native mouse-hit threshold. The original dynamic placement and quest bindings remain reconstructed. Reproducible evidence and source hashes are in [native-opening-floor-targeting.json](<%USERPROFILE%/Documents/Projects/CartoonMMO/lab/reports/native-opening-floor-targeting.json>), with raw community mesh output in `lab/reports/starting-area-floor` and `lab/reports/opening-actor-geometry`. Private helpers are `lab/tools/analyze-starting-floor.py` and `lab/tools/analyze-opening-targeting.py`.
