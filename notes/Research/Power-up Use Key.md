---
updated: 2026-10-21
---

# Power-up use key

Research for T222 (builder, 2026-10-21). Question: how does the 2014 client trigger a held combat power-up, what does it send, and can the server choose the key? Evidence tags: **[client]** native/IDA or shipped client asset, **[wiki]**, **[video]**, **(R)** reconstructed. Client addresses are FreeRealms.exe virtual addresses; IDA queries used the read-only `lab/autopilot/ida_query.py` tool.

## Summary

- **There is no dedicated combat power-up input.** A held power-up is a **job action-bar slot**, defined by the server with 36/13 + 36/5 exactly like a job ability. The client uses it when the player presses that slot's key (`ActionBarSlot3`, default **3**) or clicks the game-dock button; both send the ordinary `36/10` start-ability request for **bar 1, slot 2**. [client]
- `UsePowerup` (default **Control_L**) is **not** the combat power-up key. It is the demolition-derby/minigame action: its options row is commented out in `Resources/ControllerElements.txt`, the regular-controller input dispatcher `00b02880` has no case for action 15, and the only `Powerup` action name in the executable is the soccer action row. [client]
- The binding is **client data**: the default table is compiled into the executable (no server-shipped file), and player rebinds live in client settings. The server can neither ship `E` nor see which physical key was pressed; it only ever receives the slot index. `E` is already `StrafeRight` (row 6) and `Juke` (row 11), so it is not free. [client] The closest free default key to WASD that can run this action is the held slot's own key, default **3**; a player who wants `E` can rebind `ActionBarSlot3` in the client's Key Binds window (the row is present in `ControllerElements.txt`). We do not patch the client.
- The server already receives the use request, so T222 wires it server-side: the held kind is shown in slot 2 only while held (the locale's "add power-up to bar"), the 36/10 bar 1 slot 2 request consumes it and applies the `power_ups.json` effect, and the authored job row returns afterwards. No job row moves. **(R)** held state is session-only, so a reconnect clears it.

## 1. Which client input uses a held power-up?

### 1.1 The transport: `ActionBarSlot3` -> job-bar slot 2 -> 36/10

| Fact | Evidence | Strength |
|---|---|---|
| Default binding table at `0x01b4198c`: 81 rows x 24 bytes (`+0` name pointer, `+4` X11 keysym, `+8/+0xc/+0x10` flags, `+0x14` action index row+1). `ActionNameIndexLookup` `0094c760` and `DefaultKeyByName` `0094d0c0` read it. | IDA `decompile 0094c760`; bytes at `0x1b41b84` = pointer `0x01809648` ("ActionBarSlot3"), keysym `0x33` ('3'), index 22; `ida_query strings "ActionBarSlot[0-9]"` | [client] |
| The regular overworld controller's input dispatcher is `00b02880` (vtable `0181cc4c` slot 50). It walks queued key events and switches on the 0-based action row: `case 19..=22` calls `DispatchActionBarSlot 008a7eb0(bar, row-19, 0)`; case 21 is `ActionBarSlot3` -> slot 2, mode 0 (job bar). It also has no case 15 (`UsePowerup`), no case 12/13 (`AttackLight`/`AttackHeavy`). | IDA `decompile 00b02880`; native-map entry `show 00b02880` | [client] |
| `008a7eb0` picks a bar mode and `008a7e40` runs it; mode 1 is the job toolbar, mode 2 the item bar (the Shift/alternate action bar), mode 3 a minigame bar, mode 4 a Flash minigame. | native-map `show 008a7eb0`; `Build Guides/Gameplay/Combat and Abilities.md` §1.1 | [client] |
| The job-bar path is `009cf430` (UI gate/refusal) -> `00a36a70` (select slot) -> `00a35600` (status gate/target mode) -> `00a35150` (send), whose request object is serialized by `00a342e0` and tunneled by `00a35000`. | IDA `decompile 00a35150`; guide §1.1 rows 2-5 | [client] |
| The request is family 36 sub 10 `ClientRequestStartAbility`: `i32 bar`, `i32 slot`, `i32 target mode`, then `u64` guid (mode 0, 24 bytes) or four `f32` (other modes, 32 bytes). A self cast carries the player's own guid. | `00a342e0`/`00a35150` pseudocode; `restoration/piece_035.rs` and `piece_046.rs` (recovered constructor constants and community readers agree); our server already parses exactly these two shapes | [client] |
| The game-dock button takes the same path: Lua `GameDock.OnUserEvent "activateAbility"` calls `Ui.ClickedAbility(index)` (job bar), and the fishing/minigame docks pass bar 2 or 3. | `lab/decompiled/client-ui/ScriptsBase.lua:30481-30489` (`GameDock = { OnUserEvent = ... }`) | [client] |

So pressing the slot's key and clicking its dock button both produce **36/10 bar 1, slot <index>**; a held power-up is whatever the server defines in that slot.

### 1.2 Which slot holds it

| Fact | Evidence | Strength |
|---|---|---|
| The 2014 tutorial says the held power-up is used with key **3**: "Pick up the power up and press [3] on your keyboard to use it." | client locale hash 1441159516 (`lab/reports/combat-powerups-w153/locale-powerup-lines.tsv`) | [client] |
| The same era also says "Click the game dock button that shows the Flame Wave power-up to use the power-up attack!" | locale 1188590723, same file | [client] |
| The server adds the kind to the bar: locale dev note "power ups - add super shield to bar" (row 421005, hash 1011626935). | W153 locale report; `Combat Power-ups.md` 1.1 | [client] |
| Consumed on use, no energy: "Power-Ups go away when you use them, but they always help you and they don't even use energy!" | locale 3939331069 | [client] |
| One held at a time: "You can have one power up active at a time." | locale 4186384284 / 3152868716 / 3222513765 | [client] |
| The PS3 build uses a dedicated controller button, not a keyboard key: "Pick up the power up and press the Circle button on your controller to use it." | locale 3103689944 | [client] |
| The Hewey video's combat dock shows two numbered ability orbs and a third orb that lights up while a power-up is held and goes dark after use. | W153 `lab/reports/combat-powerups-w153/hewey-bar-crops.png` from `fktAs_GRrjM` frames 00438/00452/00456/00460 | [video], read by W153 |

Bar 1 slot 2 is the third job-bar slot, which is `ActionBarSlot3` (key 3). The server's 36/5 toolbar has eight slots; slot 0 is the basic attack, slots 1..7 are job-bar entries. The client cannot edit the job bar through 38/25 (that handler only edits item and minigame bars), so a held kind can be added only by rebuilding the toolbar with 36/13 definitions and 36/5.

### 1.3 `UsePowerup` is the minigame action, not the combat one

| Fact | Evidence | Strength |
|---|---|---|
| Default table row 15 is `UsePowerup` = Control_L (keysym `0xffe3`), index `0x10`. Rows 16 `Kick` and 17 `IgnoreCollision` share Control_L; every row with the `+8 = 2` flag is a held/controller-style action. | bytes at `0x1b41af4`; `notes/Native Map/Default Key Bindings.md` rows 15-17 | [client] |
| The shipped options file lists `UsePowerup` only inside the **commented-out** demolition-derby block (`#dd1^demolition^action_powerup...`), not in movement, action bar, general or dock. The Key Binds window is populated from this options data source. | `Resources/ControllerElements.txt` lines 16-32 (action bar) and 77-79 (commented derby block); Lua `KeyBinds` uses the same option rows (`ScriptsBase.lua:32781+`) | [client], options file |
| `00b02880` (the regular controller dispatcher) has no `UsePowerup` case; the cases are 9 (jump), 19-26 (action-bar slots 1-8), 28-39 (docks and freeform), 45 (interact), 80 (flourish), plus movement bits. | IDA `decompile 00b02880`; native-map accepted label for `00b02880` | [client] |
| The only action-name table in the executable that returns "Powerup" is the soccer one: `SoccerPlayerActionsDataSource` +0x30 slot [1] = `00cd7010`, a switch over nine soccer actions returning Ball/Dodge/Kick/SuperKick/SlideTackle/Steal/Pass/Call/`Powerup`. | IDA `disasm 00cd7010`; native-map `show 00cd7010` | [client] |
| No combat power-up command class exists: `nmq command "power"` has no match; the packet catalogue has no power-up family. | native map `command` query | [client] |
| Lua has no power-up input either: the only power-up code is the first-time-event icon page and arrow (`FtesPowerUp`, `PowerUpIcons`, `FirstTimeEvent.PowerUpFTE` arrow at `gdX + 325` on the game dock). | `lab/decompiled/client-ui/ScriptsBase.lua:40968-41105`, `41390` | [client] |

**Conclusion.** On PC the held combat power-up is used through its action-bar slot (default key **3**); `UsePowerup`/Control_L belongs to the demolition-derby/minigame controllers and never reaches the combat bar. This is the "its own key" the owner remembers: in the original the power-up had the third slot to itself, while our reconstructed job bar puts an ability row there.

## 2. What the client sends and expects back

**Sent (client -> server).** `36/10 ClientRequestStartAbility`: `u16 36`, `u16 10`, `i32 bar = 1`, `i32 slot = 2`, `i32 target mode = 0`, `u64 target` (the player's own guid for a self cast) = 24 bytes. Ground modes are 32 bytes; other bars and ground modes are refused by the server today (`abilities::unsupported_cast`). The request carries **no power-up id, kind, quantity or effect**; it names a slot only. (Client send path `00a35150`/`00a35000`, writer `00a342e0`; our parser in `abilities.rs` and `restoration/piece_046.rs` pins the 24/32 shapes.)

**Expected back (server -> client).**
- The effect: area damage results per enemy (`32/7 AttackProcessed` plus `35/5 ActorHealth`), floating heal/damage numbers (`35/35 HitPointModification`), health/energy bars (`38/1`, `38/13`), an effect tag for a timed buff (`38/16`), and animations (`35/8`, stun clip 1401). This is the same vocabulary the job abilities already use.
- HUD clear: the client keeps a held kind only as a bar slot, so the server hides it by sending the 36/13 definitions then the 36/5 toolbar with slot 2 back to its job entry (or empty). `38/25 UpdateActionBarSlot` cannot do it: its handler `009392c0` case 0x19 only updates the item and minigame bars. The client has no separate "power-up HUD state" to clear.

## 3. Is the binding data the client reads at startup?

- The **default** table is compiled into `FreeRealms.exe` at `0x01b4198c` (81 rows x 24 bytes). There is no server-shipped keymap file: `ClientConfig.ini`/`UserOptions.ini` carry no bindings, and `Resources/ControllerElements.txt` is a client asset beside the executable. [client]
- **Player rebinds** go through the client-only Key Binds window (`Ui.ChangeKeyBinding` / `Ui.SwitchKeyBinding` / `Ui.ResetKeyBinding`, `ScriptsBase.lua:32871-32921`) and are stored in client settings (`SaveSettings`/user options). The server never sees the physical key. [client]
- `E` is not free: row 6 `StrafeRight` = 'E' (0x45) and row 11 `Juke` = 'E'. [client] The nearest usable key to WASD is therefore the slot's own default, **3**; the player can move it to `E` by rebinding `ActionBarSlot3` client-side (row `a3` in `ControllerElements.txt`), which the server will accept unchanged because it only sees slot 2.

**Decision for T222.** The client already sends the request, so the task's first build branch applies: handle it server-side. We cannot and do not ship `E`; we show the held kind in its own slot (default key 3, the evidenced tutorial key) and wire the server use. The client is not patched. `E` is reported as taken (StrafeRight/Juke) and the fallback is the evidenced fixed slot key.

## 4. Evidence vs. the earlier W153 note

W153's `Combat Power-ups.md` already recorded the key-3 transport and the 36/10 path, but treated the slot as a job-bar ability slot because our reconstructed `job_abilities.json` puts abilities there. IDA wins on the transport: the client has no other combat input for a held kind, and the options file confirms `UsePowerup` is not a combat binding. Where the notes and the client disagree on which key, the client wins: **key 3 (slot 2) by default, not Control_L**.

## Evidence index

- `lab/autopilot/ida_query.py decompile 00b02880` (regular-controller input dispatch; case 21 -> slot 2), `decompile 00a35150` (ability send), `decompile 0094c760` (key-table lookup), `disasm 00cd7010` (soccer "Powerup"), `strings "ActionBarSlot[0-9]"` / `strings "UsePowerup"`.
- Native map: `show 00b02880`, `show 008a7eb0`, `show 00cd7010`, `bytes 0x1b4198c`, `bytes 0x1b41af4`, `bytes 0x1b41b84`.
- `notes/Native Map/Default Key Bindings.md`; `notes/Build Guides/Gameplay/Combat and Abilities.md` §1.1 and "Ability request (36/10)".
- Client assets: `Resources/ControllerElements.txt`; `lab/decompiled/client-ui/ScriptsBase.lua`.
- Locale: `lab/reports/combat-powerups-w153/locale-powerup-lines.tsv` (hashes 1441159516, 1188590723, 3939331069, 4186384284, 3103689944, 1011626935).
- Existing Rust evidence for the wire: `src/restoration/piece_035.rs`, `src/restoration/piece_046.rs`, `src/abilities.rs` (`unsupported_cast`), `src/opening_abilities.rs` (36/5/36/13 builders).
