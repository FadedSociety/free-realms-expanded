---
type: review
created: 2026-10-16
status: unrun
---

# Native Checklist 2026-10-16

One play session in the real 2014 client that checks everything the Claude workers landed on 2026-10-16 (W138 to W151, W155, W158, W160 to W164). Nothing below has been seen on a native client yet. Every behaviour is proven only by unit tests, gateway-level tests and, for six features, bots (`tests/bot_acceptance_1016.rs`, W154). Treat each "what should happen" line as a hypothesis from static analysis, and every field order marked (R) as a guess to confirm.

Sections follow the order a session naturally flows: login, town, shops, crafting, fights, travel, then relog. Each has Setup, Steps, Expect, Capture and the Open question the run answers. Fill the **Result** line as you go.

W170 update (HEAD `5d522cf`): sections 11 and 15 to 18 were placeholders when W165 wrote this note and now describe the real behaviour from the W158, W160, W162, W163 and W164 reports; section 5 carries the W161 and W164 detail. Sections 15 (fishing), 16 (mounts) and 18 (path trail) are wired in the working tree but only **staged, not committed**; section 17's real trade is **not wired at all** and its commit fails closed. Re-check the gates in 0.1 before you rely on this.

## 0. Before you start

### 0.1 What is live (HEAD `5d522cf`, checked 2026-10-16)

| Feature | Worker | Commit | Live in the gateway? |
|---|---|---|---|
| Equip refusals | W138 | `44c7fd0` | Yes |
| Emote restrictions, 36/12 ability lookup, explicit cast refusals | W139, W141 | `f1de554` | Yes |
| Warpstones, Atlas table, travel | W142 | `378a680` | Yes |
| Quest select (47/12), lock (47/13), breadcrumb start (49/15) | W145 | `f4e397b` | Yes |
| Gathering reconnect release | W140 | `f4e397b` | Yes |
| Recipe book at a station (26/24), pose-only save coalescing | W144 | `4ba97ad` | Yes |
| Vendor sell, 66/42 and 165/6 layout fixes | W148 | `e9088d3` | Yes (no gameplay change was needed) |
| Player click menu and Inspect | W143 | `4a7ab21` | Yes |
| Free revive (41/122) with 15 s server fallback | W147 | `f24c3ba` | Yes |
| Item bar and Small Health Potion; friend positions; group job and map ping | W146, W151 | `86b1b0e` | Yes |
| Kart race from an NPC menu; activity portal list and Play | W149, W150 | `5f06115` | Yes (kart needs `kart_gates.json`, see 0.3) |
| **Take Me There pin (47/14)** | W161 | `dc27270` | Yes |
| **Power-up drops, killer's own pickup** | W155, W158 | `dc27270` (data, engine, `gameplay.rs` load, `realm reload power_ups`) | Yes |
| **Trade unavailable answer (59/x)** | W156 | `dc27270` | Yes. This is the only trade behaviour that is live. |
| Power-up per-participant drops and big-heart group heal (slice 2b) | W158 | not committed | **No.** `encounter.rs`, `power_ups.rs` and the `participation` block in `power_ups.json` are edited in the tree, but `apply_slice2b.py` (gameplay, gateway, world, `osfr-gateway.rs`) has not been applied. Today only the killer gets a pickup and a big heart heals only its owner. |
| **Fishing entry from six guide NPCs** | W160 | staged, **not committed** | Wired in the tree (`handle_fishing`, `begin_fishing`, `fishing_spots.json`, bundle line). Needs the package to carry `fishing_spots.json` (0.3). |
| **Mounts (168)** | W162 | staged, **not committed** | Wired in the tree (`is_mounted`, `settle_mounts`, `mounts.json`). |
| **98/1 path answer (breadcrumb trail)** | W164 | staged, **not committed** | Wired in the tree (`pathing.rs`, `gameplay.rs`, label in `gateway.rs`). |
| **Real player trade** | W163 | not committed | **No.** `trade.rs` and `player_radial.rs` are edited and `tests/player_trade.rs` exists, but `lab/runs/claude/W163/patch/shared-files.diff` is not applied (no `route_trade`, no Trade radial entry in `osfr-gateway.rs`), and its `commit_trade` is a fail-closed stub even after it is applied. |

Run the gate for each "No" row just before the session and skip a section if its gate fails:

```
git -C project log --oneline -30
rg -n "load_power_ups" project/prototype/osfr-tools/src/gameplay.rs      # drops (live at dc27270)
rg -n "tracked_pin" project/prototype/osfr-tools/src/gameplay.rs          # pin (live at dc27270)
rg -n "group_heal|claim_group_heal" project/prototype/osfr-tools/src/gameplay.rs   # slice 2b (empty = not wired)
rg -n "handle_fishing" project/prototype/osfr-tools/src/gameplay.rs       # fishing
rg -n "settle_mounts" project/prototype/osfr-tools/src/gameplay.rs        # mounts
rg -n "pathing" project/prototype/osfr-tools/src/gameplay.rs              # 98/1 answer
rg -n "route_trade|commit_trade" project/prototype/osfr-tools/src/bin/osfr-gateway.rs   # real trade (empty = not wired)
git -C project status --short                                             # staged (A/M in column 1) means not committed
```

A section whose gate fails stays in this note for the day it lands. Do not count it as a native failure.

### 0.2 Two clients

| Section | Second client needed? |
|---|---|
| 1 Login and world entry | No |
| 2 Emotes | **Yes** to see the receiving animation (a bot receiving only proves the server) |
| 3 Inspect | One client plus a bot is enough; a real second client is better (it also checks AddPc) |
| 4 Friend map and group pings | **Yes**, two real clients, mutual friends |
| 5 Quest select, lock, pin | No |
| 6 Vendor | No |
| 7 Equip refusals | No |
| 8 Recipe book | No |
| 9 Item bar | No |
| 10 Free revive | No |
| 11 Power-up drops | No for the killer's own pickup. **Yes** for the per-participant drop and the big-heart group heal (slice 2b), once wired |
| 12 Activity portal | No |
| 13 Warpstones and Atlas | No (58 friend teleport is refused, so a friend is not needed) |
| 14 Kart from an NPC | No |
| 15 Fishing | No |
| 16 Mounts | **Yes** to see the rider and the ride from outside (one client checks everything else) |
| 17 Trade | The shim runs on one client; **real trade needs two clients** (and, today, only reaches the fail-closed commit) |
| 18 Path trail | No |
| 19 Relog | No |

`run-local-baseline.py` starts one client. For a second real client use a second PC over the LAN or VPN with its own account and enrolled key (see [[Playtest Host Runbook]]); this is the main blocker for sections 2, 4, 11b, 16 and 17. Until it exists, run those sections with a bot as the second player and mark the receiving-side checks as not done. A bot (`osfr-bot`) stands in as a visible second player for sections 2 (server side) and 3.

### 0.3 Build the play package

The old `round70` package holds only the campaign overlay. It has no `stores.json`, `npcs/`, `quests/`, `field_battles.json`, `recipes.json` or `power_ups.json`, so most of the features below do not exist in it. W154 and `scripts/stress/run.py` solve this by copying `data/content` first and then laying the round70 overlay over it. Do the same, on a scratch copy so Capo's checkpoint is never changed:

