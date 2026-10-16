# Original tutorial mode exit

Read-only checkpoint, October 9, 2026 (UTC). Scope: work item "original tutorial mode exit". No
production source, save, UI, content, process or capture was modified; the game was not run. This
continues [[Research/Original Opening Microtutorial Recovery]] and does not replace its accepted
packet findings, which are reused verbatim and not re-investigated.

Owned artifacts:

| Path | Role |
| --- | --- |
| this note | recovered paths, rejections, bounded candidate, native acceptance plan |
| `lab/reports/original-tutorial-mode-exit-20261009T1342Z/proof-v2.json` | private reproducible proof (the 15-function first revision `proof.json` stays beside it, superseded) |
| `lab/reports/original-tutorial-mode-exit-20261009T1342Z/trace-original-tutorial-mode-exit.py` | reproducer (same technique as the accepted microtutorial tracer) |
| `lab/reports/original-tutorial-mode-exit-20261009T1342Z/trace-run.log` | run output |

Pinned identities: original PE sha256 `26cc1c522a45e71fb57882da515f65bac45c8c8b8c7e740c0cea81c52a733351`;
original Ghidra export `f84a58674b8e7de0829d562e5f106a368f34d4750894a14b093ae745fa0befe3`; unchanged
unluac `ScriptsBase.lua` `c7c71c7cbb177c975a7cdb12ed0c2e1b74d662f5ca5ef6a4e9ff86d2d332a8f6`;
shipped `Resources/FirstTimeEvents.txt`
`164af95e76b0a33fd68e45d6298b25124cc76febf70eead72a571eef08c6b531`. Proof
`proof-v2.json` sha256 `5a800c23c7cf6431fe4e7a691e835fb38c9e433794969934bc0b3649bc0cd3c5`
(20 native function extracts, 5 machine blocks, 5 Lua ranges, the full direct-call-site scan).

## The mode flag and its manager

ORIGINAL: `Ui.IsInNPXTutorial` (binding `00c16fb0`, name string confirmed by the pointer scan) reads
the byte at **player+0x780**. `Ui.TutorialCancelled` (binding `00c16ff0`) is the Lua host function
that writes it:

```
00c16ff0 call 0x769710            ; client singleton
00c16ff5 cmp dword ptr [eax + 4], 0
00c16ff9 je 0xc1700a
00c16ffb call 0x769710
00c17000 mov ecx, dword ptr [eax + 4]   ; player
00c17003 push 0                          ; value
00c17005 call 0x8a9200                   ; player+0x780 = 0
00c1700a xor eax, eax
00c1700c ret
```

`008a9200` is a two-line setter (`*(byte *)(player + 0x780) = value`), so **`TutorialCancelled`
clears the flag and does nothing else**. It does not touch the tutorial manager.

The manager lives at **player+0x77c**. It is created only by the zone-reader true path:
`008a89a0` sets `player+0x780 = 1` and, when `player+0x77c == 0`, constructs
`ClientTutorialManager` (`00a46cd0`) and stores it. Two zone readers call it
(`009472e9`, `00947cbe`), and the accepted microtutorial evidence already records that the
false value through this setter does **not** clear a previously true flag. That finding is reused,
not re-tested.

The proof's direct-call scan over the whole mapped image (every `E8 rel32` whose target is the flag
setter) gives three direct callers of that setter (not a complete list of all writes):

| Writer | Packet / binding | Manager action |
| --- | --- | --- |
| `00aa38b8` | 41/2 `EncounterPacketPlayerEnter` | immediately disposes it (`00aa38ce`) |
| `00aa3c7f` | 41/114 `EncounterDetailsResponsePacket` | none |
| `00c17005` | `Ui.TutorialCancelled` Lua binding | none |

and the direct-call dispose scan: **one** call site, `00aa38ce`, inside 41/2, through
`008a8a30` → `00a46c40`.

Consequence for the mode exit: `Ui.TutorialCancelled()` alone is a *half* exit. It leaves the
manager object at player+0x77c alive until the player enters an encounter. The only original
operation that both rewrites the flag and tears the manager down is **entering an encounter**
(41/2).

## ClientTutorialManager side effects

All of the following is ORIGINAL native code (extracted text and hashes in `proof-v2.json`):

