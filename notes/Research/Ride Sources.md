---
type: research
updated: 2026-10-16
---

# Ride sources (W169)

How the original game granted each of the 54 seeded rides, and how our server now lets a player obtain every one. Content source rule: wiki is allowed, client evidence overrides it; reconstructed choices are marked (R). Companion notes: [[Mounts Service]] (the service), [[Item Catalogue]] (why all 54 ride items exist at runtime, W166).

## Result

Before: 4 of 54 rides had a `stores.json` row (3 with no price), 0 were in any vendor's stock, so nobody could own most of them. Now all 54 have a priced `stores.json` row for one canonical item and are in the stock of the Seaside vendor Delvin (`vendors.json`); 3 are also in the stock of the Seaside "Mysterious Merchant" Cragara, matching a real original merchant channel. Buying goes through the existing `buy_vendor` path, grants the item, and `MountCatalog::owned_mounts` then lists the ride. No payment flow was added (`orders_enabled` and `payment_sources_enabled` stay false).

## How the original gave rides (evidence)

| Channel | Evidence | Rides in our 54 |
|---|---|---|
| Station Cash Marketplace, "Rides" categories Winged, Mechanical, Exotic, Non-Winged | Wiki *Rides* ("Rides are mainly sold in the SC Marketplace"; nearly every ride page gives an SC price); Legacy `StoreBundles.json` has a bundle per ride in category groups 266 to 269 (reference); client locale 427359 "How can I buy a ride?" and 427368 "After ya buy yerself a ride, just click that stable icon" (client) | 54 have a bundle row; 52 are sold for SC (Chrome Drake and Polar Bear are not, see below) |
| In-world "stables" | Client locale 5612 and 431939 "Visit the Marketplace or head to one of the in-world stables to check out the T-Rex and Wyvern rides!", NPC titles "Stable Guide" (427444) and "Stable Hand" (427445) (client). No stable NPC exists in our NPC content. | T-Rex rides named; stock unknown |
| Mysterious Merchant (merchant-only bundles, category group 10001, 2012-12-03) | Legacy `StoreBundles.json` 50044 Dwarftech Chopper, 50053 Pogo Stick, 50057 Robgoblin Mech (reference); NPC Cragara, "Mysterious Merchant, random location" (NPC content) | 3 |
| Seasonal events (same Marketplace) | Wiki: Surfboard and Wooden Surfboard "during the summer event", Eyeball "during the Super Spooktacular" (in game 2013-10-22), Holiday Tree Board "during the Snow Days" plus a rare drop from the Holiday Gifting Tree | 4 |
| Trading Card Game virtual reward | Wiki *Chrome Drake*: redeem the series 5 loot card "Chrome Feathered Drake" (card id 5L1) | 1 |
| Quest or gift event | Wiki *Polar Bear*: 12 Days of Presents 2012 quests; wiki *Rides*: every player by collecting all Snow Days gift boxes | 1 |
| V.I.P. gate | Wiki *Sun*: the Marketplace purchase also needs V.I.P. rank 2 | 1 |
| Membership | Wiki: "Since July 1st, 2010, Membership is required for rides", but an owned ride stays usable after membership lapses; Legacy item `MembersOnly` is set on most canonical items (reference) | rule for all |
| Mystery Chests, Chest of Wonders, promo cards, SOE events, 7-11 cards, TCG questlines | Wiki *Rides* list | none of our 54 (these are rides outside the seed, for example Kangaroo, Clocktopus, Magic Carpet, Slobbersaurus-Rex) |

The client locale "Obtain a mount." achievement ladder (441400) rewards owning rides; it is not a source.

Prices: the wiki gives the Station Cash price on 48 of the 54 rides (45 regular, 3 as a member price). Legacy item `Cost` of 500 on the 2013 batch (ids 79438 to 79460: Chrome Drake, Polar Bear, Eyeball, Dwarftech Chopper, Holiday Tree Board, and Sun at 17180) is a placeholder: the Polar Bear and Chrome Drake are not sold for SC at all, and Eyeball's wiki price is 149. The matching `StoreBundles.json` rows for that batch carry `Price: 500` and a bulk launch date of 2011-04-06, so bundle dates in that batch are not release dates.

## Our sources (R)

