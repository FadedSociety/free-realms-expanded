# Class: SoeUtil::List<AssetManagement::AssetDeliveryInterface::PendingEntryData,16>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `0109b830` | `ListCtor_PendingEntryData16` | ✅ | ctor | Constructor: installs the -1 list vtable, zeroes the words at +4, +8 and +0xc, installs the 16-variant list vtable, and calls the base initializer 01025460. |
| `0109b8b0` | `Thunk_01026300_Adjust10 ?` | 🟡 | function | Adjustor thunk (bytes 83 c1 10 then jmp rel32): adds 0x10 to ECX and jumps to 01026300. |
| `0109b8c0` | `Thunk_01026370_Adjust10 ?` | 🟡 | function | Adjustor thunk (bytes 83 c1 10 then jmp rel32): adds 0x10 to ECX and jumps to 01026370. |
| `0109b8d0` | `List::~List16 → SoeUtil::List<AssetManagement::AssetDeliveryInterface::PendingE…` | ✅ | dtor | Destructor body of the 16-variant PendingEntryData list: installs the 16 vtable, drains the node list through 01093ee0, releases the auxiliary block at +0x10 (01025730 on its two words, then 00401c00), then installs the… |
| `0109b9a0` | `List::ScalarDeletingDestructor16 → SoeUtil::List<AssetManagement::AssetDelivery…` | ✅ | dtor | Scalar deleting destructor shape: calls the 16-variant list body 0109b8d0, then calls _free when bit 0 of the stack argument is set; returns the object pointer. |
