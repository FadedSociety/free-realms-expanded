---
type: design
section: Design
updated: 2026-10-20
---

# Station Cash

T230 slice 1: an earned Station Cash wallet and rides sold for Station Cash in the client's
Marketplace. This page records what the 2014 client actually reads and writes. Addresses are in
the shipped `FreeRealms.exe`. Reconstructed server policy is marked (R). No player names or chat
text are recorded.

## Client evidence

Re-verified 2026-10-20 with the read-only IDA tool in this worktree: `00b3a6e0` reads the 66/11
status and `00b2b450` the record (bool, balance, `u32`, string, string, bool); `00b3b6d0` /
`00b2ded0` read result, tracking id, string, discount, total and `00cc5470` maps 1 to success,
2/4/6/7 to order failure and 5 to balance not enough; `00b3da50` / `00b3fa00` / `00b41860` confirm
the 66/5 bundle record and its ten-field continuation; `00b3fd70` / `00b3b080` / `00b33400`,
`00b3ad20`, and `00b3d770` / `00b36bb0` / `00b2a090` confirm the navigation field order. The
T230 first pass stopped at `00b3fa00` and missed `00b41860`; that is the only reader the positions
below had contradicted, and F324 fixes it. The order field names are confirmed by the client's own
order debug string `0x01843730` (`customerUserKey`, `recipientUserKey`, `orderTrackingId`,
`customerLocaleString`, `couponCode`).

### Wallet balance: 66/10 request, 66/11 response

- 66/10 is the bare two-u16 header (`PacketInGamePurchaseWalletInfoRequest`).
- 66/11 is read by `00b3e120` through `00b3a6e0`/`00b2b450`. After the header:
  - `u32 status` (1 is success),
  - record: `u8` (unknown/first flag), `u32` **balance**, `u32` (credit-card id in the community
    name), a length-prefixed string, a second length-prefixed string, `u8` (international flag).
- `00b3e120` at `00b3e1c3`-`00b3e1ca` takes the record's first u32 (record+4), passes it to
  `00d5c770` (stores at wallet +0x1f0) and then to the `OnWalletInfo` Lua event as its third
  argument. That is the Station Cash line the player sees.
- The community reference's `WalletInfo` field order (bool, StationCash, credit-card id, two
  strings, bool) agrees with the client reader. The count is not the first u32 after the header;
  the status is. The server writes `wallet_packet` in exactly this order
  (`src/station_cash.rs`).

### Catalogue: 66/5 and its navigation packets

The Marketplace item catalogue is the store-bundle family, not the Station Cash SKU list:

- **66/5 full store** (`00b48700`). After the header: `u32` inner type 1, `u32` store id, then the
  store definition: three `u32` ids, two image strings (`00b2dfd0`, `00b45340`), then a counted
  map of bundle rows (`00b43170`). Each row is a `u32` bundle id key plus the bundle record read
  by `00b41860`/`00b3da50`/`00b370d0`:
  - four `u32` (id, name id, description id, status), two image strings, `u8` tintable,
  - tint-group string, `u32` currency type, `u32` price, two `u32` alternative prices,
  - `u32` partner id, two `u64` launch/expire, `u32` unknown, `u8` unknown, two more `u64`,
  - counted entries (quantity, game item id, marketing item id, tint-override string),
  - `u32` store id, `u32` category-group id, `u8` visible, `u32` max per order,
  - `u32` display order, `u32` category display order, `u32` discount type, `u32` sku id,
  - `u32` discount value, `u32` unknown3, `u32` unknown4, then `u8` terms id, `u8` new and
    `u8` view-only (this is the `00b3fa00` tail),
  - then `00b41860`'s continuation: `u32, u32, u8, u32, u32, u32, u8, u8, u32, u32` (31 bytes,
    names unrecovered, written as zero). `00b41860` calls `00b3fa00` first and then reads these
    ten fields; `00b43170` calls `00b41860` for every map entry.
- The T230 writer stopped after the `00b3fa00` tail and was 31 bytes short per bundle, so every
  bundle after the first misaligned and the client's reader-error flag was set. F324 appends the
  ten fields; the reader-derived byte test in `tests/station_cash.rs` walks the whole 66/5 packet,
  asserts the exact packet size (36 + 185 bytes per bundle) and requires the exact last byte.
- The bundle entry list (`00b370d0` -> `00b2f390` -> `00b24610`) is a counted list of
  (quantity, game item id, marketing item id, tint-override string); the 66/5 writer follows that
  order, which also matches the community reference's `MarketingBundleDefinition.Entry`.
- **66/7 category tree** (`00b48040`/`00b43090`/`00b3fd70`/`00b3b080`): counted map of category
  nodes (id, name id, image strings, parent id, count, display order) and a sorted byte.
