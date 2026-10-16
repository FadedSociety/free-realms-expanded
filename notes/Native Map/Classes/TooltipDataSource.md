# Class: TooltipDataSource

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00a756b0` | `TooltipDataSource::RowFieldOrError` | ✅ | virtual | Virtual slot [1]. Takes an index; when the index is below the count at this+0x248 it returns the dword at (row buffer at this+0x244) + index*32 + 4; otherwise returns the address of the literal Error!. |
| `00a76160` | `— ⚠reject` | ⚪ | virtual | Virtual slot 4 of TooltipDataSource with a stack argument index: if index is below the count at this+0x248, calls 00836c70 with the address data(this+0x244) + 0x10 + index*0x20 (the wide member of node index); otherwise… |
| `00a76930` | `TooltipDataSource::DtorBody` | ✅ | dtor | Destructor body: restores TooltipDataSource::vftable at [this] and at the secondary base slots +0x30, +0x34 and +0x254, installs SoeUtil::Array<TooltipDataSource::Node,0,1>::vftable at +0x270, calls 00a75d50 with the co… |
| `00a76a00` | `TooltipDataSource::Thunk_Slot1_Adjust254` | 🟢 | virtual | Adjustor thunk (11 bytes): subtracts 0x254 from ECX and jumps to 00a799b0. It is not a body; it forwards to 00a799b0 with this moved to the +0x254 subobject. |
| `00a76a10` | `TooltipDataSource::Slot0_GetField248` | 🟢 | virtual | Virtual slot 0 of the secondary TooltipDataSource vtable 01816410 (7 bytes): loads the 32-bit field at this+0x248 into EAX and returns it, with no arguments and no callees. |
| `00a790f0` | `— ⚠reject` | ⚪ | virtual | Virtual slot 6 of the secondary TooltipDataSource vtable 018163d4 (called with this at obj+0x254): reads the tooltip kind at this+0x2c (object +0x280). Kind 5 casts the data source at this+0x18 to AbilitiesDataSource (0… |
| `00a79880` | `TooltipDataSource::Ctor` | ✅ | ctor | Constructor: calls base constructor 00922890 with param_2, then 0081b470, stores the TooltipDataSource vtable at the primary +0 and at +0x30, +0x34 and +0x254, stores the SoeUtil::Array<TooltipDataSource::Node,0,1> vtab… |
| `00a799b0` | `TooltipDataSource::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor (27 bytes, vtable slot 0 of the primary TooltipDataSource vtable): calls the destructor body 00a76930 on this, then if bit 0 of the second argument is set calls _free (thunk 00d82463) on this,… |
