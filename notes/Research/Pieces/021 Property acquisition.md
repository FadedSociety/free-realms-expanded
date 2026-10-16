# 021 Property acquisition

Updated 2026-10-09 (UTC). Scope: work item 021 only. Status: recovered property definitions, deed
items and the recorded purchase/grant policy, with a bounded acquisition engine and a
drift-detecting checker. No native client run was made, so nothing here is a playable or
native-accepted claim.

Owned paths: `prototype/osfr-tools/src/restoration/piece_021.rs`,
`prototype/osfr-tools/tests/piece_021.rs`, `scripts/pieces/piece_021.py`,
`tests/pieces/test_piece_021.py`, this note and `lab/reports/piece-021-20261009T025932Z/`. No shared
file, manifest, gateway, content, save or release path was modified.

## What is recovered

### Property rows (ORIGINAL client content)

`lab/references/Sanctuary/src/Resources/Houses.json`, sha256
`854b2d3f009b916a928f66d57a42e2bf685dcc853e898d149580eefd26196d3c`, holds **31** property rows:
`Id`, `NameId`, `ZoneId`, `Icon`, `SpawnPosition`/`SpawnRotation` and one or more `BuildAreas`
min/max boxes. `Id` and `ZoneId` are 1:1 in every row, and rows 11-13 are absent. Three rows are
test/unreleased entries by name (`Castle Test Lot`, `Chromakey Zone`, `Snow Days Lot`); six rows
have no deed item at all (`Castle Test Lot`, `Chromakey Zone`, `Snow Days Lot`, `Snowhill Lodge`,
`Sunstone Valley Lot`, `Sunstone Dam`).

### Property deeds (ORIGINAL client content)

The client item definitions (`ClientItemDefinitions.json`, sha256
`ffe9cd2dfe54e8bd825c299a5e0bf08da52533c7ca9b9d3036c15593310d0480`) carry **36** `Type == 16`
items. Every one of them is a property: house, lot, farm, condo, apartment or party boat. The
community packet model special-cases the same type (`Sanctuary.Packet.Common/BaseItemDefinition.cs:145`
writes `ForceDisablePreview || Type == 16`).

The join between deeds and rows is `NameId`: **25 house rows** carry **26** deeds (row 21
`Wugachug Lot` has two, 10999 and 17185), and **10 deeds have no house row in the recovered
corpus** - `Large/Small/Large Briarwood House` variants, `Sandy Beach Lot` (twice), `Snowhill
Lodge`, `Merry Vale Lot`, `Sanctuary Lot`, `Wilds Rapids Lot` and `Lake Tree Lot`. Those lots have
deeds, wiki pages and prices but no `Houses.json` row, so their zone, build area and spawn are
**unrecovered gaps**, not inventions. `Snowhill Lodge` is the mirror case: a house row and a
950/855 price with **no** deed item in the dump.

The deed rows also carry raw `Cost` values (10 for `Rumbledome Lot`, 50 for `Sunset Party Boat`,
500 for `Club House`, 570 for `Wilds Farm`) and `NoSale` flags. No deed item id appears in
`CoinStoreItems.json` (sha256 `9e21fa5fc61b6db803a106c058863f7ac7bf8118ecaea1a1a0e0a68b76e74820`),
so the coin `Cost` field is not a recovered lot price and the reconstructed policy never charges
coins for a property.

### Grant and purchase packets (ORIGINAL client)

The housing family is base opcode **127** with inner opcodes; members 23/49/46/14 are confirmed in
the Ghidra export of the supplied client:

| Inner | Packet | Direction | Construction in `lab/decompiled/client/FreeRealms/pseudocode.c` |
| --- | --- | --- | --- |
| 23 | `ClientHousingPacketRequestGrant` | client to server | `FUN_00aa4fd0`, `FUN_00aa4990(0x17)`; sent once per UI state from `FUN_00ac7af0` behind the `+0x2880` guard |
| 49 | `HousingPacketNotifyHouseAdded` | server to client | `FUN_00aa86d0`, `FUN_00aa4990(0x31)` |
| 46 | `HousingPacketHousingError` | server to client | `FUN_00aa56b0`, `FUN_00aa4990(0x2e)` |
| 14 | `ClientHousingPacketRequestPlayerHouses` | client to server | `FUN_00ab3f00`, `FUN_00aa4990(0xe)` |

The client's own handler for a newly added house prints the `HousingNotifyNewHouse` key with the
fallback text **"You have received a new %s."** (`pseudocode.c:1400165`, `:1400185`), the visit
picker prints `HousingServerNoUnlockedHouses` as **"That player does not have any houses you can
visit at this time."** (`:1407515`, `:1407517`) - houses are therefore unlocked per character on
the server - and house renaming reports `CheckNameResponse.MissingHouse` (`:1282422`).
`ClientHousingPacketRequestGrant` initialises no fields beyond its opcode word, so the recovered
request is payloadless; the fields of `HousingPacketNotifyHouseAdded` (five words initialised in
the constructor) are **not** decoded.

