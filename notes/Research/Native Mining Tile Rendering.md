# Native Mining Tile Rendering

Status 2026-10-09: read-only diagnosis of the native Mining18 board rendering in run
`lab/runs/client-baseline-20261009T121834Z` (release 6935, active instance 121834, paused via the
native pause button, `state.paused=true`). **No production source was edited.** All probes and
captured frames live in `../lab/reports/native-mining-tile-20261009T123204Z/`.

## Symptom (native)

`mining-board.png` / `mining-board_small.png` (SHA-256 `96466aac1d3ea94b…`): the board is live and
pausable, quotas read **0 / 15** and **0 / 6**, but every regular tile renders as the same blue rock
even though the run's board holds values 401, 402, 403, 404 and 405, and both bins are labelled
**Iron** with identical-looking default collectable icons.

## Client contract (original ActionScript, field order)

| Step | File:line | Fact |
| --- | --- | --- |
| 1 | `game-scripts/__Packages/GameClient.as:156` | `mNetwork.OnTileInfoMsg` is registered as a client callback. |
| 2 | `game-scripts/__Packages/GameClient.as:166-169` | `OnTileInfoMsg(value, type)` calls `oTileGenerator.SetTileInfo(value, type)`; argument order is **(value, type)**. |
| 3 | `game-scripts/__Packages/GameClientNetwork.as:78` | The network layer dispatches the same two arguments: `OnTileInfoMsg(value, type)`. |
| 4 | `game-scripts/__Packages/GameServerNetwork.as:32-35` | The movie's own server script sends `AddMessage(true, "OnTileInfoMsg", value, type)`. |
| 5 | `game-scripts/__Packages/core/ServerTileGenerator.as:70` | Selectables: `network.SendTileInfo(GameData.TILE_SELECTABLE + GetType(), GetType())` -> value = 400 + type. |
| 6 | `game-scripts/__Packages/core/ServerTileGenerator.as:44` | Collectables: `network.SendTileInfo(GameData.TILE_COLLECTABLE + GetType(), GetType())` -> value = 600 + type. |
| 7 | same file, lines 31 / 57 | Bonus and lootdrop follow the same `category + type` / `type` shape (700 + type, 1000 + type). |
| 8 | `game-scripts/__Packages/core/TileGenerator.as:56-83` | `SetTileInfo(value,index)`: selectable -> `sIdName = "SelectableTile" + index` (group SELECTABLE); collectable -> `sIdName = aCollectableNames[index*2]`, `sBaseName = aCollectableNames[index*2+1]` (group COLLECTIBLE_REG). |
| 9 | `game-scripts/__Packages/core/TileGenerator.as:220-249` | The clip is attached by group (`Selectable_Combined`, `Collectible_Combined_Reg`) and its `mcContent` is stopped on the combined frame. |
| 10 | `game-scripts/__Packages/core/Tile.as:76-81`, `core/SelectableTile.as:33-49` | On every state change the client executes `mcContent.gotoAndStop(sTileContentName)`, i.e. the frame name from step 8. |
| 11 | `game-scripts/__Packages/GameClient.as:218-230` | `OnQuotaInitMsg(index,value,quota,perReward,iconId)` resolves the bin label with `LookupString(GameData.sGAME + "Value" + value)`; `GameData.sGAME` is the launch game type (`GameClient.as:46-49`), "mining" for game 18. |
| 12 | `game-scripts/__Packages/Bin.as:159-221` | `Bin.SetQuota(...)` writes that string into `mTextName.text`; the bin icon is built from the tile value through the same generator. |
| 13 | `game-scripts/__Packages/GameData.as:54-90` | Constants: `TILE_SELECTABLE 400`, `TILE_COLLECTABLE 600`, `TILE_BONUS 700`, `TILE_LOOTDROP 1000`, groups POWERUP 1 / SELECTABLE 2 / COLLECTIBLE_REG 3 / COLLECTIBLE_BONUS 4. |

There is **no** `OnTileTypes` or `SetSprite` message in the recovered Mining18 scripts, and no
tint/frame field on the wire: the tile-type binding is exactly `OnTileInfoMsg(value, type)`, and the
frame, colour and icon all derive from the resolved frame name.

## Frame inventory (game_mining.gfx)

`lab/decompiled/minigames/mining/game_mining.gfx` (SHA-256 `b3c07e23eb32d16f…`) dumped with the
pinned JPEXS 26.3.0 CLI (`ffdec-cli`, `-dumpSWF`; the `.gfx` is a CFX container, signature `CFX` +
u32 raw length + zlib). 183 `FrameLabel` tags were listed; the ones that matter:

- `Selectable_Combined`: **`SelectableTile1` … `SelectableTile5`** only — there is no
  `SelectableTile0` and no `SelectableTile400`.
