# 016 Minigame loss and retry

Status: code/test and evidence complete for this piece; native acceptance and engine wiring are
lead-owned and open. No client, save, movie, capture or shared file was modified by this worker.

Acceptance for the item: original loss/retry packets and fresh instance IDs; no reward on loss or
a stale finish.

## Owned artifacts

| Path | What |
| --- | --- |
| `prototype/osfr-tools/src/restoration/piece_016.rs` | Recovered `op39` frame codec plus the reconstructed instance ledger |
| `prototype/osfr-tools/tests/piece_016.rs` | 15 focused tests; the 4 distinct capture vectors covering all nine preserved frames are embedded and hash-pinned |
| `scripts/pieces/piece_016.py` | Read-only checker over the preserved lab corpus |
| `tests/pieces/test_piece_016.py` | 9 stdlib tests for the checker (fixtures plus real-corpus assertions) |
| `../lab/reports/piece-016-20261009T0306Z/` | `evidence.json`, red/green Rust logs, Python logs, `checker.log`, `run.log` |

The module is not registered in `src/lib.rs` (lead-owned). The test includes it with `#[path]`.

## Original, recovered

**Layout.** Every `op39` message is
`u16 39 | u8 sub-opcode | i32 StateId | i32 GroupId | i32 GameId | subtype payload` inside the
shared tunnel envelope `u16 outer | u8 flag | u32 inner length` (the inner length covers the inner
opcode, the sub-opcode and the payload). Three independent sources agree on this:

* Community Sanctuary serializer, PR #76 head `9236220c44086ba9a31393cc30b5ebd44d05787f`
  (`BaseMiniGamePacket.cs`, sha256 `548de79e7d1200fb655bd432d15e6c03b26e22049d64a01ae67ad7d791442024`;
  `MiniGameGameOverPacket.cs` `dbae4df4…`, `MiniGameKnockOutPacket.cs` `aa6888a3…`,
  `MiniGameGameStartPacket.cs` `bfe47609…`, `MiniGameStateRemovePacket.cs` `105f5ebe…`,
  `MiniGameLootWheelPackets.cs` `6cb8342c…`). The PR is closed and unmerged: it is used as
  research only, and no C# was copied.
* Native constructor `FUN_008a4420` in `lab/decompiled/client/FreeRealms/pseudocode.c`
  (sha256 `f84a58674b8e7de0829d562e5f106a368f34d4750894a14b093ae745fa0befe3`) writes `0x27`
  (39) and then the sub-opcode, the state id and two further ids.
* All nine preserved decoded frames under `lab/reports/original-minigame-packets/` parse and
  re-encode byte-exactly under this layout (`scripts/pieces/piece_016.py` asserts it).

**Captured loss-family traffic.** The three preserved captures contain exactly two `op39`
subtypes. Flow direction is fixed by the pcap 4-tuples: flow 0 is server `64.37.170.6:20267`,
flow 1 is client `192.168.1.145:61759`.

| Family/sub | Count | Direction | Frames | Decoded |
| --- | --- | --- | --- | --- |
| `39/23` `MiniGameKnockOutPacket` | 7 | server to client | soccer 3797/3799, racing 15449/15450, derby 5078/5079 | ids `-1/-1/-1`, counter `0/0`; sha256 `a6aa79938ed60a9255d0a51df9e20a02d1d2b065f97fecdbd0971f5487bdbbf9` |
| `39/23` | 1 | server to client | derby 29641 | ids `-1/-1/-1`, counter `1/0`; sha256 `08e233c11bd705f63ab943439d5d3559707358926b1244445f2a9223fde45721` |
| `39/7` (no recovered name) | 2 | client to server | derby 41374, racing 42933 | bare base header; StateId `1310785` / `917566`, other ids `-1`; sha256 `2f37481a…` / `2a3d4fd4…` |

The `39/23` payload is `i32 CurrentKnockouts | i32 MaxKnockouts`. The single preserved `1/0`
frame shows the counter can be non-zero while `Max` is `0`, so no threshold rule is inferred;
only the recovered HUD relation `remaining = Max - Current` is encoded (`KnockOut::remaining`).

**Loss and retry entry points.** The native export binds them by name:

| Native function | Log string | Wire |
| --- | --- | --- |
| `009be7d0` | `MiniGame:RequestStartGame` | C2S `39/5` with `StateId = -1`, resolved by `FUN_009bdc60` to the current state |
| `009bcb60` | `MiniGame:RequestRestartGame` | C2S `39/11` with the named `StateId` (no recovered name for this sub-opcode) |
| `009bf190` | `MiniGame:LeaveMiniGame` | C2S `39/7` with the named `StateId` (matches the two captured client frames) |
| `009ad480` | `MiniGame:OnGameOver` | S2C `39/18`; stores the `Won` byte at state `+99` and three ints at `+0x70/+0x74/+0x78` |
| `009b7170` | `MiniGame:ShowMiniGameEndDialog` | loss/win end card |

