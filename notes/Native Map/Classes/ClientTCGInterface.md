# Class: ClientTCGInterface

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00ad1640` | `ClientTCGInterface::Ctor` | ✅ | ctor | Stores the TCGInterface vtable, then replaces it with ClientTCGInterface's vtable. |
| `00ad16e0` | `GetGlobalQword3c0` | ✅ | virtual | Returns the 64-bit field at +0x3c0 of the global client's object at +0x3e0. |
| `00ad1700` | `RemoveEmitterForId` | ✅ | virtual | Obtains the global client context and calls 00b08070 with the supplied id and zero. |
| `00ad1720` | `— ⚠reject` | ⚪ | virtual | Returns the dword at +0x5ef48 of the global client's object at +0x3e0. |
| `00ad1740` | `ShowMarketplace` | ✅ | virtual | Dispatches MarketplaceHandler:Show with two zero arguments. |
| `00ad1760` | `ForwardGlobalObjectTo790d00` | ✅ | virtual | Calls the global-client getter, loads ECX from its +4 field, and tail-jumps to 00790d00. |
| `00ad1770` | `ForwardGlobalObjectTo790d10` | ✅ | virtual | Calls the global-client getter, loads ECX from its +4 field, and tail-jumps to 00790d10. |
| `00ad1780` | `GetGlobalByte940` | ✅ | virtual | Returns the byte at +0x940 of the global client object at +4. |
| `00ad1930` | `GetGlobalField41c` | ✅ | virtual | Returns the dword at +0x41c of the global client's object at +0x3e0. |
| `00ad1950` | `CopyGlobalPair3a0` | ✅ | virtual | Copies the two dwords at global-client offsets +0x3a0 and +0x3a4 into the caller's output pair. |
| `00ad1970` | `DispatchTwoGlobalHandlersA` | ✅ | virtual | Calls 00769710 and then 00b4b820 with the same two supplied arguments. |
| `00ad19a0` | `DispatchTwoGlobalHandlersB` | ✅ | virtual | Calls 00769710 and then 00b4d210 with the same two supplied arguments. |
| `00ad19d0` | `ClientTCGInterface::ScalarDeletingDtor` | ✅ | dtor | Restores the TCGInterface base vtable and frees the object when flag bit 0 is set; otherwise returns it. |
| `00ad1eb0` | `CreateTcgAudioEmitter ⚠downgrade` | 🟢 | virtual | Calls the global-client helper with zero and two shared defaults, then calls 00b07c30 with the same values and supplied object. |
| `00ad2790` | `BuildT4DetailsText` | ✅ | virtual | Builds a T4Details list from fixed input records, calls the integration API virtual slot +0x14 to resolve values, compacts the caller's text buffer, and returns whether the result is nonempty. |
| `00ad2920` | `ShowMarketplaceWithThreeArgs` | ✅ | virtual | Builds three Lua arguments from the supplied values, invokes MarketplaceHandler:Show, and destroys the temporary argument array. |
| `00ad2ac0` | `ShowMarketplaceWithTwoArgs` | ✅ | virtual | Builds two Lua arguments from the supplied values, invokes MarketplaceHandler:Show, and destroys the temporary argument array. |
| `00ad2c30` | `ShowMarketplaceWithOneArg` | ✅ | virtual | Builds one Lua argument from the supplied value, invokes MarketplaceHandler:Show, and destroys the temporary argument array. |