- **Price rule.** coins = round(SC price x 20, to 50). `stores.json` carries a top-level `ride_price_policy` (`coins_per_sc` 20, `round_to_coins` 50, `min_coins` 2,000, `max_coins` 30,000, member discount 10 percent). Prices run from 3,000 (Eyeball) to 19,000 coins (Robgoblin Mech); the whole set costs 559,000 coins. Every price is its own row and is tunable by editing it; `lab/runs/claude/W169/apply_ride_sources.py` regenerates them from the table and refuses a price outside the bounds. The member price is the existing 10 percent floor discount.
- **Item bought.** The canonical item is the candidate definition whose `Param1` equals the mount id. Name ids are one to many (older per-tint ride items such as 7960 to 7965 carry other `Param1` values and are not sold); Lawn Chair has three canonical items (79410, 79411, 79412) and 79410 is sold.
- **Vendors.** Delvin (Seaside surfboard seller, NPC 9500000572, placed, role vendor) holds all 54; this is the reconstructed stable. Troy Ridewrangler is the NPC at "the ride stable in Seaside" (client text) but has no vendor role, so he was not used.
- **Cragara** (9500000569) holds Dwarftech Chopper, Pogo Stick and Robgoblin Mech (reference evidence).
- `vendors_unresolved.json` lists the same ids as resolved for both NPCs; the `coverage` counters in `vendors.json` were raised to match (`items_resolved` and `items_with_stores_price` +57).

## All 54 rides

Columns: SC = original Station Cash price; the letter after it is the price basis (WM the wiki shows the member price and the regular price is inferred as price/0.9; E category median estimate (R); L Legacy item Cost, no wiki price). Coins and Member are our (R) prices. "Item" is the canonical item id.

