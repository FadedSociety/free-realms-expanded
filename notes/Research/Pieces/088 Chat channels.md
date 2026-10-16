# 088 Chat channels

## Scope

Recovered the original client's chat channel identity, the client's chat command surface and the
`15/1` `PacketChat` request shape, and shipped one bounded routing decision plus a parser for it.
This is a recovery and validation deliverable: it grants nothing, sends nothing and claims no
playable native chat. Acceptance for the item is "original local/party/private channel routing;
bounded message parsing and correct recipients".

Two helpers carry real behaviour, both test-first:

- `parse_request(inner)` parses one application payload in the recovered `15/1` shape and returns
  the channel, both name records, the message, and the `WorldArea`-only area id, rejecting
  truncation, unknown channels, oversized fields and trailing bytes.
- `route(request, sender_guid, peers)` decides the recipient set, the sender echo and the `/tell`
  confirmation for one accepted request.

## Evidence

| Source | Path | What it proves |
| --- | --- | --- |
| Decompiled original client UI | `lab/decompiled/client-ui/ScriptsBase.lua` | Channel names, chat log filters, bubble split, the four channel toggles, `/tell` and `/groupsay` compose |
| Decompiled original client binary export | `lab/decompiled/client/FreeRealms/pseudocode.c` | `FUN_00b978b0` (four channel numbers), `ClientChatProcessor` command table |
| Preserved original client binary | `lab/working/OSFR.Client/OSFR Client/FreeRealms.exe` | The string pool behind that table: channel names, short labels, commands, `1`/`2`/`3`/`4` |
| Original client string mapping | `lab/working/OSFR.Client/OSFR Client/Resources/CodeStringMappings.txt` | Chat tab labels `1Area`/`2Trade`/`3LFG`/`4MembersOnly` |
| Supplied server chat system | `lab/decompiled/server/Gateway/Gateway.MapChat/ChatSystem.cs` | `15/1` field order and the `messageType == 8` trailing field |
| Pinned community `Sanctuary` | `lab/references/Sanctuary/src/...` | Full enum numbering, `PacketChat` record, `NameData`, and the recipient policy used as the baseline |
| Archived wiki (`Chatting`, `In-Game Commands`, `Guilds`) | `lab/references/freerealms-wiki/wiki.sqlite` | The published slash commands and channel descriptions |

Every source is SHA-256 hashed by `scripts/pieces/piece_088.py` into
`lab/reports/piece-088-20261009T050912Z/piece-088-evidence.json`.

## Original facts

**Channel names** (client UI): `WorldSay`, `WorldShout`, `WorldTrade`, `WorldLfg`,
`WorldMembersOnly`, `WorldArea`, `GroupSay`, `GuildSay`, plus `QuickChatWorldSay` for quick chat.
The chat log filters are `ALL_CHANNEL = "WorldSay"`, `GROUP_CHANNEL = "GroupSay"`,
`GUILD_CHANNEL = "GuildSay"`, `AREA_CHANNEL = "WorldArea"`, `TRADE_CHANNEL = "WorldTrade"`,
`LFG_CHANNEL = "WorldLfg"`, `MO_CHANNEL = "WorldMembersOnly"`. The client's short label pool also
carries `Area`, `Lfg`, `Trade`, `Shout`, `Group`, `System`, `Tell` and `Say`.

**Channel numbers the client binary pins.** `FUN_00b978b0` compares the name the chat window wants
to toggle and assigns exactly four numbers: `WorldArea` 8, `WorldTrade` 6, `WorldLfg` 7,
`WorldMembersOnly` 10. These four are the only ones the client proves.

**Command surface.** The `ClientChatProcessor` constructor registers fifteen names against nine
command ids, and the binary's string pool resolves every `DAT_` argument: `say` 0, `groupsay` 1,
`tell` 2, `whisper` 2, `yell` 3, `lfg` 4, `3` 4, `area` 5, `1` 5, `tradechat` 6, `2` 6,
`guildsay` 7, `gu` 7, `mo` 8, `4` 8. The tab labels in `CodeStringMappings.txt` read
`1Area` 401341, `2Trade` 401342, `3LFG` 401343, `4MembersOnly` 401344, so the numeric shortcuts
line up with the tab order the client renders.

