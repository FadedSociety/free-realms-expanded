---
type: research
updated: 2026-10-13
---

# Player trade service

Task W29 implements the in-world player trade family in `prototype/osfr-tools/src/trade.rs`. It does not implement the separate Trading Card Game `TradeCommand_*` stack. The new module is unregistered; only the lead may register it in `src/lib.rs` and route gateway messages.

## Request trigger finding

### Recovered client facts

- Trade's player radial menu is described as server-sent command 26/9 in [[../Build Guides/Gameplay/Trade]]. Native dispatcher `00aa2360` handles command case 9 and calls `00a38570` with its parsed interaction-list data.
- `00a38570` reads 0x20-byte menu entries and the target GUID pair. In its single-entry shortcut it constructs `CommandPacketInteractionSelect` (`00a36f80`), sends it through `00a382b0`, and clears the pending selection. The constructor calls the base command constructor with sub-opcode 10 and stores three dwords at `+0x10`, `+0x14`, and `+0x18`.
- The multi-button path in `00a38570` builds Lua arguments and invokes `HandlerRadialMenu` or `HandlerAdvancedRadialMenu`, including `:onPopup`. The native code shown here stops at opening that popup.
- The general interaction-menu path is documented in [[../Build Guides/Gameplay/Combat and Abilities]]: clicking a supplied menu entry sends command 26/10 `InteractionSelect` with the selected entry id. It is the same client command class used by the single-entry shortcut above.
- Player trade itself has no family-59 request class or sender. Family 59 starts with the invite/response lifecycle; the invite is server-to-client.

### Finding and limit

**Inference:** clicking the server-supplied Trade entry requests the trade through the generic command 26/10 `InteractionSelect`, carrying the selected menu entry and target pair. The server must map its own Trade entry id to `TradeService::invite(target_guid)`. This is not a new family-59 packet.

The packet-level trigger is identified. The available native map does not trace the multi-button Lua click callback from `Ui.RadialMenuClicked` through the final call, and the Trade entry's numeric id is server content, not a universal client constant. No native trade capture confirms the full click-to-invite exchange. Do not claim those details recovered.

### Trigger trace commands and evidence

From the project root, these read-only queries used the accepted `lab/native-map/run/map.sqlite`; disassembly was checked because the `00a38570` label is not itself a verified semantic name:

```powershell
python -I scripts/native_map/nmq.py show 00a38570
python -I scripts/native_map/nmq.py callers 00a38570
python -I scripts/native_map/nmq.py show 00aa2360
python -I scripts/native_map/nmq.py show 00a36f80
python -I scripts/native_map/nmq.py show 00a39090
python -I scripts/native_map/nmq.py callers 00a39090
python -I scripts/native_map/nmq.py strings 'RadialMenuClicked|HandlerRadialMenu|AdvancedRadial|Trade'
python -I scripts/native_map/nmq.py dis 00a38570
python -I scripts/native_map/nmq.py command InteractionSelect
```

The output showed dispatcher case 9 reaching the menu routine, a 0x20-byte entry loop, a one-entry path constructing/sending command 26/10, and a multi-entry path entering Lua popup handlers. The `InteractionSelect` command guide independently records the selected-entry packet as command 26/10. Output was terminal-only; the source map is `lab/native-map/run/map.sqlite`. These observations support the packet-level inference above, not the unresolved Lua click callback or numeric entry id.

## Recovered client protocol

The sources below are the accepted native client map and [[../Build Guides/Gameplay/Trade]]. Function addresses refer to `lab/native-map/run/map.sqlite`; `nmq show` supplies the accepted label/pseudocode and `nmq dis` checks bytes. Ghidra pseudocode is inferred, not original source.

| Message | Client reader | Body after `u16 family = 59, u8 sub` |
|---|---|---|
| Invite, sub 2 | `00c6cab0` | NameData, sender GUID as two `u32`, session id `u32` |
| Start, sub 4 | `00c6cb40` | partner GUID as two `u32`, NameData, session id `u32` |
| End, sub 5 | `00c6c530` | session id `u32`; only packet that closes the trade window |
| Coin update, sub 11 | `00c6bf50` | session, counter, side `u8`, coins `u32` |
| Add item, sub 12 | `00c6ce30` | session, counter, side `u8`, ItemInstance |
| Item count, sub 13 | `00c6c000` | session, counter, side `u8`, item id, resulting quantity |
| Status, sub 15 | `00c6c630` | one status byte, no session id |
| Rejected, sub 16 | `00c6cdb0` | NameData |
| Lock update, sub 17 | `00c6c110` | session id, locked `u8` |
| Confirm, sub 18 | `00c6c5f0` | session id |

The shared NameData reader is `008e7290`: three metadata dwords followed by two length-prefixed UTF-8 strings. Their meanings are unknown. W29 writes zero metadata words (R).

The ItemInstance add-item reader is `008fa4b0`. The guide records this order: definition, tint, item guid, count, consumed count, last-cast age, rental flag and optional rental block, then stored count. The main-bag available-count predicate in `00a6d9b0` is `Count - StoredCount - ConsumedCount`.

The nine client status values are read from the switch in `00c6df10`: 1 canceled; 2 partner disconnected; 3 partner offer invalid; 4 your offer invalid; 6 completed; 7 you entered an instanced zone; 8 partner entered an instanced zone; 9 your offer exceeds the partner's max stack; 10 partner's offer exceeds your max stack. The client does not close on status alone; send sub 5 after an active cancel or completion.