| Ride (id) | Item | Original source (evidence) | SC | Our source (R) | Coins | Member |
|---|---|---|---:|---|---:|---:|
| Skeletal Dragon (715) | 10530 | SC Marketplace, Winged (bundle 4833, 2010-12-15) | 650 WM | Delvin | 13,000 | 11,700 |
| Scaled Dragon (12672) | 76856 | SC Marketplace, Winged (bundle 4832, 2010-12-15) | 549 | Delvin | 11,000 | 9,900 |
| Poison Dragon (12792) | 76905 | SC Marketplace, Winged (bundle 4888, 2011-03-29) | 550 | Delvin | 11,000 | 9,900 |
| Gold-Tipped Dragon (12671) | 76855 | SC Marketplace, Winged (bundle 4826, 2010-12-15) | 550 WM | Delvin | 11,000 | 9,900 |
| Feathered Drake (13291) | 38799 | SC Marketplace, Winged (bundle 5378, 2012-09-13) | 549 E | Delvin | 11,000 | 9,900 |
| Bat (992) | 14288 | SC Marketplace, Winged (bundle 4988, 2011-10-19) | 349 | Delvin | 7,000 | 6,300 |
| Pegasus (12601) | 76768 | SC Marketplace, Winged (bundle 4844, 2010-12-20) | 550 | Delvin | 11,000 | 9,900 |
| Fire Pegasus (12915) | 78069 | SC Marketplace, Winged (bundle 5240, 2012-04-05) | 549 | Delvin | 11,000 | 9,900 |
| Hearts Pegasus (12678) | 76870 | SC Marketplace, Winged (bundle 4827, 2010-12-15) | 550 WM | Delvin | 11,000 | 9,900 |
| Pegasus Unicorn (12679) | 76871 | SC Marketplace, Winged (bundle 4830, 2010-12-15) | 549 | Delvin | 11,000 | 9,900 |
| Tiger (12682) | 76863 | SC Marketplace, Non-Winged (bundle 4839, 2010-12-15) | 549 | Delvin | 11,000 | 9,900 |
| Lava Tiger (12688) | 76875 | SC Marketplace, Non-Winged (bundle 4828, 2010-12-15) | 550 E | Delvin | 11,000 | 9,900 |
| Saber-Tooth Tiger (12686) | 76864 | SC Marketplace, Non-Winged (bundle 4831, 2010-12-15) | 550 | Delvin | 11,000 | 9,900 |
| Stone Saber-Tooth Tiger (12687) | 76873 | SC Marketplace, Non-Winged (bundle 4836, 2010-12-15) | 550 | Delvin | 11,000 | 9,900 |
| Wolf (993) | 14289 | SC Marketplace, Non-Winged (bundle 4989, 2011-10-19) | 549 | Delvin | 11,000 | 9,900 |
| Unicorn (12680) | 76872 | SC Marketplace, Non-Winged (bundle 4840, 2010-12-15) | 549 | Delvin | 11,000 | 9,900 |
| Nightmare (12677) | 76869 | SC Marketplace, Non-Winged (bundle 4829, 2010-12-15) | 550 | Delvin | 11,000 | 9,900 |
| White-Nosed Stallion (12676) | 76868 | SC Marketplace, Non-Winged (bundle 4842, 2010-12-15) | 500 | Delvin | 10,000 | 9,000 |
| Black-Nosed Stallion (12674) | 76866 | SC Marketplace, Non-Winged (bundle 4823, 2010-12-15) | 450 | Delvin | 9,000 | 8,100 |
| Stallion (12675) | 76867 | SC Marketplace, Non-Winged (bundle 4835, 2010-12-15) | 450 E | Delvin | 9,000 | 8,100 |
| White-Nosed Mare (12673) | 76865 | SC Marketplace, Non-Winged (bundle 4841, 2010-12-15) | 450 | Delvin | 9,000 | 8,100 |
| Armored T-Rex (12689) | 76859 | SC Marketplace, Exotic (bundle 4820, 2010-12-15) | 549 | Delvin | 11,000 | 9,900 |
| Spotted T-Rex (12670) | 76854 | SC Marketplace, Exotic (bundle 4834, 2010-12-15) | 449 | Delvin | 9,000 | 8,100 |
| Striped T-Rex (12669) | 76853 | SC Marketplace, Exotic (bundle 4837, 2010-12-15) | 450 | Delvin | 9,000 | 8,100 |
| Sun (12637) | 17180 | SC Marketplace, Exotic (bundle 4857, 2011-01-10); V.I.P. rank 2 (wiki) | 350 | Delvin | 7,000 | 6,300 |
| Air T-Rex (757) | 11317 | SC Marketplace, Exotic (bundle 4862, 2011-01-14) | 550 | Delvin | 11,000 | 9,900 |
| Bowling Ball (12626) | 17197 | SC Marketplace, Exotic (bundle 4716, 2011-01-12) | 350 | Delvin | 7,000 | 6,300 |
| Eight Ball (713) | 10499 | SC Marketplace, Exotic (bundle 4723, 2010-11-24) | 349 | Delvin | 7,000 | 6,300 |
| Hydra Ride (738) | 76759 | SC Marketplace, Exotic (bundle 4858, 2011-01-10) | 699 | Delvin | 14,000 | 12,600 |
| The Hero Copter (13456) | 78394 | SC Marketplace, Mechanical (bundle 5617, 2013-07-16) | 499 | Delvin | 10,000 | 9,000 |
| The Cyber Cycle (13457) | 78395 | SC Marketplace, Mechanical (bundle 5618, 2013-06-18) | 499 | Delvin | 10,000 | 9,000 |
| The Blast Rover (13458) | 78396 | SC Marketplace, Mechanical (bundle 5619, 2013-07-31) | 499 | Delvin | 10,000 | 9,000 |
| The Fire Hawk (13459) | 78397 | SC Marketplace, Mechanical (bundle 5620, 2013-10-22) | 499 | Delvin | 10,000 | 9,000 |
| The Hover Ring (13421) | 78332 | SC Marketplace, Mechanical (bundle 100017, 2013-05-14) | 499 | Delvin | 10,000 | 9,000 |
| Bicycle (839) | 76903 | SC Marketplace, Mechanical (bundle 4907, 2011-04-06) | 549 | Delvin | 11,000 | 9,900 |
| Robgoblin Mech (1254) | 17598 | SC Marketplace, Mechanical (bundle 5376, 2012-09-13); Mysterious Merchant (merchant-only bundle) | 950 | Delvin + Cragara | 19,000 | 17,100 |
| Tractor (12907) | 78036 | SC Marketplace, Mechanical (bundle 4950, 2011-06-01) | 549 E | Delvin | 11,000 | 9,900 |
| Molten Precursor Hoverboard (8212) | 7976 | SC Marketplace, Mechanical (bundle 4329, 2010-05-17) | 550 | Delvin | 11,000 | 9,900 |
| Fiery Hoverboard (698) | 10382 | SC Marketplace, Mechanical (bundle 4562, 2010-07-12) | 550 | Delvin | 11,000 | 9,900 |
| Precursor Hoverboard (8211) | 7975 | SC Marketplace, Mechanical (bundle 4328, 2010-05-17) | 549 | Delvin | 11,000 | 9,900 |
| Clockwork Hoverboard (12668) | 76858 | SC Marketplace, Mechanical (bundle 4824, 2010-12-15) | 550 | Delvin | 11,000 | 9,900 |
| Dwarftech Hoverboard (12667) | 76857 | SC Marketplace, Mechanical (bundle 4825, 2010-12-15) | 550 | Delvin | 11,000 | 9,900 |
| Lawn Chair (13460) | 79410 | SC Marketplace, Mechanical (bundle 5603, 2010-12-15) | 399 | Delvin | 8,000 | 7,200 |
| Chrome Drake (13583) | 79454 | Trading Card Game series 5 virtual reward card (wiki); SC Marketplace, Mechanical (bundle 452525, 2011-01-10) placeholder | 500 L | Delvin | 10,000 | 9,000 |
| Stormy Hoverboard (697) | 10381 | SC Marketplace, Mechanical (bundle 4544, 2010-09-15) | 550 | Delvin | 11,000 | 9,900 |
| Fairweather Hoverboard (99) | 10393 | SC Marketplace, Mechanical (bundle 4863, 2011-01-26) | 549 | Delvin | 11,000 | 9,900 |
| Pogo Stick (13567) | 79438 | SC Marketplace, Mechanical (bundle 79438, 2011-04-06); Mysterious Merchant (merchant-only bundle) | 750 | Delvin + Cragara | 15,000 | 13,500 |
| Eyeball (13568) | 79439 | SC Marketplace, Exotic (bundle 79439, 2011-04-06); seasonal event | 149 | Delvin | 3,000 | 2,700 |
| Polar Bear (13576) | 79447 | 12 Days of Presents quests / Snow Days gift boxes (wiki); SC Marketplace, Non-Winged (bundle 79447, 2011-04-06) placeholder | 500 L | Delvin | 10,000 | 9,000 |
| Dwarftech Chopper (13587) | 79458 | SC Marketplace, Mechanical (bundle 79458, 2011-04-06); Mysterious Merchant (merchant-only bundle) | 750 | Delvin + Cragara | 15,000 | 13,500 |
| Surfboard (12161) | 15733 | SC Marketplace, Mechanical (bundle 15733, 2011-04-06); seasonal event | 399 | Delvin | 8,000 | 7,200 |
| Wooden Surfboard (12168) | 15740 | SC Marketplace, Mechanical (bundle 15740, 2011-04-06); seasonal event | 449 | Delvin | 9,000 | 8,100 |
| Basalt Surfboard (12218) | 16920 | SC Marketplace, Mechanical (bundle 16920, 2011-04-06) | 449 | Delvin | 9,000 | 8,100 |
| Holiday Tree Board (13589) | 79460 | SC Marketplace, Mechanical (bundle 79460, 2011-04-06); seasonal event | 350 | Delvin | 7,000 | 6,300 |


