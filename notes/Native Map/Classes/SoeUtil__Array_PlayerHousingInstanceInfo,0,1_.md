# Class: SoeUtil::Array<PlayerHousingInstanceInfo,0,1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00cbbd20` | `— ⚠reject` | ⚪ | virtual | Virtual slot 1 of the Array<PlayerHousingInstanceInfo> class (thiscall, count argument, out capacity pointer, flag): with flag zero, grows to (5n)/4 when the current capacity is below n, keeps the current buffer when th… |
| `00cbbdc0` | `ArrayFreeBufferAlign8 ?` | 🟡 | virtual | Virtual slot 2 of the Array<PlayerHousingInstanceInfo> class (two stack arguments, ret 8): bytes 8b 44 24 04 6a 08 50 e8 ... 83 c4 08 c2 08 00 decode to push 8 and the first argument, call 00401c00 (free with alignment … |
| `00cbbde0` | `ArrayExchangeHeadersPlayerInfo ?` | 🟡 | virtual | Virtual slot 3 of the Array<PlayerHousingInstanceInfo> class (thiscall, param_2 = other array header pointer, param_3 = byte flag). Flag nonzero: indirect call through slot 3 of *param_2 and return. Flag zero: exchanges… |
