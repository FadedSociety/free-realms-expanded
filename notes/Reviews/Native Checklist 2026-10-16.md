---
type: review
created: 2026-10-16
status: unrun
---

# Native Checklist 2026-10-16

One play session in the real 2014 client that checks everything the Claude workers landed on 2026-10-16 (W138 to W151, W155, W158, W161). Nothing below has been seen on a native client yet. Every behaviour is proven only by unit tests, gateway-level tests and, for six features, bots (`tests/bot_acceptance_1016.rs`, W154). Treat each "what should happen" line as a hypothesis from static analysis, and every field order marked (R) as a guess to confirm.

Sections follow the order a session naturally flows: login, town, shops, crafting, fights, travel, then relog. Each has Setup, Steps, Expect, Capture and the Open question the run answers. Fill the **Result** line as you go.

## 0. Before you start

### 0.1 What is live at HEAD `5f06115`

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
| **Take Me There pin (47/14)** | W161 | not committed | **No.** The builder and tests exist; the `gameplay.rs` hook is a pending shared patch. |
| **Power-up drops** | W155, W158 | data committed; engine uncommitted | **No.** `power_ups.json` is in `data/content` but `gameplay.rs` does not load it and no enemy rolls it until the lead wires it. |
| **Trade unavailable answer (59/x)** | W156 | not committed | **No.** The adapter exists; the route patch is pending. |
| **Fishing entry** | W160 | in flight | No |
| **Mounts** | W162 | in flight | No |
| **Real player trade** | W163 | in flight | No |
| **98/1 path answer (breadcrumb trail)** | W164 | in flight | No |

Run the gate for each "No" row just before the session and skip a section if its gate fails:

```
git -C project log --oneline -30
rg -n "power_up|PowerUps" project/prototype/osfr-tools/src/gameplay.rs
rg -n "Tracker|tracked_target" project/prototype/osfr-tools/src/gameplay.rs
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
| 11 Power-up drops | No (a second client is optional for the group heal, which is not built) |
| 12 Activity portal | No |
| 13 Warpstones and Atlas | No (58 friend teleport is refused, so a friend is not needed) |
| 14 Kart from an NPC | No |
| 15 to 18 (once wired) | 17 real trade needs two clients; the rest do not |
| 19 Relog | No |

`run-local-baseline.py` starts one client. For a second real client use a second PC over the LAN or VPN with its own account and enrolled key (see [[Playtest Host Runbook]]); this is the main blocker for sections 2, 4 and 17. Until it exists, run those sections with a bot as the second player and mark the receiving-side checks as not done. A bot (`osfr-bot`) stands in as a visible second player for sections 2 (server side) and 3.

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
4. Optional, to make the power-up check take minutes instead of hours (section 11): in the scratch `content\power_ups.json` set both `chance` values of the `standard` table to `1.0`. The shipped chances are 12% and 10% (R).
5. Wallet: `realm coins 5000` after login. Capo has 309 coins; the cheapest checks below cost 75 to 750.

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
| `realm reload all` | reload battles, enemies, abilities (does not reload `power_ups.json` unless the lead has added that, check) |

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

**Result:** ____ (pin part runs only if W161 is wired; see 0.1)

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

**Expect**

- Step 3: the tracker follows your selection; no popup.
- Step 4: with the lock on the tracker does not auto-switch to a newly accepted quest.
- Step 5: the selection and the lock survive the relog, and the tracker shows the same quest.
- Step 6: a pin to Ricky Danger at (-1909.5, 462.8) on the minimap and a "Take Me There" button that walks or points toward him. The NPC name is not sent, so the label reads "Objective Destination".
- Step 7: the pin clears when the objective is done and moves with the selection.
- Step 9: for an unregistered quest nothing happens (no crosswalk yet; breadcrumb starts are inert except for an owned active quest).

**Capture:** inbound 47/12, 47/13, 49/15; outbound 47/14 (`app_out`, 45 bytes with a pin, 1 byte to clear). Also the inbound 98/1 rows after Take Me There: the client sends them whenever the target is within 300 m and waits for a reply the server does not send yet (W164 is building one). Save diff: `selected_quest_id` in the quest state.

**Open questions:** (a) polarity of 47/13 (the wire byte is the inverse of the client's auto-switch flag, from `00c7c5d0`): lock once, unlock once, relog, and confirm the checkbox state matches; (b) the meaning of the pin's `a`/`b`/`c` fields (W161 sets a = quest id, b = 0, c = objective id, name 0, w 1.0): does the pin land on the right spot, and does the minimap marker and button both work; (c) with no 98/1 reply, does the client draw no breadcrumb trail, log "no path", or retry in a loop (watch the trace row count for 98/1); (d) does the login self-data read `selected_quest_id` back, so the relog shows the same selection.

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

## 11. Power-up drops (only once wired)

**Result:** ____ (skip until the gate in 0.1 passes)

**Setup:** the scratch `power_ups.json` with chance 1.0 (see 0.3 step 4), tables `standard`. The Hooligans (field_enemies) and the 3 field battles (96, 12301, 12302) carry `power_up_table: "standard"` in the uncommitted data. Kinds: health_small (15% heal), health_big (50%, group heal not built), energy, flame_wave, earth_shard, super_shield. Models 736, 737, 1949, 1950, 1951.

**Steps**

1. Kill a Hooligan. Look where it fell.
2. Walk onto a pickup (do not click).
3. Collect a health pickup while hurt, and one at full health.
4. Collect an energy pickup.
5. Collect a stored kind (flame wave, earth shard or super shield). Then try a second stored kind.
6. Leave a pickup for 30 s.
7. Kill several at once to see the 16-per-instance cap.
8. Try a click on the pickup actor.

**Expect**

- Step 1: a bobbing, spinning pickup actor spawns where the enemy died (and a "com_spawn" effect), disposition neutral so it is not Tab-targetable.
- Step 2: walking within 1.5 m collects it (proximity only; no packet is sent by the client in the design).
- Step 3: health heals by 15% and a +N number appears; collected even at full health.
- Step 4: energy fills, 38/13.
- Step 5: the first stored kind is held; the second stays on the ground and the client line "You can have one power up active at a time." appears once.
- Step 6: the pickup disappears at 30 s.
- Step 8: probably nothing.
- The held power-up is **not** shown on the bar yet (slice 3 is blocked on your key-3 slot decision).

**Capture:** outbound AddNpc/RemoveNpc rows (families 36/39 as usual for actors, check the trace), 35/35, 38/13, 15/3; the pickup sound (composite 15509). Screenshot each pickup type.

**Open questions:** (a) does a pickup actor with `name_id` 0 show an empty nameplate or the kind id string; (b) template field 133 and disposition 1 behaviour; (c) does the client send anything when you stand on or click a pickup (then the proximity-only design needs a click route); (d) does composite 15509 play; (e) the Super Shield mesh and icon are missing from the extracted packs, so that kind may render as nothing; (f) at 12% and 10% real chances, how many kills until you see one (a tuning data point, not a pass/fail); (g) not testable yet: whether the client accepts a whole-bar 36/5 mid-fight, which slice 3 (the held power-up in toolbar slot index 2, key 3) needs; slice 3 also waits for your slot-layout decision. Group heal for the big heart needs cross-player routing and is not built.

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

## 15. Fishing entry (only once wired, W160)

**Result:** ____ (skip until the gate passes)

W160 recovers how a player starts fishing (a spot, a pole, or a listed NPC (R)) and builds admission and glue; the entry has not been seen natively. Setup comes from the W160 report: `fishing_spots.json` if it adds one. Steps will follow W149's pattern: reach the entry, start, cast, catch, quit and reconnect. **Expect** exactly one server-owned payout per catch and nothing for a quit or a forged catch. **Capture:** fishing family 138 and minigame type 21 traffic, and the save diff for items and coins. **Open question:** which ordinary entry the client actually supports.

## 16. Mounts (only once wired, W162)

**Result:** ____ (skip until the gate passes)

W162 routes summon and dismount, auto-dismount on zoning, combat start, knockout and minigame entry, and supplies `is_mounted()` to the emote rules. **Setup:** an owned mount (`mounts.json`), `realm give` the mount item. **Steps:** summon, ride, have a second client watch (2 clients for the visibility check), dismount by each trigger, then try an emote while mounted (section 2 step 6) and expect the silent refusal. **Capture:** mount family 168 traffic, the AddPc or attachment the other player sees, the mounted speed. **Open questions:** what the client sends to summon (item use, a mount bar, a 168 request), what the rider looks like to others, and whether movement validation accepts the mounted speed (R: validation is unchanged).

## 17. Trade

**Result:** ____

Two states, depending on what has landed:

- **Unavailable shim (W156, when routed):** one client. Open the radial on another player (section 3); Trade is not offered. If the client reaches trade some other way, every family-59 request gets the native close: accept gets 59/5, cancel gets 59/5, offer or lock gets 59/15 status 1 then 59/5. **Expect:** the trade window never opens, or closes at once, with no crash. **Capture:** inbound 59/x and the two replies. **Open question:** how the real client renders status 1 followed by sub 5.
- **Real trade (W163, when landed):** **two clients required.** Invite, accept, offer an item and coins, lock, accept, complete, then cancel variants (walk away, zone, disconnect). **Expect:** a changed offer resets both accepts; the settlement commits atomically (the lead's own work); nothing duplicates. **Capture:** the whole family-59 lifecycle and both saves. **Open questions:** the radial's Trade entry id and label, and what the client sends after choosing it.

## 18. Take Me There path trail (only once W164 lands)

**Result:** ____ (skip until the gate passes)

With the pin from section 5 set, the client sends 98/1 when the target is within 300 m. W164 answers with a straight sampled line, or the client's failure reply when terrain is missing. **Steps:** select 3000021 within 300 m of Ricky Danger and click Take Me There, then again from over 300 m. **Expect:** breadcrumb dots along the ground toward the NPC; nothing past 300 m. **Capture:** inbound 98/1 and the reply. **Open question:** what the client draws from a straight-line reply (a trail, a walk, or a rejection), and whether it retries.

## 19. Relog and persistence pass

**Result:** ____

Run last. Exit the game normally (menu exit), wait 6 s for the pose save, then start again from the same package.

**Check** each of these survived: warpstone attunements (13), quest selection and lock (5), item-bar slots and counts (9), coins and items after the sell (6), equipment (7), Kart Driver XP and previous job (14), the last position (the pose-only save coalesces to every 5 s and flushes on logout). Also from W140: start a Mining or Harvesting run (dev tools: `realm mine` or `realm harvest`), disconnect mid-run, log back in, and check jobs, equipment and interaction all work with no "Close the mining game" refusal and no payout for the abandoned run.

**Expect:** everything above present; no run left half-open; no duplicate reward.

**Capture:** `character-before.json` against `character-after.json`; the next `world_ready` and the first packets after it.

**Open question:** after a client crash (not a clean exit) the account waits up to the 60 s idle timeout; confirm the wait and the duplicate-session refusal, and that a pose-only coalescing window does not lose a gameplay change.

## 20. Small ride-along checks

These need no setup of their own; note them as you pass.

- **Ability details (W141):** hover an ability on the bar. A known ability shows its tooltip (36/12 gets a 36/13). An empty or level-locked slot cast should give the "can't use abilities right now" line (string 286), not silence. **Open question:** does the client's pending-cast UI clear on 36/1, and does hovering a missing id ever pop that failure string (it should not).
- **Role lock (W154 proves it with bots):** if you run an account-store server without `--dev-tools`, a player account should be refused `realm give` with "your account role does not allow this command", and a builder should be able to `realm goto`/`realm here` but not `realm god`.
- **Disk and battery:** the native run is long; keep the disk free (each full-suite build dir is about 28 GB) and watch the charge.

## What to record afterwards

In a new review note named `Native Run <stamp>.md` (or by feature) record: the run id and release SHA, the package folder, pass or fail per section, the answers to each **Open question** (especially: the 41/125 order, whether a failed sell shows anything, whether the client accepts a 36/5 or 36/13 mid-fight, the vendor window opening, the Atlas with only type-7 hubs), the screenshot names, and any new `rejected` family/subtype from the trace. Then update [[Build Plan]] and [[Build Ledger]] with a verdict per feature (PROVEN, PARTLY PROVEN or NOT PROVEN) and mark every (R) rule that the run confirmed or contradicted.