**Presentation.** `OnReceiveSay` bubbles only `WorldSay`, `WorldShout` and `GroupSay`; it keeps
`WorldArea`, `WorldTrade`, `WorldLfg`, `GuildSay` and `WorldMembersOnly` chat-log only. `Tell` goes
through the whisper path (`OnReceiveWhisper`, `AddWhisper`, `TellReceived`), and the client
composes a reply as `/tell "Name" `. The minigame layer composes group chat as
`/groupsay "message"`. Members-Only is additionally gated on `Ui.IsMember()` before
`Ui.ChannelOn(self.MO_CHANNEL)`.

**Wire record.** The supplied server writes `15/1` as channel `u16`, from GUID `u64`, to GUID
`u64`, from-name, to-name, message, four floats, guild GUID `u64`, language `i32`, and a trailing
`u32` only when the channel is 8. The community `PacketChat` record matches field for field, with
`NameData` as three `i32` ids followed by the first and last strings. That is the shape
`parse_request` reads, and it is the shape the production `crate::chat::request` already reads.

## Reconstruction boundary

The full `0..10` numbering - `WorldSay` 0, `Tell` 1, `System` 2, `Whisper` 3, `GroupSay` 4,
`WorldShout` 5, `WorldTrade` 6, `WorldLfg` 7, `WorldArea` 8, `GuildSay` 9,
`WorldMembersOnly` 10 - is the pinned community enum. Four of its entries (6, 7, 8, 10) agree with
the original client binary; the remaining seven are unconfirmed and the module flags each row with
`client_number_confirmed`, which the checker cross-validates.

The recipient rules are reconstruction policy shaped by the community handler. No server binary
was recovered, so nothing here is a claim about the historical SOE server:

| Channel | Recipients | Echo |
| --- | --- | --- |
| `WorldSay` | sender's visible set | yes |
| `WorldShout` | sender's zone | yes |
| `GroupSay` | group members (party subsystem, item 089) | yes |
| `GuildSay` | guild members | yes |
| `WorldTrade`, `WorldLfg`, `WorldArea`, `WorldMembersOnly` | visible players with that channel switched on | yes |
| `Tell` | exactly one peer whose full name matches | no; the `/tell` confirmation instead |
| `System`, `Whisper` | refused: the client never composes them (`/whisper` maps onto `tell`) | - |

Two policies are deliberately stricter than the community handler and are labelled as such:
a `/tell` whose name matches more than one peer is refused instead of picking one, and peers that
ignore the sender are reported in `ignored_by` instead of silently dropped.

## Implementation

`prototype/osfr-tools/src/restoration/piece_088.rs` (test-first, `#[path]` included by
`prototype/osfr-tools/tests/piece_088.rs` until the lead registers the module):

- Recovered tables: `CHANNELS`, `CHANNEL_FACTS` (client name, confirmed number, command,
  aliases, requestable, bubble, toggle), `MAX_MESSAGE_BYTES` 2048 and `MAX_NAME_BYTES` 128.
- Parsing: `parse_request`. `Ok(None)` for another application; `Err` for a malformed `15/1`.
- Routing: `route`, `Peer`, `Delivery`, `Refusal`, `Route`.

The module has no crate-internal imports, so the lead's registration is adding
`pub mod piece_088;` to a new `src/restoration/mod.rs` and `mod restoration;` to `lib.rs`. Nothing
in that file re-implements or edits the existing `crate::chat`; the test cross-checks the two
parsers on the same vectors instead.

`scripts/pieces/piece_088.py` re-reads every pinned source and fails closed with a named check:
the client UI constants, the bubble list, the toggles, the compose strings, the client string
pool, `FUN_00b978b0`, the registration table resolved through `FreeRealms.exe`, the tab labels,
the supplied server's field order, the community enum/record/handler, and the Rust table.

## Checks

