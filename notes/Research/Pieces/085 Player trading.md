# 085 Player trading

Status: bounded recovery + reconstruction returned for lead review, 2026-10-09 05:39 UTC. Queue
acceptance was "recover offer/accept/cancel wire; atomic two-inventory transfer with disconnect and
stale offer checks". Delivered: the recovered trade opcode family and payload shapes from the
preserved native client, a checker that cross-checks them against the pinned community packet table,
and a bounded two-sided trade session with atomic transfer, disconnect handling and stale-offer
refusal. It is **not** a playability claim: no native trade was run, no capture frame was decoded,
and nothing here grants items or currency. Commands below ran from
`%USERPROFILE%\Documents\Projects\CartoonMMO\project`.

## Owned artifacts

| Path | What |
| --- | --- |
| `prototype/osfr-tools/src/restoration/piece_085.rs` | Recovered trade opcode table + wire codec, and the reconstructed two-sided session. Not registered in `src/lib.rs` (lead-owned). |
| `prototype/osfr-tools/tests/piece_085.rs` | 13 tests; `#[path]`-includes the module so it compiles before registration. |
| `scripts/pieces/piece_085.py` | Read-only checker over the preserved native pseudocode and pinned community packet table; writes the private lab bundle. |
| `tests/pieces/test_piece_085.py` | 12 stdlib tests: synthetic rejection paths plus hash/shape pins on the real preserved files. |
| `../lab/reports/piece-085-20261009T053901Z/` | Private evidence: `trading-wire.json`, `findings.json`, `provenance.json`. |

## Recovered originals

**Base opcode.** `lab/decompiled/client/FreeRealms/pseudocode.c` (SHA-256 `f84a5867…`, 90.8 MB)
`FUN_00c6a7b0` (line 1732118) installs `BasePacket::vftable`, writes `param_1[1] = 0x3b`, swaps in
`BaseTradePacket::vftable` and stores the sub-opcode in `param_1[2]`. Every trade packet therefore
travels under SOE opcode **59 (0x3b)** with a one-byte trade sub-opcode. The pinned community table
`lab/references/OpenSourceFreeRealms/ofrserver/Server/LibSOE/Core/PacketCodes.cs` (SHA-256
`f3f12876…`) lists `BaseTradePacket = 59` and the same 18 rows.

**Sub-opcode table (18 rows).** Recovered from the `FUN_00c6a7b0(n)` constructor family in the same
binary and confirmed row-for-row by the pinned community enum: `TradeInvitePacket` 2,
`TradeInviteReplyPacket` 3, `TradeStartSessionPacket` 4, `TradeEndSessionPacket` 5,
`TradeAcceptPacket` 6, `TradeCancelPacket` 7, `TradeLockPacket` 8, `TradeOfferItemPacket` 9,
`TradeCoinCountPacket` 10, `TradeUpdateCoinCountPacket` 11, `TradeUpdateAddItemPacket` 12,
`TradeUpdateItemCountPacket` 13, `TradeUpdateAcceptPacket` 14, `TradeMessagePacket` 15,
`TradeRejectedPacket` 16, `TradeUpdateLockPacket` 17, `TradeConfirmPacket` 18,
`TradeItemRemovedPacket` 19. The checker asserts built-in expectations, the native table and the
community table are all identical (0 mismatches on either side).

**Payload shapes with native reader evidence.** Three readers pin field order, session id first:
`FUN_00c6c110` (line 1733311) reads `u32` at +0xc then a bool at +0x10 and is wrapped by
`FUN_00c6c8f0` (line 1733716) for the `TradeUpdateLockPacket` dispatch, where the +0xc value is
compared against the active session id (`local_138 == param_1[1]`, line 1735082);
`FUN_00c6c180` (line 1733338) reads three `u32`s and is wrapped by `FUN_00c6c960` (line 1733746) for
`TradeItemRemovedPacket`, also comparing the first field against the active session id
(line 1735122); `FUN_00c6ce30` (line 1734007) reads `u32`, `u32`, bool and belongs to
`TradeUpdateAddItemPacket`. Constructor arity pins the remaining classes: offer item three fields,
coin count two `u32`, cancel/end-session/confirm one field, invite-reply `u32 + bool`,
accept/lock two fields, update-coin-count/update-add-item mixed.

**Client event vocabulary.** `TradeHandler` in the same binary emits `TradeCompleted` (line 1734992),
`TradeCanceledPartnerDisconnected` (1734968), `TradeCanceledPartnerOfferInvalid` (1734974),
`TradeCanceledYourOfferInvalid` (1734982), `TradeCanceledPartnerEnteredInstancedZone` (1735026),
`TradeCanceledYourOfferExceedsPartnersMaxStack` (1735044),
`TradeCanceledPartnersOfferExceedsYourMaxStack` (1735052) and shows
`TradeHandler:OtherAccepted` when the partner's lock frame flips on (1735089). That vocabulary is the
evidence behind the reconstructed cancel/disconnect/stale-offer policy.

