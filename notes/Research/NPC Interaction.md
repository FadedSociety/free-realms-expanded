# NPC interaction: menus, dialogue and vendors

Task W47 implementation evidence, 2026-10-13. The scoped module and focused tests are at
`prototype/osfr-tools/src/npc_interaction.rs` and `prototype/osfr-tools/tests/npc_interaction.rs`.
This note separates native client facts from reconstructed server policy. No native client run was
made for this task.

## Content used

`prototype/osfr-tools/data/content/npcs/*.json` contains 1,199 regional NPC rows in 17 files. The
content roles mark 192 as vendors; 177 have prose stock descriptions and 15 have none. These
descriptions are not item-definition IDs. `stores.json` contains 17,198 priced item rows, but has
no NPC-to-item binding. An unfiltered join would expose unrelated global store items at each NPC.
`vendors.json` is the resolved mapping consumed by the module; it is intentionally empty because
no NPC-to-item IDs were verified. `vendors_unresolved.json` records all 192 vendor keys, preserving
available prose and the unresolved reason. The module uses only explicit vendor mappings and their
`stores.json` prices. Missing prices stay `None` and are reconstructed policy (R).

The content NPC keys and positions are reconstructed records, not proof of live actor GUIDs or
historical placement. The lead must map the runtime actor GUID to the content key before using the
packet builder or merchant check.

## Native client facts

### Interaction request and radial menu

- `008a2820` is the base command constructor: command family 26, followed by a sub-opcode.
- NPC interaction request is 26/8. Constructor `008b6bc0` stores two dwords and one byte in the
  command object. `PlayClient::RequestInteraction` (`0092efe0`) applies client eligibility and
  rate-limit checks; `InteractionProcessor::SendInteractRequest` (`00a383e0`) sends the request.
  `gameplay.rs` independently checks the exact 13-byte shape, reads a little-endian u64 followed
  by a canonical bool byte, and a captured inner frame confirms those widths. The u64 is treated
  as an opaque target key here; semantic meaning still depends on live actor mapping.
- The server radial menu is 26/9. Reader chain `00aa2360` -> `00aa2220` -> `00aa1600` reads two
  dwords, a boolean, a count-prefixed list, a length-prefixed string and a final boolean. List
  reader `00aa04d0` calls `00a9dca0`, which reads six dwords and skips one dword per option.
  `opening_world.rs::menu` already emits this seven-dword option shape. Its one-entry NPC menu
  uses option 1 and locale string ID 3300 for Talk; the generic Interact label is 432156.
- The client sends option selection as 26/10. Constructor `00a36f80` stores three dwords and
  sender `00a39090` transmits it. The local gameplay handler and accepted menu fixture establish
  the body as target GUID (u64) plus choice ID (i32), after the four-byte family/sub-opcode header.
- The client also constructs 26/20 (`FreeInteractionNpc`, `009d9a90`) to clear the interaction
  target. That is distinct from the merchant window's server-to-client 66/46 hide message.

### Dialogue and quest prompts

- Dialogue UI command 26/3 is parsed by `00a9ef10` and handled at `00aa12e0`; `009f5ae0` passes
  dialog text, title and up to five response strings to the Lua dialog handler. The reader consumes
  the two-u16 family/subtype header, then four dwords, a bool, a float, a counted list of
  five-dword rows, two float4s, a bool, a float4, a float, three bools, a float and a dword. Thus
  the full packet is 93 + 20*n bytes for n response rows. The exact float and dword meanings remain
  unnamed. Response 26/6 has one i32 field (`009f5890`); close 26/4 has no body (`009f5760`).
- Quest offer screen is family 49/subtype 1 (`00c7bb60` reader, `00c7be00` constructor). The reader
  consumes a six-byte prefix (u16 family, i32 subtype), six dwords, a bool, a variable reward
  bundle, two dwords, a float and two bools. The empty reward bundle body is 69 bytes per the
  `008e7930` reader notes; non-empty entries are preserved opaquely by the W47b codec. Turn-in
  uses 49/13 and client reply 49/14. `QuestEngine::offer` still returns a typed state outcome.