- **66/6 category groups** (`00b44d70`/`00b3ad20`): counted map of group id plus an id and a list
  of category ids.
- **66/9 bundle groups** (`00b45560`/`00b3d770`/`00b36bb0`/`00b33350`/`00b2a090`): counted map
  of group id plus a group record (id, name id, description id, image strings, exclusive byte,
  status, then a counted entry map). Each entry is the map key, the entry's own bundle id and its
  display order; `00b2a090` reads **two** words after the key, so the value's bundle id is not the
  key. The first T230 writer omitted it and the client reader overran; the navigation test in
  `tests/station_cash.rs` walks 66/6, 66/7 and 66/9 to the last byte.
- The 66/6, 66/7 and 66/9 payloads start with the list/map count, not a status dword: their
  handlers read only the two-u16 header (`00b2dd60`) and hand the stream to the counted reader.
  The writer's leading `1` is one category or one group, not a result code.
- **66/24** enables the Marketplace (one byte); **66/33** sets the two payment-source bytes. Both
  stay server-owned; payment sources stay off.
- The client's store screen sends **66/12** (server status) when it opens
  (`RequestServerStatus` in the `InGamePurchaseStoreScreen` registration table). The server
  answers 66/13 and then pushes the catalogue. Pushing on this request is (R): the community
  reference pushes at zone entry instead, and no store traffic was ever captured. The native
  client test is still needed for this ordering.

The bundle record's *field positions* are the client reader order. The *names* (price, currency
type, limits) are reconstructed from the community packet catalogue and the client's data-source
column table `00997840`, so they are (R). The store and category name ids are data: 73197
resolves to "Marketplace" and 27547 to "Rides" in the client locale (`Global.Text.<id>`; checked
by `scripts/content_check.py`).

### Place order and its result

- **66/3** is the order object serialized by `00b3fc60` (after the header):
  - two length-prefixed strings (customer key, recipient key), a `u32` order-tracking id, four
    more strings (locale, coupon, two unknown), a `u32` detail count, then per detail
    `u32 store-bundle id`, `u32 store id`, `u32 quantity`, tint string, second string
    (`00b3af80`). The client's own order debug string `0x01843730` names
    `customerUserKey, recipientUserKey, orderTrackingId, customerLocaleString, couponCode`, so
    the coupon slot is the second string after the tracking id.
  - `CreateNewOrder` (`sub_C039F0` -> `sub_B376D0`) writes the local player to the customer field
    and the requested player to the recipient field; the ordinary store path in the client Lua
    calls `CreateNewOrder(0)`, and the binding substitutes the local player for the self
    sentinel. A self-order therefore repeats the customer key; the server treats only a
    non-empty, different recipient as a gift and refuses it.
- **66/1 preview** is the same order followed by one extra string. The server tolerates its
  absence (R) and still rejects any bytes after it. **66/2 preview response** is read by
  `00b3b450`/`00b2de20`: four u32, the first compared with 1 (result), then order id, discount
  and total.
- **66/4 place-order response** is read by `00b3b6d0`/`00b2ded0`: `u32 result` (1 is success),
  `u32 order-tracking id`, length-prefixed order-id string, `u32` discount, `u32` total. The
  client's `OnOrderResponse` receives success, order id, discount and total; a failure raises
  `PurchaseFailed` and an alert. The result-to-error map `00cc5470` confirms 2 (and 4/6/7) as
  order failure, 5 as balance not enough and 1 as success; the server uses 2 and 5.
- The community reference writes the tracking id before the result. The client compares the
  first dword with 1 in `00b3b6d0`, so **result first**; the client wins. (IDA contradiction
  with the community reference recorded here.)

### What the client shows after a successful order

- `00b3b6d0` raises `OnOrderResponse` with the success flag, tracking id, discount and total, and
  the generic path records the response on the pending order. The granted item arrives through
  the normal inventory update packets; `NotificationHandler:ItemReceived` runs for the item.
- The order handler does **not** request the wallet again. The wallet line updates when the
  server sends 66/11 (`OnWalletInfo`). T230 therefore pushes a fresh 66/11 immediately after a
  successful order so the client displays the debited balance. This push is (R).

## T230 server policy (R)

- Station Cash is a **per-character** wallet in the character save (`GameplayState::sc_wallet`).
  The original wallet was account-wide and funded with real money; the revival has no payment
  path and the character save is the atomic unit.
- `Wallet::restore` applies the data ceiling and replay-window repair only when the policy is
  enabled. A package with Station Cash off leaves a saved wallet untouched, so a content toggle
  cannot silently clamp or prune a currency it does not serve; the repair runs when it is enabled.
