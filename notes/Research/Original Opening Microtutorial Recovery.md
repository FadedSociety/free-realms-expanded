# Original opening microtutorial recovery

Read-only checkpoint, October8,2026. The smallest grounded next experiment is the original Hero Journal close objective. Camera objectives have a recovered native mechanism; the three original globe and barrier bindings are still missing. No production source, campaign, identity, movie, native input or active capture analysis was changed.

This continues [[Research/Opening Historical Coverage Ledger]]. It does not make the reconstructed17-stage opening historically complete.

## Evidence

Private reproducible proof: [original-opening-microtutorials-v2/proof.json](<%USERPROFILE%/Documents/Projects/CartoonMMO/lab/reports/original-opening-microtutorials-v2/proof.json>), SHA256`b8818121efe2bc6a0f9d1929dfa5fa250119fa223702db34760cd9bba63dc6ee`. It preserves45 original native function extracts,274 machine instructions, original Lua ranges, source identities and community provenance. Reproducer: [trace-original-opening-microtutorials.py](<%USERPROFILE%/Documents/Projects/CartoonMMO/lab/reports/trace-original-opening-microtutorials.py>). It reads preserved originals/community sources and writes only its private, identical-or-new report.

Original PE SHA256`26cc1c522a45e71fb57882da515f65bac45c8c8b8c7e740c0cea81c52a733351`; original Ghidra export`f84a58674b8e7de0829d562e5f106a368f34d4750894a14b093ae745fa0befe3`; unchanged unluac ScriptsBase.lua`c7c71c7cbb177c975a7cdb12ed0c2e1b74d662f5ca5ef6a4e9ff86d2d332a8f6`.

Community research was checked first: pinned FreeRealms-Legacy packet-name catalogue and original header/string writer, plus pinned SanctuaryPR76 objective/encounter writers. They identify the objective types and existing Tutorial fields but do not implement these complete microtutorials. Native and Lua evidence fills that specific gap; no new asset-format parser was written.

## Journal Close is a real client objective

Original `StoryBook:Hide` at ScriptsBase.lua14688 returns when Hide is disallowed or the book is already hidden. A successful Hide restores movement/keyboard and hides the original book. Only when `Ui.IsInNPXTutorial()` is true does it kill the four book tutorial timers and emit `Ui.OnObjectiveEvent("onStoryBookClose")` at14722. Ordinary journal closing outside that native mode emits no such objective event.

The original method table binds `OnObjectiveEvent` to00c1f0a0. That reaches009aaf50→00c43140, which dispatches to enabled, unfinished client objectives. `ClientObjectiveUIEvent`00cf3920 stores the objectiveID and event string; its00cf3900 callback compares the exact IString through00977030. Matching invokes00cffba0, builds `ObjectiveClientCompletePacket`00cff7d0, sends it through00cffa70 and marks the client objective completed.

The original wire formats are:

| Direction | Packet | Exact fields |
|---|---|---|
| S2C |45/10 UIEvent|u16(45),u8(10),i32 objectiveID,i32 string byte length,string bytes|
| C2S |45/7 ClientComplete|u16(45),u8(7),i32 objectiveID;7bytes total|
| S2C |45/8 ClientClear|disables the registered client objective byID|
| S2C |45/9 ClientCompleteFailed|clears its completed flag while enabled, allowing retry|

Canonical community `Write(string)` omits an on-wire NUL. Native009b2860 bounds the string and00894b10 appends its own NUL. The dispatcher additionally reconstructs a C-string through006fa730, so a NUL-bearing variant is not proved to fail matching. V1's stronger claim was incorrect and is explicitly superseded byV2; no production handler used that assumption.

Unlike45/1–4 HUD updates, the original45/6 and45/10 registration branches in009be130 do not require an active MiniGameState. Do not manufacture an encounter merely to install a journal-close objective.

## Native tutorial mode and authentic prompts

`Ui.IsInNPXTutorial`00c16fb0 reads player+0x780. Original43/31 zone readers map the community `Tutorial` byte to008a89a0; true sets that flag and creates `ClientTutorialManager`00a46cd0. False through this particular setter does not clear a previously true flag. `Ui.TutorialCancelled`00c16ff0 does clear it through008a9200. Original encounter-enter/details paths also set that flag, but their other state changes make them unsuitable as an assumed general-purpose journal-mode toggle.