1. Commit or at least review any content you want first. `power_ups.json`, `consumables.json` and `kart_gates.json` are optional files.
2. Create `lab\reports\native-1016-<stamp>\` with these steps (PowerShell, from `CartoonMMO\`):
   - copy `project\prototype\osfr-tools\data\content\*` into `...\content\`;
   - copy `lab\reports\round70-native-20261016T061000Z\content\*` over it (overwrite);
   - copy `character.json`, `identity.json`, `accounts.json` and `characters\` from the round70 folder to the new folder root.
3. Add `content\kart_gates.json` (needed by section 14):
   ```json
   {"schema_version": 1, "npcs": [9000000039], "reward_cooldown_s": 60}
   ```
   9000000039 is Cale Yardbird (Briarwood, -276.7, 1964.8). Chase Wheeler is 9000000273. Both are (R) picks. A short cooldown lets you see the cooldown refusal without waiting.
   `fishing_spots.json` (section 15) and `mounts.json` (section 16) are in `data\content`, so the copy in step 2 brings them. Check that both are in the scratch `content\`: `fishing_spots.json` is a bundle-optional file and nothing else loads it, and without it no guide offers fishing.
4. Optional, to make the power-up check take minutes instead of hours (section 11): in the scratch `content\power_ups.json` set both `chance` values of the `standard` table to `1.0`. The shipped chances are 12% and 10% (R).
5. Wallet: `realm coins 5000` after login. Section 16 needs a ride item (`realm give 14289`, the Wolf) and section 17 needs a few `realm give` items on both characters. Capo has 309 coins; the cheapest checks below cost 75 to 750.

This merged package has not been run through `run-local-baseline.py` yet. Step 1.1 is the first proof it starts.

### 0.4 Launch and capture

Command (from `project\`, per [[Current Progress]], with the new package):

```
python -I scripts\run-local-baseline.py --rust-reference lab\references\rust-gateway-reference --state-file <pkg>\character.json --content <pkg>\content --native-login --identity-file <pkg>\identity.json --journal-text lab\references\opening-native-lobby-20261009T163603Z --dev-tools --live-capture --seconds 3600
```

Raise `--seconds` for a long session (default 90). Add `--gamepad` on the handheld. Add `--native-hook add_pc` (sets: `add_pc`, `quest_offer`, `dialog_camera`, `quest_end`) only for the inspect and second-player check; the hook needs Frida.

Check the battery before launch: the dev machine dies mid-test on a low charge.

What every run leaves in `lab\runs\client-baseline-<stamp>\`:

- `server-trace.jsonl` and `login-trace.jsonl`: one JSON object per line. Incoming application packets are `event:"packet"` with `family`, `subtype`, `outcome` (`handled` or `rejected`) and `reason`. Outgoing application packets are `event:"app_out"` with `family`, `subtype` and `body_len`. `rejected` with reason `unknown application message` means the gateway has no route for that family/subtype. That reason is expected and harmless for 41/109 and 41/122 (cosmetic label, W147) and is a real finding for anything else.
- `live.log` (the console that opens beside the client), `character-before.json` and `character-after.json`, screenshots (`client-*.png`), `server-stdout.log`.
- With `--live-capture`: a UDP capture you can hash-check later.

Filter the trace for one feature (set the pairs):

```
python -I -c "import json,sys; want={(165,5),(165,6)}; [print(l.strip()[:300]) for l in open(sys.argv[1]) if (lambda r:(r.get('family'),r.get('subtype')) in want)(json.loads(l))]" lab\runs\<run>\server-trace.jsonl
```

The save diff for a section is `character-before.json` against `character-after.json` (or the state file mid-run: writes of pose-only changes are coalesced to every 5 s, so wait 6 s before reading it).

Take a screenshot at each **Expect** point. Name them `<section>-<step>.png` in the run folder so the review note can cite them. In the review note record the run id, release SHA and pass/fail per row.

### 0.5 Commands used in this note

`realm` commands need `--dev-tools` on a local run, and a native-login run uses an account store, so the account role also counts. In the round70 `accounts.json` account 1 (Capo) is `admin` and account 2 (Buddy Botsworth) is `player`, so Capo can run everything. A new account (for a second client) starts as `player`: set it with `osfr-account set-role` (offline, login stopped) or it cannot run `realm`. On an account-store server without `--dev-tools` only builders and admins get `goto`, `here`, `place` and `reload`.

| Command | Use |
|---|---|
| `realm goto <x> <z>` | land on the terrain at that point (half a metre up) |
| `realm goto <NPC name>` or `realm goto list` | stand beside an NPC (name match is a prefix) |
| `realm here` | print zone and position (the position oracle) |
| `realm give <item id> [count]` | grant an item (needs the catalogue) |
| `realm coins <n>` | grant coins (max 1,000,000) |
| `realm level <1-20>` | set the active job's level |
| `realm heal` / `realm god` | restore health and energy / toggle god mode |
| `realm battle <id>` / `realm win` | start a field battle / dev-win it |
| `realm reload all` | reload battles, enemies, abilities and power-ups |
| `realm reload power_ups` | reload `power_ups.json` alone (a bad file keeps the old tables and answers an error); use it after changing `chance` in the scratch package instead of restarting |

Landmarks used below (zone 0, the one continent; the regional `zone_id`s in content are labels):

| Place | x, z | Why |
|---|---|---|
| Cobblestone Village warpstone 7 | -1904.5, 413.3 | spawn area; Darkthorne is at -1897, 412.6 |
| Samantha, Ricky Danger, Roosey, Jones | -1879, 443; -1910, 463; -1842, 411; -1896, 567 | quests, vendor, Brawler quest |
| Hooligan group (field enemies, 220 HP) | about -1970, 540 | fights, knockout, drops |
| Hooligan Wolves (100 HP) | about -1899, 307 | easier fights |
| Cooking Table / Forge / Anvil (Farnum Farmstead) | -2109, 210; -2076, 190; -2088, 200 | recipe book stations (Chef 45, Miner 14, Blacksmith 16) |
| Sanctuary warpstone 5 | -1414.6, 351.6 | travel test |
| Briarwood warpstone 4 | -808.4, 1631.8 | travel test |
| Cale Yardbird (kart gate) | -276.7, 1964.8 | kart |

## 1. Login and world entry

**Result:** ____

**Setup:** account any role; Capo (male, Medic) or a fresh character. Nothing in the world yet.

**Steps**

1. Start the package. Confirm `world_ready` in `server-stdout.log` before touching the client.
2. Log in through the native lobby and select the character. Wait for the world.
3. Stand still for 10 seconds.

**Expect**

- The world loads. The hotbar, ability bar and item bar draw. No error popup or crash.
- Chat shows the local-play welcome line; `realm help` lists the ordinary commands.
- If Capo spawns within 12 m of the Cobblestone warpstone (-1904.5, 413.3), the chat line "attuned to the Cobblestone Village Warpstone" or similar may appear (section 13 covers it fully).

**Capture:** `app_out` rows at world entry. The ones this note cares about: 38/25 (four item-bar slots, W146), 167/1/1 (activity list, W150) and 57 (warpstone table, W142). Count `rejected` rows in the first 30 s. Family 109 rows are known and old; any other new family/subtype is a finding.

**Open question:** does the merged package start and enter the world at all (never run before), and does any new world-entry packet (38/25, 167/1/1, 57) make the client log an error? Check `client-stderr.log` and `client\Logs`.

## 2. Emotes (needs a second client for the receiving side)

**Result:** ____

**Setup:** two characters in the same area, standing 5 to 10 m apart. Player A is the sender.

**Steps**

1. A opens quick chat and the emotes menu. Pick three ordinary emotes (for example a cheer, a wave) one by one. Wait 2 s between them.
2. A picks an ordinary spoken quick-chat phrase (no animation).
3. A picks Dance (phrase 184). It is the only row of 2,470 that the server refuses (animation 3200 has no recovered slot).
4. A walks continuously, then triggers the emote that freezes movement (animation 3001; find which menu entry it is and record it).
5. A sends an emote as a tell to B, then as a group or guild line if available.
6. (After section 16, once mounts are wired) A mounts and tries an emote.

**Expect**

- Steps 1 and 2: B sees A play the animation and the text line. A sees the same.
- Step 3: nothing on B's side. A sees nothing either (silent drop; the client's own button state is unchanged). No error.
- Step 4: while A is within 500 ms of its last accepted move, the freeze emote is dropped. Standing still for a second, it plays.
- Step 5: same rules per channel.

**Capture:** `family 67 subtype 3` rows, `direction in` (A's request) and `app_out` to B. Dance should show an inbound 67/3 with no matching outbound. Screenshot B's view of the animation.

**Open question:** does the client's own Dance button still send the request (so the silent drop is visible in the trace) or does the client hide it? Do Hello/Goodbye-style emotes (animation 3333, a row W139 could not verify) animate on B? Does the receiver see an animation at all, since 67/3 carries only the phrase id and the receiving client must resolve the animation from its own startup table?

## 3. Click menu and Inspect

**Result:** ____ (one client plus a bot, or two clients)

**Setup:** a second visible player within 40 m. For a bot: `osfr-bot` on the same gateway, as Buddy Botsworth was in the earlier two-player run. Give the target some worn items and a second unlocked job so Inspect has content.

**Steps**

1. Click the other player's character once.
2. If a radial menu appears, note its buttons and click Inspect.
3. Inspect the player. Look at the jobs, ranks and outfit shown. Close the window.
4. Click a player further than 40 m (use `realm goto` to put the bot out of reach).
5. Click an NPC to confirm the ordinary menu still works.
6. Zone away and back with the menu open.

**Expect**

- Step 2: a one-button radial labelled "Inspect" (locale 3902), icon 133 (R).
- Step 3: a pedestal with the target's active job outfit, a list of unlocked jobs with ranks, "levels gained" equal to the sum of ranks minus one per job. Coins, collections, quests, stats, titles and VIP read zero (by design, R).
- Step 4: nothing happens. No error, no menu. The trace shows no 26/9 reply and no 149/1.
- Step 6: the menu is gone after zoning.

**Capture:** inbound 26/19 and 26/10, outbound 26/9 and 149/1 (`app_out`), and the inbound 149/2 on close. Screenshot the radial and the Inspect window.

**Open questions:** (a) which of the two u64s in 26/19 is the local player and which the target (inferred from decompiled code, not a capture); (b) whether `auto_select=0` with one entry shows a one-button radial or the client opens Inspect straight away; (c) whether icon 133 draws as the Inspect icon; (d) whether the About tab shows zeros without a visible fault; (e) whether a player outside 40 m or outside AddPc range can still be clicked (the client may not even send 26/19).

## 4. Friend map and group pings (two real clients)

**Result:** ____

**Setup:** A and B in the same zone, 20 to 100 m apart, both online. Make A and B mutual friends first through the Social window (add each other by name, accept). Form a group with A and B using the client's group UI (invite and accept). A third non-friend player or bot is useful as a negative control.

**Steps**

1. A opens the Atlas (the world map). Wait 35 client ticks for it to poll (about a second or two).
2. Check whether B shows as a friend dot on A's Atlas, and whether the dot is where B is.
3. B walks away. After the next poll, check A's dot follows.
4. B moves to another zone or enters a fight. Check the dot disappears.
5. A removes B from the friend list. Check B's dot is gone from A's Atlas, then re-add.
6. A pings the map (the ping control on the minimap). Check B sees the ping. Check A sees its own.
7. A uses the group's "changing job" indicator (open the job menu) and check B's group panel marks A.
8. Control: the non-friend or non-group player should see none of this.
9. A ignores B (if available), pings again, and check B does not see it.

**Expect**

- Step 2: B appears as a dot. Before W151 this could never happen because the server wrote 0 for the "in static zone" byte of online rows (client `00cb44c0`). Online rows now write 1.
- Step 3: the dot moves, with up to 30-tick lag.
- Step 4: the dot is hidden (the server sends flag 1 with no coordinates).
- Step 6: both see the ping at the same map point, so you may see a duplicate on A if the client also draws locally.
- Step 7: the "changing job" marker on A in B's group panel (40/15).
- Step 9: no ping for B.

**Capture:** inbound 26/21, outbound 74/5 (`app_out`); inbound 40/14 and 40/16, outbound 40/15 and 40/16. Screenshot both Atlas views side by side.

**Open questions:** (a) ping x,z units match the friend X,Z units (assumed equal, unverified); (b) the sender echo of 40/16 (A seeing its own ping) was a guess; (c) a marker may sit at (0,0) for one poll before the first 74/5 arrives; (d) whether the Social window shows a cross-zone friend as "in encounter" (the server sends that flag to hide the atlas dot); (e) positions of friends are only shown in the same zone and instance, so two friends in different instances of the same area should see no dot.

## 5. Quest select, lock and the Take Me There pin

**Result:** ____ (pin: committed `dc27270`, runs now. Trail in step 7b: W164, staged only, see 0.1 and section 18)

**Setup:** Cobblestone Village. Two quests available without prerequisites: 3000021 (Samantha, objective: talk to Ricky Danger) and 3000060 (Ricky Danger, objective: talk to Samantha), both in the Adventurer set. Take both so two are active. Others with objectives: 3030007 Jones (Brawler, defeat).

**Steps**

1. `realm goto Samantha`. Talk to her, accept 3000021.
2. `realm goto Ricky Danger`. Talk to him, accept 3000060.
3. Open the quest journal. Select quest 3000021, then 3000060. Watch which one the objective tracker shows.
4. Toggle the quest lock (the pin icon on the tracker or the "lock selection" control).
5. Log out and back in.
6. (Pin, if wired) With 3000021 selected, look at the minimap and the objective panel. Click "Take Me There".
7. (Pin) Walk to Ricky Danger, finish the objective, and check the pin clears. Select the other quest and check the pin moves.
8. (Pin) Leave the area through a warpstone (section 13) and back; check the pin clears and returns.
9. Breadcrumb: if the journal has a "Start" or breadcrumb button for a not-yet-taken quest (49/15), try it on one.
10. (Pin, kill objective) `realm goto Jones`, accept 3030007 (Brawler, defeat), select it. The pin should point at a Hooligan near (-1971, 540).
11. (Pin, no place) Pick a quest whose current objective is an item, gather, craft or minigame step (any of the later Adventurer or Brawler quests). Select it. Also finish every objective of 3000021 but do not turn it in.

**Expect**

- Step 3: the tracker follows your selection; no popup.
- Step 4: with the lock on the tracker does not auto-switch to a newly accepted quest.
- Step 5: the selection and the lock survive the relog, and the tracker shows the same quest.
- Step 6: a pin to Ricky Danger at (-1909.5, 462.8) on the minimap and a "Take Me There" button that walks or points toward him. The NPC name is not sent, so the label reads "Objective Destination".
- Step 7: the pin clears when the objective is done and moves with the selection.
- Step 9: for an unregistered quest nothing happens (no crosswalk yet; breadcrumb starts are inert except for an owned active quest).
- Step 10: a pin on the lowest-guid Hooligan of that kind. That is the quest-gated Hooligan at about (-1971, 540), which may not be the one nearest you (R, known weak pick).
- Step 11: no pin and no "Take Me There" button for an objective with no single place, and none once all objectives are done and only the turn-in is left (the pin clears; a turn-in pin is not built). The server sends a 1-byte clear only when a pin was showing; it sends nothing when there was none.
- A pin is sent only when the quest's zone equals the character's zone. The pin is not saved: relog resends it.

**Capture:** inbound 47/12, 47/13, 49/15; outbound 47/14 (`app_out`; the body after the 3-byte header is 1 byte inactive, or 1 + 44 bytes active: three u32 `a`, `b`, `c`, a u64 guid, a u32 name id, four floats, a u32 zone id; the earlier "45 bytes" counted the body, 48 with the header). Also the inbound 98/1 rows after Take Me There: the client sends one (55 bytes) whenever the target is within 300 m. With W164 staged the server answers with 98/2 (section 18). Without it, per W164's recovery of `009cd410`, the client waits 10 s and then **drops the Take Me There target** (it does not just show an empty trail), so a pin that vanishes after 10 s means the 98/1 reply is missing. Save diff: `selected_quest_id` in the quest state. Client log: `Breadcrumb.log` ("Kynapse path attempt success - path nodes: N").

**Open questions:** (a) polarity of 47/13 (the wire byte is the inverse of the client's auto-switch flag, from `00c7c5d0`): lock once, unlock once, relog, and confirm the checkbox state matches; (b) the meaning of the pin's `a`/`b`/`c` fields (W161 sets a = quest id, b = 0, c = objective id, name 0, w 1.0): does the pin land on the right spot, and does the minimap marker and button both work; (c) whether the pin and button survive more than 10 s (they should only if a 98/2 reply arrives; if W164 is not in the build, expect the target to clear at about 10 s, which is the proof that the reply matters); (d) does the login self-data read `selected_quest_id` back, so the relog shows the same selection.

## 6. Vendor buy and sell

**Result:** ____

**Setup:** `realm coins 5000`. Roosey in Cobblestone Village, at (-1842.2, 411.2). Stock (price/sell): Brawler's Ring of Energy I 75/37, Brawler's Mallet of Sweeps 750/375, Brawler's Necklace of Bark Skin I 750/375. Roosey has no regional zone label in content; the earlier test `realm_goto_reaches_and_interacts_with_zone_less_roosey` exercised her. If her window does not open, try another listed vendor with stock (Sanctuary: Shephard, Linraist, Florina, Wake) and record which opened. W95 found only 62 of 192 vendors fully grantable.

**Steps**

1. `realm goto Roosey`. Talk to her and open her shop.
2. Look at the stock list and the coin total. Buy the 75-coin ring. Buy a second one.
3. Open the sell side. Check the sell list shows your items and any price.
4. Sell one ring. Sell the other. Check the coin total after each.
5. Open the buy-back list. Buy one back.
6. Try to sell an equipped item (equip the ring first), then try to sell more than you own, then try to sell something the shop will not buy (a quest item).
7. Walk 20 m away with the window open, then try to sell.
8. Close the window and talk to her again.
9. Relog and check the coin total and inventory match.

**Expect**

- Step 1: the shop window opens (the old 66/42 list had an extra pair and a seventh dword, so the client would refuse it; that is now fixed, unproven natively).
- Step 2: coins drop by 75 each; the item appears in inventory (38/19 balance update).
- Step 4: coins rise by the server's `sell_price_coins` (37 for the ring). The window's displayed sell price comes from the item definition's resell field (sent via 38/34) and may differ from what is paid (the two tables are generated separately and were not compared).
- Step 5: the sold ring is in the buy-back list and buying it back restores the exact coins.
- Step 6: refused. The server answers with a 165/6 failure. The client only raises an alert popup for a BUY failure; a SELL failure only logs "Failed transaction", so you may see nothing at all and the window may stay frozen or reset.
- Step 7: refused (range), same silence.
- Step 8: the shop reopens fresh. The server-side shop session stays open until the next interaction or range exit because 26/43 (close) is not routed.

**Capture:** inbound 66/43 (buy), 165/4 (buy), 165/5 (sell); outbound 66/42 (list), 165/6 (result), 38/19, 38/34. For each refused sell record the 165/6 `result` code (5 invalid merchant, 3 invalid item, 4 invalid quantity, 8 no-sale or equipped) and what the screen did. Screenshots: the shop window, the sell list, the buy-back list, the failed-sell moment. Save diff: coins and the ring stack before and after.

**Open questions (the main ones for this build):** (a) does the vendor window open at all with the rewritten 66/42; (b) does the SELL tab show a sold list and the coin count; (c) **does a failed sell show anything on screen** (static analysis says no popup for SELL failures); (d) is the price the window shows the price paid; (e) does the client send merchant 0 for sells (static analysis says yes, because the merchant globals are never written) or a real id; (f) are the three R fields in 165/6 (merchant, transaction id, timestamp slots, and the second leading dword) accepted without a client error; (g) does buy-back by the completion's row id work.

## 7. Equip refusals

**Result:** ____

**Setup:** Capo is male (identity `gender: 1`). `realm give` these (all on the catalogue): 76476 (freestyle bootcut pants, slot 4, Adventurer profile 1: eligible), 76497 (freestyle girls vest, slot 3, **girls only**), 75859 (Brawler layered shirt, slot 3, **needs class rank 4** on profile 43). Also buy Roosey's Brawler ring 75260 from section 6 as a class case. If your test character is a girl, swap 76497 for a boys-only item (look up a row with gender usage 2; the catalogue's `serialized_hex` carries it per W138).

**Steps**

1. Open the outfit or inventory window. Equip 76476.
2. Try to equip 76497.
3. Switch to the Brawler job (rank 1) and try to equip 75859.
4. Switch back. Try the Brawler ring on Medic.
5. Relog and check the outfit.

**Expect**

- Step 1: equips, appearance updates (38/5, 35/6).
- Step 2: "This item is only for girls." in the chat line, equipment unchanged.
- Step 3: "This item needs class rank 4." in chat, equipment unchanged.
- Step 5: only the eligible item is worn.

**Capture:** inbound 42/3 or 42/8; outbound 15/3 (the refusal line) and the absence of 38/5. Save diff: `equipment` rows unchanged after refusals.

**Open question:** does the client ever let you attempt a refused equip? W138 notes the client only greys items out. If the client blocks it, no 42/3 is sent and the server rule is never exercised natively (still correct, just invisible). Also whether the one-line chat feedback is visible to the player, and that no popup or freeze follows. Rental expiry is not enforced (the clock unit is unproven); members-only enforcement needs a members-only item (880 rows) and a non-member state.

## 8. Recipe book at a station

**Result:** ____

**Setup:** Farnum Farmstead. Cooking Table at (-2109.2, 210.3). Switch to Chef (profile 45) with `realm level` as needed (rank requirement checked). Give ingredients with `realm give` for the stew (recipe id 70, re-added on load); an exact ingredient list is shown in the client recipe window. For the Miner and Blacksmith stations: the Forge (-2076, 190) and Anvil (-2088, 200). Note that the miner catalog has no `recipe_id`, so the book cannot start smelting; only Chef and Blacksmith can be tried.

**Steps**

1. Stand beside the Cooking Table on Chef. Open the recipe book (the cooking window's recipe list).
2. Pick Stew. Start it. Play the minigame to the end.
3. Repeat from 30 m away.
4. Start again with missing ingredients.
5. Switch to Miner (not Chef) and try to start the stew.
6. Start a second recipe while one is running.

**Expect**

- Step 2: the cooking minigame opens, completes, items are consumed and the result is granted once.
- Steps 3 to 6: refused with a chat line (out of range; missing ingredients; wrong job; busy). Nothing is consumed.

**Capture:** inbound 26/24 (8 bytes: `26,0,24,0` + recipe id); outbound 39/x minigame start, 38/x inventory updates; the refusal 15/3 line. Save diff: ingredient counts.

**Open questions:** (a) does the client accept the book route's start with the same opening replies the tutorial path sends, and does the completion grant items without a pending tutorial objective; (b) does a recipe window reply to a refusal in any way besides the chat line; (c) the station list is only the opening world's three actors, so recipe starts elsewhere should be refused with the range line, confirm; (d) if a tutorial Craft objective is pending, a book-started run also advances it (R, W144's open decision).

## 9. Item bar and the Small Health Potion

**Result:** ____

**Setup:** `realm give 11002 5` (Small Health Potion, in the captured inventory). Take damage first (fight a Hooligan Wolf at about -1899, 307, or let a Hooligan hit you) so the heal is visible. `realm level 5` helps survive. Leave `realm god` off.

**Steps**

1. Open the inventory Consumables tab. Drag a potion onto item-bar slot 1.
2. Drag a second stack or the same item onto slot 2, then clear slot 2.
3. With less than full health, press the slot-1 key (or click it). Watch health.
4. Press it again immediately.
5. After 10 s press it again. Use until the stack is gone.
6. Use it at full health.
7. Drag a non-consumable (a hat) onto the bar.
8. Relog and check the bar.
9. Mid-fight, use the potion.

**Expect**

- Step 1: the slot shows the potion icon, name and a count.
- Step 3: health jumps by about 35% of max (instant, R; the original is 2% plus 11%/s for 3 s), the count drops by one, a recast timer shows.
- Step 4: refused (the recast is 10 s per definition, R); count unchanged; you may see the generic "can't use abilities right now" line (string 286).
- Step 5: works again; the slot empties and unbinds when the stack hits zero.
- Step 6: allowed (a potion is consumed even at full health, R).
- Step 7: the drag snaps back with one chat line.
- Step 8: the bar and counts return.

**Capture:** inbound 42/6 or 42/9 (assign; 42/9 is slot, definition, tint, 16 bytes), 36/10 with bar 2, 42/15 and 42/16 (use confirmation); outbound 38/25 (four slots), 38/30 (confirm), 35/35 (heal number), 36/1 (failure). Screenshot the bar before and after use.

**Open questions:** (a) which assign message the client actually sends for a drag (42/6 by item instance or 42/9 by record); (b) for the use, does it send 36/10 bar 2 with its own guid or with 0 as the self target (the server accepts both); (c) does the recast graphic animate from `total_refresh_time` and `cast_age`; (d) is the heal visible and is the cast animation or sound absent without harm; (e) the stack count on the bar goes stale after a sell or grant until the next bar action (known, not wired). The `consumables.json` file is optional; leave it absent for the first run to see the built-in numbers.

## 10. Free revive

**Result:** ____

**Setup:** an overworld knockout. Stand beside the Hooligans at about (-1970, 540) with `realm god` off and the active job set low (`realm level 1`) so they can finish you. Record your health first.

**Steps**

1. Let a Hooligan or wolf knock you out. Do not click anything.
2. Read the respawn window. Note the timer and both buttons.
3. Click "Revive at safe location".
4. Get knocked out again. This time wait without clicking until the timer ends.
5. Get knocked out again. Click the paid button ("Revive here: N coins") if it is enabled.
6. Get knocked out and close the game from the window. Log back in.
7. (Instance case) Lose a battle: `realm battle 96` and let the minions win. Use the lost-result Close or revive.

**Expect**

- Step 2: the window shows two buttons, "Revive here: N coins" greyed because the price sent is `i32::MAX`, and "Revive at safe location: Free". A countdown of about 10 s.
- Step 3: you wake at the world checkpoint at full health with about 8 s of enemy grace. A 38/1 health packet and 41/126 (hide window).
- Step 4: the client auto-sends the free revive when the timer ends. If the client never shows the window, the server revives you itself after 15 s.
- Step 5: the paid button should be greyed. If it is clickable, the server refuses, shows a chat line and re-sends the window.
- Step 6: you wake in place at full health on relog.
- Step 7: leaving a lost instance uses the existing exit.

**Capture:** outbound 41/125 (window), 41/126 (hide), 38/1; inbound 41/122 (13 bytes: header plus one bool, flag 0 free, 1 paid). The trace labels 41/122 `rejected: unknown application message`, which is cosmetic. Record the timer value shown and the cost shown. Screenshot the window and the post-revive spot. Save diff: health and position.

**Open questions:** (a) **the 41/125 field order**: the server sends `timer ms` then `cost`, inferred from the arguments at `00904300`. If the screen shows a countdown of thousands of seconds or a cost of 10000, the order is reversed; (b) does the window appear for an overworld knockout (the client-side "overworld" predicate has an unrecovered writer at client `+0x80`); (c) does it show two buttons or only the countdown; (d) the avatar has no knocked-out pose (the 35/20 bit is not sent); does the character just stand with 0 health; (e) is the zoning to the checkpoint smooth or a visible pop; (f) 41/125 built by `abilities.rs` (`build_respawn_window`) is 12 bytes and would be misparsed (needs the 20-byte layout); it is not on the world route but check no other knockout path shows a garbled window (battle knockouts).

## 11. Power-up drops

**Result:** ____ (11a, the killer's own pickup, is committed `dc27270` and runs now. 11b, per-participant drops and the big-heart group heal, needs two clients and is not wired, see 0.1)

**Setup**

- Package from 0.3, `power_ups.json` with both `standard` `chance` values at `1.0` (0.3 step 4). If you edit it mid-session, run `realm reload power_ups`.
- Solo, god mode off, `realm level 5` so the Hooligans do not end the test. Stand near the Hooligan group at about (-1970, 540). All 16 field enemies and the Bixie Soldier (12301) and Robgoblin (12302) battles carry `power_up_table: "standard"`; battle 96 is deliberately untagged and the tutorial cave drops nothing.
- To see health pickups work, get hurt first. To see energy work, spend some energy first.
- Kinds: `health_small` 15% heal, `health_big` 50% heal and a heart for the group (model 736, scale 1.5), `energy` +100 (clamped to the bar, model 737), and the held kinds `flame_wave` (1949), `earth_shard` (1950), `super_shield` (1951). Pickup radius 1.5 m, lifetime 30 s, at most 16 alive per instance.
- One client for 11a. 11b needs two clients in a group in the same zone (or a client plus `osfr-bot`; a bot stands in for the damage side but not for what the second player sees).

**Steps (11a, one client)**

1. Kill a Hooligan. Look where it fell.
2. Walk onto the pickup. Do not click.
3. Collect a health pickup while hurt, and one at full health.
4. Collect an energy pickup.
5. Collect a stored kind (flame wave, earth shard or super shield), then try to collect a second stored kind while holding the first.
6. Leave a pickup alone for 30 s.
7. Kill several enemies in a row without collecting to see the 16 cap.
8. Click the pickup actor.
9. Zone away (warpstone, section 13) with a pickup on the ground, then come back.
10. Log out holding a stored kind and log back in.

**Expect (11a)**

- Step 1: a bobbing, spinning pickup actor appears where the enemy died, with the "com_spawn" effect. Neutral disposition, so it cannot be Tab-targeted. Several drops from one kill sit 0.8 m apart.
- Step 2: walking within 1.5 m collects it (horizontal distance, from the server position; the client sends nothing). The pickup sound (composite 15509) plays.
- Step 3: a heal of 15% (small) or 50% (big) of max health and a "+N" number. It is consumed even at full health (R).
- Step 4: energy rises, clamped at 100, and the energy bar updates.
- Step 5: the first stored kind is held (invisible: it is **not** on the bar yet). The second stays on the ground, the client line "You can have one power up active at a time." appears **once** per pickup, and the pickup still despawns on its 30 s lifetime. Hearts and energy are still collected while holding a stored kind.
- Step 6: the pickup disappears at 30 s. At exactly the deadline despawn wins.
- Step 7: pickups beyond 16 alive are simply not created.
- Step 8: probably nothing (the pickup has no click route).
- Step 9: pickups are cleared on the zone change, with no removal packets. Pickups never outlive the 30 s lifetime anyway.
- Step 10: the held kind is lost (session only, by design).
- The pickup is personal to the killer for now; a second player standing next to you sees nothing at all (the pickup and its removal go only to you).

**Steps and expect (11b, two clients A and B, in a group, wired only after `apply_slice2b.py`)**

1. A and B both hit the same Hooligan; A lands the last hit.
2. A and B each stand off and wait 10 s, then A and B each kill another where only A hits it.
3. B stands in the group but never hits or gets targeted; A kills.
4. A picks up a big heart while B is hurt and in range. Then get a second big heart from the same kill if you can.
5. A picks up a big heart while B is in another zone.

- Step 1: **each of A and B gets their own pickup**, and neither sees the other's. A's appears immediately, B's on B's next server tick. Rolls are independent, so with chance 1.0 both see one.
- Step 2: a player who hit the enemy in the last 30 s or was its target at death takes part; a credit not claimed within 10 s lapses.
- Step 3: B (a bystander) gets nothing.
- Step 4: A heals 50% and B (same group, same zone and instance, alive) heals 50% of B's own max health once. A second big heart from the same kill adds no second group heal. It never revives a knocked-out member.
- Step 5: no heal for B (and the kill's group claim is spent).

**Capture:** AddNpc and RemoveNpc rows for the pickup (families 35/2 and 35/3), 35/35 (the "+N" number), 38/1 and 38/13 (health and energy), 15/3 (the held-kind line), and the pickup sound. For 11b: B's own AddNpc/35/3 rows, and a 38/1 on B at the moment A collects. Screenshot each pickup type, plus both players' screens in 11b.

**Open questions (native):** (a) does a pickup actor with `name_id` 0 show an empty nameplate or the kind id string; (b) does disposition 1 and template field 133 behave as non-targetable and non-attackable; (c) does the client send anything when you stand on or click a pickup (if so, the proximity-only design needs a click route); (d) does composite 15509 play, and from the right spot (W158 plays it on guid 1 in the personal tick); (e) the Super Shield mesh and icon are missing from the extracted packs, so that kind may render as nothing; (f) tuning data point, not pass/fail: at the shipped 12% and 10% chances, how many kills until you see one; (g) not testable yet: whether the client accepts a whole-bar 36/5 mid-fight, which slice 3 (the held power-up in toolbar slot index 2, key 3) needs, and slice 3 also waits for the owner's slot-layout decision; (h) 11b: does the group heal land on the right player and number, and does A's big heart heal the others in a group the client treats as the same group as the social service.

**Known gaps for this run:** participants are credited only for players in instance 0 with the world loaded; the engaged-target rule uses the target at death only; `Session::heal_group` has no automated test (only a socket test with two bots in a group would cover it), so 11b is the first proof.

## 12. Activity portal

**Result:** ____

**Setup:** any combat job at level 1 or higher. Three battles are published: 96, 12301, 12302 (from `field_battles.json`, all `required_combat_level` 0). A non-combat job (a gathering job) is the negative case.

**Steps**

1. Open the activity finder or Activities window (the book or minigame browser). Open the Adventure or "hack and slash" category.
2. Note the three entries, their names, icons and whether Play is enabled.
3. Click Play on one.
4. Click Play twice quickly.
5. Win or `realm win`, then close the result.
6. Switch to a non-combat job. Open the list and try Play.
7. Switch back to a combat job. Check the list updates (the client sends a refresh on job level change).

**Expect**

- Step 2: three rows with the battle's title and icon (art `hacknslash_detail.dds`, R), Play enabled for a combat job.
- Step 3: you are zoned into the battle (same as the guide entry).
- Step 4: the second Play is refused (busy), a chat line.
- Step 6: Play disabled or refused with a chat line.
- Step 7: the list is correct without relogging.

**Capture:** outbound 167/1/1 (list, once at world entry plus once per refresh, each about 330 bytes); inbound 167/1/4 (refresh, 4 bytes) and 167/1/2 (Play, 12 bytes). Screenshot the list and the detail pane.

**Open questions:** (a) does the client accept the list rows (the layout is built from the reader chain, never run), including the ImageSetId being the battle's `icon_id`, and the unknown dwords sent as 0; (b) does the portal also need the 147/1 profile list (not sent); (c) a refusal has no 147/2 body (unrecovered), so a refusal is only a chat line; is anything frozen or half-open afterwards; (d) the client sends the refresh through both tunnels, so two lists come back; any flicker; (e) does the Activity Finder show a Play button for these category-18 rows at all.

## 13. Warpstones and the Atlas

**Result:** ____

**Setup:** nine curated stones (ids 3 to 10 and 15): Snowhill (51.4, 363.2), Briarwood (-808.4, 1631.8), Sanctuary (-1414.6, 351.6), Greenwood Forest (-1583.1, -547.8), Cobblestone Village (-1904.5, 413.3), Shrouded Glade (1.2, 1115.0), Lakeshore (-734.0, 946.0), Briarheart (44.5, 2804.6), Lavender Coast Pass (-2124.8, -292.4). Attune radius 12 m, travel cooldown 5 s (R). Travel works only in the open world and not while fighting, knocked out, or in a crafting, mining, harvesting or kart session.

**Steps**

1. Open the Atlas before attuning anything. Note which hubs are listed.
2. Stand 25 m from a stone (for Sanctuary, `realm goto -1440 352`). Check nothing attunes.
3. Walk to within 8 m. Look for the chat line "attuned to the Sanctuary Warpstone".
4. Open the Atlas. Check the stone is now listed. Check an unattuned one is not.
5. `realm goto` near a second stone, then walk in the last 25 m. Attune it.
6. From the Atlas, click the first stone to travel there.
7. Click again immediately (within 5 s).
8. Try travel mid-fight, then while knocked out, then in the cooking minigame.
9. Press the world's "safe location" or let the client fall below the world (jump off a cliff, if the world floor can be hit) to trigger 122.
10. Relog and check attunements and position.

**Expect**

- Step 1: the Atlas shows no travel hubs until you attune (the server lists only attuned stones).
- Step 3: attunement by walking, saved.
- Step 6: you move to the stored position (raised to terrain+0.5 if the terrain is higher). 38/12 relocation. Facing is kept unless the original has one.
- Step 7: refused, no cooldown spent on the refusal.
- Step 8: refused with a message, no movement.
- Step 9: 122 moves you to the nearest attuned stone to your server position (the 5 s cooldown for a verified fall below terrain, otherwise a 300 s "stuck" cooldown).

**Capture:** outbound 57 (the table, re-sent on each attunement), 38/12 and 15/3; inbound 56, 90, 122, 58. Screenshots: the Atlas before and after, arrival spot, chat line.

**Open questions:** (a) **does the type-7-only POI table break the zone-change notification** (`CurrentPOI`) the client builds from it, since only notification-type-7 hubs are sent; (b) does the Atlas list only attuned hubs and open without an error; (c) does a click send a 90 with the right id and does the client wait for a reply; (d) arrival facing and ground height at non-captured hubs; (e) is the 38/12 trailer of 0 correct (as the developer teleport); (f) friend teleport 58 is refused by design (no friend lookup yet); (g) whether walking to a stone is needed at all, or the client's own discovery already shows hubs.

## 14. Kart from an NPC

**Result:** ____

**Setup:** `kart_gates.json` in the package (0.3 step 3). Cale Yardbird near Briarwood. Travel there with warpstone 4 (section 13) or `realm goto Cale`. The race content (`kart_entry.json`, `kart_lobby.json`) is in round70.

**Steps**

1. Talk to Cale Yardbird. Read his menu.
2. Pick the "Kart Race Lobby" option.
3. Drive the race to the finish (3 laps, about 270 s).
4. Close the results. Check your job, outfit, coins and XP.
5. Talk to him again within the cooldown and pick the race option.
6. Talk to a non-gate NPC and confirm no race option.
7. During a race, quit from the menu.
8. Relog after a finish.

**Expect**

- Step 1: the ordinary NPC menu with an extra race option labelled with locale 3052 "Kart Race Lobby" (R).
- Step 2: the kart loading screen and the track, with the pickup boxes (per the earlier native kart runs).
- Step 3: 50 coins and 25 Kart Driver XP, paid once.
- Step 4: you return to the overworld with your previous job restored (not stuck on Kart Driver).
- Step 5: refused within the cooldown (60 s in the scratch file; shipped bundle default 300 s). A chat line.
- Step 6: no race option.
- Step 7: leaving a race pays nothing and restores the job.

**Capture:** inbound 26/8 (interact), 26/10 (choice), outbound 26/9 (menu), then the kart family-33 and 101/102 traffic as in the earlier kart runs; a screenshot of the NPC menu; the save diff for coins, XP and job.

**Open questions:** (a) whether a kart race can be entered from the ordinary menu path and loads (the earlier native kart runs entered via `realm race`); (b) the menu label; (c) the Kart Driver outfit (the native default outfit of 4 items is not in our starter content, so the jersey may be missing: W88 catalogue adds 13419); (d) is the race sound and HUD intact when entered through the NPC; (e) the reward cooldown message and whether the lead wants "Kart Driver job must be unlocked" instead of auto-unlock on first entry.

## 15. Fishing from a guide NPC (W160, staged, not committed)

**Result:** ____ (skip until `rg -n "handle_fishing" ...gameplay.rs` finds it in the build you launch; see 0.1)

**Setup**

- `fishing_spots.json` must be in the package's `content\` (0.3). Without it no guide offers fishing, and that is the first thing to check if the menu has no fishing entry.
- No job is required (a quest video shows an Archer fishing), and no rod or item needs to be given. The first trip unlocks Fisherman (R) and does **not** change the active job. Any role; `realm goto <name>` works with a name prefix.
- Six guides stand in for the original boat, which is a world object our server does not route (R). All six are NPC content:

| Guide | NPC id | Area (minigame row) | Notes |
|---|---|---|---|
| Reed Stillwater (Stillwater Crossing) | 9000000181 | Sacred Grove Shallows (599) | strongest evidence: client text says use the boat behind him |
| Selvi Slowreel (Lakeshore docks) | 9000000194 | Rainbow Lake (598) | client text names her |
| Lila Darklure (Briarwood) | 9000000377 | Darklit Lagoon (596) | client text names her |
| Gunther (Briarwood) | 9000000339 | Brambleback's Bayou (597) | weak (R) pick, the real guide is unnamed |
| Big J (Snowhill) | 9000000024 | Wintery Basin (600) | weak (R) pick |
| Chip Numbwing (Snowhill) | 9500000736 | Frostbitten Banks (601) | client text names him |

- Menu label: locale 73552, "Go fish!" (R). The "Learn to Fish!" tutorial row 595 has no entry.
- The server anchors the fishing spot at the position where you clicked the menu entry and accepts casts 2 to 35 m from it, so stand next to real water when you click. A round ends for free if you walk more than 50 m from the anchor or change zone.
- One client. Optional: a second client watches for the cast visuals.

**Steps**

1. `realm goto Reed`. Talk to him and read the menu.
2. Talk to a non-guide NPC. Check no fishing entry.
3. Choose "Go fish!". Then click the Go or start button on the minigame dialog the client shows.
4. Cast at the water. Wait for the bite, reel in.
5. Repeat for three catches. Check the inventory and the Fisherman job (it appears unlocked, with XP; the active job is unchanged).
6. Reel too early, and let a bite pass without reeling.
7. Quit from the fishing menu.
8. Choose "Go fish!" again and, in the middle of a cast, close the game. Log back in.
9. Choose "Go fish!" and leave the dialog open without pressing Go. Walk 60 m away.
10. Start a round and, while fishing, try to start a kart race, an activity from the portal and a second guide's trip.
11. Visit the other five guides (`realm goto Selvi`, `Lila`, `Gunther`, `Big J`, `Chip`) and start a round at each. Note which ones put you at a place with water.

**Expect**

- Step 1: the ordinary menu plus one extra row "Go fish!". Only the six guides have it.
- Step 3: the dialog camera ends (26/4), a type-21 minigame offer appears (39/16) with the area's name and icon; pressing Go makes the client send 39/5 and the server answer 41/2 and a player-status update (35/20) with **bit 22 set** (0x400001). That bit is what turns the client's fishing processor on, so the cast UI should appear.
- Step 4: a catch pays exactly one fish item and 10 Fisherman XP, once. Fish are small to large per species; sizes without an item are never caught. The bag floor is raised so a full inventory does not block a catch.
- Step 5: three casts, three payments. Every hooked reel pays (catch chance 1.0, no cooldown, about one catch every 2.4 s), so this is a farm until tuned in `fishing.json`.
- Step 6: early or late reels pay nothing. A fish gets away only for a late reel, an unbound size or an unstorable catch.
- Step 7: nothing paid, the status bit is cleared (back to 1), and a new trip can start.
- Step 8: the earlier catches are kept, the cast in flight is lost, and a new trip offers fresh. Nothing is paid for the abandoned cast.
- Step 9: the round stays "offered" (busy) until 26/42, 50 m of walking, a zone change or a reconnect. Walking away ends it without a payout.
- Step 10: all three refused (fishing counts as busy and blocks warps).
- Step 11: each guide opens its own game row 596 to 601.

**Capture:** 26/8, 26/9 (menu with the extra row), 26/10 (choice), 26/4, 39/16, 39/5, 41/2, 35/20 (read the status dword), the family-138 traffic (subs 3, 4, 8, 10, 11 and 14, all (R) reconstructions), 39/19 on a server-initiated end, 41/109 and 39/6 on quit, 26/42 on dialog cancel, and 38/x inventory updates. Save diff: fish items, Fisherman unlock and XP, active job unchanged. Screenshots: the menu, the cast UI, the catch.

**Open question from the report:** is the entry the client really supports a guide NPC rather than only the boat object? Native unknowns: (a) does the 35/20 bit really start the client's fishing processor on our 39/16 plus 41/2 sequence (static analysis of `0092f460` says yes); (b) do the guides stand where a ray probe finds water within cast reach; (c) what the client shows on a server-initiated end (39/19), since type 21 has no score screen; (d) does the "Go fish!" label render; (e) do the cast, bite and reel subs behave as reconstructed; (f) the rod and lure choice from the Fish Finder is not routed (the client message is unknown), so a lure choice would do nothing.

## 16. Mounts (W162, staged, not committed)

**Result:** ____ (skip until `rg -n "settle_mounts" ...gameplay.rs` finds it; see 0.1)

**Setup**

- `mounts.json` in `content\` (it is in `data\content`). Mount state is session only: it is not saved and is gone after a relog.
- The cheapest ride to own: `realm give 14289` (Wolf, ride id 993, top speed 12.5 against a base of 10). Only **24 of the 54** catalogued rides can be owned today because 30 of them have no item definition in the captured catalogue. Ownable: Scaled Dragon, Poison Dragon, Gold-Tipped Dragon, Bat, Fire Pegasus, Tiger, Lava Tiger, Stone Saber-Tooth Tiger, Wolf, Nightmare, Armored T-Rex, Striped T-Rex, Air T-Rex, Hydra, Fire Hawk, Bicycle, Robgoblin Mech, Tractor, five Hoverboards (Molten Precursor, Precursor, Clockwork, Fiery, Stormy) and Surfboard. A ride you were not given must be refused.
- **Two clients** for the visibility rows (a second player in range, and a third who logs in afterwards); everything else runs on one.
- Near the Hooligans (about -1970, 540) for the combat trigger.

**Steps**

1. `realm give 14289`. Open the Rides panel (the dock stable icon) and look for the Wolf.
2. Summon it from the panel. Then dismount, and summon it again from the inventory item instead.
3. Run, jump and stop while mounted. Watch both your own view and, if you have B, B's view.
4. B stands within range from the start. A third client C logs in nearby while A is already mounted.
5. Dismount from the UI.
6. Try to summon a ride you do not own (from the panel if it lists others).
7. Summon twice within a second.
8. Mounted, trigger each server dismount: walk through a warpstone zone change (section 13), start a field fight (let a Hooligan chase you), get knocked out (`realm level 1`, god off), start the cooking minigame (section 8), start the kart (section 14).
9. Mounted, use an ability.
10. Mounted, send an emote (section 2 step 6), then dismount and send it again.
11. Try to summon while a Hooligan is chasing you, and while knocked out.
12. Mounted, log out. A and B and C: watch what each sees. Log back in.

**Expect**

- Step 1: the Wolf is listed; the unowned rides are not usable.
- Step 2: every summon makes the client send a dismount (168/3) first, then 168/6 (panel) or 168/8 (item). The server answers with the ride as a world actor (AddNpc), a roster refresh (168/5), and the board (168/2). When the ride actor appears, the client itself sends 168/1 (automount), which also boards. You end up seated on the ride.
- Step 3: the rider moves normally; no rubber-banding (movement validation is unchanged, the Wolf is 1.25x base and the limit is 40x). **Open gap 1:** the server moves only the rider and never moves the ride actor, so B may see the ride stay where it spawned.
- Step 4: B sees the ride and the rider seated. C, who logs in later, sees an AddNpc and a 168/2 after A's AddPc (this is a replay, not the original AddPc mount tail).
- Step 5: one step: 168/4, the ride is removed and the roster is refreshed. W25 used to need a second request.
- Step 6: refused with "You do not own that ride." and a roster refresh. Nothing changes.
- Step 7: the second summon inside 1 s is ignored.
- Step 8: each trigger ends the ride: zoning, a chasing field enemy (at the next tick after the engage flag), an instance battle, a knockout, a minigame start, and a kart race. The rider is dismounted (168/4) and the ride removed.
- Step 9: the client sends 168/3 before an ability, so it dismounts.
- Step 10: the emote is dropped silently while mounted (no animation, no text for anyone), and plays after dismounting.
- Step 11: refused with "You cannot use a ride here right now." and a roster refresh.
- Step 12: the ride disappears for B and C when A disconnects. After relog A is not mounted.

**Capture:** inbound 168/1, 168/3, 168/6 and 168/8 (check the real byte lengths; the report allowed 8 to 24 body bytes for 168/1 and 0 to 16 for 168/3), outbound 168/5, 168/2, 168/4 and the AddNpc/35/3 for the ride, the 15/3 notices, and 67/3 on the emote. A screenshot of A mounted from A's and B's view. Save diff: the inventory only (nothing else is saved).

**Open question from the report:** what does the real client send to summon (the Rides panel's 168/6, the item's 168/8, or its own automount 168/1), and does the rider draw attached to the ride? Also: (a) does B see the ride follow the rider (open gap 1; if not, an NPC move packet per rider pose is needed and is not built); (b) the 168/1 and 168/3 serializers and the refusal bytes are unrecovered (R); (c) the ride GUID is the owner's id XOR `0xc000000000000000` (R); (d) the 168/2 trailing fields, seat and queue values are (R); (e) any zone allows a ride (R), with no dungeon, duel, member or trial rule; (f) the late-join route differs from the native AddPc mount tail (W162 gap 5).

## 17. Trade (W156 shim live, W163 real trade not wired)

**Result:** ____

Two states, depending on which build you launch. Check them with the gates in 0.1.

### 17a. Unavailable shim (W156, committed `dc27270`, one client)

**Setup:** any other player in range (a bot is enough). Radial menu from section 3.

**Steps:** open the radial on the other player; look for Trade. If the client can reach trade some other way, send each of accept, cancel, offer and lock.

**Expect:** the radial offers Inspect only; Trade is not offered. Every family-59 request that gets through is answered with the native close: accept 59/3 gets 59/5, cancel 59/7 gets 59/5, offer 59/9 or 59/10, lock 59/8 or accept 59/6 gets 59/15 status 1 then 59/5. The trade window never opens or closes at once, with no crash.

**Capture:** inbound 59/x and the two replies.

**Open question:** how does the real client render status 1 followed by sub 5? (W163 recovered that status 1 alone does not close the window; only 59/5 does.)

### 17b. Real player trade (W163, **two clients**, not wired)

Run this only when the gates `route_trade` and a real `commit_trade` pass (0.1). `lab/runs/claude/W163/patch/shared-files.diff` replaces W156's hunks, and `tests/trade_unavailable_gateway.rs` must be deleted when it is applied. Until the lead replaces the fail-closed commit, the happy path ends in a cancel by design, so only steps 1 to 9 are meaningful.

**Setup:** characters A and B in the same zone and instance, within 40 m, both online and registered, each having been sent the other's AddPc (stand in view of each other first), neither marked away (not in a battle or dungeon), B not ignoring A, no pending invite or trade. `realm give` A a stackable item (for example `realm give 11002 5`, five Small Health Potions), a piece of gear to equip and keep equipped, an item to leave in the bag, and `realm coins 500`. Give B a stack of the same potion and some coins so a merge and a max-stack case exist. Add a no-trade item and a rental if you can find one. The radial's Trade row is entry id **1002** (R), label locale 3299 "Trade", icon 43188 (not checked against client assets), listed before Inspect.

**Steps**

1. A clicks B's character. Read the radial: Trade and Inspect.
2. A chooses Trade. Look at B.
3. B declines. A re-invites; B accepts.
4. A offers 3 potions and then raises it to 4. A offers 100 coins.
5. B watches the window. B offers coins and an item.
6. A tries to offer the equipped item, the no-trade item, the rental, more coins than A has, and more items than A owns.
7. A locks. B locks (note the order). A then changes an offer while both are locked.
8. Lock both again. Wait for the 10-second timer. Click Accept on both.
9. Cancel variants, each on a fresh session: A cancels; A walks more than 40 m away; A changes zone; A disconnects; B disconnects; A starts a battle; leave an invite unanswered for 60 s; leave an open session for 300 s.
10. After the commit lands: complete a trade and check both inventories; trade into a nearly full stack of the same item; trade a coin total that would overflow.

**Expect**

- Step 1: a two-row radial.
- Step 2: B gets an invite toast (59/2). A gets nothing yet.
- Step 3: B's decline gives A the "rejected" text (59/16) and frees both. An accept opens the window on both (59/4).
- Step 4: B sees A's offer rows update (59/11, 59/12, 59/13, side 0 for B). A's own window updates optimistically.
- Step 6: each refusal changes nothing; a refused item offer answers A with the real row so A's optimistic row snaps back. An over-balance coin offer is not corrected on the client (the client ignores a side-1 coin update).
- Step 7: a lock with a stale counter is refused and answered with an unlock; any change to an offer resets both locks and both accepts and tells both sides. With both locked the "other accepted" marker appears (59/17).
- Step 8: after both locks the confirm timer runs 10 s (59/18), then Accept enables. After both accept the server builds a settlement and calls `commit_trade`. **Today the commit fails closed**: both players see the cancelled status and 59/5, and the save diff shows **nothing moved**. When the lead's commit is wired: coins and items move atomically, both see status 6 and 59/5, per-definition totals are conserved, and the receiver gets fresh instance ids.
- Step 9: a client cancel, distance, zone change, a battle or an expiry gives status 1 then 59/5 to both; either side leaving gives the other status 2 and 59/5; entering an instanced zone gives 7 or 8. A withdrawn pending invite uses status 2, whose text reads as if the other left the game (the only status that removes the toast).
- Step 10: a blocked max stack cancels with status 9 or 10, a coin overflow with 3 or 4.

**Capture:** the whole family-59 lifecycle on both sides (59/2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 15, 16, 17, 18), the radial 26/9 and the 26/10 choice (check that the Trade choice really arrives as a 26/10 with the entry id), `app_out` for each reply, and both save diffs. Screenshots: the radial, the window on both sides, the timer, the cancel.

**Open questions:** (a) the radial's Trade entry id, label, icon and order, and what the client sends after choosing it (static analysis says a 26/10 with the menu guid and the entry id); (b) the counter semantics beyond "running maximum" (the original server's reply to a stale counter is unknown); (c) is the client's own optimistic coin box ever corrected; (d) the account gates (members only, trade-banned, "already trading" lines) are not traced and silent refusals give the clicker no message; (e) the missing trade-completed hook for quests; (f) the commit itself is the lead's: it must refuse unless each saved state still equals `before`, write both `after` states in one step, and then call `settled`.

## 18. Take Me There path trail (W164, staged, not committed)

**Result:** ____ (skip until `rg -n "pathing" ...gameplay.rs` finds it; see 0.1)

**Setup:** the pin from section 5, with quest 3000021 selected. Ricky Danger is at (-1909.5, 462.8) and Samantha at (-1879, 443), about 36 m apart, so stand within 300 m of him for the near case. The far case needs a spot more than 300 m away, for example the Sanctuary warpstone area (-1414.6, 351.6), about 500 m off. One client, any role. Open the client log for `Breadcrumb.log`.

**Steps**

1. Stand 100 m from Ricky Danger (use `realm goto` to a point away from him). Select 3000021 and click Take Me There.
2. Walk toward him and stay still for 10 s.
3. Walk more than 10 m away from where you started, then wait 5 s.
4. Go to the far spot (`realm goto -1414 352`) and click Take Me There again.
5. Walk back inside 300 m.
6. (Optional) `realm goto` to a zone the server has no terrain for and try again.
7. Click the button many times in a second.

**Expect**

- Step 1: the client sends one 98/1 (55 bytes, kind 1) and the server answers with a 98/2 whose points follow the terrain from you to Ricky, 5 m apart (61 points at 300 m, never more than 80). The client plays a moving breadcrumb effect along the ground and `Breadcrumb.log` reads "Kynapse path attempt success - path nodes: N".
- Step 2: the target and the trail are **still there** after 10 s. Without a reply the client drops the Take Me There target at 10 s.
- Step 3: the client re-requests (one request per 5 s while you walk and are more than 10 m from the last start) and the trail follows.
- Step 4: nothing is sent past 300 m. The client logs "Kynapse path attempt ... is too long. Not attempting". The server sees no 98/1.
- Step 5: the trail comes back.
- Step 6: a 15-byte 98/2 with a count of 0. The client logs "no path generated (tried fallback N)", makes one immediate retry and then holds, and the target survives.
- Step 7: at most 4 answered requests per second. Refused requests (non-finite numbers, a start more than 300 m from your server position, a point outside the zone grid, a wrong length) get silence, no error and no disconnect.

**Capture:** inbound 98/1 and outbound 98/2 (header 15 bytes plus 16 per point, `kind` and `aux` echoed), the row count per second, `Breadcrumb.log`. Screenshot the trail. Kind 2 requests (the point-of-interest teleport) and click-to-move kinds 3 and 4 are **not** answered and still show as `rejected: unknown application message`; that is expected and not a finding unless the client needs them.

**Open question from the report:** what the client draws from a straight-line reply (a trail, a walk, or a rejection), and whether it retries. Confirm the three assumptions W164 could not test: (a) `from` is the start and `to` the goal and nodes run start to goal; (b) echoing `kind` and `aux` is harmless; (c) `w` is interpolated between the request's two values. Expect the line to cut through walls, cliffs and water (no obstacle avoidance); the client prepends its own nav-graph nodes, which mitigates short hops. If you see false out-of-zone refusals after a teleport, the player-near check (it relies on a fresh `character.position`) is the cause.

## 19. Relog and persistence pass

**Result:** ____

Run last. Exit the game normally (menu exit), wait 6 s for the pose save, then start again from the same package.

**Check** each of these survived: warpstone attunements (13), quest selection and lock (5), item-bar slots and counts (9), coins and items after the sell (6), equipment (7), Kart Driver XP and previous job (14), the last position (the pose-only save coalesces to every 5 s and flushes on logout). Features that are **session only by design** and must be gone after the relog: the mount (16), a held power-up and uncollected pickups (11), a fishing round in flight (15; earlier catches, the Fisherman unlock and its XP persist), and a trade session (17). The Take Me There pin is not saved but is resent (5). Also from W140: start a Mining or Harvesting run (dev tools: `realm mine` or `realm harvest`), disconnect mid-run, log back in, and check jobs, equipment and interaction all work with no "Close the mining game" refusal and no payout for the abandoned run.

**Expect:** everything above present; no run left half-open; no duplicate reward.

**Capture:** `character-before.json` against `character-after.json`; the next `world_ready` and the first packets after it.

**Open question:** after a client crash (not a clean exit) the account waits up to the 60 s idle timeout; confirm the wait and the duplicate-session refusal, and that a pose-only coalescing window does not lose a gameplay change.

## 20. Small ride-along checks

These need no setup of their own; note them as you pass.

- **Ability details (W141):** hover an ability on the bar. A known ability shows its tooltip (36/12 gets a 36/13). An empty or level-locked slot cast should give the "can't use abilities right now" line (string 286), not silence. **Open question:** does the client's pending-cast UI clear on 36/1, and does hovering a missing id ever pop that failure string (it should not).
- **Role lock (W154 proves it with bots):** if you run an account-store server without `--dev-tools`, a player account should be refused `realm give` with "your account role does not allow this command", and a builder should be able to `realm goto`/`realm here` but not `realm god`.
- **Disk and battery:** the native run is long; keep the disk free (each full-suite build dir is about 28 GB) and watch the charge.

## What to record afterwards

In a new review note named `Native Run <stamp>.md` (or by feature) record: the run id and release SHA, the package folder, pass or fail per section, the answers to each **Open question** (especially: the 41/125 order, whether a failed sell shows anything, whether the client accepts a 36/5 or 36/13 mid-fight, the vendor window opening, the Atlas with only type-7 hubs, whether the 35/20 bit starts fishing, whether the client sends a 98/1 and keeps the target for more than 10 s, what the client sends to summon a mount and whether the ride follows the rider, the radial's Trade entry and the 26/10 it sends, and whether a pickup actor shows a nameplate), the screenshot names, and any new `rejected` family/subtype from the trace. Then update [[Build Plan]] and [[Build Ledger]] with a verdict per feature (PROVEN, PARTLY PROVEN or NOT PROVEN) and mark every (R) rule that the run confirmed or contradicted.