- A ride row in `data/content/stores.json` carries `sc_price` (the original Station Cash value)
  and may carry `earn_only: true`. The Marketplace lists every `sc_price` row except
  `earn_only` at `round(sc_price * station_cash.sc_price_scale)`, with the scale, price bounds,
  wallet ceiling, replay-window size, store/category ids and name ids in `station_cash`.
- A row with a Station Cash price cannot also have a coin purchase or sell price and must be
  `no_sale`. The two currencies never meet in one store row. Coins no longer buy rides: the
  Seaside and Mysterious Merchant ride stock was removed.
- An order:
  - parses a bounded 66/3 body (4 KiB, 256-byte strings, 16 details, exact end);
  - requires exactly one detail, quantity 1, the configured store id, a listed bundle, no gift
    recipient, no coupon;
  - prices from the server catalogue (the packet carries no price);
  - refuses when busy (any active in-Gameplay activity through `Gameplay::activity_busy`; an
    active player trade is not in that table, see "Open gaps"), when the balance is short
    (result 5), when the bag is full, or when the item cannot be granted (result 2);
  - stages the inventory grant and only then commits the debit, the inventory, and a persisted
    replay record, all in one `place_order` call.
- The replay record is keyed by the full order (tracking id, store, bundle, quantity, tint). A
  duplicate returns its success without a second debit or grant. The persisted window is
  `station_cash.order_history` (default 256, capped at 4096); a replay older than the window is
  outside this slice's proof. The client's tracking id is a process-global counter that starts
  at 1 each time the client runs (`00b28360`), so a restarted client can reuse an id for a new
  purchase; the whole-order key keeps that from being mistaken for a replay of a different
  order, and a window old enough to drop the record is this slice's documented bound (F324
  review item 7; making it absolute needs the lead decision recorded in Open gaps).
- `Chrome Drake` (item 79454) and `Polar Bear` (item 79447) are `earn_only`; slice 2 makes them
  rewards.
- Wallet funding, membership and real-money orders stay refused. There is no gameplay Station
  Cash source in slice 1; slice 2 adds the earned source.

## Tradeability finding (item data)

Every ride row in the generated item catalogue has the NO_TRADE byte at +44 set
(`Inventory::trade_flags`; `piece_096` layout). Provenance matters here: ride ids are absent from
the 302-row shipped `CharacterCreate/ItemDefinitions.txt` and from the 1,703 captured client
definitions, so the ride rows come from Legacy `ClientItemDefinitions.json` (`source:
legacy+client-if-present`) and `NoTrade` is tagged reconstructed. It is therefore **not a packet
capture**. The existing trade gate refuses a bought ride with `UntradableItem`; T230 pins the flag
for all 54 catalogue rows in `tests/station_cash.rs` and does not override item data.

**Goal B (tradeable SC rides) is explicitly deferred** (F324 review item 9): overriding a
reconstructed NO_TRADE flag is a lead economy decision, so version 1 keeps the existing trade
gate and the pin. Reopen with a captured client flag for the ride ids; do not assume the
reconstructed byte is wrong.

## Open gaps

- Native client acceptance of the catalogue push (66/12 → 66/24/33/7/6/9/5) and of the order
  response is not yet run; the bundle field names beyond the reader order are (R). Bundle
  launch/expire are written as zero (no time-bounded sale assumed); the client's date
  filter/display semantics are unrecovered. `currency_type` is 0 from data and the client's
  price/currency symbol mapping is unrecovered.
- The 66/12 hook is a reconstruction; the original may push the catalogue at login.
- The welcome screen's `starting_wallet_balance` stays 0; the 66/11 wallet is the authoritative
  display.
- Slice 2 (earning) is not implemented, so a fresh character has no Station Cash.
- The replay window is bounded and persisted; see the note above. A replayed packet older than
  the window is charged again (a duplicate purchase, not a free grant); making that absolute
  needs a lead decision because the client tracking id restarts at 1 per client process.
- Trade mutual exclusion is deferred (F324 review item 2): T230 derived the trade-busy flag from
  the shared trade service in the per-message staged clone, which changes the lead-only session
  transaction staging path. F324 reverts that call, so an order is gated by every in-Gameplay
  activity through `activity_busy` but not by an active player trade. Wiring trade into the busy
  table needs a lead-approved per-request input (not a staged flag) and a real `TradeService`
  test. The existing trade path still revalidates offers at settlement, and the order commit
  syncs the inventory back into the trade service (`refresh_trade_player`).
- `CoinShop::handle_marketplace` takes the shared busy answer as an argument now; the hub passes
  `Gameplay::activity_busy` on every Marketplace packet.
