# 048 Ability bar

Status: code plus tests returned for lead review, 2026-10-09 04:42 UTC. This piece recovers the
**original action-bar binding request, the slot refresh frame and the item-bar binding layout**, and
ships a bounded Rust module plus a checker that keeps persisted bindings limited to owned, available
actions. It is not a playable claim: no native input was sent and the request/reply path has not been
exercised against the live client.

Owned paths: `prototype/osfr-tools/src/restoration/piece_048.rs`,
`prototype/osfr-tools/tests/piece_048.rs`, `scripts/pieces/piece_048.py`,
`tests/pieces/test_piece_048.py`, this note and `lab/reports/piece-048-20261009T0442Z/`. No shared
`lib.rs`, gameplay, gateway, Cargo manifest, launcher, content, state or native UI file was touched.

## Recovered original facts

**Assign request (42/6).** The native inventory base constructor `00a10400` writes
`param_1[1] = 0x2a` (family 42) and copies the sub opcode; `00c14720` constructs
`InventoryPacketItemActionBarAssign` with `FUN_00a10400(6)`, a zero-based slot and an owned item
instance id; the header writer `00a1ada0` emits the family and sub opcode as two 16-bit fields. The
request is therefore exactly 12 bytes:

| Offset | Type | Field |
|---|---|---|
| 0 | u16 | family 42 |
| 2 | u16 | sub 6 |
| 4 | i32 | zero-based item-bar slot |
| 8 | i32 | owned item instance id (0 unassigns) |

The pinned community deserializer reads `Slot` then `Guid` and rejects any trailing byte, which
matches the native order.

**By-item-record variant (42/9).** `00a122e0` constructs
`InventoryPacketItemActionBarAssignByItemRecord` with `FUN_00a10400(9)`, a slot and three copied
item-record fields (`param_3 + 4/8/0xc`). The serialized record order is **not** recovered, so the
module classifies sub 9, records the observed length and refuses to parse or apply it.

**Item bar identity.** The original client assigns dock consumables with
`Ui.AssignToItemActionBar(button - 5, itemId)`, the consumable equipment view (map 18) with
`Ui.AssignToItemActionBar(slot - 18, itemId)`, and activates them with `Ui.ClickedAbility(button - 5, 2)`.
Fishing assigns its lure to slot 0 and unassigns it with item id `0`. The pinned community handler
hardcodes bar id 2 for the same bar.

**Slot count.** The preserved captured self packet
(`working/OSFR.Server/OSFR Server/Customize/PacketSendSelfToClient.json`) carries one `ItemActionBar`
with bar id 2 and exactly four slots `0..3`. Three are bound, disabled, with quantities 59/28/33, and
slot 1 carries the never-cast cast-age sentinel `2147483647`; slot 3 is empty. The native reader
accepts any i32 slot, so four is server policy rather than a wire constraint.

**Slot refresh (38/25).** `008dadd0` consumes the bar id and the slot index as two i32; `008dae30`
consumes one 51-byte slot. The packet is `4 header + 8 bar/slot + 51 slot = 63` bytes.

| Slot offset | Type | Recovered role |
|---:|---|---|
| 0 | bool | empty flag; activation of an empty slot is refused natively |
| 1 | i32 | icon id |
| 5 | i32 | icon tint id |
| 9 | i32 | localized name id |
| 13 | i32 | first targeting selector; the capture uses 1 |
| 17 | i32 | second targeting selector; the capture uses 2 (self) |
| 21 | f32 | targeting parameter, NaN-checked by the native reader |
| 25 | i32 | mana cost |
| 29 | bool | server enable flag, independent of the item's own activation field |
| 30 | i32 | community `Unknown10` (elapsed/refresh variant) |
| 34 | i32 | total refresh time |
| 38 | i32 | community `Unknown12` (elapsed/refresh variant) |
| 42 | i32 | displayed quantity, the owned stack count |
| 46 | bool | force-dismount flag copied into the native cast state |
| 47 | i32 | cast age; `2147483647` selects the native never-cast sentinel |

The byte layout is cross-checked three ways: the native read order above, the pinned community
serializer (`ActionBarSlot` writes 15 fields, 51 bytes) and the preserved capture, whose slot values
render to the same bytes the module carries as `CAPTURED_SLOT_0/1/2` and `CAPTURED_EMPTY_SLOT`.

