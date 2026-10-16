# 010 Trading Card Game

Status: recovered catalogue + bounded rule slice, 2026-10-09 03:46 UTC. Queue
item 010. The decks, the victory rule, the card text block and the minigame
wire family are recovered from preserved first-party inputs and pinned
community sources. The client shipped the game as a Scaleform UI over a native
card engine, so this note is a data/protocol recovery plus one validated rule
slice. It is **not** a playability claim: no match was played, and the scoring
actions themselves are not recovered.

Owned artifacts (all new):

| Path | What |
| --- | --- |
| `scripts/pieces/piece_010.py` | Read-only audit over the shipped minigame tables, the locale export and the community packet catalogue; writes the private evidence bundle |
| `tests/pieces/test_piece_010.py` | 20 stdlib tests: caret grammar, deck parsing, card-text classification, opcode cross-check, real-source counts, determinism, output writing |
| `prototype/osfr-tools/src/restoration/piece_010.rs` | Deck catalogue shape + `TcgMatch` rule slice (first to 12 points, replay protection, once-only reward, reconnect snapshot) |
| `prototype/osfr-tools/tests/piece_010.rs` | 18 behavioral tests for the module; `#[path]`-includes it so no shared `lib.rs` edit is needed |
| `../lab/reports/piece-010-20261009T034635Z/` | Private bundle: `tcg-catalogue.json` (100,879 B), `findings.json`, `provenance.json`, `run.log` |

## Provenance (read-only inputs)

| Input | Bytes | SHA-256 |
| --- | --- | --- |
| `lab/working/OSFR.Client/OSFR Client/Resources/MiniGameData.txt` | 30,734 | `8ce3cc61a2ad324912465e8d1785d246ccd680a01712e256fddf702ce856c252` |
| `.../MiniGameTypeData.txt` | 805 | `2d200849d113c0f4b071b869774843b2fe9d07095235e7a6e907be83156e27d0` |
| `lab/reports/locale-id-to-text.json` (85,932 entries) | 6,788,031 | `f1f22a30107c824af4b3021c8c7c323c927d3c6a9b82efc387e9eaba6ae62844` |
| `lab/references/OpenSourceFreeRealms/ofrserver/Server/LibSOE/Core/PacketCodes.cs` | pinned community catalogue | cross-checked, 8/8 opcodes match |
| `lab/decompiled/client/FreeRealms/pseudocode.c` | 95 MB Ghidra export | symbol and constructor evidence below |
| `lab/references/freerealms-wiki/wiki.sqlite` page 2874 "Trading Card Game" | revision 2023-09-29T12:34:12Z | community card list |
| `lab/manifests/rust-asset-catalog.tsv` | 108,547 rows | TCG asset paths below |

Regenerate the bundle:

```powershell
& '..\lab\tools\wiki-env\Scripts\python.exe' 'scripts\pieces\piece_010.py'
```

## Original recovered facts

**Rows.** `MiniGameData.txt` declares four TYPE 3 rows (id 7, 11, 107, 108) and
124 TYPE 16 rows. `MiniGameTypeData.txt` marks TYPE 3 as Flash and TYPE 16 as
Flash. Parsed families across the 124 TYPE 16 rows: 44 campaign rounds
(`<Opponent>'s First..Fourth Round`), 67 named decks, 3 `Battle <NPC>` rows, 1
lobby, 1 tutorial, 1 single-player challenge (Kai), 2 one-off names and 5
unnamed rows. 119 rows resolve a localized name, 30 distinct opponents appear,
and the four shipped deck archetypes are `resource`, `trick`, `creature` and
`tournament` (12 rows each).