## Choices for the owner (all reconstructed)

1. **Coin rate of 20 coins per SC.** Cheap rides land at 3,000 to 9,000 coins, the dragons and hoverboards at 11,000, the Robgoblin Mech at 19,000. Other high-end store goods go up to 100,000 coins, but quest payouts in content are 5 to a few hundred coins, so a ride is a long goal. Change `coins_per_sc` or single rows.
2. **Chrome Drake (TCG promo) and Polar Bear (event gift)** are plain purchases at 10,000 coins. They are the natural quest or achievement rewards, but no event or TCG quest exists in the quest content, so none was wired. If you want them scarce, raise their rows or pull them from Delvin's `item_ids`.
3. **Seasonal rides (both surfboards, Eyeball, Holiday Tree Board) are always on sale.** The server has no event calendar.
4. **Sun** is sold without the V.I.P. rank 2 gate. **Membership is not enforced** by the vendor buy (no `RESULT_NOT_A_MEMBER` path in `buy_once`), although the original required membership to buy rides. `vendors.json` keeps `member_only` null because the vendor test pins `member_only_flags_recovered` at 0.
5. **Estimated prices (E, 4 rides):** Feathered Drake 549 SC, Tractor 549, Lava Tiger 550, Stallion 450. **Inferred from the wiki member price (WM, 3):** Skeletal Dragon 650, Gold-Tipped Dragon 550, Hearts Pegasus 550.
6. **One shop for 54 rides.** Delvin lists all 54 in one window. A split by category over several vendors, or a Stable Hand NPC, would match the Marketplace tabs better.

## Gaps this does not close

- Ride items occupy bag slots (the mount service derives the roster from inventory) and a player can buy the same ride twice; the original kept rides out of the bag and said duplicates give no benefit.
- Buying needs a loadable definition: true for all 54 with `item-catalogue.json` beside `inventory.json` (W166). Without the catalogue the buy fails with `RESULT_INVALID_ITEM`.
- The vendor window with a 54-item list of type 19 items has not been run natively.
- Rides outside the seed (the wiki lists about 100) have no definitions here and so no source.
- `vendors.json` came from a W118 generator that is not in the repo. Re-running it would drop the two ride vendors; `lab/runs/claude/W169/apply_ride_sources.py` re-applies them and is idempotent.

## Verification

See `lab/runs/claude/W169/final.md`: `content_check.py --strict` 0 errors, the existing vendor, coin shop, mount and quest tests, and the new `tests/ride_sources.rs` (red before the content change, green after).
