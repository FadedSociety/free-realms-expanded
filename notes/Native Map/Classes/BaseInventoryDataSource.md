# Class: BaseInventoryDataSource

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00a6d870` | `BaseInventoryDataSource::BaseInventoryDataSource ⚠downgrade` | 🟢 | ctor | Constructor body: calls 00cb1bb0 with the constructor arguments, installs BaseInventoryDataSource vtables at offsets 0x0, 0x30, 0x34 and 0x254, stores param_2 at +0x2dc, and zeroes the dwords at +0x2e0 and +0x2e4. |
| `00a6d960` | `BaseInventoryDataSource::ColumnLabel_PriceNoSale` | ✅ | virtual | Virtual slot 1: returns the static string Price for argument 0x1b and NoSale for 0x1c. Other arguments are tail-jumped to 00cb09d0 with the same argument (e9 jump, 41 bytes). |
| `00a6d990` | `BaseInventoryDataSource::QueryUnlessKind11 ?` | 🟡 | virtual | Virtual slot 2 (17 bytes, fastcall with ECX=this): when the dword at this+0x2b0 equals 0x11 it returns 0; otherwise it tail-jumps to 00cb0c60 with the same this. |
| `00a6d9b0` | `— ⚠reject` | ⚪ | virtual | Virtual slot 15 (+0x3c) row predicate. Resolves the row id through 00b0d810 (must be nonzero), then tests the owner at this+0x2dc: field +0x1bc, the manager flags and +0x3e0, the id against 008a6920 on this+0x2e0, and t… |
| `00a6dcf0` | `BaseInventoryDataSource::SecondaryAdjustorToDtor ?` | 🟡 | virtual | Adjustor thunk, 11 bytes: subtracts 0x254 from ECX and jumps to 00a6de20 (BaseInventoryDataSource scalar deleting destructor). It is slot 1 of the secondary BaseInventoryDataSource vtable at 018157b4. |
| `00a6de20` | `— ⚠reject` | ⚪ | dtor | Scalar deleting destructor: calls 00cb1b10 (SEH-prolog-style helper) with the frame cookie, then frees this through _free (thunk 00d82463) when bit 0 of the flag argument is set; returns this. The body has no member tea… |
| `00a6e5f0` | `BaseInventoryDataSource::GetRowValueKind1bOr1c ⚠downgrade` | 🟢 | virtual | Virtual slot 4 (+0x10): first calls 00cb1160 with (row, kind, out) and returns its value when nonzero. Otherwise, for kind 0x1b it looks up the row object through 009909c0 in the list at this+0x248 (row*4) and writes it… |
| `00cb0b30` | `— ⚠reject` | ⚪ | virtual | Sets a one-byte dirty flag and tail-dispatches through the base datasource vtable at slot 4. |
| `00cb1340` | `BaseInventoryDataSource::RebuildRows` | ✅ | virtual | Collects qualifying records, selects a comparator based on the mode field, sorts a temporary list, copies record identifiers into the output array, updates capacity/count state, and clears the dirty flag. |
| `00cb1500` | `BaseInventoryDataSource::GetCellText` | ✅ | virtual | Switches on the requested column, reads item or client state, formats numbers, booleans and strings into the output value, and returns whether the column was handled. |
