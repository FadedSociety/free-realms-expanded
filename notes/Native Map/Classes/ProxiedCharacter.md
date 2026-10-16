# Class: ProxiedCharacter

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `009580d0` | `ProxiedCharacter::DestructorAdjustor ? ⚠rename` | 🟡 | virtual | Adds 0x154 to ECX and jumps to 00bf1ee0; decoded bytes show this is an adjustor thunk, not the destructor body. |
| `009580f0` | `ProxiedCharacter::LogAttachmentGroupError` | 🟢 | virtual | Calls 00704dd0 with the exact log name #AttachmentGroupErrors.log\|AttachmentGroupErrors.log, a global configuration pointer, and the supplied value. |
| `00958490` | `ProxiedCharacter::Virtual29 ?` | 🟡 | virtual | Calls vtable slot 0x70 with param_2 and destination this+0x610. |
| `00958630` | `ProxiedCharacter::MarkFlagAt100 ?` | 🟡 | virtual | Stores byte one at offset 0x100 of the object referenced by this+8. |
| `00959ab0` | `ProxiedCharacter::GetPair24 ?` | 🟡 | virtual | Copies two dwords from this+0x24 and this+0x28 to the output pair. |
| `0095a120` | `ProxiedCharacter::HasActiveEffect ? ⚠rename` | 🟡 | virtual | Walks an effect list and returns false if any active node's field +0x20 satisfies 0072a1c0; returns true when none do. |
| `0095a170` | `ProxiedCharacter::SetEffectsEnabled ?` | 🟡 | virtual | Walks active effect nodes and calls 0072ae80 on each node's field +0x20 with the inverse of param_2 as its second argument. |
| `0095a250` | `— ⚠reject` | ⚪ | virtual | Queries a name/string helper, resolves an object using this+0xb4, invokes a virtual callback with a resolved field, then conditionally calls a second interface callback. |
| `0095a3c0` | `ProxiedCharacter::SetContext504 ? ⚠rename` | 🟡 | virtual | Calls the object at this+0x734 virtual slot 4 with this+0x504 as its argument. |
| `00960480` | `— ⚠reject` | ⚪ | virtual | Calls virtual slot 7, processes param_2 through two helpers, copies four words to this+0xf44, clears a field at +0x3f0, then empties a linked list. |
| `00960520` | `— ⚠reject` | ⚪ | virtual | For non-owner characters, forwards two vectors to virtual setters and normalizes direction through FUN_0095dae0; for the owner, copies position and direction into fields, reads control/input state, and may update a mode… |
| `00961550` | `ProxiedCharacter::DispatchCurrentAction ? ⚠rename` | 🟡 | virtual | Reads this+0x1884; if it is zero and the relevant flag/value condition holds, calls vtable slot +0x24, otherwise dispatches the selected value through the object at this+0x734 slot +0x10. |
| `00963410` | `ProxiedCharacter::OnProfileSwapComplete` | ✅ | virtual | Sets byte +0x18d4, clears +0x18d5 when set, and for the owning character optionally logs JobHandler:OnProfileSwapComplete; if the profile predicate and other state checks pass, obtains an object and calls FUN_007dc190. |
| `0096aaf0` | `ProxiedCharacter::SetRecordValue_v30 ?` | 🟡 | virtual | Virtual slot 30 of ProxiedCharacter::vftable. For index 0..17 it finds or creates the record as in 0096aa00, releases record+0x1d0 through 00969910 when set, and stores param_2 at record+0x1cc. |
| `0096bd90` | `ProxiedCharacter::SwitchToTemporaryAppearance` | 🟢 | method | Thiscall(param_1=P, param_2=request record). Returns when record+0x18 equals P[0x3cb]. Otherwise, when P[0x1ce]+0x959 is zero and the vtable call on P returns nonzero, and the checks 0076c590, 008c4930, P[0x61c] and P[0… |
| `0096fa70` | `ProxiedCharacter::Ctor` | ✅ | ctor | Calls ProxiedObject construction, installs ProxiedCharacter vtables, initializes identifiers, flags, transforms, targets, strings, collections, event lists, manager pointers, and numerous default fields. |
| `00970770` | `ProxiedCharacter::GetField200 → ProxiedCharacter::SubobjectAtC8` | ✅ | virtual | Returns the address this+200. |
| `00970780` | `ProxiedCharacter::SetField128` | ✅ | virtual | Stores param_2 at this+0x128 and returns. |
| `00970790` | `ProxiedCharacter::Field200AdjustorThunk` | ✅ | virtual | Subtracts 0x54 from ECX and tail-jumps to 00970770, which returns the adjusted object's address plus 200. |
| `009707a0` | `ProxiedCharacter::ScalarDeletingDtorAdjustorThunk50` | ✅ | virtual | Subtracts 0x50 from ECX and tail-jumps to 009731e0, the scalar deleting destructor implementation. |
| `009707b0` | `ProxiedCharacter::ScalarDeletingDtorAdjustorThunk54` | ✅ | virtual | Subtracts 0x54 from ECX and tail-jumps to 009731e0, the scalar deleting destructor implementation. |
| `009707c0` | `ProxiedCharacter::IsFlagBit7ClearAtEC` | ✅ | virtual | Reads the byte at this+0xec, inverts it, converts to unsigned, shifts right seven bits, and returns the resulting byte. |
| `009707d0` | `ProxiedCharacter::GetFlagBit0At144` | ✅ | virtual | Reads the byte at this+0x144 and returns its low bit. |
| `009707e0` | `ProxiedCharacter::AdjustorThunkTo00961550` | ✅ | virtual | Subtracts 0x54 from ECX and tail-jumps to 00961550. |
| `009707f0` | `ProxiedCharacter::SetFlagBit0At144` | ✅ | virtual | Sets bit 0 at this+0x144 when param_2 is nonzero; otherwise clears that bit while preserving the other seven bits. |
| `00970810` | `ProxiedCharacter::CreateStringFromField56C` | ✅ | virtual | Initializes an output IString object with the shared empty string, then passes the field at this+0x56c to a string helper and returns the output object. |
| `00970880` | `ProxiedCharacter::CreateStringFromField58C` | ✅ | virtual | Initializes an output IString object with the shared empty string, then passes the field at this+0x58c to a string helper and returns the output object. |
| `00972570` | `ProxiedCharacter::Dtor` | ✅ | dtor | Restores ProxiedCharacter vtables, destroys subordinate state, calls RemoveRider, and frees the allocated mount-state block. |