### Prices and grant channels (ORIGINAL wiki record)

Lot pages in `lab/references/freerealms-wiki/wiki.sqlite` (sha256
`b9d7b9f1adabb42db54c3253bf43fddc249d6d70e3f2075b134a9441e2fcd911`) state the purchase channel in
their raw wikitext; the extractor reads the `{{SC|n}}` template from that raw text rather than a
derived field. `Seaside Lot` revision 49520: "available to everyone through the
[[Station Cash Marketplace|SC Marketplace]] at the regular price of {{SC|300}}, and to [[Membership|Members]]
at the 10% discount price of {{SC|270}}". **19** house rows have such a pair:

| Id | Property | Deed item(s) | Regular SC | Member SC | Wiki rev |
| --- | --- | --- | --- | --- | --- |
| 1 | Blackspore Swamp Lot | 17567 | 300 | 270 | 49557 |
| 3 | Bog Shore Lot | 17192 | 400 | 360 | 49508 |
| 4 | Briar Falls Lot | 17193 | 500 | 450 | 49509 |
| 5 | Briarwood Lot | 7485 | 300 | 270 | 49511 |
| 6 | Briarwall Towers Lot | 17189 | 500 | 450 | 49510 |
| 8 | Shrouded Stairs Lot | 10338 | 400 | 360 | 49523 |
| 9 | Shrouded Glade Lot | 17182 | 300 | 270 | 49521 |
| 10 | Lonely Island Lot | 10386 | 499 | 449 | 49516 |
| 14 | Seaside Lot | 10384 | 300 | 270 | 49520 |
| 16 | Crystal Mines Lot | 17191 | 400 | 360 | 49559 |
| 19 | Snowhill Lot | 7486 | 300 | 270 | 47062 |
| 20 | Vale Stream Lot | 17194 | 400 | 360 | 49530 |
| 21 | Wugachug Lot | 10999, 17185 | 300 | 270 | 49532 |
| 22 | Blackspore Swamp House | 11110 | 950 | 855 | 49620 |
| 26 | Large Wilds House | 2214 | 750 | 675 | 49515 |
| 27 | Small Wilds House | 2213 | 549 | 494 | 49611 |
| 28 | Club House | 11140 | 1499 | 1349 | 49512 |
| 29 | Snowhill Lodge | none recovered | 950 | 855 | 49526 |
| 32 | Sunstone Dam | none recovered | 1499 | 1349 | 49528 |

Every recorded pair is the floor of a 10% member discount (`300 -> 270`, `499 -> 449`,
`549 -> 494`, `1499 -> 1349`), which matches the wiki sentence, the item `MemberDiscount: 10`
field and the membership benefit "10% discount on SC and Coin items" (`Membership`, rev 48146).

Four free/grant channels are recorded, each pinned to a revision and quoted by the extractor:

| Property | Channel | Evidence |
| --- | --- | --- |
| Apartment | starter grant | "formerly awarded to every new player"; on 2012-01-26 the Wilds Condo replaced it and "Players who already had an Apartment before this date kept their Apartment and received a new Wilds Condo" (rev 49504) |
| Wilds Condo | starter grant + bonus grant | "awarded to every new player, as of January 26, 2012" and the same migration sentence (rev 49531) |
| Small Wilds House | membership claim | "can also be acquired through the [[Members Only New House Package]], which is awarded to players upon first obtaining Membership" (rev 49611); the deed item is `MembersOnly: true` |
| Sunset Party Boat | bonus grant | "included in February 2014's Monthly Membership Pack" (rev 49527) |
| Rumbledome Lot | bonus grant | "acquired through redeeming the Trading Card Game Series 5 Virtual Reward loot card" (rev 49518) |

Ownership is instanced per owner, not one owner per lot globally: the pinned community server keys
zones by `(zoneDefinitionId, ownerId)` and creates a housing instance per owner
(`Sanctuary.Game/ZoneManager.cs:160`, `:162`, `:189`), and the Small Wilds House is a repeated
membership benefit rather than a unique object. Its ownership table is still a TODO there -
`PlayerOwnsHouse` throws `NotImplementedException` with a commented-out
`Houses.SingleOrDefault(house => house.CharacterId == ownerId && house.ZoneDefinitionId ==
zoneDefinitionId)` lookup (`ZoneManager.cs:249`, `:253`).

## Reconstructed policy (not original)

Everything in this section is invented, bounded behaviour. No original server code for a property
purchase was recovered.

- Channel names `starter_grant`, `marketplace`, `membership_claim` and `bonus_grant`; a property with
  no recovered channel (`Castle Test Lot`, `Bat Cavern Lot`, `Shrouded Gloam Lot`, `Snow Days Lot`,
  `Chromakey Zone`, `Sunstone Valley Lot`, `Briarwood Farm`, `Wilds Farm`) stays known and refuses
  every channel instead of inventing one.
- Once-only ownership per (character, property): a second acquisition of the same property is
  refused with `AlreadyOwned` and never charges again.