`FirstTimeEventHandler.FtesNpxBookExit` at40819 shows the original book, allows Hide, disables hotkeys and starts `book_exit` tutorial-arrow flow. Its original text ID is5100079. `FtesNpxDppEnd` reenables hotkeys and job switching; `FtesNpxDppHide` hides the directed prompt. Neither is proved to clear native+0x780. A clean mode-exit path and all ClientTutorialManager side effects therefore remain a prerequisite to production integration.

Original107/5 `FirstTimeEventScriptPacket` parses a length-prefixed script name, integer parameter list, eventID and clear flag.00c893f0 splits an object.method name, reconstructs object:method and calls the Lua bridge009797a0 with the object as self. This is an original fixed callback mechanism. Historical eventIDs, flags and server ordering for the NPX4.0 sequence are not recovered; an implementation must disclose any reconstructed ordering and whitelist the exact original callbacks it needs.

## Camera and barriers

Original45/6 `ObjectiveLookAtPacket` carries objectiveID, targetGUID, then either a Vector4 target position for the native empty-GUID sentinel or a float vertical offset for an actor target, followed by two32-bit distance/angle parameters.009be130 converts those last integer values for `ClientObjectiveLookAt`. The actor offset is scaled by the original float0.1 at017e795c; angle is converted from degrees using0.01745329238474369 at01b7dc74.00cf36a0 resolves the target/camera direction, applies the configured range/alignment test and emits the same45/7. Per-objective historical values and the runtime empty-GUID value are not recovered here. Ordinary126 camera telemetry is not that completion request.

Original NPX4.0 callbacks `FtesNpxDppCamera1/2/3` show text IDs10258/10259/10260, play sounds18264/18265/18266 and blink the right mouse tutorial overlay. Movement1/2/3 use10261/10262/10263, WASD and then Space. Older retained CameraMovement tutorials are a separate path; their coexistence does not establish one combined sequence.

Original Models.txt3380 identifies model4198 as `sg_npx_cavern_01_rockbarrier_01.adr`. The corresponding retract, shoot-up and deployed animations are present in the original asset catalogue. The existing community cave exports are room meshes, not a proved dynamic-actor layout. No original three barrier coordinates, GUIDs, triggers, collision transitions or camera globe model are established. Model875 `invisible_cube_orb_purple.adr` is only a candidate and must not be installed as the globe without evidence. Generic45/11 `ObjectiveFirstMovementPacket` is recovered, but its relationship to barrier goals remains unproved.

## Narrow candidate and acceptance

Prepare a fresh private tutorial campaign branch after the current native baseline milestone. Use an explicitly reconstructed test objectiveID if the original cannot be recovered, install only `onStoryBookClose`, and activate the authentic original journal/arrow path. Measure tutorial-mode entry and exit before connecting it to campaign rewards. Keep the current17-stage content/save unchanged.

The server must accept only exact7byte45/7, the expected outstandingID, correct player/zone/phase and once. It cannot infer which UI event occurred from that packet alone. Add rejection checks for unknown/stale/duplicateIDs, premature or wrong-zone completion and interrupted restart. Reuse the original community writer and native reader as the framing oracle.

Actual acceptance requires the original journal and arrow to appear, normal player Close to emit the real45/7 and advance only that objective, no completion from hidden/disallowed/outside-mode Hide, normal controls restored, exactly-once reward if any and restart preservation. Root alone drives ordinary input. No packet injection, movie/script replacement or memory modification is part of this candidate. Camera globes and barriers remain outside implementation scope until their missing original bindings are recovered or separately disclosed reconstructions are approved.

Independent read-only review passed: `../../../lab/reports/original-opening-microtutorials-independent-review.json`, SHA-256 `9b8d7b3d22ac17f67e1f801756cd0ee22ca375c5c03b11a1968632145b980142`. The reviewer rehashed14 sources, matched45 function extracts/four Lua ranges/274 Capstone instructions and original method-table entries, and separately preserved the original00c1f0a0 wrapper plus00402ce0 NUL-scanning string constructor omitted from the v2 extract set. No semantic blocker was found. Native acceptance remains pending. The whole historical starting-area acceptance remains open.