The self-targeting pair is recovered from `00a35600`, which sets the self flag for exactly
`selector 1 && (target selector 2 || target selector 4)`. The captured slot uses 1/2, so that is the
template the module renders for a fresh bind.

**Original client UI columns.** `ScriptsBase.lua` defines `ItemActionBarCol` as
`IsEmpty 0, IconID 1, Name 2, IsEnabled 3, RefreshTimeLeft 4, TotalRefreshTime 5, Quantity 6,
IsTargeting 7, IconTintValue 8`. The dock reads icon, name, quantity, tint and enabled state per row
and shows a cooldown timer from total/refresh time. Which wire field feeds `RefreshTimeLeft` is still
unmapped.

**Pinned community policy (facts, not copied source).** Sanctuary revision
`4f028dcc274f628b65ef665fd59b184f5f3d3e3e`, AGPL-3.0: on `Guid == 0` the handler clears the slot and
replies with an empty slot; an unknown item instance logs a warning and sends nothing; a known item
renders `Icon.Id`, the tint fallback (`Tint == 0 ? definition tint : Tint`), `NameId` and
`Quantity = clientItem.Count`; a scheduled re-enable is cancelled when the slot is reassigned.

## Reconstructed policy in the module

Label these as reconstruction, not history:

1. The item bar is bounded to the four captured slots; out-of-range slots are refused with no reply.
2. Persisted bindings are `(slot, item instance, definition)` and are pruned against the live
   inventory on every change and on reconnect, so only owned instances with a positive stack and an
   unchanged definition survive. A consumed item or a definition drift drops its binding.
3. A fresh bind renders the captured self-targeting template (selectors 1/2, zero targeting
   parameter, zero mana/refresh, never-cast sentinel) with `serverEnabled = true`, because the
   captured reference slot is disabled and the original assignment's enable policy is not recovered.
4. Quantity comes from the live stack, never from a cached value, and never exceeds i32.
5. The module only reads the caller's inventory projection. It never grants, consumes, moves or
   mutates an item, coin, quest or save field, and it invents no effect, cooldown or balance number.

## Contradictions and gaps

* The community handler writes `Unknown7 = 15` as an integer where the native reader treats offset 21
  as a float (the community's bytes are a 1.5e-44 denormal); the module writes the captured `0.0`.
* The community uses `Unknown6 = 4` for self while the capture uses 2; the recovered native cast path
  forces self for selector 1 with target 2 **or** 4.
* The 42/9 serialized record order is unrecovered; the module refuses it and records the observed
  length for the next recovery attempt.
* `Unknown10`/`Unknown12`/`Unknown15` have no recovered meaning beyond the sentinel branch, and the
  client's `RefreshTimeLeft` column is not mapped to a wire field yet.
* Four slots come from the capture, not from a native limit; the bound is server policy.
* No native acceptance: no real client input was sent.

## Verification

```powershell
# RED (module absent), then GREEN: 16 tests
& .\scripts\cargo.ps1 test --test piece_048 --manifest-path .\prototype\osfr-tools\Cargo.toml

# RED (checker absent, 11 errors), then GREEN: 11 tests
& '..\lab\tools\wiki-env\Scripts\python.exe' tests/pieces/test_piece_048.py -v

# 30 checks, 0 failing, 6 gaps, 3 contradictions
& '..\lab\tools\wiki-env\Scripts\python.exe' scripts/pieces/piece_048.py `
  --output ..\lab\reports\piece-048-20261009T0442Z\piece-048-evidence.json
```

A negative control mutated one captured quantity byte and both the Rust vector test and the checker's
`module.captured_vectors_equal_the_rendered_capture` check failed as expected
(`negative-control-*.log`), then the mutation was reverted.

## Integration boundary for the lead

Wire `piece_048::apply_parse` into the existing inventory dispatch in `src/gameplay.rs` where family
42 is currently handed to `inventory.handle` (which intentionally leaves the toolbar outside its
layer), keep the `ActionBarState` in the version-2 snapshot behind `#[serde(default)]`, implement
`ActionBarItems` over the live inventory, and forward the returned reply bytes through the existing
`Outcome.replies`. Combat/ability toolbars stay on the recovered 36/5 set-definition path and are not
touched by this piece.