- One starter home per character. The 2012-01-26 Wilds Condo migration is modelled as an explicit
  `BonusGrant`, because the wiki records it as a separate event rather than a second starter grant.
- Station Cash accounting: the marketplace charges `member_price` for members and `regular` for
  everyone else; a claim, grant or migration is free. Insufficient funds refuse the acquisition and
  leave ownership, the starter flag and the balance untouched. Every successful charge returns a
  `StationCashDebit` proposal for the lead-owned atomic economy layer; this module keeps only a
  working copy of the balance and never mints currency or items.
- Check order: owner, known property, already owned, channel allowed, membership, starter grant
  already used, funds, then commit. Every check runs before the first mutation.

## Verification

All commands ran from `%USERPROFILE%\Documents\Projects\CartoonMMO\project` on 2026-10-09 and are
kept in `lab/reports/piece-021-20261009T025932Z/`.

| Command | Result |
| --- | --- |
| `scripts/cargo.ps1 test --manifest-path prototype/osfr-tools/Cargo.toml --test piece_021` | RED first (`couldn't find file tests\..\src\restoration\piece_021.rs`), then 13 passed, 0 failed, 1 ignored |
| `scripts/cargo.ps1 -CargoArgs @('test',...,'--test','piece_021','real_recovered_catalogue_loads_and_charges_the_recorded_prices','--','--ignored','--exact')` with `OSFR_PIECE_021_CATALOGUE` set | 1 passed: the real 31-row catalogue loads and charges the recorded 270 Station Cash member price |
| `wiki-env\Scripts\python.exe -m unittest discover -s tests/pieces -p "test_piece_021.py" -v` | RED first (script missing), then 13 passed |
| `wiki-env\Scripts\python.exe scripts/pieces/piece_021.py --out lab/reports/piece-021-20261009T025932Z` | 31 properties, 19 priced, 10 deeds without a property row |
| `wiki-env\Scripts\python.exe scripts/pieces/piece_021.py --check <report>\property-acquisition.json` | exit 0, no problems |
| `scripts/cargo.ps1 test --manifest-path prototype/osfr-tools/Cargo.toml --test housing --test inventory --test game_store --test character_store --test piece_021` | 26 + 12 (+2 ignored) + 1 + 8 + 13 (+1 ignored) passed, 0 failed |

Two real defects were found by these checks rather than by inspection: the test's first member-price
vector mutated the wrong catalogue row (it hit the unpriced `Bat Cavern Lot`, which surfaced as an
unreadable catalogue instead of the discount rule), and the first price count written into the
Python test (21) was wrong - the corpus records 19 priced rows. Corpus drift is covered by the
negative path recorded in `negative-path.txt`: a moved wiki revision or a page that no longer
contains the quoted sentence makes the extractor fail instead of emitting a stale claim.

## Gaps

- `ClientHousingPacketRequestGrant` (23) and `HousingPacketNotifyHouseAdded` (49) payload bodies are
  not decoded; the grant request looks payloadless in its constructor, so the server must decide
  which property is granted from state it owns. `HousingPacketHousingError` (46) has no recovered
  body either.
- No original server code for a purchase, grant or refund was recovered. `ClientHousingPacketPayUpkeep`
  (7) exists in the client and the community model but has no recovered payload, so upkeep billing,
  discounts on upkeep and any rent are **unmodelled**.
- `StoreBundles.json` carries lot-shaped bundles (`Robgoblin Mobile Apartment`, id 4524) but its
  `Price`/`AltCurrencyPrice` fields are scrubbed to `1` in the dump, so the marketplace price could
  not be cross-checked there; the wiki record is the only recovered price source.
- `Param1`/`Param2` on the `Type == 16` items are still unexplained. They are not the matched row's
  `Id` or `ZoneId` (checked for all 36 deeds), and they are not house display indices either.
- 10 deeds have no property row and 1 property row (`Snowhill Lodge`) has no deed. The missing
  `Houses.json` rows are a content gap the lead must fill from the client's own zone/build data, not
  from this note.

## Integration gates

1. Lead registration: `src/restoration/piece_021.rs` is not in `src/lib.rs`; the test includes it by
   `#[path]` and the lead drops that include when registering the module.
2. Wire gate: mapping `ClientHousingPacketRequestGrant` (23) and `HousingPacketNotifyHouseAdded` (49)
   needs the payload layouts; only then can a purchase answer the native client.
3. Economy gate: `StationCashDebit` must be committed by the lead-owned atomic economy layer together
   with the ownership row; this module's local balance is a working copy.
4. Housing gate: `src/housing.rs` currently refuses a second owner of the same property id, while the
   recovered model is one housing instance per `(zoneDefinitionId, ownerId)`. The lead must widen that
   grant rule before item 022/030 build on it; this item deliberately did not edit `housing.rs`.
5. Native gate: no client run was made, so "the player buys a lot and the client shows the new house"
   is **unproven**.

Cross-links: [[Housing Restoration]], [[MMO Worker Contract]], [[Rust Revival Roadmap]].