- NPC content supplies speaker-name and quest-title string IDs. No generic NPC dialogue body or
  phase binding was recovered. The module returns a `DialogueContext` with those IDs; it does not
  fabricate a 26/3 packet or original dialogue text.

### Merchant window and transactions

- The in-game purchase dispatcher is `00b48a10`, family 66 (`NpcMerchantV2`). Subtype 42 parses a
  merchant list (`00b40a60`) and raises `NpcMerchantV2:Show`. `00b3d5f0` reads two signed 16-bit
  values, one dword and one nested record (`00b3aa30`). The group reader `00b33020` consumes seven
  dwords and a counted child list. Each row (`00b29cd0`, reached from `00b2e620`) reads six dwords,
  a bool, three dwords and a bool (38 bytes). Field meanings remain unknown; W47b encodes this
  exact shape with opaque names.
- Subtype 43 is a native buy request: constructor `00b2a290`, serializer `00b3ab20`, tunnel sender
  `00b40b90`; UI callers include `00c0aab0` and `00c0ac10`. The serializer writes two dwords, three
  dwords, a length-prefixed byte string, then three dwords (40 bytes plus string length). String
  framing is through `00911c30`. The meanings of these fields and transaction behavior remain
  unknown. Subtype 44 is its result (`00b3b6d0`), subtype 45 updates stock (`00b3c0d0`), and subtype
  46 hides the window with no body. The dispatcher raises `NpcMerchantV2:Hide` for 46.
- Coin Store buy/sell are separate packets: family 165/subtype 4 is 28 bytes total (header, u64
  merchant ID, u32 client transaction ID, i32 item ID, u32 quantity, u32 caller value); subtype 5
  is 20 bytes total (header, u64 merchant ID, u32 item-instance ID, u32 quantity). The existing
  `CoinShop::handle` validates both and deduplicates buy retries. Its numeric result policy is
  reconstructed in `coin_shop.rs`.

## Reconstructed server policy (R)

- Interaction range is a 12 m 3D sphere in one content zone. The server opening interactions use
  this radius; it is not the client's prompt-acquisition algorithm. Missing/non-finite positions,
  wrong zones and unknown NPCs are rejected.
- Content role and quest bindings produce menu actions. Talk is first; subsequent option IDs are
  assigned sequentially. The Shop label reuses generic Interact string ID 432156 because a Shop
  locale label was not recovered. Offers already marked offered, active or completed are hidden.
  Active accepted quests at the NPC are exposed as turn-in choices.
- One active menu/shop is held per `NpcInteraction` session. Content NPC keys stand in for the
  live interaction GUID and Coin Store merchant ID until the lead supplies an authoritative map.
- `set_vendor_stock_items` accepts a separately reviewed item-ID mapping. Buy IDs must be in the
  active vendor list before the request is passed to `CoinShop::handle`; sell requests use the same
  CoinShop validation. Prices are joined from `stores.json`. There is no guessed global catalogue
  fallback.
- Close clears the local shop state and returns native family 66/subtype 46 bytes.

## Implementation and verification

`npc_interaction.rs` loads bounded NPC, price and vendor-mapping JSON, resolves a target by content key, checks
zone/range, builds the 26/9 menu bytes, consumes the 26/10 choice, returns content-backed dialogue
IDs, invokes `QuestEngine` for offer/turn-in outcomes, builds a typed merchant window, routes
Coin Store transactions through `CoinShop`, and emits the 66/46 hide packet. It also decodes 26/8,
encodes/decodes 26/3, encodes/decodes 49/1 around an opaque serialized reward body, encodes/decodes
66/42, and parses 66/43 only when a shop is open. The 66/43 parser does not apply a transaction.
The module is included
by `#[path]` in its integration test; the lead owns later `lib.rs`/gateway registration.