`39/18` (`MiniGameGameOverPacket`, S2C) is `u8 Won | i32 | i32 | i32`, with the recovered
serializer defaults `0, 0, 1` after the byte. The community note and the earlier native loss run
(`notes/Research/Native Loss Boss Display.md`) agree that this byte selects the win card versus
the `TRY AGAIN!` card. `39/17` (`MiniGameGameStartPacket`, S2C) and `39/19`
(`MiniGameStateRemovePacket`, S2C) are bare headers; `39/17` echoes the ids of the `39/5` start
request, which is the recovered "fresh instance" acknowledgement.

**Original text.** From the preserved client locale export
(`lab/reports/locale-id-to-text.json`, sha256 `f1f22a30107c824af4b3021c8c7c323c927d3c6a9b82efc387e9eaba6ae62844`):
locale id `401577` = `TRY AGAIN!` and `432500` = `Retry`. This export is community-reader output
over `locale/en_us_data.dat`, not a first-party table.

**Community name table.** `OpenSourceFreeRealms` rev `ace6a82e0ab07111be0fc8e899b5a0716724e137`,
`ofrserver/Server/LibSOE/Core/PacketCodes.cs` sha256
`f3f1287671de00cd009dd86ceaeec4765be581cd823d1da3306910361ccd0869` still lists
`MiniGameInfoPacket = 16`, `MiniGameGameOverPacket = 18`, `MiniGameKnockOutPacket = 23`,
`MiniGameGameEndScorePacket = 47`, `MiniGameCreateGameResultPacket = 67`,
`CommandPacketClosedMinigameEndScreen = 42` (family 26) and
`PacketInviteAndStartMiniGameStartGamePacket = 3` (family 180). The table carries names only.

## Explicit gaps

* No preserved frame carries `39/16`, `39/18`, `39/19`, `39/47` or `39/67`, and family 26 never
  appears in the three captures. The win/loss result card and end-screen close traffic were
  therefore never captured; their layouts come from the community serializer and native function
  binding only, and the loss path has never been replayed against a Rust server.
* `39/11` (restart) and `39/7` (leave) have no recovered names; `39/7` is identified from the
  native sender that produces it plus the two captured client frames.
* `GroupId` versus `GameId` is unverified: both are `-1` in every preserved frame.
* No reward contents, quantities, drop tables or score rows are recovered. Nothing in this piece
  grants items or currency.

## Reconstructed, labelled

`Ledger` in the module is server policy, not recovered code; the original server that owned the
`StateId` lifecycle was not recovered. It enforces, per player and per minigame stream:

1. `start` mints a fresh `StateId` and replies with `39/17`; a second start before the current
   instance settles is rejected.
2. `finish` accepts only the current, still-playing `StateId`. A loss or quit settles the
   instance with `39/18` (`Won = false`, loss only) plus `39/19` and **no** reward; a repeated
   finish is rejected.
3. A finish naming any other `StateId` — for example one issued before a retry — is rejected as
   stale, with no packets and no reward.
4. `retry` requires the current instance to be terminal and always mints a new `StateId`, so ids
   are never reused.
5. Only a win on the current, unsettled instance returns one `RewardProposal{state_id, game_id}`.
   It carries ids only; amounts and any atomic grant stay with the owning engine and the
   lead-owned economy layer.

## Checks run

| Command | Result |
| --- | --- |
| `.\scripts\cargo.ps1 test --manifest-path prototype\osfr-tools\Cargo.toml --test piece_016` against the naive baseline | RED: `3 passed; 12 failed` (`red-tests.log`), including "a loss must never propose a reward" and the stale-finish case |
| same command after the implementation | GREEN: `15 passed; 0 failed; 0 ignored` (`green-tests.log`) |
| `rustfmt --check --edition 2024 src\restoration\piece_016.rs tests\piece_016.rs` | clean |
| `..\lab\tools\wiki-env\Scripts\python.exe scripts\pieces\piece_016.py --out ..\lab\reports\piece-016-20261009T0306Z` | exit 0, `verified: 9` frames (`evidence.json`, `checker.log`) |
| `..\lab\tools\wiki-env\Scripts\python.exe -m unittest discover -s tests\pieces -p "test_piece_016.py" -v` | `Ran 9 tests ... OK` (`python-tests.log`) |

The checker is the reproducible evidence path: it re-hashes the community serializer files against
`SOURCE.json`, re-decodes and re-encodes every preserved frame, re-finds the five native entry
points and the `0x27` constructor, re-reads the two locale rows, and fails if any loss/retry-named
subtype appears in the captures.

## Remaining gates

* Lead integration: register `restoration::piece_016` in `lib.rs` and decide which engine consumes
  the ledger (the Type8 mining/harvesting engine already emits its own `39/17/18/19` sequence and
  keeps board logic private, so the wiring is an interface decision, not a copy).
* Native acceptance: no client run was made by this worker. A real loss card (`TRY AGAIN!`), a
  real Retry that mints a new `StateId`, and a stale finish that pays nothing still have to be
  observed in the shared client by the lead.
* Capture gap: a loss/retry pcap that contains `39/18`, `39/19` and the end-screen close would
  replace the community-derived layouts with captured ones and let the ledger be diffed rather
  than reconstructed.
