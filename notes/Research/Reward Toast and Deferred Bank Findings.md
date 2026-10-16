# Free Realms reward toast and deferred-bank findings

October 9, 2026. Scope: Windows client **1.910.1.530630**, executable SHA-256 `26cc1c522a45e71fb57882da515f65bac45c8c8b8c7e740c0cea81c52a733351`. Addresses below are virtual addresses for that executable at image base `0x00400000`. Other builds/platforms need their own reader check.

**The main distinction: normal reward notifications and the non-member deferred bank are different paths.** `FUN_00b8a640` dispatches reward-family packets. `RewardPatternOne` is a localization key for a single reward. `SetCurrentXpBank` / `SetCurrentCoinBank` update a deferred-bank UI; they are not the wallet/class-progression setters. The actual Lua callback is **`ShowXpCoinBankReward`**, rather than `ShowXpCoinBankAward`.

Thanks for the supplied Lua archive, locale hasher and pack extractor. They helped corroborate these findings against the original client. This note contains derived layouts/behavior, not redistributed game scripts or decompiled functions.

## 1. Framing: this family uses a byte subtype

Inside the existing gateway/tunnel framing:

```
u16 little-endian base opcode = 50 (0x0032)
u8 subtype
subtype-specific body
```

For example, a bundle starts `32 00 01`, not `32 00 01 00`. Do not count the SOE/tunnel wrapper as part of the body offsets below. Native base reader `0x00b89190` consumes two bytes followed by one byte; constructor `0x00b88ad0` sets base opcode `0x32`.

| Subtype | Behavior in `0x00b8a640` |
| --- | --- |
| 1 | Reads a reward bundle and queues a player reward event |
| 2 | Reads a non-bundled item and uses the item-notification helper |
| 6 | Adds deferred stars/XP and coins to cached bank totals |
| 7 | Clears the native bank caches and shows the granted-bank membership panel when a supplied amount is positive |

## 2. Normal quest reward toast: separate coins and XP

Trace **`0x00b89ca0`** in addition to `0x00b8a640`. The former handles bundle reward presentation. Relevant getters in the ClientRewardBundle vtable:

| Vtable slot | Function | Value returned |
| --- | --- | --- |
| `+0x3c` | `0x0091c290` | object `+0x48`, reward XP/stars amount |
| `+0x50` | `0x0091c2e0` | object `+0x50`, reward coin amount |
| `+0x84` | `0x0091c380` | object `+0x54`, reward kind |

The original bundle reader **`0x008e7930`** establishes these offsets **relative to the subtype-1 body**, excluding its three-byte header:

| Body offset | Width | Meaning established here |
| --- | --- | --- |
| `0` | 1 | Success flag |
| `1` | 4 | Coin award amount |
| `5` | 4 | XP/stars award amount |
| `9` | 4 | Not resolved in this investigation |
| `13` | 4 | Reward kind, used for item-notification behavior |

Thus full inner-packet offsets are coins **4**, XP **8**, reward kind **16**. All numeric fields here are little-endian. These are award amounts, not total wallet balances.

The remaining empty-entry body is 69 bytes: further fields, multiplier and GUIDs precede icon/name fields, entry count at body61, then a final four-byte field after the entries (body65 when there are none). A synthetic empty-entry inner packet is therefore 72 bytes. This is reader-layout evidence; zero/default values for unresolved fields are not recovered historical settings.

A practical naming trap: in the **pinned Sanctuary serializer we inspected**, the first integer is called `Unknown1`, the second `RewardKind`, and the fourth `Unknown3`. The native getters establish those positions as **coins, XP and reward kind**, respectively. Check byte positions before trusting those names; they may differ in another fork/revision.

### Template selection

Original `Resources/CodeStringMappings.txt` maps:

- `RewardPatternOne` -> locale ID2: `You receive #count([*item*])`
- `RewardPatternTwo` -> locale ID3: `You receive #count([*experience*]) and #count([*coins*])`
- `Coin` -> ID4; `Experience` -> ID2517. In this supplied locale those label strings are `coin` and `star`.

`0x00b89ca0` first obtains the XP amount and applies the player-side adjustment helper `0x00be8980`, then reads coins separately. Its signed-positive checks select:

| Effective XP | Coins | Template branch |
| --- | --- | --- |
| 0 | 16 | RewardPatternOne, coins |
| 16 | 0 | RewardPatternOne, XP/stars |
| 16 | 16 | RewardPatternTwo, both |
| 0 | 0 | Skips this currency/XP message construction; separate item handling can still continue |

Therefore **encoding 16 in both award fields causes the combined message by design**. For coins-only, encode coins=16 / XP=0; for XP-only, encode coins=0 / XP=16. Do not edit the locale sentence to hide a wrongly populated award field. If a packet really contains one zero amount but still renders both, inspect subtype width, offsets, build identity and which queued event produced the notification before concluding this is the same bug. We have not inspected the contributor's failing packet bytes.

## 3. Deferred non-member bank: subtypes 6 and 7

Subtype6 reader **`0x00b892b0`** reads three four-byte words after the header:

| Body offset | Meaning |
| --- | --- |
| `0` | First word: not resolved here; this dispatch does not use it for the totals |
| `4` | Deferred stars/XP delta |
| `8` | Deferred coin delta |

The handler **adds**, rather than replaces:

```
manager.coinBank += coinDelta   // manager +0x3c
manager.xpBank   += xpDelta     // manager +0x40
```

If the resulting XP total is positive it calls `CallHandler:SetCurrentXpBank(total)`. If the resulting coin total is positive it calls `CallHandler:SetCurrentCoinBank(total)`. These callbacks receive **accumulated totals**, although the packet supplies **deltas**. Repeating this packet adds again; it is not an idempotent absolute-balance update. Zero/non-positive totals do not invoke these setters in this handler.

The Lua forwards those totals to `XpButton`, converts them to decimal strings, and calls `XpButton.gfx` methods `setXp_lua` and `setCoin_lua`. Those ActionScript functions assign the displayed `starCount` and `coinCount`. This does not credit spendable coins or the class rank/XP fields. UI terminology is stars; do not assume it is the separate ability-star spending balance.

Subtype7 reader **`0x00b894a0`** reads:

```
i32 grantedCoins
i32 rowCount
rowCount times: { i32 firstWord, i32 grantedStars }
```

The list reader is `0x00b89340`; each two-word row is read by `0x00b88a70`. The first row word is not resolved here. The handler clears its native coin/XP caches, sums the **second word** of every row, and calls:

```
CallHandler:ShowXpCoinBankReward(sumOfStars, grantedCoins)
```

It does so only when coins or the summed stars are positive. The panel amounts come from this packet, not from the cache totals that were just cleared. This handler does not explicitly call the two Lua setters with zero, so do not assume the cached Lua/movie text was reset by those writes alone.

Lua forwards the call to `UnifiedMessageWindow`, which invokes **`showMembershipStarsCoinsReward_lua`** and shows the modal membership reward panel. Locale IDs990/992 provide its header/description, ID23 the OK button, ID388887 the Stars label, and ID4923 Coins. The original `StarsCoinRewardPanel` ActionScript puts the two values into distinct star/coin fields. This is a different UI from the ordinary small reward toast.

## 4. Why Mystery Chest appears in the same investigation

Item-notification helper **`0x00b899f0`** checks reward kind against **16 / 0x10**. For that branch, a successful item-definition lookup leads to `CallHandler:ShowMysteryChestReward(icon, tint, title, body)`, then `UnifiedMessageWindow:ShowRewardPanel`.

For bundled eligible item entries, the kind comes from the bundle getter at vtable `+0x84` (body13). This explains why a reward-family packet can also open a Mystery Chest panel. It does **not** establish how to spawn an explorer chest, acquire its X-interaction, choose historical loot or persist ownership.

Another distinction: subtype2's reviewed dispatch passes fixed notification type8 into the helper. The supplied `NotificationHandler:ItemReceived` deliberately returns early for type8. Do not use that path as a substitute for the normal bundle currency/XP toast without checking the intended notification behavior.

## 5. State updates versus presentation

Our existing reconstructed quest path separately updates the authoritative save and sends **38/19 coin totals**, **38/14 profile progress**, and **38/18 rank notification** when required. Those updates and persistence are native-tested in our local opening campaign. The newly traced reward presentation has not been integrated or visually accepted in a fresh original-client run.

For an isolated test, keep actual wallet/profile changes separate from deferred-bank display experiments. Verify one committed grant, the corresponding totals, the requested presentation, duplicate-request conservation, and reconnect persistence. Sending another display packet must not inadvertently perform another server grant.

## 6. What was actually verified

**36 executable probe checks passed, zero failed in the final suite.** They include:

- Executing the original x86 subtype6 reader under CPU emulation, with the complete message and every shorter truncation.
- Executing the original x86 empty-entry bundle reader for coins-only, XP-only, both and neither; exact 69-byte consumption and expected native fields.
- Executing original x86 bank addition/reset/list-sum blocks and all four currency/XP template branches.
- Executing the supplied CallHandler/XpButton/UnifiedMessageWindow Lua in a Lua5.1 VM with recording UI stubs.
- Opening the original UnifiedMessageWindow, XpButton and notificationSm GFX with official JPEXS26.3.0, exporting 104 ActionScript files, and inspecting the relevant setter, panel and notification functions.
- Comparing the supplied pack reader's 6,155 entries in one complete pack against our existing catalogue, and verifying the extracted membership-panel movie bytes/CRC.
- Checking the supplied locale hasher against all85,932 already-recovered IDs. All hashes/first lines match;85,918 full texts match. **14 multiline strings are truncated by the helper's line-based text reader**. Use the existing full locale reader for complete text. This was found after an initial overly strict full-text comparison failed; that failure is retained in our private evidence.

This is static analysis plus executable CPU/Lua probes, **not a live-client rendering or end-to-end grant test**. The unknown fields identified above remain unknown. No contributor's private work was contacted or requested, no raw game code/assets are included here, and no production reward/save/balance was modified.