The original vendor test uses item 8 as an explicit fixture mapping because the regional prose does not map
to item IDs. It verifies a 26/9 menu byte-for-byte, 15-coin debit, one item grant, idempotent retry,
resale, and 66/46 close. The quest test checks the Talk string-ID context and that selecting the
content offer invokes `QuestEngine::offer`. Unknown NPC and out-of-range cases are rejected. Five
W47b fixtures add a real captured 26/8 request and synthetic round-trips for 26/3, 49/1, 66/42 and
66/43.

Commands and results:

```text
python -I scripts/native_map/nmq.py show 008b6bc0 0092efe0 00a383e0 00a36f80 00a39090 00aa1600 00aa04d0
  Read-only native-map lookups; evidence addresses summarized above.
python -I scripts/native_map/nmq.py show 00a99bd0 00aa12e0 009f5890 009f5760 00b48a10 00b40a60 00b3d5f0 00b33020 00b2e620 00b3ab20 00b3c0d0
  Read-only native-map lookups; evidence addresses summarized above.
& '%USERPROFILE%\Documents\Projects\CartoonMMO\lab\tools\rustup\toolchains\stable-x86_64-pc-windows-gnu\bin\rustfmt.exe' src\npc_interaction.rs tests\npc_interaction.rs
  Exit 0; formatted only the two owned Rust files.
$env:CARGO_TARGET_DIR = '%USERPROFILE%\Documents\Projects\CartoonMMO\lab\build\W47'; & '..\..\scripts\cargo.ps1' test --test npc_interaction
  Exit 0; 4 passed, 0 failed. Existing unrelated warning remains in src/housing.rs:567.
```

## Gaps and lead integration

- 26/8 now has a strict shape decoder and a real captured request fixture. Its target-key mapping
  and server-side actor resolution remain outside this module.
- 26/3, 49/1 and 66/42 have reader-derived wire codecs, but no corresponding client frames were
  found in the scanned application captures. The round-trip fixtures are synthetic, not native
  acceptance evidence. 49/1 preserves the nested reward body without validating reward entry types.
- 66/43 now has a strict request parser and an active-shop session gate (R), but item, merchant,
  quantity and result meanings are unresolved; it does not mutate inventory. It remains separate
  from the verified Coin Store 165/4 and 165/5 path.
- The existing accepted opening files do not serialize the claimed 26/3 or 49/1 messages:
  `opening_world.rs::menu` writes 26/9; `QuestEngine::offer` creates typed `QuestInfo` but
  `QuestMessage::encode_exact` returns `None`; opening quest acceptance tests cover 49/3 QuestAdd.
  Thus the 26/3 and 49/1 layouts here come from client reader bytes, not those opening writers.
- No NPC item IDs are resolved. `vendors.json` is empty and all 192 vendor-role NPC keys are in
  `vendors_unresolved.json`; 177 preserve stock prose, and 15 have no prose. The extracted client
  vendor resources and pinned Legacy records provided no verified NPC-to-item mapping.
- The lead integration must map live actor GUIDs and runtime positions to content keys, supply
  reviewed item-ID stock mappings, register the module, persist `CoinShop` state before replying,
  and connect these codecs to gateway routing. No native playability or historical vendor-stock
  completeness is claimed.

## W47b evidence and commands (2026-10-13)

### Recovered wire fields

- **26/8 request:** exact capture `lab/runs/client-baseline-20261009T103605Z/analysis/application-0066.bin`
  is 20 bytes including the seven-byte outer capture envelope. The inner 13 bytes are
  `1a000800050000004e45504f00`: 4-byte family/subtype, 8-byte little-endian target and one bool.
  `gameplay.rs:1508` independently enforces length 13 and bool `0..=1`. W47b decoder mirrors that.
- **26/3 dialogue:** constructor `00a99bd0` sets family 26/subtype 3. Reader `00a9ef10` calls the
  two-u16 family/subtype reader `00a9cb50`, then reads four dwords, bool,
  f32, count plus five-dword rows (`00a9d760` / `00a9b480`), then two float4s, bool, float4, f32,
  three bools, f32 and dword. The full packet is 93 bytes plus 20 per choice, including the 4-byte
  header. All body values are kept as raw integer/float bits or normalized bools because meanings are
  not established. No 26/3 frame was found in the application capture scan.