`TradeAcceptTimer` is 10 seconds of in-game time. The client enables Accept when it fires. Sub 18 starts that timer after both sides lock. A family-59 client accept is sub 6 and includes session id plus the current counter; a lock is sub 8 and includes session id, counter and a byte.

## Reconstructed server policy (R)

- Keep one pending or active trade per participant. Check both players are online and reject a target who ignores the inviter. A busy target's client silently drops a new sub-2 invite, so the server checks before sending.
- Allocate a fresh `u32` session id, skipping the client's `0xffffffff` no-session sentinel. Start the client's counter at 1. Raise it on each changed item or coin offer. Reject stale lock/accept counters.
- Treat item offers as resulting totals. Require ownership and available quantity, a definition not marked `NO_TRADE`, no equipment link, and no pet-equipment membership. The current `inventory::Definition` does not expose `NO_TRADE`; the host supplies `TradeItemDefinition` metadata.
- Treat `TradeInventory::bag_capacity` as a host-provided count of occupied item rows. The original bag limit is not recovered. `max_stack: None` means uncapped; the native `MAX_STACK_SIZE = -1` meaning remains unresolved.
- Changing an offer clears both locks, both accepts, and the confirmation timer. Send sub 17 unlocked so each client clears its popup. When both lock again, start the 10-second minimum accept delay. A session timeout has no recovered duration; the host calls `timeout` when its selected timeout fires.
- Revalidate both offers, balances, stack limits and bag capacity after the second valid accept. Clone both inventory snapshots, apply all debits/credits to the clones, and replace both only after every check passes. Bag capacity has no dedicated status, so W29 uses the generic invalid-offer statuses 3/4 (R). Stack overflow uses 9/10 from the failed offer's and receiver's point of view.
- Allocate a new item instance id in the receiver's inventory and merge only a complete fitting stack with matching definition/tint when both rows are non-rental. These ownership/merge rules follow the current local inventory grant behavior and are not recovered original trade rules (R).
- Active cancel sends status 1 then sub 5 to each side. Disconnect sends status 2 then sub 5. Instance entry sends viewer-relative status 7/8 then sub 5. Successful settlement sends status 6 then sub 5. Declining an invite sends sub 16 with the refuser's NameData; no session window opened.

## Implemented surface and lead integration

`TradeService` consumes the live `PlayerRegistry` binding and `SocialService` presence/ignore list for invitations. Callers register a `TradeInventory` snapshot with definition policies, pet-equipped ids and bag capacity. The service returns recipient-tagged inner family-59 messages; gateway wrapping remains the caller's job. It holds the staged/committed `InventoryState` snapshots in memory.

Lead integration steps:

1. Register `trade.rs` in the crate and replace the `#[path]` inclusion in `tests/trade.rs` when ready.
2. Route command 26/10's server-owned Trade entry id to `invite` with the selected target GUID. Confirm that id in the actual radial-menu content before enabling the route.
3. Register each online player with an inventory snapshot and recovered definition metadata. Add a trusted `NO_TRADE` adapter; provide pet-equipped ids and the configured bag capacity. Reject or defer trading when any required metadata is absent.
4. Dispatch client sub 3/6/7/8/9/10 to invite reply, accept, cancel, lock, item offer and coin offer. Pass in-game time to lock/accept and call disconnect/instance-exit/timeout before the player session is removed.
5. Persist both committed inventory snapshots as one transaction, then emit the inventory-family updates plus the returned trade messages. W29 returns trade messages only; no family-38 inventory refresh or persistence transaction is wired here.
6. Run the two-client native window/inventory acceptance. The guide has no decoded player-trade datagram, so byte-layout tests do not prove original-client acceptance.

The earlier isolated `restoration/piece_085.rs` remains untouched. Its `WireFrame` uses a one-byte base field; W29's output follows the current guide's inner header of a little-endian `u16` family plus one-byte sub-opcode. The old prototype also has no `InventoryState`, definition eligibility, capacity, or coin-transfer integration.

## Verification

From the project root, format the two owned Rust files; then run the focused target from `prototype/osfr-tools` with the project's private toolchain:

```powershell
$rustfmt = '%USERPROFILE%\Documents\Projects\CartoonMMO\lab\tools\rustup\toolchains\stable-x86_64-pc-windows-gnu\bin\rustfmt.exe'
& $rustfmt --edition 2024 'prototype\osfr-tools\src\trade.rs' 'prototype\osfr-tools\tests\trade.rs'
$env:CARGO_TARGET_DIR = '%USERPROFILE%\Documents\Projects\CartoonMMO\lab\build\W29'
Push-Location 'prototype\osfr-tools'
& '..\..\scripts\cargo.ps1' test --test trade
Pop-Location
```

The focused target passed 11 tests, 0 failed. It covers exact message shapes, item/coin conservation, timer enforcement, lock reset, eligibility and available quantity, bag-capacity rollback, ignore checks, decline, cancel, timeout, disconnect, instance exit and duplicate accept. The Rust build emitted one pre-existing `housing.rs:567` dead-code warning for `instance_index`.

The test executable was `lab/build/W29/debug/deps/trade-588b05bce4fddf43.exe`. The build output stays in `lab/build/W29`. No client was launched and no native trade was run.
