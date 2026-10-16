# Class: CoinStoreScreen

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00bffe30` | `CoinStoreScreen::BuyItem` | 🟢 | callback | Requires three truthy Lua arguments and an available manager, extracts three values, calls 00c77a30, then pushes its result. |
| `00c002d0` | `CoinStoreScreen::RequestStoreSort` | 🟢 | callback | Requires two Lua arguments and an available manager, extracts the first value and second string, calls 00c77ca0, then pushes its result. |
| `00c00730` | `CoinStoreScreen::RequestBuyBackItem` | 🟢 | callback | Requires one or two Lua arguments and an available manager, extracts item and optional value, calls 00c76c60, and pushes its result. |
| `00c00a50` | `CoinStoreScreen::RequestGiftPurchase ? ⚠rename` | 🟡 | callback | Validates four Lua arguments, parses one string as a 64-bit id, builds a temporary item list from three values, calls 00c79370, and pushes its result. |
| `00c00b50` | `CoinStoreScreen::RebuildFilteredList` | 🟢 | callback | When one Lua string argument and a store manager exist, copies the string into a temporary object, calls 00c75bd0 to rebuild filtered items, and pushes the result. |