- **49/1 quest offer:** `00c7acd0` reads a u16 and u32 packet prefix; `00c7b7a0` then reads six
  dwords, bool, reward bundle (`008e7930`), two dwords, f32 and two bools. Empty reward base is
  69 bytes per `notes/Research/Reward Toast and Deferred Bank Findings.md`. The codec preserves
  reward bytes between the fixed prefix and 14-byte tail. No 49/1 frame was found. The Rust
  `QuestInfo` intent still contains only a quest ID and is not passed into this raw-wire struct.
- **66/42 merchant list:** dispatcher `00b48a10` selects reader `00b40a60`; `00b3d5f0` reads two
  i16s and one dword, then allocates one nested record through `00b3aa30`. `00b33020` reads seven
  dwords plus the child count. `00b2e620` iterates rows parsed at `00b29cd0`: six dwords, bool,
  three dwords, bool. No 66/42 frame was found. Struct field names are intentionally opaque.
- **66/43 merchant request:** constructor `00b2a290` sets family 66/subtype 43; serializer
  `00b3ab20` writes fields at object offsets `+0x18`, `+0x1c`, `+0x20..+0x28`, string `+0x2c`,
  then `+0x3c`, `+0x40`, `+0x44`. The string writer `00911c30` writes a dword byte length then those
  bytes. The codec parses the exact order and bounded length. No 66/43 frame was found; field
  meanings and transaction semantics remain open.

Capture scan: application `.bin` files under every `client-baseline-*/analysis/` and
`client-baseline-*/analysis-live/` directory (137,912 files) were scanned for the exact little-endian
family/subtype signatures. Counts: 26/8 184; 49/3 21; 26/3, 49/1, 49/2, 66/42 and 66/43 zero. This
is a signature scan, not a full tunnel/frame parser. Captured 49/3 packets are QuestAdd, not offers.

Commands run:

```text
python -I scripts/native_map/nmq.py dis 00a9ef10 600
python -I scripts/native_map/nmq.py dis 00a99bd0 180
python -I scripts/native_map/nmq.py dis 00a9cb50 100
python -I scripts/native_map/nmq.py dis 00c7b7a0 650
python -I scripts/native_map/nmq.py dis 00c7acd0 100
python -I scripts/native_map/nmq.py dis 00b48a10 160
python -I scripts/native_map/nmq.py dis 00b40a60 230
python -I scripts/native_map/nmq.py dis 00b3d5f0 140
python -I scripts/native_map/nmq.py dis 00b3aa30 230
python -I scripts/native_map/nmq.py dis 00b33020 150
python -I scripts/native_map/nmq.py dis 00b2e620 120
python -I scripts/native_map/nmq.py dis 00b29cd0 380
python -I scripts/native_map/nmq.py dis 00b2a290 110
python -I scripts/native_map/nmq.py dis 00b3ab20 150
python -I scripts/native_map/nmq.py dis 00911c30 260
python -I -c "
from pathlib import Path; import collections
root=Path(r'%USERPROFILE%\Documents\Projects\CartoonMMO\lab\runs')
pats=['client-baseline-*/analysis/application-*.bin','client-baseline-*/analysis-live/application-*.bin']
sigs={(26,3):bytes([26,0,3,0]),(26,8):bytes([26,0,8,0]),(49,1):bytes([49,0,1,0,0,0]),(49,2):bytes([49,0,2,0,0,0]),(49,3):bytes([49,0,3,0,0,0]),(66,42):bytes([66,0,42,0]),(66,43):bytes([66,0,43,0])}
files=set(p for pat in pats for p in root.glob(pat)); c=collections.Counter(); by=collections.Counter()
for p in sorted(files):
 b=p.read_bytes()
 for op,s in sigs.items():
  n=b.count(s); c[op]+=n; by[p.name,op]+=n
print('files',len(files)); print('counts',dict(sorted(c.items())))
print('hits',[(k,v) for k,v in by.items() if v and k[1] != (26,8)][:50])
"
& '%USERPROFILE%\Documents\Projects\CartoonMMO\lab\tools\rustup\toolchains\stable-x86_64-pc-windows-gnu\bin\rustfmt.exe' src\npc_interaction.rs tests\npc_interaction.rs
$env:CARGO_TARGET_DIR='%USERPROFILE%\Documents\Projects\CartoonMMO\lab\build\W47b'; & '..\..\scripts\cargo.ps1' test --test npc_interaction
$env:CARGO_TARGET_DIR='%USERPROFILE%\Documents\Projects\CartoonMMO\lab\build\W47b'; & '..\..\scripts\cargo.ps1' build --lib
```