- `Collectible_Combined_Reg`: `Iron_LowQual`, `Iron_MedQual`, `Iron_HighQual`, `Tin_*`, `Silver_*`,
  `Gold_*`, `Copper_*` (three quality frames each).
- `Collectible_Combined_Bonus`: `Gem_Tourmaline`, `Gem_Topaz`, `Gem_Onyx`, `Gem_Moonstone`,
  `Gem_Turquoise`, `Gem_Amethyst`, `Gem_Garnet`, `Gem_Ruby`, `Gem_Fireagate`, `Gem_Malachite`.

Summary: `label-summary.json` / `label-summary.txt`.

## Captured native initialize_board comparison

The live run's captured frames (run `…T121834Z/frames/`) contain exactly seven `OnTileInfoMsg`
payloads; the parsed app messages and hashes are in `native-onTileInfo-frames.txt`:

| Frame file | App payload text | App bytes (hex, tail = tab-delimited text) |
| --- | --- | --- |
| `003562.bin` | `OnTileInfoMsg\t401\t400` | `27000e…160000004f6e54696c65496e666f4d7367093430310934303000` |
| `003563.bin` | `OnTileInfoMsg\t402\t400` | `…093430320934303000` |
| `003564.bin` | `OnTileInfoMsg\t403\t400` | `…093430330934303000` |
| `003565.bin` | `OnTileInfoMsg\t404\t400` | `…093430340934303000` |
| `003566.bin` | `OnTileInfoMsg\t405\t400` | `…093430350934303000` |
| `003567.bin` | `OnTileInfoMsg\t601\t600` | `…093630310936303000` |
| `003568.bin` | `OnTileInfoMsg\t602\t600` | `…093630320936303000` |

Source side (`prototype/osfr-tools/src/mining.rs`, revision SHA-256 `a66073ca463d84dc…`,
`initialize_board` lines 1106-1114): `message(out, …, &format!("OnTileInfoMsg\t{}\t{}",
kind.value, kind.category))`. So the second argument is `kind.category` (400/600), not the per-kind
index. Message order (`OnTileInfoMsg…` then `OnTileQueueMsg -1 board` then `OnBuildMapMsg`) and the
`39/14` framing (instance, `-1`, game 18, u32 length, tab-delimited text, NUL) match the client
contract; only the second field's value is wrong.

## Precise cause

1. **All regular tiles render as one blue rock.** The client evaluates
   `sIdName = "SelectableTile" + 400 = "SelectableTile400"` for every selectable because the server
   sent `400` as the type. `Selectable_Combined` has no such frame label, so the AS2
   `mcContent.gotoAndStop("SelectableTile400")` fails and each tile stays on the combined clip's
   initial frame; five different values therefore draw the same sprite. The recovered server
   invariant is `value = 400 + type`; the board values 401..405 imply types 1..5, which are exactly
   the five labels the movie ships. Only the emitted index is wrong.
2. **Collectables use the default frame instead of an iron-quality frame.** With type `600`, the
   client evaluates `aCollectableNames[1200]`, which is `undefined`, so `sIdName`/`sBaseName` are
   undefined and the collectable clip keeps the default `Collectible_Combined_Reg` frame (the orange
   sprites seen in the grid and bins). The same invariant gives type 1 for value 601 and type 2 for
   value 602, resolving to the `Iron_MedQual` and `Iron_HighQual` frames that exist in the movie.
3. **Both bins reading "Iron" is original client behaviour, not a placeholder.** The bin label is the
   pure string lookup `miningValue<value>`: the shipped `CodeStringMappings.txt` maps 601, 602 and
   603 to locale 117144 = **Iron** (three iron qualities share the metal name), 604-606 to Tin,
   607-609 Silver, 610-612 Gold, 613-614 Copper, and 401-405 to Black/Blue/Orange/Red/Green (see
   `mining-value-strings.txt`). So two Iron bins with quotas 15 and 6 clusters (12/2) are what the
   client is supposed to show for this board; the missing `OnTileInfoMsg` index only changes the
   *frames*, not those labels.

## Bounded recommendation (no edits made)

- Emit `OnTileInfoMsg(value, value - category)` for every configured kind: `401..405 -> 1..5` and
  `601/602 -> 1/2` for the current `mining_spec()`. That matches the recovered server invariant and
  every frame label the client has. Do not add tint/frame fields; the client has none.
- Optionally fail the content load when a kind's index has no frame label (selectables outside
  401..405, collectables outside the 600+type table, bonus outside 700+type) instead of silently
  rendering a fallback.
- Open question to settle natively: the string table groups 601..603 as Iron (Low/Med/High) while
  the frame pairing `aCollectableNames[type*2]` maps value 601 (type 1) to `Iron_MedQual`, value 602
  to `Iron_HighQual`; the first Iron frame (`Iron_LowQual`, type 0) would be value 600, which has no
  `miningValue600` string. Whether the live server numbered the iron qualities from 0 or 1 is not
  settled by the recovered scripts, so after the index fix the lead should confirm the observed
  quality sprites and, if Low/Med/High for 601/602/603 matters, adjust the configured values rather
  than the renderer.