| Check | Command | Result |
| --- | --- | --- |
| Rust RED | `scripts/cargo.ps1 test --test piece_088 --manifest-path prototype/osfr-tools/Cargo.toml` before the module existed | `couldn't find file tests\..\src\restoration\piece_088.rs` - `red-test-rust.log` |
| Rust GREEN | same command | `12 passed; 0 failed; 0 ignored`, exit 0 - `green-test-rust.log`, repeated after `rustfmt` as `green-test-rust-postfmt.log` |
| Python suite | `lab/tools/wiki-env/Scripts/python.exe tests/pieces/test_piece_088.py -v` | `Ran 11 tests ... OK`, exit 0 - `green-test-python-postfmt.log` |
| Evidence document | `lab/tools/wiki-env/Scripts/python.exe scripts/pieces/piece_088.py --output lab/reports/piece-088-20261009T050912Z/piece-088-evidence.json` | `piece 088 evidence ok: 11 channels, 4 client-confirmed numbers, 15 command registrations, 6 gaps` - `checker-run-postfmt.log` |
| Fails closed | same checker with a drifted client channel name | `ClientChannelNames: ChatBubbles.WORLDSAY_CHANNEL is 'Say', expected 'WorldSay'`, exit 2 - `checker-fails-closed.log` |
| Format | `rustfmt.exe --edition 2024 src/restoration/piece_088.rs tests/piece_088.rs` | clean |

The Python suite needed three corrections before it passed (an over-narrow `ChatLog` window, an
enum parser that ignored the trailing `// Looking for group` comment, and a missing output key).
Each run overwrote `python-suite.log`, so only the final GREEN transcript is retained; the
fail-closed property is independently preserved by `checker-fails-closed.log` and by the nine
tamper cases the suite runs.

## Artifacts

| Artifact | sha256 |
| --- | --- |
| `prototype/osfr-tools/src/restoration/piece_088.rs` | `6301a3398b2a916b84de3960d631c357b3c1fb6f23feca0506b0dfbdf297f4cf` |
| `prototype/osfr-tools/tests/piece_088.rs` | `8ad3f9918c1457d1293626201597a44b69835deaa4a1ab6f8f7d7e838410c132` |
| `scripts/pieces/piece_088.py` | `2632922a384593abcd552f9ef9ba92a5556406dda72956d6a94d7ec43ee63690` |
| `tests/pieces/test_piece_088.py` | `c14ff3a805ab744466c8dcd0cef079a6056f84557a955df0c0d1a65adb44e409` |
| `lab/reports/piece-088-20261009T050912Z/piece-088-evidence.json` | `7d0341f41bc569b6a72408a3815f54a2485c896f5b440d558611f00534141a5b` |

Hashes are of the final revision. Recompute with `Get-FileHash` before relying on them;
`provenance.txt` in the same report folder records the list plus the report inventory.

## Skills applied

- **reverse-engineering**: strings -> cross-reference -> function on the real client binary. The
  channel-name pool at `0x181d6a4`, `FUN_00b978b0`, and the `ClientChatProcessor` registration
  table were found this way, then resolved through the PE image with a stdlib-only VA->offset
  reader. No guessing was needed for the names or the four confirmed numbers.
- **game-research-websearch**: used the read-only wiki corpus already pinned in the lab for the
  published slash commands and channel descriptions; no live web fetch was needed, and no new
  format work was started, so no external reader search was applicable.
- **ponytail**: reused the existing `crate::chat` parse shape and protocol understanding instead
  of designing a new one, kept to one module plus one checker with stdlib only, and made the
  existing parser a cross-check oracle in the Rust test.
- **rtk**: commands were run normally; only the harness condensed the output.
- **caveman**: terse progress messages; this note and the evidence JSON stay in plain, complete
  prose.

## Limitations and next acceptance

- Wire numbers `0/1/2/3/4/5/9` are not pinned by any recovered binary. A captured client frame
  that sends group, guild or shout chat would close that gap; the run archives hold client-to-
  server frames but they are encrypted with the session cipher, so the lead's analyzer would have
  to decode them.
- Group membership is not recovered here (party lifecycle is item 089). `route` takes the group
  set from the caller, so `GroupSay` cannot be native-accepted until that subsystem lands.
- The per-recipient membership gate for Members-Only, the server-side mute and the recovered
  `ChatSpeed` suppression defaults (Any 330 ms, Dup 1000 ms, Dup3 2000 ms, Max 30000 ms) are not
  modelled.
- The QuickChat send-to-channel record (`67/3`) is out of scope; only the typed `15/1` request is
  parsed.
- The routing decision is not wired into `gateway.rs`, and no native client run sends a routed
  chat in this item. Native acceptance stays with the lead.
- No shared file, gateway, save, content or release path was touched.