Native-map lookups confirmed the listed reader call/field sequence; rustfmt exited 0; focused tests
passed 9/9 (including all four W47 tests). The test run reports one existing dead-code warning at
`src/housing.rs:567`. `build --lib` did not complete: the first attempt caught `E0063` while the
unowned `src/login.rs` initializers were being edited; three retries then consistently stopped on
`E0382` because `flags` is moved at line 221 and reused at line 236. That compile blocker is outside
the assigned paths. An initial rustfmt invocation using the stale
`%USERPROFILE%\.codex\tools\rustup` path failed; the private lab toolchain path above succeeded.
No native game client was launched.

## Vendor stock (W47c, 2026-10-13)

### Evidence and provenance

- Native client merchant parsing remains structural evidence only. The native map's `label` rows
  and reviewed bytes identify family 66/subtype 42's reader at `00b40a60`, called by dispatcher
  `00b48a10`; nested records are read through `00b3d5f0`, `00b3aa30`, `00b33020`, and `00b2e620`.
  The nested item row at `00b29cd0` has six dwords, a bool, three dwords and a bool. Their meanings
  are not established. A fresh signature scan across 137,912 application captures under every
  `lab/runs/client-baseline-*/analysis/` and `analysis-live/` found no family 66 subtype 42, 43, 44,
  45 or 46 messages. SoE traffic was not decoded as proof of merchant stock. No native NPC stock,
  price or membership flag is claimed.
- Item names and candidate IDs come from the W51-pinned
  `lab/references/FreeRealms-Legacy/src/Resources/ClientItemDefinitions.json`
  (SHA-256 `40a3cc79cfd8adfd50dc689e58de18dc14e21bedd73993b6eab0a43d3d95f025`) joined by NameId
  to `lab/reports/locale-id-to-text.json`. Only IDs present in
  `lab/content/startup-snapshot/startup.json` (SHA-256
  `da3a4ac1654522e212e7364fcf3870f5315e055a45b66f7ecdb116c904c0f276`) were mapped as resolved
  items. Item definitions provide item identity, not vendor association.
- NPC associations and stock names come from the regional NPC content and local wiki corpus
  (`lab/references/freerealms-wiki/wiki.sqlite`, CC BY-SA). The 94 NPC pages with a `Sells` list
  contribute 1,131 distinct listed names; each row retains its page, revision, section, and
  license. Wiki lists are historical reference claims and are not native family 66 observations.
- The local video review corpus (`lab/video-archive/review/haiku/B*.jsonl`) was checked for
  merchant and item frames. The Merchant Store frame
  `yBwnL7jnweI/00030.jpg` shows a Luau Party Ticket at 18,000 coins with a 2,000-coin sale
  discount, but does not identify seller or zone and the regular price is unreadable. Wiki page
  `Tiger the Party Animal` says he sells tickets, but three regional NPC keys share that name
  (`9500000069`, `9500000540`, `9500000774`); this is retained as an unresolved association
  inference with no item ID. Generic Coin Store rows are not assigned to NPC vendors.
- `stores.json` (SHA-256
  `8b66a80323f81923566ec30c093e53e1aebbca8493fb2a85c96d9867190900a9`) supplies reference price
  fields where an item ID exists. They are not captured NPC quotes. Its member discount price does
  not prove member-only stock; zero member-only flags were recovered.