- Expected native result after the fix: five distinct regular sprites (black/blue/orange/red/green
  labels), two distinct iron-quality collectable sprites, bins still labelled Iron/Iron.

## Provenance and hashes (SHA-256 prefixes)

| Artifact | Hash |
| --- | --- |
| `game-scripts/__Packages/GameClient.as` | `57ef9dd82e748dda…` |
| `game-scripts/__Packages/GameClientNetwork.as` | `571592993e9b9d72…` |
| `game-scripts/__Packages/GameServerNetwork.as` | `8c2b5c5a90b3cdc4…` |
| `game-scripts/__Packages/GameData.as` | `fc779888deb39e9d…` |
| `game-scripts/__Packages/Bin.as` | `c87376a8d2ff08df…` |
| `game-scripts/__Packages/core/TileGenerator.as` | `95554996ebd8a108…` |
| `game-scripts/__Packages/core/ServerTileGenerator.as` | `e4c7c0790957e152…` |
| `game-scripts/__Packages/core/Tile.as` | `da3f921420263aa6…` |
| `game-scripts/__Packages/core/SelectableTile.as` | `b157849ab4e4c991…` |
| `game_mining.gfx` | `b3c07e23eb32d16f…` |
| `Resources/CodeStringMappings.txt` | `1da9690fcc034657…` |
| `runs/client-baseline-20261009T121834Z/mining-board.png` | `96466aac1d3ea94b…` |
| `prototype/osfr-tools/src/mining.rs` (emitting revision) | `a66073ca463d84dc…` |

## Limits

Read-only: no production file was modified, no client relaunch, no save or process change. The exact
identity of the fallback frame (which colour the combined clip shows before a valid label is applied)
is inferred from the observed blue sprite plus the label inventory; the host-supplied game-type
string "mining" is evidenced by the `miningValue*` keys and the `TileGenerator` `case "mining"`
switch, not by decompiling the launcher. Native re-verification belongs to the lead.

Cross-links: [[Mining Restoration]], [[Type8 Mining Gateway Integration]].

## Fix applied (2026-10-09, narrow repair)

Owned files only: `prototype/osfr-tools/src/mining.rs` (revision
`e6e421d8e7cd0e923b9f033638f24fb04d648613e9b6719e78a28c3fc45e090e`) and the new
`prototype/osfr-tools/tests/mining_tile_rendering.rs`
(`500c6a828cbf6a0eae62068455d6d9233a8b56a4e82d801d799b51a35e8b5a68`). No other production file was
touched; no content package, UI, save or release change.

Change: `initialize_board` now emits `OnTileInfoMsg\t<value>\t<value - category>` instead of the
category, using `checked_sub` and refusing content whose value sits below its category
(`"mining tile value below its category"`). The `Type8Kind.category` field, the content schema, the
existing `mining.json` bytes, the public API, reward/economy handling, goal labels and quota
balances are unchanged. Both "Iron" bin labels are correct client behaviour and were deliberately
not renamed.

Test-first evidence (bundle `../lab/reports/native-mining-tile-fix-20261009T124532Z/`):

| Check | Result | Log |
| --- | --- | --- |
| RED on the old source | 3 tests: the new expectation failed with the captured `401\t400 … 602\t600` form, and the malformed kind wrapped into a successful build instead of refusing | `red-mining-tile-rendering.txt` (`f828611a…`) |
| GREEN after the fix | `mining_tile_rendering` 3 passed / 0 failed, exit 0 (index 401..405 -> 1..5, 601/602 -> 1/2; underflow refused; run state and snapshot bytes unchanged) | `green-mining-tile-rendering.txt` (`38556be7…`) |
| Engine regressions | `mining` 21 passed, `mining_review` 13 passed, exit 0 | `focused-mining-regression.txt` (`127812d1…`) |
| Gateway regression | `type8_gateway` 10 passed, exit 0 | `gateway-regression.txt` (`2ea07c54…`) |
| Combined final focused run | 21 + 13 + 3 + 10 passed / 0 failed, cargo exit 0 | `final-focused-suite.txt` (`e7cc4c4f…`) |
| Scoped formatting | `rustfmt --check` exit 0 on both owned files (empty output) | `fmt-check-mining.rs.txt`, `fmt-check-mining_tile_rendering.rs.txt` |

Remaining native verification (lead-owned): relaunch the board and confirm the five distinct regular
sprites and the two distinct iron-quality collectable frames, with both bins still reading
`Iron 0/15` and `Iron 0/6`. The quality-index question from the diagnosis stands: value 601 resolves
to `Iron_MedQual` under the recovered `value = category + type` invariant; if the original intent was
`Iron_LowQual` for 601, that is a content-value decision, not a renderer change.