| Function | Recovered behaviour |
| --- | --- |
| `00a46cd0` ctor | sets the `ClientTutorialManager::vftable`, stores the player at `+4`, clears `+8` (started) and `+9` (preloaded), constructs its base object in place (the base constructor at `00ae8360` installs `DelayedAssetLoader::vftable` and stores the argument at `+0xd`), then calls `00ae3740` to register itself |
| `00a46c40` cleanup | sets `+8 = 1`, then `FUN_00769710()` (client singleton) → `[singleton+0x18]` → `00ae3750`, which stores the manager in the global slot `+0x280c8` and calls that object's vtable slot 4 — an unregister/notify step |
| `00a46c60` preload | when `+8 != 0`, `+9 == 0` and `player+0x959 == 0`, marks `+9 = 1` and calls `00ae79b0("GamePreLoadAssets.txt", 0x10)`; when both flags are set it calls `00ae6b90` |
**Correction (2026-10-12, S0297, verified from the vtables):** `00a46e40`, `00a46eb0` and `00a46ee0` are not
ClientTutorialManager methods. They are overhead UI element virtuals: `00a46e40` is
`OverHeadTextElement::vftable[16]` (a `Vector4` store), `00a46eb0` is `OverHeadChatBubbleElement::vftable[18]`, and
`00a46ee0` is `OverHeadHpBarElement::vftable[18]` (scaled accessors). `00a46e90` has no vtable slot. Its only
caller is `ProxiedCharacter::Process` (`00973200`). The "timeline" row that used to stand here was wrong.

Two honest limits. First, no recovered string or community model names the manager's fields, so they are
recorded as **structure**. Second, the manager's own methods (ctor `00a46cd0`, preload `00a46c60`, cleanup
`00a46c40`, dtor body `00a46bf0`) cover only construction, registration/unregistration and asset preload. They do
no hotkey, camera or job-switch gating. That gating is Lua UI state, documented next.

## Where the entry/exit gating actually lives (Lua)

ORIGINAL, `ScriptsBase.lua`. The first-time-event methods are keys of the global
`FirstTimeEvent` table (`40564`), which is what the 107/5 channel addresses:

| Line | Callback | Gating it performs |
| --- | --- | --- |
| 40799 / 40809 / 40819 | `FtesNpxBookSticker` / `FtesNpxBookTab` / `FtesNpxBookExit` | `TutorialArrow` flows, `StoryBook:Show()`, `HotKeyEvents:SetEnabledAll(false)`; `BookExit` additionally sets `StoryBook.shouldAllowHide = true` and `NotificationHandler:SkipItemOff()` |
| 40830 | `FtesNpxDppQuests` | `HotKeyEvents:SetEnabledAll(false)`, `GameDock:SetState(NPX_STATE)`, `GroupHandler:SetJobSwitchOnOrOff(0)`, `SetIsInMiniGame(1)`, `QuestHelper:ShowHelper()` |
| 40847 | `FtesNpxDppReturnHud` | same gating again for a returning HUD |
| 40861 | `FtesNpxDppEnd` | **exit restore**: `HotKeyEvents:SetEnabledAll(true)`, `GroupHandler:SetJobSwitchOnOrOff(1)`, `GroupHandler:SetIsInMiniGame(0)` |
| 40867 | `FtesNpxDppHide` | `DirectedPlayPrompt:Hide()` |

Movement and keyboard are gated by the UI context stack, not by the tutorial manager:
`Context:Push` stores the current movement value and calls `Ui.SetMovementEnabled(0)` (`1686`);
`Context:Pop` restores `Ui.SetKeyboardEnabled(...)` and `Ui.SetMovementEnabled(...)` from that
stored value (`1700`–`1701`). `StoryBook:Hide` (accepted extract, `14688`–`14727`) restores
movement and keyboard explicitly and, **only when `Ui.IsInNPXTutorial()` is true**, kills the four
book tutorial timers and emits `Ui.OnObjectiveEvent("onStoryBookClose")`. The outbound 45/7
completion is the accepted microtutorial finding and was not re-investigated.

## The exact flow that calls `Ui.TutorialCancelled`

`Ui.TutorialCancelled()` has exactly one call site in the shipped UI:

* `ScriptsBase.lua:47272`, inside `MinigameStartScreen:Hide(self, mode, go)` in the `mode == 0`
  branch, next to `Ui.OnObjectiveEvent("onMinigameClose")` and `Ui.MinigameScreenClosed()`.

That branch is reached only when `Hide` is called **with an explicit 0**:

* `ScriptsBase.lua:39153`, `Lobby:Show(...)`, calls `MinigameStartScreen.Hide(0)` when the activity
  portal is not open — i.e. the player opens the multiplayer lobby.
* `ScriptsBase.lua:47225` pushes `MinigameStartScreen.Hide` as a `CONTEXT_HIDE` callback, and
  `Context:CallQuit()` (`1717`) invokes the stored function **with no arguments**; `mode` is then
  `nil` and the code takes the final `else` branch, which never calls `TutorialCancelled`.

So the exact original flow is: player opens the Lobby while the minigame start screen is up →
`Lobby:Show` → `MinigameStartScreen.Hide(0)` → `Ui.TutorialCancelled()` → `player+0x780 = 0`.
It is a **minigame-lobby side effect**, available in normal play but outside tutorial mode and not
scoped to the starting tutorial. There is no recovered tutorial-scoped caller.

## The 107/5 script channel

ORIGINAL chain, all pinned in `proof-v2.json`:

`107/5 FirstTimeEventScriptPacket` body → `00c897b0` (parse; when its header word equals 2 it also
constructs a `FirstTimeEventStatePacket` with id `-1` through `00c88240`) → per script entry
`00c893f0`, which reads the object name up to `.`
(`FUN_006fc080(..., 0x2e, 1)`), rebuilds `object:method` with the `:` separator
(`FUN_006fc030(0x3a)`), fills a `SoeUtil::Array<ClientLuaParam,20>` (`00a8d810`) with the packet's
integer parameters, and calls the generic Lua bridge `009797a0` with the object as `self`.

`Resources/FirstTimeEvents.txt` (92 rows, id→name) is a shipped first-time-event name table, and the
NPX callbacks above live on the `FirstTimeEvent` table, so `FirstTimeEvent.FtesNpxDppEnd` and
`FirstTimeEvent.FtesNpxDppHide` are the original, intended targets of this channel. The pinned
Ghidra export never references that filename, and the dispatch path performs **no whitelist
lookup**: any name that exists as a Lua global is callable, so the channel can also reach e.g.
`MinigameStartScreen.Hide` with an integer 0. That is a generic Lua-call channel, not an FTE
whitelist, and it is rejected below for the production candidate.

Historic 107/5 event ids, clear-flag values and ordering for the NPX 4.0 sequence remain
**unrecovered**; a candidate that uses this channel must disclose a reconstructed ordering and
whitelist the exact original callbacks it names.

The same body parser separates the family's two inner messages, which matters for any exit flow:

* inner **2** builds `FirstTimeEventStatePacket` (`00c88240(-1, 0)`) and applies it; applying it
  (`00c89270`) reaches `00c896f0`, which — when the state flag transitions to 0 — walks the queued
  script list and calls every entry's Lua callback through the same bridge
  (`FUN_009797a0(puVar2[3], puVar2 + 6, 0)`). That is the original "run the queued first-time-event
  callbacks" mechanism.
* inner **5** parses one script entry (`00c88ee0`) and either queues it (`00c88a00`) or dispatches
  it immediately (`00c893f0`).

So the original server pattern is: queue the intended FTE callbacks, then close the state — not
invent a callback name. A production candidate that wants the authentic `FtesNpxDppEnd` UI restore
should follow that pattern with names that exist in the shipped `FirstTimeEvent` table.

## Bounded production candidate and rejections

**Candidate.** Exit tutorial mode only through the original encounter transition. The starting
tutorial already ends by entering an activity/encounter; the server's existing 41/2
`EncounterPacketPlayerEnter` is the only original operation that rewrites player+0x780 *and* tears
down the manager, and the same packet is already required for the minigame HUD. Keep 43/31
`Tutorial = false` exactly as it is: it does not clear, which is now a recovered fact rather than a
workaround. Do not add a Lua call, a client patch or a memory write.

Rejections, each grounded in the evidence above:

1. **41/2 with no launched encounter** — hidden state: the packet would end tutorial mode and
   dispose the manager without the encounter the client is being told about.
2. **41/114 outside an encounter-details request** — it rewrites the same flag plus the encounter
   profile bytes (`+0x704`, `+0x958`, `+0x781`–`+0x783`, `+0x78c`) and is not an exit.
3. **Duplicate or replayed 41/2** — the same case also calls `009b9500` and drives the
   minigame HUD; replays churn client state. Accept once per encounter instance.
4. **107/5 `MinigameStartScreen.Hide` with parameter 0** — technically reaches
   `Ui.TutorialCancelled()`, but it is outside-mode (the start screen may be closed), it is not an
   FTE name, and it turns the FTE channel into an arbitrary Lua caller. Rejected.
5. **Memory writes, injected Lua, client edits, save edits, packet injection** — out of scope by
   instruction and unnecessary for this candidate.

## Native acceptance plan (lead-only, after integration)

1. Fresh character; enter the reconstructed starting tutorial through the original 43/31
   `Tutorial = true` path. Observe: `Ui.IsInNPXTutorial()` true, NPX arrow/book prompts appear,
   hotkeys off, job switching off, movement gated while a tutorial window is up.
2. Walk the reconstructed 17-stage opening; the journal close must still emit its real 45/7 once
   (accepted evidence, unchanged).
3. Enter the real activity/encounter the tutorial ends with, using only original client input.
4. After that transition, verify read-only that player+0x780 and player+0x77c are both clear, that
   `StoryBook:Hide` is no longer gated by `Ui.IsInNPXTutorial()`, and that hotkeys, job switching,
   movement and camera are restored (`FtesNpxDppEnd` effects).
5. Counter-checks: no completion from a hidden/disallowed `StoryBook:Hide`, no completion while
   outside the zone, no second HUD/manager from a replayed 41/2, no double reward.
6. Restart the server and client and confirm the character, tutorial state and rewards persist.

Unresolved and required for that gate: which parsed 41/2 field feeds player+0x780 (the community
model names only `EncounterId`, `InstanceId`, `PlayerGuid`, `Unknown`; the client reads a dword
0x18 bytes into its parse buffer), whether that value reaching 0 is exactly what happens in
practice, and the historic 107/5 ids/order. Those are client observations for the acceptance run,
not assumptions to code against.

## Verification

Run from the report directory with the pinned lab Python:

```
& '..\..\..\lab\tools\wiki-env\Scripts\python.exe' 'trace-original-tutorial-mode-exit.py'
{"flag_writers": ["00aa38b8", "00aa3c7f", "00c17005"], "functions": 15, "machine_blocks": 5,
 "proof_sha256": "5a800c23c7cf6431fe4e7a691e835fb38c9e433794969934bc0b3649bc0cd3c5"}
```

The reproducer asserts every pinned source hash, the three-writer and one-dispose call-site sets,
the `TutorialCancelled` and `IsInNPXTutorial` method-table names, the Lua range contents, and each
machine block's first instruction address, then writes `proof-v2.json` with `write_exact` semantics
(an existing different file fails). Log: `trace-run.log`.

Cross-links: [[Research/Original Opening Microtutorial Recovery]], [[Research/Housing Restoration]],
[[../Design/MMO Worker Contract]].

## Independent field-mapping review correction

[[Reviews/Original Tutorial Mode Exit Field Review]] (SHA256 `59fd826cf95ddb853f1edaacde1264ef2d8cfe442085749423807416333cae7a`) independently maps41/2 body byte20 (community `EncounterPacketPlayerEnter.Unknown`) to player+0x780. Current real cave enter writes0 and already clears that flag while disposing the manager.41/114 is a different field, object+0x4e (`Tutorial`), with other state writes; it is not interchangeable. The three E8rel32 callers above are direct setters only: review also finds inline runtime clear00936d71 and constructor zeroing0094239c; computed/indirect writes are not excluded. Dispose does not null player+0x77c; owner teardown does. Use genuine cave entry rather than inventing an exit packet or extra Lua/memory operation. Original tutorial pilot still needs normal native storybook/objective/entry validation, and no historical quest binding is inferred from this field proof.