### Resolved and unresolved data

`prototype/osfr-tools/data/content/vendors.json` contains 64 vendors with item-name mappings and
211 resolved item rows. Exact name matches with multiple item definitions remain ambiguous unless
the sources disambiguate them. Reba's `Certified Postman Hat` resolves to ID 75992: its shared
NameId candidates are distinguished by the item-definition comments, and only 75992 is the
Postman hat present in the startup catalogue. The remaining coverage is 18 ambiguous names, 908
unmatched names, including 524 unique item-name matches absent from the startup catalogue. Five
additional named client-prose claims are kept unresolved: Bait for Big J and Bluerose, Spice and
Bat Meat for Mae Farnum, and Mae's recipe. Bait maps to ID 57012 but that ID is absent from startup;
Spice has candidates 2900/30576, Bat Meat 2862/10008/30573, and the recipe maps to absent ID 848.

There are 185 mapped rows with `stores.json` prices and 26 mapped rows without them. The rows keep
all 211 item mappings and their source fields. `(R)` `item_ids` contains only the 185 priced rows:
the existing `NpcInteraction::load` at `src/npc_interaction.rs` rejects vendor item IDs without a
store-price row. This leaves 62 of the 64 mapped vendors with runtime item IDs; two retain
evidence-backed descriptive rows but no runnable stock IDs. Any server authorization of these
historical/wiki associations remains reconstructed policy. `vendors_unresolved.json` keeps all
192 vendor NPC keys with a reason and item-level ambiguity/unmatched/missing-catalogue records.

The client Build Guide distinguishes family 165 Coin Store purchases from Marketplace currency
transactions and family 66 NPC Merchant V2. Only item-specific NPC wiki/content claims are used
here; global Coin Store and Marketplace inventory is not attributed to NPC merchants.

### Checks and current gate

The added `tests/npc_interaction_vendors.rs` loads production content through `NpcInteraction`,
checks every mapped item ID against the startup item catalogue, validates all vendor keys and roles,
compares available prices to `stores.json`, checks source fields, and prints the coverage above.
It passes. `cargo build --lib` passes. The existing `npc_interaction` integration suite has 8/9
passing: `merchant_buy_66_43_is_parsed_only_for_an_open_vendor_session` expects Alexander's shop
(`9500000339`) to be empty, but the verified wiki/client-name mapping adds priced item 75475. The
other eight tests pass. That test is outside W47c's owned paths and was not edited; its empty-stock
fixture expectation must be reviewed before this acceptance gate is green.

Commands run from `project/prototype/osfr-tools` (build artifacts under `lab/build/W47c`):

```text
& '%USERPROFILE%\Documents\Projects\CartoonMMO\lab\tools\rustup\toolchains\stable-x86_64-pc-windows-gnu\bin\rustfmt.exe' tests\npc_interaction_vendors.rs
$env:CARGO_TARGET_DIR='%USERPROFILE%\Documents\Projects\CartoonMMO\lab\build\W47c'; $cargoArgsW47c = @('test','--test','npc_interaction_vendors','--','--nocapture'); & '..\..\scripts\cargo.ps1' @cargoArgsW47c
$env:CARGO_TARGET_DIR='%USERPROFILE%\Documents\Projects\CartoonMMO\lab\build\W47c'; & '..\..\scripts\cargo.ps1' test --test npc_interaction
$env:CARGO_TARGET_DIR='%USERPROFILE%\Documents\Projects\CartoonMMO\lab\build\W47c'; & '..\..\scripts\cargo.ps1' build --lib
```

Results: rustfmt exited 0; the vendor acceptance test passed 1/1 and printed 192 keys, 62 runtime
vendors, 211 mapped items and 185 priced items; `npc_interaction` passed 8/9 with the fixture
mismatch above; `build --lib` finished successfully. Both crate builds show pre-existing dead-code
warnings in `src/chess.rs:1064` and `src/housing.rs:567`. An initial direct wrapper invocation did
not forward `-- --nocapture`; argument-array splatting above did. No native client was launched.
