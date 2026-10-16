# Class: InGamePurchaseSessionStats<GameCommerce::ClientInGamePurchaseOrder>::IdEntry<int>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00b21090` | `IdEntryIntDtor` | ✅ | dtor | Destructor body storing the IdEntry<int> vtable at [param_1] and then Loggable::vftable; reached from nine EH unwind funclets. |
| `00b214a0` | `IdEntryIntDeletingDtor` | ✅ | dtor | Virtual slot 1 of the IdEntry<int> vtable: stores the IdEntry vtable, stores Loggable::vftable and frees the object with 00d82463 when bit 0 of param_2 is set; returns the object pointer. |
| `00b21fb0` | `InGamePurchaseSessionStats::IdEntryCtor` | ✅ | ctor | Constructor for IdEntry<int>: writes Loggable::vftable then the IdEntry<int> vftable to *this, copies the int key from *param_2 into this+4, zeroes this+8, and stores the 8-byte value pair returned by FUN_00702670 (addr… |
| `00b22200` | `InGamePurchaseSessionStats_IdEntryIntDtor ⚠downgrade` | 🟢 | dtor | Destructor body: writes the IdEntry<int> vtable then the Loggable base vtable to *this and returns; body is byte-identical to the copy at 00b22250. |
| `00b22250` | `InGamePurchaseSessionStats_IdEntryIntDtorVariant ⚠downgrade` | 🟢 | dtor | Second byte-identical copy of the IdEntry<int> destructor body: writes the IdEntry<int> vtable then Loggable::vftable to *this and returns. |
| `00b22b80` | `InGamePurchaseSessionStats_IdEntryIntDefaultCtor` | ✅ | ctor | Constructor variant: writes Loggable::vftable then the IdEntry<int> vtable to *this (base then own order) and zeroes this+4 and this+8; it does not load the timestamp pair that the keyed constructor at 00b21fb0 stores. |
| `00b25bc0` | `IdEntryIntDerivedCtor` | 🟢 | ctor | Inside an SEH frame, calls FUN_00b21fb0 with the incoming this pointer and param_3 as the key pointer, copies *param_2 to this+0x20, and returns this. |
| `00b25c20` | `IdEntryIntDerivedCtorVariant` | 🟢 | ctor | Inside an SEH frame, calls FUN_00b21fb0 with the incoming this pointer and param_3 as the key pointer, copies *param_2 to this+0x20, and returns this. |
| `00b26260` | `InGamePurchaseSessionStats_IdEntryIntDeletingDtor` | ✅ | dtor | Scalar deleting destructor: writes the IdEntry<int> vtable then Loggable::vftable and frees this via _free when the flags low bit is set. |
| `00b262d0` | `InGamePurchaseSessionStats_IdEntryIntDeletingDtorVariant ⚠downgrade` | 🟢 | dtor | Second copy of the IdEntry<int> scalar deleting destructor: own vtable, then Loggable::vftable, then _free on the flags low bit. |
| `00b27b00` | `IdEntryIntInitThenSetField20` | 🟢 | ctor | Inside an SEH frame runs the default IdEntry<int> constructor FUN_00b22b80 with this, then stores *param_2 at this+0x20 and returns this. |
| `00b293c0` | `HashListRemoveIdEntryIntNode6` | 🟢 | function | Removes an int-keyed entry: unlinks the node from its 6-bucket chain via FUN_00b22520, splices it out of the list (next/prev at +0x18/+0x1c, head this+8, tail this+0xc), decrements the count at this+0x10, writes the IdE… |
| `00b29460` | `HashListRemoveIdEntryIntNode` | 🟢 | function | Removal of an int-keyed entry with its own chain walk: bucket = param_2[8] & 0xf, chain link param_2[9], table this+0x14; then splices the list (next/prev +0x18/+0x1c), decrements the count, writes the IdEntry<int> vtab… |
| `00b357c0` | `IntIdSessionStat::Log` | ✅ | virtual | Formats entry timestamp then logs entry id, count and last-occurrence string. |