**Community search result (negative).** The pinned wiki corpus
`lab/references/freerealms-wiki/wiki.sqlite` has **no player-trading page**; the only title matches
are "Free Realms Treasure Trader" and "Treasure Trader" (NPCs). No community prose documents the
player flow, so the wire is the only first-party evidence for it.

## Reconstructed (not historical)

`piece_085.rs` labels these at every call site: the two-sided session machine (offer -> lock ->
confirm -> complete), the offer-revision stale-lock rule (any offer change bumps the revision and
makes earlier locks/confirmations stale with `TradeError::OfferChanged`), the whole-or-nothing
transfer plan (both inventories are cloned, validated and only then replaced, so a bad offer on
either side moves nothing), and the disconnect policy (either side dropping closes the session with
`TradeClose::PartnerDisconnected` and no transfer). Field order for `TradeOfferItemPacket` is
marked inferred from the sibling readers; the second field of `TradeAcceptPacket`/`TradeLockPacket`
is carried as a slot but not named.

## Verification

```
& scripts/cargo.ps1 test --test piece_085      # run from prototype/osfr-tools
# RED before implementation: test result: FAILED. 7 passed; 6 failed
#   a_lock_frame_for_another_session_is_stale ... FAILED
#   a_replayed_complete_does_not_transfer_twice ... FAILED
#   cancel_and_disconnect_close_the_session_without_moving_items ... FAILED
#   changing_an_offer_after_lock_invalidates_the_lock ... FAILED
#   confirm_before_both_sides_lock_is_refused ... FAILED
#   session_commits_both_offers_atomically_on_confirm ... FAILED
# GREEN: test result: ok. 13 passed; 0 failed; 0 ignored

& '..\lab\tools\wiki-env\Scripts\python.exe' -m unittest discover -s tests/pieces -p "test_piece_085.py" -v
# RED during authoring: 2 failures from regexes leaking past a function boundary
#   (field slots and reader shapes picked up the next Ghidra function)
# GREEN: Ran 12 tests ... OK

& '..\lab\tools\wiki-env\Scripts\python.exe' 'scripts\pieces\piece_085.py'
# {"base_opcode": 59, "community_mismatches": [], "native_mismatches": [],
#  "rows": 18, "reports_dir": "...\piece-085-20261009T053901Z",
#  "findings": ["packet-field-order-partly-unrecovered",
#  "packet-field-order-partly-unrecovered", "invite-partner-name-region-unrecovered",
#  "offer-item-field-order-inferred", "wiki-has-no-player-trading-page"]}
```

## Gaps and remaining gates

* **No capture/decrypt.** The pinned community repo ships raw `.pcap/.pcapng` sessions
  (`ofrserver/Packets/`), but SOE game frames are not decoded here; the wire rests on constructors,
  readers and the community enum, not on a literal captured trade. That is the largest gap.
* Unrecovered field detail: the partner-name region of `TradeInvitePacket` (native offsets
  0x26/0x27/0x28), the second field of `TradeAcceptPacket`/`TradeLockPacket`, the
  `TradeUpdateCoinCountPacket` field mix, and the item-limits rule (the max-stack cancellation
  strings imply one, but no numeric limit is recovered).
* Item eligibility (bound/NoTrade flags) is not checked here: the native client has a `NoTrade`
  and `ItemNoTrade` path (`pseudocode.c` lines 1347882, 1843558) that this piece does not decode.
* Integration: the module is unregistered; the lead owns `src/lib.rs`. The transfer must run inside
  the lead-owned atomic inventory/economy layer (pieces 081/086) before any grant is real.
* Native acceptance is lead-controlled (shared client window, and trade needs two clients —
  piece 099 owns the multi-client harness). `code/test-candidate` is the strongest status this
  piece supports.

## Skills applied

`reverse-engineering` for the native constructor family, the three field readers, the session-id
comparison and the `TradeHandler` event vocabulary; `game-research-websearch` for the pinned
community repositories and the read-only wiki corpus (negative result recorded); `game-recon` to
keep the SOE/ForgeLight wire separate from UI/Scaleform data. `asset-pipeline` was not needed (no
asset output) and `game-automation` was not used because native testing is lead-controlled. RTK
(normal commands first), Ponytail (reused the existing hash/checker conventions and stdlib only, no
new dependencies) and Caveman (concise reporting, full detail persisted here) were followed.

Cross-links: [[../../MMO Work Queue]], [[081 Inventory instance lifecycle]], [[083 Shops and purchases]],
[[084 Selling and buyback]], [[086 Currency and economy audit]], [[099 Multi-client integration harness]],
[[../Free Realms Recovery]].
