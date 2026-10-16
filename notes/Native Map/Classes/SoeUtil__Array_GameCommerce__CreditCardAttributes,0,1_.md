# Class: SoeUtil::Array<GameCommerce::CreditCardAttributes,0,1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00b244c0` | `— ⚠reject` | ⚪ | virtual | Virtual storage-allocation slot of the array template: when the caller is not forcing an exact size it grows the requested count by 1.25x (5/4) or shrinks it by 1.2x (6/5) depending on the current capacity at this+0xc, … |
| `00b24560` | `SoeUtil::ArrayCreditCardAttributesFreeStorage` | ✅ | virtual | Virtual free slot (vtable index 2) of the dynamically-allocated array: calls FreeMemory(param_1, 4) on the buffer pointer argument, with no inline-buffer check. |
| `00b24580` | `— ⚠reject` | ⚪ | virtual | Virtual storage-swap slot (vtable index 3): when param_3 is nonzero it forwards to the other array's own slot 3 ((**(code **)(*param_2 + 0xc))()); otherwise it XOR-swaps the buffer pointer (this+4 with param_2[1]) and t… |