**Victory rule.** 116 of the 124 TYPE 16 rows carry the sentence "Be the first
to score 12 points to win the match". 108 of them additionally name the card
the winner receives, e.g. "…and a Pummel trick card from his deck." The
remaining 8 rows (499 lobby, 537 tutorial, 700-703 empty placeholders, 1036
Kai's challenge, 9997 unnamed) state no target; 16 rows name no reward card.
Both gaps are recorded by id rather than filled in.

**Card text block.** The shipped `en_us` locale carries the Free Realms TCG
text block. Ids 433394-433613 hold 112 entries classified as 37 names, 25 game
texts and 50 flavor texts; 108 ids in that range are absent and every gap is
listed. Examples: 433462 "Robgoblin Sorceress", 433477 "Angry Yeti", 433468
"Weaponsmith", plus card types (Animal, Pixie, Plant, Forest Troll, Ride,
Ship, Gem). Game text uses a recoverable vocabulary: duel, treasure, base,
attack, defend, power, colour, discard, flip, hand, shuffle, recover, destroy.
The same range also carries the tournament/UI strings (433372-433391: collection
sync, tournament system, event calendar, leaderboards, "Waiting for
Opponent…"), organized-play rules (433980-433997) and rarities (434012-434015:
Common, Rare, Promo, Uncommon). The later Clone Wars set the same client
shipped sits at 433836-433887 (14 entries present).

**Protocol.** The pinned community catalogue documents `BaseMiniGamePacket = 39`
with sub-opcodes `RequestTCGChallengePacket = 38`, `TradingCardStartGamePacket
= 40`, `MiniGameGameOverPacket = 18`, `MiniGameGameEndScorePacket = 47`,
`MiniGameStarsEarnedPacket = 21`, `MiniGameInfoPacket = 16` and
`PacketInviteAndStartMiniGamepacket = 180`. The native client agrees: the
Ghidra export shows `TradingCardStartGamePacket::vftable` built by
`FUN_009a9de0` calling `FUN_008a4420(0x28, …)` with two payload fields, and
`RequestTCGChallengePacket::vftable` built by `FUN_009a9ec0` calling
`FUN_008a4420(0x26, …)` with one. 0x28 = 40 and 0x26 = 38.

**Client shape.** `TradingCardGameHandler` loads `TradingCardAssets.txt`, loads
the `flash` movie `TradingCardMain`, and reports four load stages:
`OnCampaignLoadComplete`, `OnTournamentLoadComplete`, `OnCollectionLoadComplete`
and `OnMatchmakingLoadComplete`. `ClientTradingCardGameManager::Startup()` logs
realm, session ticket, launch ticket and station name, and `OnDisconnect()`
logs the same session identity; the UI movie path in the client is
`.\UI\TradingCard\TCCW_Main.swf` (loaded with scaleform handle 0x1c). The client
also contains a native card engine (`BaseCardData`, `FlipCardData`,
`SendCardData`) and a scripted-effect evaluator (`ZPSumExpression::getCardList`,
`ZPEachExpression::getCardList`) — card behaviour is scripted, not hard-coded.

**Assets.** `Resources/TradingCardAssets.txt` (3,864 B) lists the TCCW UI
lists, card templates and Clone Wars art (`thumb_anakin_skywalker.dds` and 40
other thumbnails); `Resources/SimpleCardAssets.txt` (6,129 B) lists the
Free-Realms-side card destroyers (`destroyerNinja/Robgoblin/Yeti`), card
templates and `campaignImage_101..` art. Pack catalogue: `TC_cardTemplates.lst`
`Assets_003.pack` entry 83, `TCCW_Main.lst` `Assets_004.pack` entry 413,
`cardTemplates.lst` `Assets_002.pack` entry 392, plus
`icon_category_minigames_tradingcardgame_{32,64,128}.dds`.

**Community card list.** The read-only wiki page (revision 2023-09-29) records
the physical/virtual series structure and 357 set-coded entries, including 101
`1L*` virtual-reward codes (`1L1` Passport to Free Realms … ), 60 rare, 60
uncommon, 60 common and 52 promo codes. It states that normal booster packs
hold 10 random cards plus 1 virtual reward card, that members received one
complimentary Series 4 pack each month, and that packs, individual cards and
loot cards could be traded through the TCG Lobby.

## Reconstruction boundary

Original: the four TYPE 3 rows, the 124 TYPE 16 rows and their locale text, the
12-point sentence, the named reward sentences, the locale card text block, the
opcode values, the client symbols and asset manifests, the wiki card list.

Reconstructed (marked as policy, not history): the audit's classification
vocabulary (`name`/`game_text`/`flavor`/`ui`/`reward`/`unknown`), the `RowKind`
taxonomy, the deck-name regexes, strict monotonic event ids for replay
protection, the `MatchError` taxonomy, the snapshot shape, and "the loser
receives nothing". Unclassified locale entries stay `unknown` and are counted
rather than guessed.

## Coverage gaps (explicit)

| Gap | Evidence for the gap |
| --- | --- |
| Deck-to-card mapping | Deck rows name only the winner's reward card; no shipped table lists the cards in each deck |
| Card numbers (cost/power/colour/rarity) | Not present as static client data; `BaseCardData`/`FlipCardData`/`SendCardData` show the client receiving card data messages. Rarity is localizable text (434012-434015) only |
| Match scoring actions | No recovered rule text or packet describes how points are earned; only the 12-point target is shipped |
| Matchmaking wire shape | `RequestTCGChallengePacket`(38) has one payload field and `TradingCardStartGamePacket`(40) two, but field meaning is not recovered; `ClientTradingCardGameManager` logs session/launch tickets that presumably authorize the match |
| Card effect runtime | The client evaluates card text through `ZP*Expression` handlers; the mapping from text to handlers is not recovered |
| Row anomalies | 8 TYPE 16 rows state no target; 16 name no reward card; `MiniGameData` row 700-703 use `NAME_ID=-1` (missing locale id); TYPE 3 rows 7 and 108 resolve to non-TCG locale text and row 107 has no locale entry |

## Verification

Test-first: the Python module was written after its test module, and the Rust
module after its test file. Recorded RED evidence:

```text
& '..\lab\tools\wiki-env\Scripts\python.exe' -m unittest discover -s tests\pieces -p "test_piece_010.py" -v
FileNotFoundError: [Errno 2] No such file or directory: '...\scripts\pieces\piece_010.py'
Ran 1 test in 0.007s — FAILED (errors=1)

& '.\scripts\cargo.ps1' test --manifest-path prototype\osfr-tools\Cargo.toml --test piece_010
error: couldn't find file `tests\..\src\restoration\piece_010.rs`
  --> tests\piece_010.rs:14:1
```

Green, after implementation (both re-run to completion):

```text
& '..\lab\tools\wiki-env\Scripts\python.exe' -m unittest discover -s tests\pieces -p "test_piece_010.py"
Ran 20 tests in 0.225s — OK

& '.\scripts\cargo.ps1' test --manifest-path prototype\osfr-tools\Cargo.toml --test piece_010
running 18 tests ... test result: ok. 18 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
(warning count: 0)

& '..\lab\tools\wiki-env\Scripts\python.exe' 'scripts\pieces\piece_010.py'
lobby=4 decks=124 opponents=30 score_targets=[12] cards_fr=112 cards_cw=14
wrote %USERPROFILE%\Documents\Projects\CartoonMMO\lab\reports\piece-010-20261009T034635Z\tcg-catalogue.json
```

What the checks actually assert: malformed caret rows raise with source and
line; lenient mode collects them; deck rows yield opponent, archetype,
difficulty, target and reward; Kai's challenge and the rewardless Player's Club
battles keep explicit gaps; `-1` difficulty is not a difficulty; card text
classification separates names, game text, flavor, UI and reward strings;
locale id gaps are listed, not skipped; every recorded opcode matches the
community catalogue (8 checked, 0 mismatches); the real audit reproduces
783/4/124 rows, `score_targets == [12]`, 30 opponents and 108 reward rows, and
is deterministic across runs with a fixed stamp. On the Rust side: opcodes and
the 12-point target match the recovered values; the four deck archetypes are
the only accepted archetypes; a match refuses points before the start packet,
refuses non-positive and over-target events, refuses replayed or out-of-order
event ids, finishes at the first side to reach 12, refuses every later event,
grants exactly the winner's named card and nothing while unfinished or when
the row names no card, and round-trips a reconnect snapshot that still rejects
a replayed event.

## Skills applied

`game-recon` (engine split: Scaleform UI movie plus native card engine versus
declared Flash flag), `reverse-engineering` (Ghidra packet constructors, handler
symbols, `MiniGameData` parsing), `game-research-websearch` (community packet
catalogue and the read-only wiki corpus consulted before writing any reader),
plus RTK (normal commands first), Ponytail (reused the shipped tables, the
existing locale export and the community packet catalogue instead of new
readers; the auditor is one script) and Caveman (concise status reporting while
this note stays complete). `asset-pipeline` was not needed (no asset
conversion) and `game-automation` is lead-controlled, so no native run was made
by this worker.

## Remaining integration and native gates

1. The module is not registered in `lib.rs` (shared file, lead-owned); the test
   includes it by path. The lead decides whether the gateway hosts
   `TcgMatch`.
2. No wire encoding is claimed. The two constructors recover sub-opcodes and
   field counts, not field meanings; a `TradingCardStartGamePacket` writer needs
   a captured or decompiled serialization before any client sees it.
3. Native acceptance is unstarted: the four `TradingCardGameHandler` stages and
   the `.\\UI\\TradingCard\\TCCW_Main.swf` load path suggest the original client
   expects a server-side TCG service (session/launch ticket), which is not
   implemented. Only the lead can run the shared client window.
4. The economy layer is lead-owned: this slice returns the reward card name and
   grants nothing. Wiring it to item rewards needs the same atomic path as the
   other activity rewards.

Cross-links: [[../../MMO Work Queue]], [[Original Activity Coverage]],
[[Free Realms Recovery]], [[Wiki Research Database]].
